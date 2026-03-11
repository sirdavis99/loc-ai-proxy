use crate::api::models::{ChatCompletionRequest, ChatCompletionResponse, Model};
use crate::utils::errors::Result;
use crate::providers::opencode::OpencodeProvider;

pub mod opencode;

pub struct ProviderRegistry {
    providers: Vec<Provider>,
}

pub enum Provider {
    Opencode(OpencodeProvider),
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Opencode(_) => "opencode",
        }
    }
    
    pub async fn is_available(&self) -> bool {
        match self {
            Provider::Opencode(p) => p.is_available().await,
        }
    }
    
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        match self {
            Provider::Opencode(p) => p.list_models().await,
        }
    }
    
    pub async fn create_session(&self) -> Result<String> {
        match self {
            Provider::Opencode(p) => p.create_session().await,
        }
    }
    
    pub async fn chat_completion(
        &self,
        session_id: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        match self {
            Provider::Opencode(p) => p.chat_completion(session_id, request).await,
        }
    }
    
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        match self {
            Provider::Opencode(p) => p.close_session(session_id).await,
        }
    }
    
    pub async fn health_check(&self) -> Result<()> {
        match self {
            Provider::Opencode(p) => p.health_check().await,
        }
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }
    
    pub fn register(&mut self, provider: Provider) {
        self.providers.push(provider);
    }
    
    pub async fn get_provider(&self, name: &str) -> Option<&Provider> {
        for provider in &self.providers {
            if provider.name() == name {
                return Some(provider);
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
