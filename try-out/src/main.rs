use anyhow::Context;
use gst::prelude::*;
use gstreamer as gst;
use std::sync::Arc;

use shared::observability::*;

fn main() -> anyhow::Result<()> {
    let _droppables = setup_observability()?;

    info!("Initializing gstreamer...");
    gst::init().context("Failed to initialize GStreamer")?;

    let pipeline = gst::parse_launch("videotestsrc ! nvvideoconvert ! filesink")?;

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
