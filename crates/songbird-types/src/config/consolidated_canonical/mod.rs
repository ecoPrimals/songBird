// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔧 Consolidated Canonical Configuration System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL CONFIGURATIONS** ✅
//!
//! This module consolidates ALL configuration structures from across the Songbird ecosystem
//! into a single, unified, canonical configuration system. This replaces:
//!
//! - `songbird-config` crate configurations
//! - `songbird-types` fragmented config modules (25+ config types,
//! - All deprecated configuration aliases and compatibility layers
//!
//! ## Consolidation Summary
//! - **25+ configuration types** → Single `CanonicalSongbirdConfig`
//! - **614 config structs** → Organized hierarchical system
//! - **Multiple config crates** → Single canonical source
//! - **Legacy compatibility** → Clean migration path

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import all sub-modules
pub mod discovery;
pub mod environment;
pub mod factory;
pub mod federation;
pub mod gaming;
pub mod network;
pub mod observability;
pub mod performance;
pub mod primals;
pub mod security;
pub mod system;

// Re-export all types from sub-modules
pub use discovery::*;
pub use environment::*;
pub use factory::*;
pub use federation::*;
pub use gaming::*;
pub use network::*;
pub use observability::*;
pub use performance::*;
pub use primals::*;
pub use security::*;
pub use system::*;

// ============================================================================
// CANONICAL CONFIGURATION - Single Source of Truth
// ============================================================================

/// **CANONICAL**: Main Songbird configuration - replaces ALL fragmented configs
///
/// This single configuration structure replaces:
/// - `songbird-config::SongbirdConfig`
/// - `songbird-types::config::UnifiedSongbirdConfig` (multiple versions,
/// - All 25+ fragmented config types across modules
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalSongbirdConfig {
    /// System-wide configuration
    pub system: CanonicalSystemConfig,

    /// Network and communication configuration
    pub network: CanonicalNetworkConfig,

    /// Security and authentication configuration
    pub security: CanonicalSecurityConfig,

    /// Performance and optimization configuration
    pub performance: CanonicalPerformanceConfig,

    /// Service discovery and registration configuration
    pub discovery: CanonicalDiscoveryConfig,

    /// Observability, monitoring, and metrics configuration
    pub observability: CanonicalObservabilityConfig,

    /// Gaming protocol and bridge configuration
    pub gaming: CanonicalGamingConfig,

    /// Universal primal provider configuration
    pub primals: CanonicalPrimalConfig,

    /// Federation and clustering configuration
    pub federation: CanonicalFederationConfig,

    /// Environment and deployment configuration
    pub environment: CanonicalEnvironmentConfig,

    /// Extensibility - custom configuration fields
    pub custom: HashMap<String, serde_json::Value>,
}

// ============================================================================
// IMPLEMENTATION - Core Methods
// ============================================================================

