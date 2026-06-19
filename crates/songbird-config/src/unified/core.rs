// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// Core unified configuration types
//
// This module provides the foundational types and structures for the unified
// configuration system, replacing fragmented configuration patterns.

use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use songbird_types::primal_names::SELF_NAME;
use std::collections::HashMap;

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
            name: SELF_NAME.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: format!("instance-{}", std::process::id()),
            tags: vec![SELF_NAME.to_owned()],
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
            metrics_port: crate::defaults::ports::metrics_port(),
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
pub fn get_unified_config() -> UnifiedCoreConfig {
    UnifiedCoreConfig::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.name, "songbird");
        assert!(!config.version.is_empty());
        assert!(config.instance_id.starts_with("instance-"));
        assert_eq!(config.tags.len(), 1);
        assert_eq!(config.tags[0], "songbird");
    }

    #[test]
    fn test_environment_config_default() {
        let config = EnvironmentConfig::default();
        assert!(!config.environment.is_empty());
        assert!(!config.log_level.is_empty());
        // Debug format should work
        assert!(format!("{config:?}").contains("environment"));
    }

    #[test]
    fn test_observability_config_default() {
        let config = CanonicalObservabilityConfig::default();
        assert!(config.metrics_enabled);
        assert!(config.tracing_enabled);
        assert!(config.health_check_enabled);
        assert!(config.metrics_port > 0);
    }

    #[test]
    fn test_unified_core_config_from_env() {
        let config = UnifiedCoreConfig::from_env();
        assert_eq!(config.service.name, "songbird");
        assert!(config.observability.metrics_enabled);
        assert!(config.extensions.is_empty());
    }

    #[test]
    fn test_unified_core_config_is_production() {
        let mut config = UnifiedCoreConfig::from_env();

        // Test development
        config.environment.environment = String::from("development");
        assert!(!config.is_production());

        // Test production
        config.environment.environment = String::from("production");
        assert!(config.is_production());

        // Test staging (not production)
        config.environment.environment = String::from("staging");
        assert!(!config.is_production());
    }

    #[test]
    fn test_get_unified_config() {
        let config = get_unified_config();
        assert_eq!(config.service.name, "songbird");
    }

    #[test]
    fn test_unified_core_config_extensions() {
        let mut config = UnifiedCoreConfig::from_env();

        // Test empty extensions
        assert!(config.extensions.is_empty());

        // Add extension
        config.extensions.insert(String::from("custom_key"), serde_json::json!({"value": 42}));
        assert_eq!(config.extensions.len(), 1);
        assert!(config.extensions.contains_key("custom_key"));
    }

    #[test]
    fn test_serde_unified_core_config() {
        let config = UnifiedCoreConfig::from_env();

        // Test serialization
        let json = serde_json::to_string(&config).expect("should serialize");
        assert!(!json.is_empty());
        assert!(json.contains("songbird"));

        // Test deserialization
        let deserialized: UnifiedCoreConfig =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.service.name, config.service.name);
        assert_eq!(
            deserialized.observability.metrics_enabled,
            config.observability.metrics_enabled
        );
    }

    #[test]
    fn test_songbird_config_type_alias() {
        // Test that type alias works
        let _config: SongbirdConfig = UnifiedCoreConfig::from_env();
        // If this compiles, the type alias is correct
    }

    #[test]
    fn test_service_config_clone() {
        let config = ServiceConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.name, config.name);
        assert_eq!(cloned.version, config.version);
        assert_eq!(cloned.instance_id, config.instance_id);
    }
}
