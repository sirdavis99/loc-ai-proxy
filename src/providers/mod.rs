use async_trait::async_trait;
use crate::utils::errors::{ProxyError, Result};
use crate::api::models::{ChatCompletionRequest, ChatCompletionResponse, Model};

pub mod opencode;

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Provider name
    fn name(&self) -> &'static str;
    
    /// Check if provider is available
    async fn is_available(&self) -> bool;
    
    /// List available models
    async fn list_models(&self) -> Result<Vec<Model>>;
    
    /// Create a new session
    async fn create_session(&self) -> Result<String>;
    
    /// Send a chat completion request
    async fn chat_completion(
        &self,
        session_id: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse>;
    
    /// Close a session
    async fn close_session(&self, session_id: &str) -> Result<()>;
    
    /// Health check
    async fn health_check(&self) -> Result<()>;
}

pub struct ProviderRegistry {
    providers: Vec<Box<dyn ProviderAdapter>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    pub fn register(&mut self, provider: Box<dyn ProviderAdapter>) {
        self.providers.push(provider);
    }
    
    pub async fn get_provider(&self, name: &str) -> Option<&dyn ProviderAdapter> {
        for provider in &self.providers {
            if provider.name() == name {
                return Some(provider.as_ref());
            }
        }
        None
    }
    
    pub async fn list_all_models(&self) -> Result<Vec<Model>> {
        let mut all_models = Vec::new();
        
        for provider in &self.providers {
            match provider.list_models().await {
                Ok(models) => all_models.extend(models),
                Err(e) => {
                    tracing::warn!("Failed to list models for {}: {}", provider.name(), e);
                }
            }
        }
        
        Ok(all_models)
    }
}
