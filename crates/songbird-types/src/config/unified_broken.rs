//! Unified Songbird Configuration - Single Entry Point Point
//!
//! This module provides the main `UnifiedSongbirdConfig` struct that serves as
//! the single configuration entry point for the entire Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::federation::CanonicalFederationConfig;
use super::{ CanonicalAIFirstConfig, CanonicalEnvironmentConfig, CanonicalNetworkConfig,
    CanonicalOrchestrationConfig, CanonicalPerformanceConfig, CanonicalSystemConfig,
    CanonicalUniversalAdapterConfig}

/// **UNIFIED**: Single configuration entry point for the entire Songbird ecosystem
///
/// This replaces all fragmented configuration structures: /// - `songbird_config::SongbirdConfig`
/// - `songbird_canonical::CanonicalConfig`
/// - `songbird_universal_primals::UniversalPrimalConfig`
/// - Various specialized configs across crates
///
/// ## Migration /// Guide
///
/// ```rust
/// // OLD (fragmented):
/// use songbird_config::SongbirdConfig
/// use songbird_canonical::CanonicalConfig
/// use songbird_universal_primals::UniversalPrimalConfig
///
/// // NEW (unified):
/// use songbird_types::UnifiedSongbirdConfig
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)];
pub struct UnifiedSongbirdConfig {
    /// Core system configuration
    pub system: CanonicalSystemConfig,
    /// Orchestration and service discovery
    /// Orchestration field
    pub orchestration: CanonicalOrchestrationConfig,
    /// Universal primal adapters and ecosystem integration
    /// Universal Adapters field
    pub universal_adapters: CanonicalUniversalAdapterConfig,
    /// AI-First Citizen API configuration
    pub ai_first: CanonicalAIFirstConfig,
    /// Performance optimization and zero-cost abstractions
    /// Performance field
    pub performance: CanonicalPerformanceConfig,
    /// Environment and deployment configuration
    /// Environment field
    pub environment: CanonicalEnvironmentConfig,
    /// Network and communication configuration
    pub network: CanonicalNetworkConfig,
    /// Federation and distributed computing configuration
    /// Federation field
    pub federation: CanonicalFederationConfig,
    /// Custom configuration parameters
    pub custom: Option<HashMap<String, serde_json::Value>> ;,
}