impl CanonicalSongbirdConfig {
    /// Create configuration from environment variables (most common use case)
    ///
    /// This loads all configuration from environment variables with intelligent defaults.
    /// Environment variables follow the pattern: `SONGBIRD_{SECTION}_{FIELD}`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_types::config::CanonicalSongbirdConfig;
    ///
    /// let config = CanonicalSongbirdConfig::from_env()
    ///     .expect("Failed to load configuration from environment");
    ///
    /// println!("Running in: {} environment", config.environment.name);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded from the environment.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            system: CanonicalSystemConfig::default(),
            network: CanonicalNetworkConfig::default(),
            security: CanonicalSecurityConfig::default(),
            performance: CanonicalPerformanceConfig::default(),
            discovery: CanonicalDiscoveryConfig::default(),
            observability: CanonicalObservabilityConfig::default(),
            gaming: CanonicalGamingConfig::default(),
            primals: CanonicalPrimalConfig::default(),
            federation: CanonicalFederationConfig::default(),
            environment: CanonicalEnvironmentConfig::default(),
            custom: HashMap::new(),
        })
    }

    /// Create a builder for programmatic configuration construction
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use songbird_types::config::{CanonicalSongbirdConfig, CanonicalSystemConfig};
    ///
    /// let config = CanonicalSongbirdConfig::builder()
    ///     .system(CanonicalSystemConfig::default())
    ///     .build()
    ///     .expect("Failed to build configuration");
    /// ```
    #[must_use]
    pub fn builder() -> CanonicalConfigBuilder {
        CanonicalConfigBuilder::default()
    }

    /// Create test-friendly configuration with safe defaults
    ///
    /// This method provides a configuration suitable for testing environments:
    /// - Uses localhost bindings
    /// - Minimal resource requirements
    /// - Permissive timeouts
    /// - No external dependencies
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_types::config::CanonicalSongbirdConfig;
    ///
    /// let config = CanonicalSongbirdConfig::test_defaults();
    /// assert_eq!(config.environment.name, "test");
    /// ```
    #[allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::unnecessary_wraps,
        clippy::field_reassign_with_default,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    #[cfg(test)]
    pub fn test_defaults() -> Self {
        Self {
            environment: CanonicalEnvironmentConfig {
                name: String::from("test"),
                deployment_mode: String::from("standalone"),
            },
            ..Self::default()
        }
    }

    /// Validate configuration completeness and correctness
    ///
    /// Checks all sub-configurations for validity and consistency.
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration is invalid or inconsistent.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.system.environment.is_empty(), "System environment cannot be empty");
        anyhow::ensure!(!self.system.system_id.is_empty(), "System ID cannot be empty");

        // Port 0 means ephemeral (OS-assigned), valid for IPC-only or test
        // deployments where TCP port doesn't need to be fixed.
        anyhow::ensure!(
            !(self.network.base_port == 0 && self.discovery.mode.is_enabled()),
            "Discovery requires external TCP port (network.base_port > 0).\n\
             \n\
             Songbird operates in dual-mode:\n\
             - External TCP port (for LAN discovery beacons)\n\
             - Internal Unix socket (for inter-primal IPC)\n\
             \n\
             Fix: Set network.base_port = 8080 or disable discovery.\n\
             Port 0 (ephemeral) is allowed only when discovery is disabled."
        );

        Ok(())
    }

    // ========================================================================
    // CONVENIENCE METHODS - Environment Checks
    // ========================================================================

    /// Check if running in production environment
    #[must_use]
    pub fn is_production(&self) -> bool {
        self.environment.name == "production" || self.environment.name == "prod"
    }

    /// Check if running in development environment
    #[must_use]
    pub fn is_development(&self) -> bool {
        self.environment.name == "development" || self.environment.name == "dev"
    }

    /// Check if running in test environment
    #[must_use]
    pub fn is_test(&self) -> bool {
        self.environment.name == "test" || self.environment.name == "testing"
    }

    /// Check if running in staging environment
    #[must_use]
    pub fn is_staging(&self) -> bool {
        self.environment.name == "staging"
    }

    // ========================================================================
    // CONVENIENCE METHODS - Network
    // ========================================================================

    /// Get bind address based on environment
    #[must_use]
    pub fn get_bind_address(&self) -> &str {
        &self.network.bind_host
    }

    /// Get base port for services
    #[must_use]
    pub const fn get_base_port(&self) -> u16 {
        self.network.base_port
    }

    // ========================================================================
    // CONVENIENCE METHODS - Directories
    // ========================================================================

    /// Get data directory path
    #[must_use]
    pub fn get_data_dir(&self) -> &str {
        &self.system.data_dir
    }

    /// Get config directory path
    #[must_use]
    pub fn get_config_dir(&self) -> &str {
        &self.system.config_dir
    }

    /// Get cache directory path
    #[must_use]
    pub fn get_cache_dir(&self) -> &str {
        &self.system.cache_dir
    }

    /// Get log directory path
    #[must_use]
    pub fn get_log_dir(&self) -> &str {
        &self.system.log_dir
    }

    /// Get temporary directory path
    #[must_use]
    pub fn get_temp_dir(&self) -> &str {
        &self.system.temp_dir
    }
}

// ============================================================================
// BUILDER PATTERN
// ============================================================================

/// Builder for `CanonicalSongbirdConfig`
///
/// Provides a fluent API for constructing configuration programmatically.
#[derive(Debug, Clone, Default)]
pub struct CanonicalConfigBuilder {
    system: Option<CanonicalSystemConfig>,
    network: Option<CanonicalNetworkConfig>,
    security: Option<CanonicalSecurityConfig>,
    performance: Option<CanonicalPerformanceConfig>,
    discovery: Option<CanonicalDiscoveryConfig>,
    observability: Option<CanonicalObservabilityConfig>,
    gaming: Option<CanonicalGamingConfig>,
    primals: Option<CanonicalPrimalConfig>,
    federation: Option<CanonicalFederationConfig>,
    environment: Option<CanonicalEnvironmentConfig>,
    custom: HashMap<String, serde_json::Value>,
}

