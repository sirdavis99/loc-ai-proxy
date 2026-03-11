use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, debug, error};

use crate::providers::ProviderAdapter;
use crate::api::models::{ChatCompletionRequest, ChatCompletionResponse, Model, Message, Choice, Usage};
use crate::utils::errors::{ProxyError, Result};

pub struct OpencodeProvider {
    client: Client,
    config: crate::config::OpencodeConfig,
}

#[derive(Debug, Serialize)]
struct OpencodeSessionRequest {
    title: String,
}

#[derive(Debug, Deserialize)]
struct OpencodeSessionResponse {
    id: String,
}

#[derive(Debug, Serialize)]
struct OpencodePromptRequest {
    parts: Vec<OpencodePart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<OpencodeModel>,
}

#[derive(Debug, Serialize)]
struct OpencodeModel {
    provider_id: String,
    model_id: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OpencodePart {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct OpencodeMessageResponse {
    id: String,
    role: String,
    parts: Vec<OpencodeResponsePart>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum OpencodeResponsePart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "reasoning")]
    Reasoning { text: String },
}

impl OpencodeProvider {
    pub fn new(config: crate::config::OpencodeConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");
        
        Self { client, config }
    }
    
    fn parse_model_id(&self, model: &str) -> (String, String) {
        // Handle formats:
        // - "opencode/anthropic/claude-3.5-sonnet" -> ("anthropic", "claude-3.5-sonnet")
        // - "anthropic/claude-3.5-sonnet" -> ("anthropic", "claude-3.5-sonnet")
        
        let parts: Vec<&str> = model.split('/').collect();
        
        if parts.len() >= 3 && parts[0] == "opencode" {
            (parts[1].to_string(), parts[2..].join("/"))
        } else if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            // Fallback: assume it's just a model ID
            ("anthropic".to_string(), model.to_string())
        }
    }
    
    async fn opencode_get_models(&self) -> Result<Vec<Model>> {
        // For now, return a static list of known opencode models
        // In production, this would query opencode's API
        let models = vec![
            Model {
                id: "opencode/anthropic/claude-3.5-sonnet".to_string(),
                object: "model".to_string(),
                created: 0,
                owned_by: "anthropic".to_string(),
            },
            Model {
                id: "opencode/google/gemini-2.5-pro".to_string(),
                object: "model".to_string(),
                created: 0,
                owned_by: "google".to_string(),
            },
            Model {
                id: "opencode/groq/llama-3.3-70b".to_string(),
                object: "model".to_string(),
                created: 0,
                owned_by: "groq".to_string(),
            },
            Model {
                id: "opencode/mistral/mistral-large".to_string(),
                object: "model".to_string(),
                created: 0,
                owned_by: "mistral".to_string(),
            },
            Model {
                id: "opencode/openrouter/deepseek/deepseek-chat".to_string(),
                object: "model".to_string(),
                created: 0,
                owned_by: "deepseek".to_string(),
            },
        ];
        
        Ok(models)
    }
}

#[async_trait]
impl ProviderAdapter for OpencodeProvider {
    fn name(&self) -> &'static str {
        "opencode"
    }
    
    async fn is_available(&self) -> bool {
        match self.health_check().await {
            Ok(_) => true,
            Err(_) => {
                if self.config.auto_start {
                    // Try to start opencode
                    if let Err(e) = self.start_opencode().await {
                        error!("Failed to auto-start opencode: {}", e);
                        return false;
                    }
                    // Retry health check
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    self.health_check().await.is_ok()
                } else {
                    false
                }
            }
        }
    }
    
    async fn list_models(&self) -> Result<Vec<Model>> {
        self.opencode_get_models().await
    }
    
    async fn create_session(&self) -> Result<String> {
        let url = format!("{}/session", self.config.url);
        
        let request = OpencodeSessionRequest {
            title: "loc-ai-proxy session".to_string(),
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Failed to create session: {}", e)))?;
        
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!("Failed to create session: {}", text)));
        }
        
        let session: OpencodeSessionResponse = response
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Failed to parse session response: {}", e)))?;
        
        debug!("Created opencode session: {}", session.id);
        Ok(session.id)
    }
    
    async fn chat_completion(
        &self,
        session_id: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let url = format!("{}/session/{}/prompt", self.config.url, session_id);
        
        // Convert messages to opencode parts
        let last_message = request.messages.last()
            .ok_or_else(|| ProxyError::InvalidRequest("No messages provided".to_string()))?;
        
        let (provider_id, model_id) = self.parse_model_id(&request.model);
        
        let opencode_request = OpencodePromptRequest {
            parts: vec![OpencodePart::Text {
                text: last_message.content.clone(),
            }],
            model: Some(OpencodeModel {
                provider_id,
                model_id,
            }),
        };
        
        debug!("Sending request to opencode: {:?}", opencode_request);
        
        let response = self.client
            .post(&url)
            .json(&opencode_request)
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!("opencode error: {}", text)));
        }
        
        // Parse response
        let message: OpencodeMessageResponse = response
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Failed to parse response: {}", e)))?;
        
        // Convert opencode response to OpenAI format
        let content = message.parts.iter()
            .filter_map(|part| match part {
                OpencodeResponsePart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        
        let reasoning = message.parts.iter()
            .filter_map(|part| match part {
                OpencodeResponsePart::Reasoning { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        
        let choice = Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: content.clone(),
            },
            finish_reason: Some("stop".to_string()),
        };
        
        Ok(ChatCompletionResponse {
            id: message.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: request.model.clone(),
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: 0,    // opencode doesn't provide these easily
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
    }
    
    async fn close_session(&self, _session_id: &str) -> Result<()> {
        // opencode sessions auto-cleanup, but we could explicitly close if needed
        Ok(())
    }
    
    async fn health_check(&self) -> Result<()> {
        let url = format!("{}/global/health", self.config.url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProxyError::ProviderUnavailable(format!("Health check failed: {}", e)))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(ProxyError::ProviderUnavailable(format!("Health check returned: {}", response.status())))
        }
    }
}

impl OpencodeProvider {
    async fn start_opencode(&self) -> Result<()> {
        use std::process::Command;
        
        info!("Auto-starting opencode...");
        
        let _child = Command::new("opencode")
            .args(["serve"])
            .spawn()
            .map_err(|e| ProxyError::ProviderError(format!("Failed to start opencode: {}", e)))?;
        
        // We intentionally don't wait for the child process
        // It will run independently
        
        Ok(())
    }
}
