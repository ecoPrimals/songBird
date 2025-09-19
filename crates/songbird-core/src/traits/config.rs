// Module imports
//! Configuration Provider Trait
//!
//! Defines the interface for pluggable configuration backends,
//! supporting file-based, environment, Consul, and other configuration sources.

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use songbird_errors::SongbirdResult;
// Import all concrete config types from the config module
pub use songbird_config::*;
/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider<T>: Send + Sync
where
    T: serde::de::DeserializeOwned + Clone + Send + Sync,
{
    /// Load configuration from the provider
    async fn load_config(&self) -> SongbirdResult<T>;
    /// Reload configuration (useful for file-based configs)
    async fn reload_config(&self) -> SongbirdResult<T>;
    /// Watch for configuration changes
    async fn watch_config(&self) -> impl Stream<Item = SongbirdResult<T>>;
    /// Validate configuration before loading
    async fn validate_config(&self, config: &T) -> SongbirdResult<()>;
    /// Get provider information
    fn provider_info(&self) -> ConfigProviderInfo;
}
/// Information about a configuration provider
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Env,
}
