// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.system.environment.is_empty(), "System environment cannot be empty");
        anyhow::ensure!(!self.system.system_id.is_empty(), "System ID cannot be empty");
        anyhow::ensure!(
            self.network.ports.orchestrator != 0,
            "Network orchestrator port must be greater than 0"
        );
        Ok(())
    }

    /// Get bind address from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn get_bind_address_from_env(
        &self,
        env: &std::collections::HashMap<String, String>,
    ) -> String {
        env.get("SONGBIRD_BIND_ADDRESS").cloned().unwrap_or_else(|| {
            if self.is_production() {
                crate::constants::PRODUCTION_BIND_ADDRESS.to_string()
            } else {
                crate::constants::DEVELOPMENT_BIND_ADDRESS.to_string()
            }
        })
    }

    /// Get bind address based on environment
    #[must_use]
    pub fn get_bind_address(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", {
            if self.is_production() {
                crate::constants::PRODUCTION_BIND_ADDRESS
            } else {
                crate::constants::DEVELOPMENT_BIND_ADDRESS
            }
        })
    }

    /// Get data directory from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn get_data_dir_from_env(&self, env: &std::collections::HashMap<String, String>) -> String {
        env.get("SONGBIRD_DATA_DIR").cloned().unwrap_or_else(|| {
            if self.is_production() {
                String::from("/var/lib/songbird")
            } else {
                let home = env.get("HOME").map_or("/tmp", String::as_str);
                format!("{home}/.local/share/songbird")
            }
        })
    }

    /// Get data directory path
    #[must_use]
    pub fn get_data_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_DATA_DIR", {
            if self.is_production() {
                String::from("/var/lib/songbird")
            } else {
                format!(
                    "{home}/.local/share/songbird",
                    home = SafeEnv::get_or_default("HOME", "/tmp")
                )
            }
        })
    }

    /// Get config directory from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn get_config_dir_from_env(
        &self,
        env: &std::collections::HashMap<String, String>,
    ) -> String {
        env.get("SONGBIRD_CONFIG_DIR").cloned().unwrap_or_else(|| {
            if self.is_production() {
                String::from("/etc/songbird")
            } else {
                let home = env.get("HOME").map_or("/tmp", String::as_str);
                format!("{home}/.config/songbird")
            }
        })
    }

    /// Get config directory path
    #[must_use]
    pub fn get_config_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_CONFIG_DIR", {
            if self.is_production() {
                String::from("/etc/songbird")
            } else {
                format!("{home}/.config/songbird", home = SafeEnv::get_or_default("HOME", "/tmp"))
            }
        })
    }

    /// Get cache directory from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn get_cache_dir_from_env(
        &self,
        env: &std::collections::HashMap<String, String>,
    ) -> String {
        env.get("SONGBIRD_CACHE_DIR").cloned().unwrap_or_else(|| {
            if self.is_production() {
                String::from("/var/cache/songbird")
            } else {
                let home = env.get("HOME").map_or("/tmp", String::as_str);
                format!("{home}/.cache/songbird")
            }
        })
    }

    /// Get cache directory path
    #[must_use]
    pub fn get_cache_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_CACHE_DIR", {
            if self.is_production() {
                String::from("/var/cache/songbird")
            } else {
                format!("{home}/.cache/songbird", home = SafeEnv::get_or_default("HOME", "/tmp"))
            }
        })
    }

    /// Get log directory from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn get_log_dir_from_env(&self, env: &std::collections::HashMap<String, String>) -> String {
        env.get("SONGBIRD_LOG_DIR").cloned().unwrap_or_else(|| {
            if self.is_production() {
                String::from("/var/log/songbird")
            } else {
                let home = env.get("HOME").map_or("/tmp", String::as_str);
                format!("{home}/.local/share/songbird/logs")
            }
        })
    }

    /// Get log directory path
    #[must_use]
    pub fn get_log_dir(&self) -> String {
        SafeEnv::get_or_default("SONGBIRD_LOG_DIR", {
            if self.is_production() {
                String::from("/var/log/songbird")
            } else {
                format!(
                    "{home}/.local/share/songbird/logs",
                    home = SafeEnv::get_or_default("HOME", "/tmp")
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

    /// Check if running in test mode from environment map (for testing - concurrent safe)
    #[must_use]
    pub fn is_test_from_env(env: &std::collections::HashMap<String, String>) -> bool {
        env.get("SONGBIRD_ENV").map(String::as_str) == Some("testing")
            || env.get("NODE_ENV").map(String::as_str) == Some("test")
            || env.contains_key("CI")
    }

    /// Check if running in test mode
    #[must_use]
    pub fn is_test() -> bool {
        SafeEnv::get_or_default("SONGBIRD_ENV", "") == "testing"
            || SafeEnv::get_or_default("NODE_ENV", "") == "test"
            || SafeEnv::get_required("CI").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;

    fn roundtrip<T>(v: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let a: Value = serde_json::to_value(v).expect("serialize");
        let back: T = serde_json::from_value(a.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&back).expect("serialize again"), a);
    }

    #[test]
    fn unified_songbird_config_default() {
        let c = UnifiedSongbirdConfig::default();
        assert!(!c.system.system_id.is_empty());
        assert!(!c.system.environment.is_empty());
        assert!(c.network.ports.orchestrator > 0);
    }

    #[test]
    fn unified_new_matches_default() {
        let a = UnifiedSongbirdConfig::new("h", 1, "p");
        let b = UnifiedSongbirdConfig::default();
        assert_eq!(serde_json::to_value(&a).unwrap(), serde_json::to_value(&b).unwrap());
    }

    #[test]
    fn validate_ok_for_default() {
        assert!(UnifiedSongbirdConfig::default().validate().is_ok());
    }

    #[test]
    fn validate_errors_on_empty_environment() {
        let mut c = UnifiedSongbirdConfig::default();
        c.system.environment.clear();
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("System environment cannot be empty"));
    }

    #[test]
    fn validate_errors_on_empty_system_id() {
        let mut c = UnifiedSongbirdConfig::default();
        c.system.system_id.clear();
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("System ID cannot be empty"));
    }

    #[test]
    fn validate_errors_on_zero_orchestrator_port() {
        let mut c = UnifiedSongbirdConfig::default();
        c.network.ports.orchestrator = 0;
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("Network orchestrator port must be greater than 0"));
    }

    #[test]
    fn get_bind_address_from_env_override() {
        let c = UnifiedSongbirdConfig::default();
        let mut env = HashMap::new();
        env.insert(String::from("SONGBIRD_BIND_ADDRESS"), String::from("10.0.0.1"));
        assert_eq!(c.get_bind_address_from_env(&env), "10.0.0.1");
    }

    #[test]
    fn get_data_dir_from_env_non_prod_uses_home() {
        let c = UnifiedSongbirdConfig::default();
        let mut env = HashMap::new();
        env.insert(String::from("HOME"), String::from("/home/u"));
        assert_eq!(c.get_data_dir_from_env(&env), "/home/u/.local/share/songbird");
    }

    #[test]
    fn get_config_cache_log_dirs_from_env_non_prod() {
        let c = UnifiedSongbirdConfig::default();
        let mut env = HashMap::new();
        env.insert(String::from("HOME"), String::from("/home/u"));
        assert_eq!(c.get_config_dir_from_env(&env), "/home/u/.config/songbird");
        assert_eq!(c.get_cache_dir_from_env(&env), "/home/u/.cache/songbird");
        assert_eq!(c.get_log_dir_from_env(&env), "/home/u/.local/share/songbird/logs");
    }

    #[test]
    fn is_test_from_env_detects_ci() {
        let mut env = HashMap::new();
        env.insert(String::from("CI"), String::from("1"));
        assert!(UnifiedSongbirdConfig::is_test_from_env(&env));
    }

    #[test]
    fn serde_roundtrip_unified_songbird_config() {
        roundtrip(&UnifiedSongbirdConfig::default());
    }
}
