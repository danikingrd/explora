//! Logging-related configuration common to all binaries.
//!
//! This is useful because we can change our logging settings everywhere from here.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing/logging.
///
/// The default filter level is set to `INFO`.
pub fn init() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::default()
                .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
        )
        .init();

    let level = tracing::level_filters::LevelFilter::current();
    tracing::info!("Tracing level set to `{}`.", level);
}
