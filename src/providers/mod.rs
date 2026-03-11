use crate::api::models::{ChatCompletionRequest, ChatCompletionResponse, Model};
use crate::utils::errors::Result;
use crate::providers::opencode::OpencodeProvider;
use crate::providers::anthropic::AnthropicProvider;

pub mod opencode;
pub mod anthropic;

pub struct ProviderRegistry {
    providers: Vec<Provider>,
}

pub enum Provider {
    Opencode(OpencodeProvider),
    Anthropic(AnthropicProvider),
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Opencode(_) => "opencode",
            Provider::Anthropic(_) => "anthropic",
        }
    }

    pub async fn is_available(&self) -> bool {
        match self {
            Provider::Opencode(p) => p.is_available().await,
            Provider::Anthropic(p) => p.health_check().await.unwrap_or(false),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<Model>> {
        match self {
            Provider::Opencode(p) => p.list_models().await,
            Provider::Anthropic(_) => {
                // Return Anthropic models
                Ok(vec![
                    Model {
                        id: "anthropic/claude-3-opus-20240229".to_string(),
                        object: "model".to_string(),
                        created: 0,
                        owned_by: "anthropic".to_string(),
                    },
                    Model {
                        id: "anthropic/claude-3-5-sonnet-20241022".to_string(),
                        object: "model".to_string(),
                        created: 0,
                        owned_by: "anthropic".to_string(),
                    },
                    Model {
                        id: "anthropic/claude-3-5-haiku-20241022".to_string(),
                        object: "model".to_string(),
                        created: 0,
                        owned_by: "anthropic".to_string(),
                    },
                ])
            }
        }
    }

    pub async fn create_session(&self) -> Result<String> {
        match self {
            Provider::Opencode(p) => p.create_session().await,
            Provider::Anthropic(_) => {
                // Anthropic doesn't use sessions, return a placeholder
                Ok(format!("anthropic-{}-{}-direct", std::process::id(), chrono::Utc::now().timestamp()))
            }
        }
    }

    pub async fn chat_completion(
        &self,
        _session_id: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        match self {
            Provider::Opencode(p) => p.chat_completion(_session_id, request).await,
            Provider::Anthropic(p) => p.chat_completion(request).await,
        }
    }

    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        match self {
            Provider::Opencode(p) => p.close_session(session_id).await,
            Provider::Anthropic(_) => {
                // Anthropic doesn't use sessions, nothing to close
                Ok(())
            }
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        match self {
            Provider::Opencode(p) => p.health_check().await,
            Provider::Anthropic(p) => {
                if p.health_check().await? {
                    Ok(())
                } else {
                    Err(crate::utils::errors::ProxyError::ProviderError(
                        "Anthropic API key not configured".to_string()
                    ))
                }
            }
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
