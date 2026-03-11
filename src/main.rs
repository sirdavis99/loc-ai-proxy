use clap::{Parser, Subcommand};
use tracing::{info, error};
use std::process;

mod api;
mod cli;
mod config;
mod models;
mod providers;
mod server;
mod session;
mod utils;

use crate::config::Config;
use crate::server::Server;

#[derive(Parser)]
#[command(name = "locaiproxy")]
#[command(about = "OpenAI-compatible proxy for local AI agents")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Port to listen on
    #[arg(short, long, env = "LOC_AI_PROXY_PORT", default_value = "9110")]
    pub port: u16,
    
    /// Host to bind to
    #[arg(short = 'H', long, env = "LOC_AI_PROXY_HOST", default_value = "127.0.0.1")]
    pub host: String,
    
    /// Configuration file path
    #[arg(short, long, env = "LOC_AI_PROXY_CONFIG")]
    pub config: Option<String>,
    
    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Configure a provider
    Configure {
        /// Provider to configure
        #[arg(value_enum)]
        provider: cli::ProviderChoice,
    },
    /// Show status of all providers
    Status,
    /// List available models
    ListModels,
    /// Run diagnostics
    Doctor,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("loc_ai_proxy={}", log_level))
        .init();
    
    info!("Starting loc-ai-proxy v{}", env!("CARGO_PKG_VERSION"));
    
    match cli.command {
        Some(Commands::Configure { provider }) => {
            cli::configure_provider(provider).await;
        }
        Some(Commands::Status) => {
            cli::show_status().await;
        }
        Some(Commands::ListModels) => {
            cli::list_models().await;
        }
        Some(Commands::Doctor) => {
            cli::run_diagnostics().await;
        }
        None => {
            // Start server mode
            run_server(cli).await;
        }
    }
}

async fn run_server(cli: Cli) {
    // Load configuration
    let config = match Config::load(cli.config.as_deref()).await {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };
    
    // Override with CLI args
    let port = cli.port;
    let host = cli.host;
    
    info!("Configuration loaded successfully");
    info!("Binding to {}:{}", host, port);
    
    // Create and start server
    let server = match Server::new(config, host, port).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create server: {}", e);
            process::exit(1);
        }
    };
    
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        process::exit(1);
    }
}
