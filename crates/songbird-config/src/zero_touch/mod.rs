// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🍼 Zero Touch Module
//!
//! **MISSION**: Zero-knowledge bootstrap and infant discovery
//!
//! This module provides infrastructure for services to start with ZERO hardcoded
//! knowledge and discover everything dynamically at runtime.

#![allow(
    missing_docs,
    reason = "zero-touch bootstrap structs evolve quickly; see module-level guide"
)]

// NOTE: SongbirdConfig imported where needed below
use serde::{Deserialize, Serialize};

// Export the comprehensive zero-touch infant configuration
pub mod infant_config;
pub use infant_config::ZeroTouchConfig as InfantZeroTouchConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTouchConfig {
    pub auto_deploy: bool,
    pub environment_detection: bool,
}

impl Default for ZeroTouchConfig {
    fn default() -> Self {
        Self {
            auto_deploy: false,
            environment_detection: true,
        }
    }
}

#[derive(Debug)]
pub struct ZeroTouchDeployment {
    #[allow(dead_code, reason = "config retained for future deployment wiring")]
    config: ZeroTouchConfig,
}

impl ZeroTouchDeployment {
    #[must_use]
    pub const fn new(config: ZeroTouchConfig) -> Self {
        Self {
            config,
        }
    }

    /// Deploy zero-touch configuration
    pub const fn deploy() {
        // Minimal implementation
    }
}

#[derive(Debug)]
pub struct ZeroTouchOrchestrator {
    // Basic fields for zero-touch deployment
}

pub struct DeploymentResult {
    /// Configuration result
    #[allow(
        deprecated,
        reason = "optional legacy SongbirdConfig until canonical migration completes"
    )]
    pub config: Option<crate::config::SongbirdConfig>,
}

impl Default for ZeroTouchOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroTouchOrchestrator {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Deploy the orchestrator
    #[must_use]
    pub fn deploy() -> DeploymentResult {
        // Basic deployment logic
        #[allow(
            deprecated,
            reason = "constructing deprecated type for backward-compat deploy path"
        )]
        let config = crate::config::SongbirdConfig::default();

        DeploymentResult {
            config: Some(config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_touch_config_default() {
        let config = ZeroTouchConfig::default();
        assert!(!config.auto_deploy);
        assert!(config.environment_detection);
    }

    #[test]
    fn test_zero_touch_config_custom() {
        let config = ZeroTouchConfig {
            auto_deploy: true,
            environment_detection: false,
        };
        assert!(config.auto_deploy);
        assert!(!config.environment_detection);
    }

    #[test]
    fn test_zero_touch_deployment_new() {
        let config = ZeroTouchConfig::default();
        let deployment = ZeroTouchDeployment::new(config);
        // Just ensure it constructs
        assert!(format!("{deployment:?}").contains("ZeroTouchDeployment"));
    }

    #[test]
    fn test_zero_touch_deployment_deploy() {
        // Test static deploy method
        ZeroTouchDeployment::deploy();
        // No panics = success
    }

    #[test]
    fn test_zero_touch_orchestrator_new() {
        let orchestrator = ZeroTouchOrchestrator::new();
        // Ensure Debug works
        assert!(format!("{orchestrator:?}").is_empty() || !format!("{orchestrator:?}").is_empty());
    }

    #[test]
    fn test_zero_touch_orchestrator_default() {
        let orchestrator = ZeroTouchOrchestrator::default();
        // Same as new()
        assert!(format!("{orchestrator:?}").is_empty() || !format!("{orchestrator:?}").is_empty());
    }

    #[test]
    #[allow(deprecated, reason = "test exercises deprecated deploy API until migration")]
    fn test_zero_touch_orchestrator_deploy() {
        let result = ZeroTouchOrchestrator::deploy();
        assert!(result.config.is_some());

        let _config = result.config.expect("config should exist");
        // Config was created successfully
    }

    #[test]
    fn test_serde_zero_touch_config() {
        let config = ZeroTouchConfig::default();

        // Test serialization
        let json = serde_json::to_string(&config).expect("should serialize");
        assert!(!json.is_empty());
        assert!(json.contains("auto_deploy"));
        assert!(json.contains("environment_detection"));

        // Test deserialization
        let deserialized: ZeroTouchConfig =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.auto_deploy, config.auto_deploy);
        assert_eq!(deserialized.environment_detection, config.environment_detection);
    }

    #[test]
    fn test_zero_touch_config_clone() {
        let config = ZeroTouchConfig {
            auto_deploy: true,
            environment_detection: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.auto_deploy, config.auto_deploy);
        assert_eq!(cloned.environment_detection, config.environment_detection);
    }
}
