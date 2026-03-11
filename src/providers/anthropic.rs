use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

use crate::api::models::{ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Usage};
use crate::utils::errors::{ProxyError, Result};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    #[serde(rename = "stop_reason")]
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(mut config: crate::config::AnthropicConfig) -> Self {
        // Try to auto-detect API key from environment if not configured
        if config.api_key.is_none() {
            config.api_key = Self::detect_api_key_from_env();
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key: config.api_key.unwrap_or_default(),
            base_url: config.url,
        }
    }

    /// Try to detect Anthropic API key from environment
    fn detect_api_key_from_env() -> Option<String> {
        std::env::var("ANTHROPIC_API_KEY").ok()
    }

    /// Get API key or return error
    fn get_api_key(&self) -> Result<&str> {
        if self.api_key.is_empty() {
            return Err(ProxyError::ProviderError(
                "Anthropic API key not configured. \
                Set ANTHROPIC_API_KEY environment variable or configure api_key in the config file. \
                Get your API key from: https://console.anthropic.com/settings/keys".to_string()
            ));
        }
        Ok(&self.api_key)
    }

    pub async fn chat_completion(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let api_key = self.get_api_key()?;

        // Convert OpenAI format to Anthropic format
        let anthropic_request = self.convert_request(request)?;

        debug!("Sending request to Anthropic API");

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| {
                ProxyError::ProviderError(format!("Failed to send request to Anthropic: {}", e))
            })?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Anthropic API error (HTTP {}): {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("Failed to parse Anthropic response: {}", e))
        })?;

        debug!("Received response from Anthropic API");

        // Convert Anthropic response to OpenAI format
        Ok(self.convert_response(&anthropic_response, &request.model))
    }

    fn convert_request(&self, request: &ChatCompletionRequest) -> Result<AnthropicRequest> {
        // Parse model ID - Anthropic models don't need the "anthropic/" prefix
        let model = if request.model.starts_with("anthropic/") {
            request
                .model
                .strip_prefix("anthropic/")
                .unwrap_or(&request.model)
                .to_string()
        } else if request.model.starts_with("claude-") {
            request.model.clone()
        } else {
            // Try to resolve alias
            crate::models::resolve_model_alias(&request.model)
                .strip_prefix("anthropic/")
                .unwrap_or(&request.model)
                .to_string()
        };

        // Convert messages - Anthropic uses "user" and "assistant" roles
        let messages: Vec<AnthropicMessage> = request
            .messages
            .iter()
            .map(|msg| AnthropicMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect();

        Ok(AnthropicRequest {
            model,
            max_tokens: 4096, // Default max tokens
            messages,
        })
    }

    fn convert_response(
        &self,
        response: &AnthropicResponse,
        original_model: &str,
    ) -> ChatCompletionResponse {
        // Extract text content from response
        let content = response
            .content
            .iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("");

        let choice = Choice {
            index: 0,
            message: Message {
                role: response.role.clone(),
                content,
                name: None,
            },
            finish_reason: response.stop_reason.clone(),
        };

        ChatCompletionResponse {
            id: response.id.clone(),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: original_model.to_string(),
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: response.usage.input_tokens as i32,
                completion_tokens: response.usage.output_tokens as i32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as i32,
            },
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        match self.get_api_key() {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_parsing() {
        let config = crate::config::AnthropicConfig {
            api_key: Some("test-key".to_string()),
            url: "https://api.anthropic.com".to_string(),
            timeout_seconds: 30,
        };

        let provider = AnthropicProvider::new(config);

        // Should accept claude-3.5-sonnet alias
        let request = ChatCompletionRequest {
            model: "claude-3.5-sonnet".to_string(),
            messages: vec![crate::api::models::Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            stream: None,
        };

        let anthropic_req = provider.convert_request(&request).unwrap();
        assert!(anthropic_req.model.contains("claude"));
    }
}
