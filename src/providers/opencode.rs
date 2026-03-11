use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, debug, error, warn};

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
    #[serde(rename = "providerID")]
    provider_id: String,
    #[serde(rename = "modelID")]
    model_id: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OpencodePart {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Clone, Deserialize)]
struct OpencodeMessageHistoryResponse {
    info: OpencodeMessageInfo,
    parts: Vec<OpencodeResponsePart>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpencodeMessageInfo {
    role: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum OpencodeResponsePart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "reasoning")]
    Reasoning { text: String },
}

impl OpencodeProvider {
    pub fn new(mut config: crate::config::OpencodeConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");
        
        // Try to auto-detect auth if not configured
        if config.auth.is_none() {
            config.auth = Self::detect_auth_from_env();
        }
        
        Self { client, config }
    }
    
    /// Try to detect opencode authentication from environment
    fn detect_auth_from_env() -> Option<crate::config::OpencodeAuth> {
        use std::env;
        
        let username = env::var("OPENCODE_SERVER_USERNAME").ok()?;
        let password = env::var("OPENCODE_SERVER_PASSWORD").ok()?;
        
        Some(crate::config::OpencodeAuth { username, password })
    }
    
    /// Get authentication credentials or return error
    fn get_auth(&self) -> Result<&crate::config::OpencodeAuth> {
        self.config.auth.as_ref().ok_or_else(|| {
            ProxyError::ProviderError(
                "Authentication not configured. \
                Set OPENCODE_SERVER_USERNAME and OPENCODE_SERVER_PASSWORD environment variables, \
                or configure auth in the config file. \
                You can also use CLI mode which handles auth automatically.".to_string()
            )
        })
    }
    
    /// Create authenticated request builder
    fn auth_request(&self, method: reqwest::Method, url: String) -> Result<reqwest::RequestBuilder> {
        let auth = self.get_auth()?;
        Ok(self.client
            .request(method, url)
            .basic_auth(&auth.username, Some(&auth.password)))
    }
    
    pub async fn is_available(&self) -> bool {
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
    
    pub async fn list_models(&self) -> Result<Vec<Model>> {
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
    
    pub async fn create_session(&self) -> Result<String> {
        let url = format!("{}/session", self.config.url);
        
        let request = OpencodeSessionRequest {
            title: "loc-ai-proxy session".to_string(),
        };
        
        let response = self.auth_request(reqwest::Method::POST, url)?
            .json(&request)
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Failed to create session: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(ProxyError::ProviderError(
                    "Authentication failed. Please check your OPENCODE_SERVER_USERNAME and OPENCODE_SERVER_PASSWORD.".to_string()
                ));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!("Failed to create session: HTTP {} - {}", status, text)));
        }
        
        let session: OpencodeSessionResponse = response
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Failed to parse session response: {}", e)))?;
        
