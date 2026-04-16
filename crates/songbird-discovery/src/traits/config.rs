// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
//! Configuration Provider Trait
//!
//! Defines the interface for pluggable configuration backends
//! supporting file-based, environment, Consul, and other configuration sources.
//!
//! # Native Async Traits
//!
//! This module uses native async fn in traits (AFIT) for zero-cost abstraction.

#![allow(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
// Import all concrete config types from the discovery config module
pub use crate::discovery::config::*;

/// Configuration provider trait
///
/// Provides pluggable configuration backends with async loading,
/// reloading, and watching capabilities.
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

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{ConfigFormat, ConfigMetadata, ConfigProviderInfo};

    #[test]
    fn config_provider_info_serde_roundtrip() {
        let info = ConfigProviderInfo {
            name: "file".into(),
            version: "1".into(),
            supports_reload: true,
            description: "d".into(),
            provider_type: "t".into(),
            supports_watch: false,
        };
        let json = serde_json::to_string(&info).expect("ser");
        let back: ConfigProviderInfo = serde_json::from_str(&json).expect("de");
        assert_eq!(back.name, "file");
        assert!(back.supports_reload);
    }

    #[test]
    fn config_metadata_serde_roundtrip() {
        let meta = ConfigMetadata {
            source: "s".into(),
            last_modified: chrono::Utc::now(),
            checksum: "abc".into(),
            version: 9,
        };
        let json = serde_json::to_string(&meta).expect("ser");
        let back: ConfigMetadata = serde_json::from_str(&json).expect("de");
        assert_eq!(back.checksum, "abc");
        assert_eq!(back.version, 9);
    }

    #[test]
    fn config_format_json_roundtrip() {
        let fmt = ConfigFormat::Json;
        let v = serde_json::to_value(&fmt).expect("to value");
        let back: ConfigFormat = serde_json::from_value(v).expect("from value");
        assert!(matches!(back, ConfigFormat::Json));
    }
}
