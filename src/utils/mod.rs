pub mod errors;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(level: &str) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("loc_ai_proxy={}", level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
