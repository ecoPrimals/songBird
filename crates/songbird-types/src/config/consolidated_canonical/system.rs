// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # System Configuration Module
//!
//! **CANONICAL SYSTEM CONFIGURATION** ✅
//!
//! This module provides system-wide configuration structures for the Songbird ecosystem.

use crate::primal_names::SELF_NAME;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// SYSTEM CONFIGURATION
// ============================================================================

/// **CANONICAL**: System-wide configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSystemConfig {
    /// Environment (development, staging, production,
    pub environment: String,

    /// System identifier
    pub system_id: String,

    /// Application name
    pub app_name: String,

    /// Application version
    pub version: String,

    /// Instance identifier
    pub instance_id: String,

    /// Data directory
    pub data_dir: String,

    /// Config directory
    pub config_dir: String,

    /// Cache directory
    pub cache_dir: String,

    /// Log directory
    pub log_dir: String,

    /// Temporary directory
    pub temp_dir: String,

    /// Logging configuration
    pub logging: CanonicalLoggingConfig,

    /// Resource limits and management
    pub resources: CanonicalResourceConfig,

    /// Graceful shutdown configuration
    pub shutdown: CanonicalShutdownConfig,
}

/// **CANONICAL**: Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLoggingConfig {
    /// Log level (trace, debug, info, warn, error,
    pub level: String,

    /// Log format (json, pretty, compact,
    pub format: String,

    /// Log output destinations
    pub outputs: Vec<CanonicalLogOutput>,

    /// Log rotation settings
    pub rotation: Option<CanonicalLogRotation>,

    /// Structured logging fields
    pub structured_fields: HashMap<String, String>,
}

/// **CANONICAL**: Log output destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLogOutput {
    /// Output type (stdout, stderr, file, syslog,
    pub output_type: String,

    /// Target (file path for file output, etc.,
    pub target: Option<String>,

    /// Output-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// **CANONICAL**: Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLogRotation {
    /// Maximum file size before rotation
    pub max_size_mb: u64,

    /// Maximum number of log files to keep
    pub max_files: u32,

    /// Rotation frequency (daily, weekly, monthly,
    pub frequency: String,
}

/// **CANONICAL**: Resource limits and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalResourceConfig {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,

    /// CPU limit as percentage (0-100,
    pub cpu_limit_percent: f64,

    /// Maximum file descriptors
    pub max_file_descriptors: u32,

    /// Resource monitoring interval
    pub monitoring_interval: Duration,

    /// Resource cleanup settings
    pub cleanup: CanonicalResourceCleanup,
}

/// **CANONICAL**: Resource cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalResourceCleanup {
    /// Enable automatic cleanup
    pub enabled: bool,

    /// Cleanup interval
    pub interval: Duration,

    /// Memory threshold for cleanup trigger
    pub memory_threshold_percent: f64,

    /// Resource age threshold
    pub max_resource_age: Duration,
}

/// **CANONICAL**: Graceful shutdown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalShutdownConfig {
    /// Graceful shutdown timeout
    pub timeout: Duration,

    /// Force shutdown after timeout
    pub force_after_timeout: bool,

    /// Shutdown hooks to execute
    pub hooks: Vec<String>,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for CanonicalSystemConfig {
    fn default() -> Self {
        let home = songbird_process_env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Self {
            environment: "development".to_string(),
            system_id: "songbird-1".to_string(),
            app_name: SELF_NAME.to_string(),
            version: "0.1.0".to_string(),
            instance_id: format!("{}-{}", SELF_NAME, std::process::id()),
            data_dir: format!("{home}/.local/share/songbird"),
            config_dir: format!("{home}/.config/songbird"),
            cache_dir: format!("{home}/.cache/songbird"),
            log_dir: format!("{home}/.local/share/songbird/logs"),
            temp_dir: "/tmp/songbird".to_string(),
            logging: CanonicalLoggingConfig::default(),
            resources: CanonicalResourceConfig::default(),
            shutdown: CanonicalShutdownConfig::default(),
        }
    }
}

impl Default for CanonicalLoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            outputs: vec![CanonicalLogOutput::default()],
            rotation: None,
            structured_fields: HashMap::new(),
        }
    }
}

impl Default for CanonicalLogOutput {
    fn default() -> Self {
        Self {
            output_type: "stdout".to_string(),
            target: None,
            config: HashMap::new(),
        }
    }
}

impl Default for CanonicalResourceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1_073_741_824, // 1GB
            cpu_limit_percent: 80.0,
            max_file_descriptors: 1024,
            monitoring_interval: Duration::from_secs(60),
            cleanup: CanonicalResourceCleanup::default(),
        }
    }
}

impl Default for CanonicalResourceCleanup {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(300), // 5 minutes
            memory_threshold_percent: 85.0,
            max_resource_age: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl Default for CanonicalShutdownConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            force_after_timeout: true,
            hooks: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), json);
    }

    #[test]
    fn default_canonical_system_config() {
        let c = CanonicalSystemConfig::default();
        assert_eq!(c.environment, "development");
        assert_eq!(c.version, "0.1.0");
        assert!(c.data_dir.contains("songbird"));
        assert_eq!(c.logging.level, "info");
        assert_eq!(c.resources.max_memory_bytes, 1_073_741_824);
    }

    #[test]
    fn default_logging_resource_cleanup_shutdown() {
        assert_eq!(CanonicalLoggingConfig::default().format, "json");
        assert_eq!(CanonicalResourceConfig::default().cpu_limit_percent, 80.0);
        assert!(CanonicalResourceCleanup::default().enabled);
        assert_eq!(CanonicalShutdownConfig::default().timeout, Duration::from_secs(30));
    }

    #[test]
    fn roundtrip_canonical_system_config() {
        assert_json_roundtrip(&CanonicalSystemConfig::default());
    }

    #[test]
    fn roundtrip_logging_log_output_rotation() {
        assert_json_roundtrip(&CanonicalLoggingConfig::default());
        assert_json_roundtrip(&CanonicalLogOutput::default());
        let rot = CanonicalLogRotation {
            max_size_mb: u64::MAX,
            max_files: u32::MAX,
            frequency: "daily".into(),
        };
        assert_json_roundtrip(&rot);
    }

    #[test]
    fn roundtrip_resource_config_and_cleanup() {
        assert_json_roundtrip(&CanonicalResourceConfig::default());
        assert_json_roundtrip(&CanonicalResourceCleanup::default());
    }

    #[test]
    fn roundtrip_shutdown_config() {
        let mut s = CanonicalShutdownConfig::default();
        s.hooks.push("hook1".into());
        assert_json_roundtrip(&s);
    }
}
