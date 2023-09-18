use anyhow::Context;

pub use tracing::instrument;
pub use tracing::{debug, error, info, trace, warn};

use crate::MultiDropGuard;

#[instrument()]
pub fn setup_tracing() -> anyhow::Result<MultiDropGuard> {
    use tracing_subscriber::prelude::*;

    let mut droppables = MultiDropGuard::default();

    dotenv::dotenv()?;
    let gstreamer_verbose_loggers_level = "info";
    let gstreamer_verbose_loggers = [
        "GST_ELEMENT_FACTORY",
        "GST_PLUGIN_LOADING",
        "GST_POLL",
        "GST_REFCOUNTING",
        // "GST_REGISTRY",
        "default",
        "structure",
    ];
    let mut env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .context("Couldn't load tracing's env-filter from environment")?
        .add_directive("tracing_gstreamer::callsite=warn".parse()?);
    for logger in gstreamer_verbose_loggers.iter() {
        env_filter = env_filter.add_directive(
            format!("gstreamer::{}={}", logger, gstreamer_verbose_loggers_level).parse()?,
        );
    }

    let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    droppables.add(Box::new(guard));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_tracy::TracyLayer::new())
        .with(chrome_layer)
        .with(tracing_subscriber::fmt::Layer::default())
        .with(tracing_error::ErrorLayer::default())
        .init();

    Ok(droppables)
}
