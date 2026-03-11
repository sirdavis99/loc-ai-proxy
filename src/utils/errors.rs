use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Request timeout")]
    Timeout,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            ProxyError::ProviderUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "provider_unavailable",
                msg.clone(),
            ),
            ProxyError::ProviderError(msg) => {
                (StatusCode::BAD_GATEWAY, "provider_error", msg.clone())
            }
            ProxyError::SessionNotFound(msg) => {
                (StatusCode::NOT_FOUND, "session_not_found", msg.clone())
            }
            ProxyError::InvalidModel(msg) => {
                (StatusCode::BAD_REQUEST, "invalid_model", msg.clone())
            }
            ProxyError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, "invalid_request", msg.clone())
            }
            ProxyError::Timeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "timeout",
                "Request timed out".to_string(),
            ),
            ProxyError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_error",
                msg.clone(),
            ),
            ProxyError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
            ),
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "type": error_type,
                "code": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, ProxyError>;
