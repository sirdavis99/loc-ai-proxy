pub mod api;
pub mod cli;
pub mod config;
pub mod models;
pub mod providers;
pub mod server;
pub mod session;
pub mod utils;

use tracing::info;

pub async fn run() -> anyhow::Result<()> {
    info!("Starting loc-ai-proxy");
    Ok(())
}
