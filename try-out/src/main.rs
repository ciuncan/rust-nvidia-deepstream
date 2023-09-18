use anyhow::Context;
use gst::prelude::*;
use gstreamer as gst;
use std::sync::Arc;

use shared::observability::*;

fn main() -> anyhow::Result<()> {
    let _droppables = setup_observability()?;

    info!("Initializing gstreamer...");
    gst::init().context("Failed to initialize GStreamer")?;

    let src_elem = gst::ElementFactory::make_with_name("videotestsrc", Some("source"))?;
    let convert_elem = gst::ElementFactory::make_with_name("nvvideoconvert", Some("convert"))?;
    let sink_elem = gst::ElementFactory::make_with_name("fakesink", Some("sink"))?;

    // unsafe {
    //     // gst_buffer_get_nvds_batch_meta
    //     let batch_meta_mut_ptr: *mut nvds::ffi::NvDsBatchMeta =
    //         nvds::ffi::gst_buffer_get_nvds_batch_meta(gst_buffer_mut_ptr);
    //     let frame_meta_list: nvds::NvDsFrameMetaList =
    //         (*batch_meta_mut_ptr).frame_meta_list.into();
    //     let frame_meta = *frame_meta_list.next().unwrap();
    //     frame_meta.
    // }

    let pipeline = gst::Pipeline::with_name("test-pipeline");

    let elems = [&src_elem, &convert_elem, &sink_elem];
    for (p_elem, &n_elem) in elems.iter().zip(elems.iter().skip(1)) {
        p_elem.link(n_elem)?;
    }
    pipeline.add_many(elems)?;

    let pipeline = Arc::new(pipeline);

    pipeline.set_state(gst::State::Playing)?;

    let handler_pipeline_ref = Arc::clone(&pipeline);
    ctrlc::set_handler(move || {
        println!("Handler run");
        handler_pipeline_ref
            .set_state(gst::State::Null)
            .expect("Couldn't set pipeline to null during shutdown!");
    })?;

    let bus = pipeline
        .bus()
        .context("Bus wasn't available on pipeline!")?;

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                error!(
                    source = ?err.src().map(|s| s.path_string()),
                    error = ?err.error(),
                    debug = ?err.debug(),
                    "Pipeline error!",
                );
                break;
            }
            m => trace!(?m, "Unhandled message"),
        }
    }

    // Shutdown pipeline
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn setup_observability() -> anyhow::Result<shared::MultiDropGuard> {
    let droppables = setup_tracing()?;
    gst::debug_remove_default_log_function();
    gst::debug_set_default_threshold(gst::DebugLevel::Memdump);
    tracing_gstreamer::integrate_events();
    tracing_gstreamer::integrate_spans();
    Ok(droppables)
}
