// Core unified configuration types
//
// This module provides the foundational types and structures for the unified
// configuration system, replacing fragmented configuration patterns.

use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Core unified configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCoreConfig {
    /// Service configuration
    pub service: ServiceConfig,
    /// Environment configuration
    pub environment: EnvironmentConfig,
    /// Observability configuration
    pub observability: CanonicalObservabilityConfig,
    /// Additional configuration extensions
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service instance ID
    pub instance_id: String,
    /// Additional metadata tags
    pub tags: Vec<String>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "songbird".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: format!("instance-{}", std::process::id()),
            tags: vec!["songbird".to_owned()],
        }
    }
}

/// Environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment field
    pub environment: String,
    /// Debug field
    pub debug: bool,
    /// Log Level field
    pub log_level: String,
    /// Config Path field
    pub config_path: Option<String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: SafeEnv::get_or_default("SONGBIRD_ENV", "development"),
            debug: SafeEnv::get_required("SONGBIRD_DEBUG").is_ok(),
            log_level: SafeEnv::get_or_default("SONGBIRD_LOG_LEVEL", "info"),
            config_path: SafeEnv::get_required("SONGBIRD_CONFIG_PATH").ok(),
        }
    }
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalObservabilityConfig {
    /// Metrics Enabled field
    pub metrics_enabled: bool,
    /// Tracing Enabled field
    pub tracing_enabled: bool,
    /// Health Check Enabled field
    pub health_check_enabled: bool,
    /// Metrics Port field
    pub metrics_port: u16,
}

impl Default for CanonicalObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            health_check_enabled: true,
            metrics_port: SafeEnv::get_port("SONGBIRD_METRICS_PORT", 9090),
        }
    }
}

/// Type alias for backwards compatibility
pub type SongbirdConfig = UnifiedCoreConfig;

impl UnifiedCoreConfig {
    /// Create configuration from environment variables
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            service: ServiceConfig::default(),
            environment: EnvironmentConfig::default(),
            observability: CanonicalObservabilityConfig::default(),
            extensions: HashMap::new(),
        }
    }

    /// Check if running in production environment
    #[must_use]
    pub fn is_production(&self) -> bool {
        self.environment.environment == "production"
    }
}

/// Get unified configuration from environment
#[must_use]
pub fn get_unified_config() -> Result<UnifiedCoreConfig, String> {
    Ok(UnifiedCoreConfig::from_env())
}
