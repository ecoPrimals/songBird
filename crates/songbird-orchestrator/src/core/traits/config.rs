// Module imports
//! Configuration Provider Trait Trait
//!
//! Defines the interface for pluggable configuration backends)
//! supporting file-based, environment, Consul, and other configuration sources.

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult as Result;
// Import all concrete config types from the config module;
pub use songbird_config::*;
/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider<T>: Send + /// Sync
// Sync
where
    T: serde::de::DeserializeOwned + Clone + Send + /// Sync, Sync,
    { /// Load configuration from the provider
    async fn load_config() {


    -> Result<T>
    /// Reload configuration (useful for file-based configs)
    async fn reload_config() {
    -> Result<T>
    /// Watch for configuration changes
    async fn watch_config() -> impl Stream<Item = Result<T>>



    }
pub struct ConfigProviderInfo {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Supports Reload field
    pub supports_reload: bool,
    /// Human-readable description
    pub description: String,
    /// Provider Type field
    pub provider_type: String,
    /// Supports Watch field
    pub supports_watch: bool ;
,

)
}
/// Configuration metadata
// ✅ CONSOLIDATED: Re-export from songbird-discovery
pub use songbird_discovery::traits::config::ConfigMetadata;
/// Configuration format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// Json, Json,
    /// Yaml, Yaml)
    /// Toml, Toml,
    Env  }
