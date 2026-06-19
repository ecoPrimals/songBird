// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core System /// Configuration capability Configuration
//!
//! This module contains the fundamental system-level configuration
//! that applies across the entire Songbird ecosystem.

use crate::SafeEnv;
use serde::{Deserialize, Serialize};

/// **CANONICAL**: Core system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSystemConfig {
    /// Environment (development, staging, production)
    /// Environment field
    pub environment: String,
    /// System identifier
    pub system_id: String,
    /// Instance identifier
    /// Instance Id field
    pub instance_id: String,
    /// System version;
    /// Version string
    pub version: String,
}

impl Default for CanonicalSystemConfig {
    fn default() -> Self {
        Self {
            environment: SafeEnv::get_or_default("SONGBIRD_ENVIRONMENT", "development"),
            system_id: SafeEnv::get_or_default("SONGBIRD_SYSTEM_ID", "songbird-default"),
            instance_id: SafeEnv::get_or_default("SONGBIRD_INSTANCE_ID", "default-instance"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SongbirdError;

    #[test]
    fn test_default_system_config() {
        let config = CanonicalSystemConfig::default();
        assert_eq!(config.environment, "development");
        assert_eq!(config.system_id, "songbird-default");
        assert_eq!(config.instance_id, "default-instance");
        assert!(!config.version.is_empty());
    }

    #[test]
    fn test_custom_system_config() {
        let config = CanonicalSystemConfig {
            environment: String::from("production"),
            system_id: String::from("songbird-prod-1"),
            instance_id: String::from("instance-42"),
            version: String::from("1.0.0"),
        };
        assert_eq!(config.environment, "production");
        assert_eq!(config.system_id, "songbird-prod-1");
        assert_eq!(config.instance_id, "instance-42");
        assert_eq!(config.version, "1.0.0");
    }

    #[test]
    fn test_system_config_environments() {
        let dev = CanonicalSystemConfig {
            environment: String::from("development"),
            system_id: String::from("test"),
            instance_id: String::from("test"),
            version: String::from("0.1.0"),
        };
        assert_eq!(dev.environment, "development");

        let staging = CanonicalSystemConfig {
            environment: String::from("staging"),
            system_id: String::from("test"),
            instance_id: String::from("test"),
            version: String::from("0.1.0"),
        };
        assert_eq!(staging.environment, "staging");

        let prod = CanonicalSystemConfig {
            environment: String::from("production"),
            system_id: String::from("test"),
            instance_id: String::from("test"),
            version: String::from("0.1.0"),
        };
        assert_eq!(prod.environment, "production");
    }

    #[test]
    fn test_system_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = CanonicalSystemConfig {
            environment: String::from("production"),
            system_id: String::from("songbird-1"),
            instance_id: String::from("inst-1"),
            version: String::from("2.0.0"),
        };

        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some(String::from("JSON")),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        assert!(json.contains("production"));
        assert!(json.contains("songbird-1"));
        assert!(json.contains("inst-1"));
        assert!(json.contains("2.0.0"));
        Ok(())
    }

    #[test]
    fn test_system_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{
            "environment": "staging",
            "system_id": "songbird-staging",
            "instance_id": "staging-1",
            "version": "1.5.0"
        }"#;

        let config: CanonicalSystemConfig =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some(String::from("JSON")),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(config.environment, "staging");
        assert_eq!(config.system_id, "songbird-staging");
        assert_eq!(config.instance_id, "staging-1");
        assert_eq!(config.version, "1.5.0");
        Ok(())
    }

    #[test]
    fn test_system_config_clone() {
        let config1 = CanonicalSystemConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.environment, config2.environment);
        assert_eq!(config1.system_id, config2.system_id);
        assert_eq!(config1.instance_id, config2.instance_id);
        assert_eq!(config1.version, config2.version);
    }

    #[test]
    fn test_system_config_debug() {
        let config = CanonicalSystemConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CanonicalSystemConfig"));
        assert!(debug_str.contains("environment"));
        assert!(debug_str.contains("system_id"));
    }

    #[test]
    fn test_version_not_empty() {
        let config = CanonicalSystemConfig::default();
        assert!(!config.version.is_empty());
        assert!(config.version.contains('.'));
    }
}
