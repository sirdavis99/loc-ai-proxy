use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};

use crate::config::{Config, ProviderSettings};
use crate::api::create_router;
use crate::providers::{ProviderRegistry, opencode::OpencodeProvider};
use crate::session::SessionManager;
use crate::utils::errors::ProxyError;

pub struct Server {
    config: Config,
    host: String,
    port: u16,
    router: Router,
}

impl Server {
    pub async fn new(config: Config, host: String, port: u16) -> anyhow::Result<Self> {
        // Initialize providers
        let mut provider_registry = ProviderRegistry::new();
        
        // Register opencode if configured
        if let Some(opencode_config) = config.providers.get("opencode") {
            if opencode_config.enabled {
                if let ProviderSettings::Opencode(settings) = &opencode_config.settings {
                    info!("Registering opencode provider");
                    let provider = OpencodeProvider::new(settings.clone());
                    provider_registry.register(Box::new(provider));
                }
            }
        }
        
        let provider_registry = Arc::new(provider_registry);
        
        // Initialize session manager (30 minute TTL)
        let session_manager = Arc::new(SessionManager::new(30));
        
        // Create router
        let router = create_router(session_manager, provider_registry);
        
        Ok(Self {
            config,
            host,
            port,
            router,
        })
    }
    
    pub async fn run(self) -> anyhow::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| {
                error!("Failed to bind to {}: {}", addr, e);
                ProxyError::Internal(format!("Failed to bind: {}", e))
            })?;
        
        info!("🚀 Server running at http://{}", addr);
        info!("Health check: http://{}/health", addr);
        
        axum::serve(listener, self.router)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
        
        Ok(())
    }
}
