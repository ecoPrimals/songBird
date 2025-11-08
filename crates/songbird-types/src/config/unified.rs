//! Unified Configuration System
//!
//! **CANONICAL**: Single Source of Truth for all Songbird configuration
//!
//! # Future Direction
//!
//! This configuration structure is stable and actively used. For new projects,
//! consider `CanonicalSongbirdConfig` in `consolidated_canonical` module which provides
//! additional structure and organization. Both are fully supported.

use super::{
    ai_first::CanonicalAIFirstConfig, health::CanonicalHealthConfig,
    migration::CanonicalMigrationConfig, network::CanonicalNetworkConfig,
    orchestration::CanonicalOrchestrationConfig, performance::CanonicalPerformanceConfig,
    security::CanonicalSecurityConfig, system::CanonicalSystemConfig,
};
use crate::SafeEnv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// **CANONICAL**: Unified Songbird Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedSongbirdConfig {
    /// System configuration
    pub system: CanonicalSystemConfig,
    /// Network configuration
    pub network: CanonicalNetworkConfig,
    /// Security configuration
    pub security: CanonicalSecurityConfig,
    /// Performance configuration
    pub performance: CanonicalPerformanceConfig,
    /// Health monitoring configuration
    pub health: CanonicalHealthConfig,
    /// Orchestration configuration
    pub orchestration: CanonicalOrchestrationConfig,
    /// AI-First API configuration
    pub ai_first: CanonicalAIFirstConfig,
    /// Migration configuration
    pub migration: CanonicalMigrationConfig,
    /// Custom configuration fields
    pub custom: Option<HashMap<String, Value>>,
}

impl UnifiedSongbirdConfig {
    /// Create a new unified configuration with defaults
    #[must_use]
    pub fn new(_host: impl Into<String>, _port: u16, _protocol: impl Into<String>) -> Self {
        Self::default()
    }

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration values are invalid
    pub fn validate(&self) -> Result<(), String> {
        // Validate system configuration
        if self.system.environment.is_empty() {
            return Err("System environment cannot be empty".to_string());
        }
        if self.system.system_id.is_empty() {
            return Err("System ID cannot be empty".to_string());
        }

        // Validate network configuration
        if self.network.ports.orchestrator == 0 {
            return Err("Network orchestrator port must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get bind address based on environment
    #[must_use]
    pub fn get_bind_address(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", {
            if self.is_production() {
                "0.0.0.0"
            } else {
                "127.0.0.1"
            }
        })
    }

    /// Get data directory path
    #[must_use]
    pub fn get_data_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_DATA_DIR", {
            if self.is_production() {
                "/var/lib/songbird".to_string()
            } else {
                format!(
                    "{}/.local/share/songbird",
                    SafeEnv::get_or_default("HOME", "/tmp")
                )
            }
        })
    }

    /// Get config directory path
    #[must_use]
    pub fn get_config_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_CONFIG_DIR", {
            if self.is_production() {
                "/etc/songbird".to_string()
            } else {
                format!(
                    "{}/.config/songbird",
                    SafeEnv::get_or_default("HOME", "/tmp")
                )
            }
        })
    }

    /// Get cache directory path
    #[must_use]
    pub fn get_cache_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_CACHE_DIR", {
            if self.is_production() {
                "/var/cache/songbird".to_string()
            } else {
                format!(
                    "{}/.cache/songbird",
                    SafeEnv::get_or_default("HOME", "/tmp")
                )
            }
        })
    }

    /// Get log directory path
    #[must_use]
    pub fn get_log_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_LOG_DIR", {
            if self.is_production() {
                "/var/log/songbird".to_string()
            } else {
                format!(
                    "{}/.local/share/songbird/logs",
                    SafeEnv::get_or_default("HOME", "/tmp")
                )
            }
        })
    }

    /// Check if running in production mode
    #[must_use]
    pub fn is_production(&self) -> bool {
        self.system.environment == "production"
            || SafeEnv::get_or_default("SONGBIRD_ENV", "development") == "production"
            || SafeEnv::get_or_default("NODE_ENV", "development") == "production"
    }

    /// Check if running in development mode
    #[must_use]
    pub fn is_development(&self) -> bool {
        self.system.environment == "development"
            || SafeEnv::get_or_default("SONGBIRD_ENV", "") == "development"
            || SafeEnv::get_or_default("NODE_ENV", "") == "development"
    }

    /// Check if running in test mode
    #[must_use]
    pub fn is_test() -> bool {
        SafeEnv::get_or_default("SONGBIRD_ENV", "") == "testing"
            || SafeEnv::get_or_default("NODE_ENV", "") == "test"
            || SafeEnv::get_required("CI").is_ok()
    }
}
