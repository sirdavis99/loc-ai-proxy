use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use tracing::{info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub providers: HashMap<String, ProviderConfig>,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub settings: ProviderSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderSettings {
    #[serde(rename = "opencode")]
    Opencode(OpencodeConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpencodeConfig {
    pub url: String,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_health_interval")]
    pub health_check_interval_seconds: u64,
    #[serde(default = "default_session_ttl")]
    pub session_ttl_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub format: LogFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Pretty
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                port: 9110,
                host: "127.0.0.1".to_string(),
                request_timeout_seconds: 120,
            },
            providers: HashMap::new(),
            logging: LoggingConfig {
                level: default_log_level(),
                format: LogFormat::default(),
            },
        }
    }
}

fn default_timeout() -> u64 {
    120
}

fn default_health_interval() -> u64 {
    30
}

fn default_session_ttl() -> i64 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    pub async fn load(path: Option<&str>) -> Result<Self> {
        // Try specified path first
        if let Some(p) = path {
            debug!("Loading config from: {}", p);
            return Self::load_from_file(p).await;
        }
        
        // Try standard locations
        let config_paths = vec![
            dirs::config_dir().map(|d| d.join("loc-ai-proxy/config.yaml")),
            dirs::home_dir().map(|d| d.join(".config/loc-ai-proxy/config.yaml")),
            Some(PathBuf::from("./config.yaml")),
        ];
        
        for path in config_paths.into_iter().flatten() {
            if path.exists() {
                debug!("Found config at: {}", path.display());
                return Self::load_from_file(&path.to_string_lossy()).await;
            }
        }
        
        // Return default config
        info!("No config file found, using defaults");
        Ok(Config::default())
    }
    
    async fn load_from_file(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;
        
        info!("Loaded configuration from: {}", path);
        Ok(config)
    }
    
    pub async fn save(&self, path: Option<&str>) -> Result<()> {
        let path = path.map(PathBuf::from)
            .or_else(|| dirs::config_dir().map(|d| d.join("loc-ai-proxy/config.yaml")))
            .or_else(|| dirs::home_dir().map(|d| d.join(".config/loc-ai-proxy/config.yaml")))
            .context("Could not determine config directory")?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;
        
        tokio::fs::write(&path, content).await
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        info!("Saved configuration to: {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_load_default_config() {
        let config = Config::load(None).await.unwrap();
        assert_eq!(config.server.port, 9110);
        assert_eq!(config.server.host, "127.0.0.1");
    }
    
    #[tokio::test]
    async fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let mut config = Config::default();
        config.server.port = 9999;
        
        config.save(Some(config_path.to_str().unwrap())).await.unwrap();
        
        let loaded = Config::load(Some(config_path.to_str().unwrap())).await.unwrap();
        assert_eq!(loaded.server.port, 9999);
    }
}
