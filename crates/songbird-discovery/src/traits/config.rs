// Module imports
//! Configuration Provider Trait
//!
//! Defines the interface for pluggable configuration backends,
//! supporting file-based, environment, Consul, and other configuration sources.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_errors::SongbirdResult;
type Result<T> = SongbirdResult<T>;
// Import all concrete config types from the discovery config module
pub use crate::discovery::config::*;

/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider<T>: Send + Sync
where
    T: serde::de::DeserializeOwned + Clone + Send + Sync,
{
    /// Load configuration from the provider
    async fn load_config(&self) -> Result<T>;
    /// Reload configuration (useful for file-based configs)
    async fn reload_config(&self) -> Result<T>;
    /// Watch for configuration changes (returns a receiver for config updates)
    async fn watch_config(&self) -> Result<tokio::sync::watch::Receiver<T>>;
    /// Validate configuration before loading
    async fn validate_config(&self, config: &T) -> Result<()>;
    /// Get provider information
    fn provider_info(&self) -> ConfigProviderInfo;
}

/// Provider information struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProviderInfo {
    pub name: String,
    pub version: String,
    pub supports_reload: bool,
    pub description: String,
    pub provider_type: String,
    pub supports_watch: bool,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub source: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
    pub version: u64,
}

/// Configuration format enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Env,
}