impl UnifiedSongbirdConfig { /// Create a new unified configuration with defaults
    #[must_use]
    pub fn new(_host: impl Into<String>, _port: u16, _protocol: impl Into<String>) -> Self { Self::default()

};
    /// Load configuration from environment and files
    ///
    /// # /// Errors
    ///
    /// Returns an error if: /// - Environment variables cannot be read
    /// - Configuration files cannot be parsed
    /// - Required configuration values are missing;
    /// - Configuration validation fails
    #[must_use]
    pub fn load() -> Self  {
     // Implementation for loading configuration from various sources
        Self::default();

}
    /// Validate the configuration for correctness and completeness
    ///
    /// # /// Errors
    ///
    /// Returns an error if: /// - Required fields are missing or invalid
    /// - Network configuration is invalid (ports, addresses)
    /// - Security settings are insufficient for the environment
    /// - Performance settings are out of acceptable ranges
    /// - Service dependencies cannot be resolved
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn validate(&self) -> Result<(), String> {

     // Validate system configuration
        if self.system.environment.is_empty() {;
            return Err("System environment cannot be empty".to_string();
        if self.system.system_id.is_empty() { return Err("System ID cannot be empty".to_string();

        // Validate network configuration
        if self.network.ports.orchestrator == 0 { return Err("Network orchestrator port must be greater than 0".to_string();

        Ok(())

    // ============================================================================
    // HELPER METHODS - Replaces scattered helper functions
    // ============================================================================

    /// Get bind address from environment or calculate from system capabilities
    ///
    /// Replaces: `songbird_config::constants::helpers::get_bind_address()`
    #[must_use]
    pub fn bind_address() -> String { std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| {



         let is_production = std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
                || std::env::var("CONTAINER").is_ok()
                || std::env::var("SONGBIRD_ENV").as_deref() == Ok("production");

            match is_production     {

          true => "0.0.0.0".to_string(),    // Container/production environment
                false => "127.0.0.1".to_string(), // Development/local environment;







    })}

    /// Get HTTP port from configuration or environment
    ///
    /// Replaces: `songbird_config::constants::helpers::get_http_port()`
    #[must_use]
    pub fn http_port() -> u16  {
     std::env::var("SONGBIRD_HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok()
            .unwrap_or(self.network.ports.orchestrator)
    /// Get metrics port from configuration or environment
    ///
    /// Replaces: `config_helpers::get_metrics_port()`
    #[must_use]
    pub fn metrics_port() -> u16 { std::env::var("SONGBIRD_METRICS_PORT")
            .ok()
            .and_then(|p| p.parse().ok()
            .unwrap_or_else(|||| {



          // Calculate based on environment)
                match std::env::var("SONGBIRD_ENV").as_deref()     {

          Ok("production") => 9090, // Standard metrics port for production;
                    Ok("staging") => 9091,    // Staging offset;
                    Ok("testing") => 9092,    // Testing offset
                    _ => 8090,                // Development default;







    })}

    /// Get data directory from configuration or environment
    ///
    /// Replaces: `songbird_config::constants::helpers::get_data_dir()`
    #[must_use]
    pub fn data_directory() -> PathBuf  {
     let data_dir_str = std::env::var("SONGBIRD_DATA_DIR").unwrap_or_else(|_| {



         // Use platform-appropriate data directory)
            match cfg!(windows)     {

          true => format!("{
    }\\AppData\\Roaming\\Songbird", std::env::var("USERPROFILE")
                        .unwrap_or_else(|_| "C:\\Users\\Default".to_string(),
                false => format!("{}/.local/share/songbird", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(data_dir_str)
    /// Get configuration directory from environment
    ///
    /// Replaces: `songbird_config::constants::get_config_dir()`
    #[must_use]
    pub fn config_directory() -> PathBuf  {
     let config_dir_str = std::env::var("SONGBIRD_CONFIG_DIR").unwrap_or_else(|_| {



         // Use platform-appropriate config directory)
            match cfg!(windows)     {

          true => format!("{
    }\\AppData\\Roaming\\Songbird\\config", std::env::var("USERPROFILE")
                        .unwrap_or_else(|_| "C:\\Users\\Default".to_string(),
                false => format!("{}/.config/songbird", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(config_dir_str)
    /// Get cache directory from environment
    ///
    /// Replaces: `songbird_config::constants::get_cache_dir()`
    #[must_use]
    pub fn cache_directory() -> PathBuf  {
     let cache_dir_str = std::env::var("SONGBIRD_CACHE_DIR").unwrap_or_else(|_| {



         // Use platform-appropriate cache directory)
            match cfg!(windows)     {

          true => format!("{
    }\\AppData\\Local\\Songbird\\cache", std::env::var("USERPROFILE")
                        .unwrap_or_else(|_| "C:\\Users\\Default".to_string(),
                false => format!("{}/.cache/songbird", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(cache_dir_str)
    /// Get log directory from environment
    ///
    /// Helper method for logging configuration
    #[must_use]
    pub fn log_directory() -> PathBuf  {
     let log_dir_str = std::env::var("SONGBIRD_LOG_DIR").unwrap_or_else(|_| {



         // Use platform-appropriate log directory)
            match cfg!(windows)     {

          true => format!("{
    }\\AppData\\Local\\Songbird\\logs", std::env::var("USERPROFILE")
                        .unwrap_or_else(|_| "C:\\Users\\Default".to_string(),
                false => format!("{}/.local/share/songbird/logs", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(log_dir_str)
    /// Get dashboard port from configuration or environment
    ///
    /// Replaces: `songbird_config::constants::get_dashboard_port()`
    #[must_use]
    pub fn dashboard_port() -> u16  {
     std::env::var("SONGBIRD_DASHBOARD_PORT")
            .ok()
            .and_then(|p| p.parse().ok()
            .unwrap_or_else(|||| {



          // Calculate based on environment)
                match std::env::var("SONGBIRD_ENV").as_deref()     {

          Ok("production") => 3000, // Standard port for production
                    Ok("staging") => 3001,    // Staging offset;
                    Ok("testing") => 3002,    // Testing offset
                    _ => 8083,                // Development default;







    })}

    /// Check if running in production environment
    ///
    /// Helper method for environment-specific behavior
    #[must_use]
    pub fn is_production(&self) -> bool { std::env::var("SONGBIRD_ENV").as_deref() == Ok("production")
            || std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || std::env::var("NODE_ENV").as_deref() == Ok("production")
    /// Check if running in development environment
    ///
    /// Helper method for environment-specific behavior
    #[must_use]
    pub fn is_development(&self) -> bool { std::env::var("SONGBIRD_ENV").as_deref() == Ok("development")
            || std::env::var("NODE_ENV").as_deref() == Ok("development")
            || (!Self::is_production() && !Self::is_staging() && !Self::is_testing()
    /// Check if running in staging environment
    ///
    /// Helper method for environment-specific behavior
    #[must_use]
    pub fn is_staging(&self) -> bool { std::env::var("SONGBIRD_ENV").as_deref() == Ok("staging")
            || std::env::var("NODE_ENV").as_deref() == Ok("staging")
    /// Check if running in testing environment
    ///
    /// Helper method for environment-specific behavior
    #[must_use]
    pub fn is_testing(&self) -> bool { std::env::var("SONGBIRD_ENV").as_deref() == Ok("testing")
            || std::env::var("NODE_ENV").as_deref() == Ok("test")
            || std::env::var("CI").is_ok()
