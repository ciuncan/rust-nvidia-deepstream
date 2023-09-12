use anyhow::Context;
use gst::prelude::*;
use gstreamer as gst;

fn main() -> anyhow::Result<()> {
    setup_observability()?;

    tracing::info!("Initializing gstreamer...");
    gst::init().context("Failed to initialize GStreamer")?;

    let pipeline = gst::parse_launch("videotestsrc ! nvvideoconvert ! filesink")?;

    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline
        .bus()
        .context("Bus wasn't available on pipeline!")?;

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                tracing::error!(
                    source = ?err.src().map(|s| s.path_string()),
                    error = ?err.error(),
                    debug = ?err.debug(),
                    "Pipeline error!",
                );
                break;
            }
            _ => (),
        }
    }

    // Shutdown pipeline
    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn setup_observability() -> anyhow::Result<()> {
    use tracing_subscriber::prelude::*;

    dotenv::dotenv()?;
    let gstreamer_verbose_loggers_level = "info";
    let gstreamer_verbose_loggers = [
        "GST_ELEMENT_FACTORY",
        "GST_PLUGIN_LOADING",
        "GST_POLL",
        "GST_REFCOUNTING",
        "GST_REGISTRY",
        "default",
        "structure",
    ];
    let mut env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .context("Couldn't load tracing's env-filter from environment")?
        .add_directive("tracing_gstreamer::callsite=warn".parse()?);
    // .add_directive("my_crate::module=trace".parse()?)
    // .add_directive("my_crate::my_other_module::something[some_inner_span]=info".parse()?)
    for logger in gstreamer_verbose_loggers.iter() {
        env_filter = env_filter.add_directive(
            format!("gstreamer::{}={}", logger, gstreamer_verbose_loggers_level).parse()?,
        );
    }

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_tracy::TracyLayer::new())
        .with(tracing_subscriber::fmt::Layer::default())
        .with(tracing_error::ErrorLayer::default())
        .init();
    gst::debug_remove_default_log_function();
    gst::debug_set_default_threshold(gst::DebugLevel::Memdump);
    tracing_gstreamer::integrate_events();
    tracing_gstreamer::integrate_spans();
    Ok(())
}
