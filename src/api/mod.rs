pub mod models;

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::Extension,
};
use std::sync::Arc;
use tracing::{info, debug, error};

use crate::api::models::{
    ChatCompletionRequest, ChatCompletionResponse, 
    ModelsResponse, HealthResponse
};
use crate::session::SessionManager;
use crate::providers::ProviderRegistry;
use crate::utils::errors::Result;

pub fn create_router(
    session_manager: Arc<SessionManager>,
    provider_registry: Arc<ProviderRegistry>,
) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        .route("/health", get(health_check))
        .layer(Extension(session_manager))
        .layer(Extension(provider_registry))
}

async fn chat_completions(
    Extension(session_manager): Extension<Arc<SessionManager>>,
    Extension(provider_registry): Extension<Arc<ProviderRegistry>>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>> {
    debug!("Received chat completion request for model: {}", request.model);
    
    // Determine provider from model ID
    let provider_name = extract_provider_name(&request.model);
    
    let provider = provider_registry
        .get_provider(&provider_name)
        .await
        .ok_or_else(|| {
            error!("Provider not found: {}", provider_name);
            crate::utils::errors::ProxyError::InvalidModel(format!("Provider not found: {}", provider_name))
        })?;
    
    // Get or create session
    let conversation_id = request.conversation_id.clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let session_id = session_manager
        .get_or_create_session(&conversation_id, provider_name.to_string(), provider)
        .await?;
    
    debug!("Using session: {} for conversation: {}", session_id, conversation_id);
    
    // Send request to provider
    let response = provider.chat_completion(&session_id, &request).await?;
    
    info!(
        "Completed chat completion - model: {}, tokens: {}",
        request.model,
        response.usage.total_tokens
    );
    
    Ok(Json(response))
}

async fn list_models(
    Extension(provider_registry): Extension<Arc<ProviderRegistry>>,
) -> Result<Json<ModelsResponse>> {
    let models = provider_registry.list_all_models().await?;
    
    Ok(Json(ModelsResponse {
        object: "list".to_string(),
        data: models,
    }))
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

fn extract_provider_name(model_id: &str) -> String {
    // Extract provider from model ID
    // Format: "provider/vendor/model" or "provider/model"
    let parts: Vec<&str> = model_id.split('/').collect();
    
    if parts.len() >= 2 {
        parts[0].to_string()
    } else {
        // Default to opencode if no provider prefix
        "opencode".to_string()
    }
}