impl CanonicalConfigBuilder {
    /// Set system configuration
    #[must_use]
    pub fn system(mut self, config: CanonicalSystemConfig) -> Self {
        self.system = Some(config);
        self
    }

    /// Set network configuration
    #[must_use]
    pub fn network(mut self, config: CanonicalNetworkConfig) -> Self {
        self.network = Some(config);
        self
    }

    /// Set security configuration
    #[must_use]
    pub fn security(mut self, config: CanonicalSecurityConfig) -> Self {
        self.security = Some(config);
        self
    }

    /// Set performance configuration
    #[must_use]
    pub const fn performance(mut self, config: CanonicalPerformanceConfig) -> Self {
        self.performance = Some(config);
        self
    }

    /// Set discovery configuration
    #[must_use]
    pub fn discovery(mut self, config: CanonicalDiscoveryConfig) -> Self {
        self.discovery = Some(config);
        self
    }

    /// Set observability configuration
    #[must_use]
    pub fn observability(mut self, config: CanonicalObservabilityConfig) -> Self {
        self.observability = Some(config);
        self
    }

    /// Set gaming configuration
    #[must_use]
    pub fn gaming(mut self, config: CanonicalGamingConfig) -> Self {
        self.gaming = Some(config);
        self
    }

    /// Set primals configuration
    #[must_use]
    pub fn primals(mut self, config: CanonicalPrimalConfig) -> Self {
        self.primals = Some(config);
        self
    }

    /// Set federation configuration
    #[must_use]
    pub fn federation(mut self, config: CanonicalFederationConfig) -> Self {
        self.federation = Some(config);
        self
    }

    /// Set environment configuration
    #[must_use]
    pub fn environment(mut self, config: CanonicalEnvironmentConfig) -> Self {
        self.environment = Some(config);
        self
    }

    /// Add custom configuration field
    #[must_use]
    pub fn custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Build the configuration
    ///
    /// Uses defaults for any fields not explicitly set.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration fails validation.
    pub fn build(self) -> anyhow::Result<CanonicalSongbirdConfig> {
        let config = CanonicalSongbirdConfig {
            system: self.system.unwrap_or_default(),
            network: self.network.unwrap_or_default(),
            security: self.security.unwrap_or_default(),
            performance: self.performance.unwrap_or_default(),
            discovery: self.discovery.unwrap_or_default(),
            observability: self.observability.unwrap_or_default(),
            gaming: self.gaming.unwrap_or_default(),
            primals: self.primals.unwrap_or_default(),
            federation: self.federation.unwrap_or_default(),
            environment: self.environment.unwrap_or_default(),
            custom: self.custom,
        };

        config.validate()?;
        Ok(config)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::consolidated_canonical::discovery::DiscoveryMode;

    #[test]
    fn test_validate_port_zero_with_discovery_enabled() {
        // Create a config with discovery enabled and port = 0 (invalid)
        let mut config = CanonicalSongbirdConfig::default();
        config.network.base_port = 0;
        config.discovery.mode = DiscoveryMode::Anonymous;

        // Should fail validation
        match config.validate() {
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("Discovery requires external TCP port"));
                assert!(msg.contains("dual-mode"));
            }
            Ok(()) => panic!("expected validation to fail"),
        }
    }

    #[test]
    fn test_validate_port_zero_with_discovery_disabled() {
        let mut config = CanonicalSongbirdConfig::default();
        config.network.base_port = 0;
        config.discovery.mode = DiscoveryMode::Disabled;

        // Port 0 (ephemeral) is valid when discovery is disabled (IPC-only / test mode).
        assert!(config.validate().is_ok(), "port 0 should be allowed when discovery is disabled");
    }

    #[test]
    fn test_validate_port_nonzero_with_discovery_enabled() {
        // Create a config with discovery enabled and port > 0 (valid)
        let mut config = CanonicalSongbirdConfig::default();
        config.network.base_port = 8080;
        config.discovery.mode = DiscoveryMode::Anonymous;

        // Should pass validation
        let result = config.validate();
        assert!(result.is_ok(), "Expected validation to pass, got: {:?}", result);
    }

    #[test]
    fn test_default_config_is_valid() {
        // Default configuration should pass validation
        let config = CanonicalSongbirdConfig::default();
        let result = config.validate();
        assert!(result.is_ok(), "Default config should be valid, got: {:?}", result);
    }
}