        debug!("Created opencode session: {}", session.id);
        Ok(session.id)
    }
    
    pub async fn chat_completion(
        &self,
        _session_id: &str,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        // Use HTTP-based approach (more reliable for programmatic access)
        self.chat_completion_http(request).await
    }
    
    /// CLI-based chat completion using opencode run command
    /// More reliable than HTTP API as it handles auth, sessions, and model routing automatically
    async fn chat_completion_cli(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let start_time = std::time::Instant::now();
        let max_duration = Duration::from_secs(60); // 60 second timeout
        
        // Extract the last message content
        let last_message = request.messages.last()
            .ok_or_else(|| ProxyError::InvalidRequest("No messages provided".to_string()))?;
        
        // Parse model ID for CLI
        let model_arg = if request.model.starts_with("opencode/") {
            // Remove the "opencode/" prefix
            request.model.strip_prefix("opencode/").unwrap_or(&request.model).to_string()
        } else {
            request.model.clone()
        };
        
        debug!("Starting CLI chat completion with model: {}", model_arg);
        debug!("Prompt: {}...", &last_message.content.chars().take(100).collect::<String>());
        
        // Build environment variables for CLI
        let mut env_vars = std::collections::HashMap::new();
        
        // Pass through authentication if available
        if let Some(auth) = &self.config.auth {
            env_vars.insert("OPENCODE_SERVER_USERNAME".to_string(), auth.username.clone());
            env_vars.insert("OPENCODE_SERVER_PASSWORD".to_string(), auth.password.clone());
        }
        
        // Set server URL
        env_vars.insert("OPENCODE_SERVER_URL".to_string(), self.config.url.clone());
        
        // Build the CLI command
        // Using 'opencode run' with message as positional argument
        let mut cmd = Command::new("opencode");
        cmd.arg("run")
           .arg("--model")
           .arg(&model_arg)
           .arg("--format")
           .arg("json")
           .arg(&last_message.content)  // Pass message as positional argument
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Apply environment variables
        cmd.envs(&env_vars);
        
        debug!("Spawning opencode CLI command");
        
        // Spawn the process with timeout
        let result = timeout(max_duration, async {
            let child = cmd.spawn().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ProxyError::ProviderError(
                        "opencode CLI not found. Please install opencode and ensure it's in your PATH. \
                        Visit https://opencode.ai for installation instructions.".to_string()
                    )
                } else {
                    ProxyError::ProviderError(format!("Failed to spawn opencode CLI: {}", e))
                }
            })?;
            
            // Wait for the process with a timeout
            let output = child.wait_with_output().await.map_err(|e| {
                ProxyError::ProviderError(format!("Failed to read opencode output: {}", e))
            })?;
            
            Ok(output)
        }).await;
        
        let output = match result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(ProxyError::Timeout);
            }
        };
        
        // Check if the process succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);
            
            error!("opencode CLI exited with code {}: {}", exit_code, stderr);
            
            return Err(ProxyError::ProviderError(format!(
                "opencode CLI failed (exit code {}): {}. \
                This usually means the model '{}' is not available or the server is not running. \
                Try running 'opencode run --model {} --help' to verify the setup.",
                exit_code, stderr, model_arg, model_arg
            )));
        }
        
        // Parse the stdout
        let stdout = String::from_utf8_lossy(&output.stdout);
        let content = self.extract_content_from_cli_output(&stdout);
        
        let elapsed = start_time.elapsed();
        debug!("CLI chat completion completed in {:?}", elapsed);
        
        // Generate a unique ID for this completion
        let completion_id = format!("chatcmpl-{}-opencode-cli", chrono::Utc::now().timestamp());
        
        let choice = Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content,
                name: None,
            },
            finish_reason: Some("stop".to_string()),
        };
        
        Ok(ChatCompletionResponse {
            id: completion_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: request.model.clone(),
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
    }
    
    /// Extract content from CLI output (handles both JSON and plain text)
    fn extract_content_from_cli_output(&self, output: &str) -> String {
        let trimmed = output.trim();
        
        // Try to parse as JSON first
        if trimmed.starts_with('{') {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                // Try various JSON paths that opencode might use
                if let Some(content) = json.get("content").and_then(|v| v.as_str()) {
                    return content.to_string();
                }
                if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                    return text.to_string();
                }
                if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                    return message.to_string();
                }
                if let Some(response) = json.get("response").and_then(|v| v.as_str()) {
                    return response.to_string();
                }
                if let Some(output) = json.get("output").and_then(|v| v.as_str()) {
                    return output.to_string();
                }
            }
        }
        
        // Fallback: return the raw output (strip any JSON formatting if present)
        trimmed.to_string()
    }
    
    /// HTTP-based chat completion with polling (fallback method)
    async fn chat_completion_http(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        // Create a session first
        let session_id = self.create_session().await.map_err(|e| {
            ProxyError::ProviderError(format!(
                "Failed to create session: {}. \
                Please verify that opencode server is running at {} and authentication is configured correctly. \
                You can check with 'opencode status' or try auto-start by setting auto_start: true in config.",
                e, self.config.url
            ))
        })?;
        
        // Send the prompt using prompt_async endpoint (returns 204 No Content)
        let prompt_url = format!("{}/session/{}/prompt_async", self.config.url, session_id);
        
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
        
        debug!("Sending async request to opencode HTTP API: {:?}", opencode_request);
        
        let response = self.auth_request(reqwest::Method::POST, prompt_url)?
            .json(&opencode_request)
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!(
                "Failed to send prompt to HTTP API: {}. \
                The opencode server at {} may be unreachable. \
                Check if the server is running with 'curl {}/global/health'.",
                e, self.config.url, self.config.url
            )))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(
                format!("Failed to send prompt: HTTP {} - {}. \
                This usually indicates an authentication issue or invalid model selection.",
                status, text)
            ));
        }
        
        // Poll the message endpoint to get the assistant response
        let message_url = format!("{}/session/{}/message", self.config.url, session_id);
        let start_time = std::time::Instant::now();
        let max_duration = Duration::from_secs(30);
        let poll_interval = Duration::from_millis(500);
        
        let assistant_message = loop {
            // Check timeout
            if start_time.elapsed() > max_duration {
                return Err(ProxyError::Timeout);
            }
            
            // Poll the message endpoint
            let response = self.auth_request(reqwest::Method::GET, message_url.clone())?
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Failed to poll messages: {}", e)))?;
            
            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(ProxyError::ProviderError(
                    format!("Failed to poll messages: HTTP {} - {}", status, text)
                ));
            }
            
            // Parse message history
            let messages: Vec<OpencodeMessageHistoryResponse> = response
                .json()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Failed to parse message history: {}", e)))?;
            
            debug!("Polled {} messages from HTTP API", messages.len());
            
            // Find the latest assistant message with content
            if let Some(assistant_msg) = messages.iter().rev().find(|m| {
                m.info.role == "assistant" && !m.parts.is_empty()
            }) {
                break assistant_msg.clone();
            }
            
            // Wait before polling again
            tokio::time::sleep(poll_interval).await;
        };
        
        // Extract text content from assistant response
        let content = assistant_message.parts.iter()
            .filter_map(|part| match part {
                OpencodeResponsePart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        
        // Clean up the session
        let _ = self.close_session(&session_id).await;
        
        // Generate a unique ID for this completion
        let completion_id = format!("chatcmpl-{}-opencode-http", chrono::Utc::now().timestamp());
        
        let choice = Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content,
                name: None,
            },
            finish_reason: Some("stop".to_string()),
        };
        
        Ok(ChatCompletionResponse {
            id: completion_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: request.model.clone(),
            choices: vec![choice],
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
    }
    
    pub async fn close_session(&self, _session_id: &str) -> Result<()> {
        // opencode sessions auto-cleanup, but we could explicitly close if needed
        Ok(())
    }
    
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/global/health", self.config.url);
        
        // If auth is configured, use it; otherwise try without auth (for older opencode versions)
        let response = if let Some(auth) = &self.config.auth {
            self.client
                .get(&url)
                .basic_auth(&auth.username, Some(&auth.password))
                .send()
                .await
        } else {
            self.client.get(&url).send().await
        };
        
        let response = response
            .map_err(|e| ProxyError::ProviderUnavailable(format!("Health check failed: {}", e)))?;
        
        match response.status() {
            status if status.is_success() => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(ProxyError::ProviderError(
                    "Authentication failed. Please set OPENCODE_SERVER_USERNAME and OPENCODE_SERVER_PASSWORD environment variables.".to_string()
                ))
            }
            _ => Err(ProxyError::ProviderUnavailable(format!(
                "Health check returned: {}", response.status()
            )))
        }
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
