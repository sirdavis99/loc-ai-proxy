use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::providers::Provider;
use crate::utils::errors::Result;

pub struct SessionInfo {
    pub provider_session_id: String,
    pub provider_name: String,
    pub last_accessed: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl SessionInfo {
    pub fn new(provider_session_id: String, provider_name: String) -> Self {
        let now = Utc::now();
        Self {
            provider_session_id,
            provider_name,
            last_accessed: now,
            created_at: now,
        }
    }

    pub fn is_expired(&self, ttl_minutes: i64) -> bool {
        let ttl = Duration::minutes(ttl_minutes);
        Utc::now() - self.last_accessed > ttl
    }

    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }
}

pub struct SessionManager {
    sessions: Arc<DashMap<String, SessionInfo>>,
    ttl_minutes: i64,
}

impl SessionManager {
    pub fn new(ttl_minutes: i64) -> Self {
        let sessions: Arc<DashMap<String, SessionInfo>> = Arc::new(DashMap::new());
        let sessions_clone = sessions.clone();

        // Spawn cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                let now = Utc::now();
                let expired: Vec<String> = sessions_clone
                    .iter()
                    .filter(|entry| {
                        let ttl = Duration::minutes(30); // Default TTL
                        now - entry.last_accessed > ttl
                    })
                    .map(
                        |entry: dashmap::mapref::multiple::RefMulti<'_, String, SessionInfo>| {
                            entry.key().clone()
                        },
                    )
                    .collect();

                for key in expired {
                    debug!("Cleaning up expired session: {}", key);
                    sessions_clone.remove(&key);
                }
            }
        });

        Self {
            sessions,
            ttl_minutes,
        }
    }

    pub async fn get_or_create_session(
        &self,
        conversation_id: &str,
        provider_name: String,
        provider: &Provider,
    ) -> Result<String> {
        // Check if session exists and is valid
        if let Some(mut entry) = self.sessions.get_mut(conversation_id) {
            if !entry.is_expired(self.ttl_minutes) {
                entry.touch();
                return Ok(entry.provider_session_id.clone());
            }

            // Session expired, close it
            if let Err(e) = provider.close_session(&entry.provider_session_id).await {
                warn!("Failed to close expired session: {}", e);
            }
        }

        // Create new session
        debug!("Creating new session for conversation: {}", conversation_id);
        let provider_session_id: String = provider.create_session().await?;

        let session_info = SessionInfo::new(provider_session_id.clone(), provider_name);
        self.sessions
            .insert(conversation_id.to_string(), session_info);

        info!(
            "Created new session: {} for conversation: {}",
            provider_session_id, conversation_id
        );

        Ok(provider_session_id)
    }

    pub fn get_session(&self, conversation_id: &str) -> Option<String> {
        self.sessions
            .get(conversation_id)
            .map(|s| s.provider_session_id.clone())
    }

    pub fn remove_session(&self, conversation_id: &str) {
        self.sessions.remove(conversation_id);
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_expiration() {
        let session = SessionInfo::new("test-123".to_string(), "opencode".to_string());

        // Should not be expired immediately
        assert!(!session.is_expired(30));
    }
}
