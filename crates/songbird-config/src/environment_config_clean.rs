//! # 🔧 Environment Configuration - PEDANTIC PERFECT
//!
//! **PEDANTIC QUALITY**: Zero errors, zero warnings, perfect environment handling
//!
//! This module provides clean, error-free environment configuration management.

use std::env;
// use songbird_config; // FIXED: Circular import removed

/// **PEDANTIC**: Environment configuration manager
#[derive(Debug, Clone, Default)]
pub struct EnvironmentConfig;

impl EnvironmentConfig {
    /// Get the Songbird orchestrator endpoint from environment or calculate from config
    #[must_use]
    pub fn songbird_endpoint() -> String {
        std::env::var("SONGBIRD_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = &crate::constants::network::DEFAULT_HOST;
            let port = std::env::var("SONGBIRD_ORCHESTRATOR_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get service endpoint by capability instead of hardcoded primal names
    #[must_use]
    pub fn service_endpoint_by_capability(capability_type: &str, default_port: u16) -> String {
        std::env::var("SONGBIRD_ENDPOINT")
            .or_else(|_| env::var(format!("{capability_type}_ENDPOINT")))
            .unwrap_or_else(|_| {
                format!("http://{}:{}", crate::constants::network::DEFAULT_HOST, default_port)
            })
    }

    /// Get the `ToadStool` compute endpoint from environment or calculate from config
    #[must_use]
    pub fn toadstool_endpoint() -> String {
        std::env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = &crate::constants::network::DEFAULT_HOST;
            let port =
                std::env::var("TOADSTOOL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8081);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get the `NestGate` storage endpoint from environment or calculate from config
    #[must_use]
    pub fn nestgate_endpoint() -> String {
        std::env::var("NESTGATE_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = &crate::constants::network::DEFAULT_HOST;
            let port =
                std::env::var("NESTGATE_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8082);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get the Squirrel AI endpoint from environment or calculate from config
    #[must_use]
    pub fn squirrel_endpoint() -> String {
        std::env::var("SQUIRREL_ENDPOINT").unwrap_or_else(|_| {
            let bind_addr = &crate::constants::network::DEFAULT_HOST;
            let port =
                std::env::var("SQUIRREL_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8083);
            format!("http://{bind_addr}:{port}")
        })
    }

    /// Get configuration value from environment with fallback
    #[must_use]
    pub fn get_env_or_default(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get configuration value from environment as integer with fallback
    #[must_use]
    pub fn get_env_int_or_default(key: &str, default: u16) -> u16 {
        std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    /// Get configuration value from environment as boolean with fallback
    #[must_use]
    pub fn get_env_bool_or_default(key: &str, default: bool) -> bool {
        std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    /// Check if running in development mode
    #[must_use]
    pub fn is_development() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "development"
    }

    /// Check if running in production mode
    #[must_use]
    pub fn is_production() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "production"
    }

    /// Get bind address from environment
    #[must_use]
    pub fn bind_address() -> String {
        if Self::is_production() {
            Self::get_env_or_default("BIND_ADDRESS", "0.0.0.0")
        } else {
            Self::get_env_or_default("BIND_ADDRESS", crate::constants::network::DEFAULT_HOST)
        }
    }

    /// Get orchestrator port from environment
    #[must_use]
    pub fn orchestrator_port() -> u16 {
        Self::get_env_int_or_default("SONGBIRD_ORCHESTRATOR_PORT", 8080)
    }

    /// Get discovery port from environment
    #[must_use]
    pub fn discovery_port() -> u16 {
        Self::get_env_int_or_default("SONGBIRD_DISCOVERY_PORT", 8001)
    }

    /// Get registry port from environment
    #[must_use]
    pub fn registry_port() -> u16 {
        Self::get_env_int_or_default("SONGBIRD_REGISTRY_PORT", 8002)
    }

    /// Get metrics port from environment
    #[must_use]
    pub fn metrics_port() -> u16 {
        Self::get_env_int_or_default("SONGBIRD_METRICS_PORT", 8004)
    }

    /// Get federation port from environment
    #[must_use]
    pub fn federation_port() -> u16 {
        Self::get_env_int_or_default("SONGBIRD_FEDERATION_PORT", 8005)
    }
}

// ============================================================================
// PEDANTIC PERFECT HELPER FUNCTIONS
// ============================================================================

/// **PEDANTIC**: Get environment variable or panic with helpful message
///
/// # Panics
///
/// Panics if the environment variable is not set
#[must_use]
pub fn get_required_env(key: &str) -> String {
    std::env::var(key)
        .unwrap_or_else(|_| panic!("Required environment variable '{key}' is not set"))
}

/// **PEDANTIC**: Get environment variable as integer or panic with helpful message
///
/// # Panics
///
/// Panics if the environment variable is not set or is not a valid integer
#[must_use]
pub fn get_required_env_int(key: &str) -> u16 {
    std::env::var(key)
        .unwrap_or_else(|_| panic!("Required environment variable '{key}' is not set"))
        .parse()
        .unwrap_or_else(|_| panic!("Environment variable '{key}' is not a valid integer"))
}

/// **PEDANTIC**: Get environment variable as boolean or panic with helpful message
///
/// # Panics
///
/// Panics if the environment variable is not set or is not a valid boolean
#[must_use]
pub fn get_required_env_bool(key: &str) -> bool {
    std::env::var(key)
        .unwrap_or_else(|_| panic!("Required environment variable '{key}' is not set"))
        .parse()
        .unwrap_or_else(|_| panic!("Environment variable '{key}' is not a valid boolean"))
}

// ============================================================================
// PEDANTIC PERFECT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_songbird_endpoint_default() {
        // Clear environment variable if set
        std::env::remove_var("SONGBIRD_ENDPOINT");
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");

        let endpoint = EnvironmentConfig::songbird_endpoint();
        assert_eq!(
            endpoint,
            format!(
                "http://{}:{}",
                crate::constants::network::DEFAULT_HOST,
                crate::constants::network::DEFAULT_ORCHESTRATOR_PORT
            )
        );
    }

    #[test]
    fn test_songbird_endpoint_from_env() {
        std::env::set_var("SONGBIRD_ENDPOINT", "http://custom:9000");

        let endpoint = EnvironmentConfig::songbird_endpoint();
        assert_eq!(endpoint, "http://custom:9000");

        std::env::remove_var("SONGBIRD_ENDPOINT");
    }

    #[test]
    fn test_service_endpoint_by_capability() {
        let endpoint = EnvironmentConfig::service_endpoint_by_capability("COMPUTE", 8081);
        assert_eq!(endpoint, format!("http://{}:8081", crate::constants::network::DEFAULT_HOST));
    }

    #[test]
    fn test_get_env_or_default() {
        std::env::set_var("TEST_VAR", "test_value");
        assert_eq!(EnvironmentConfig::get_env_or_default("TEST_VAR", "default"), "test_value");
        assert_eq!(EnvironmentConfig::get_env_or_default("NON_EXISTENT", "default"), "default");
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_get_env_int_or_default() {
        std::env::set_var("TEST_INT", "42");
        assert_eq!(EnvironmentConfig::get_env_int_or_default("TEST_INT", 0), 42);
        assert_eq!(EnvironmentConfig::get_env_int_or_default("NON_EXISTENT", 123), 123);
        std::env::remove_var("TEST_INT");
    }

    #[test]
    fn test_get_env_bool_or_default() {
        std::env::set_var("TEST_BOOL", "true");
        assert!(EnvironmentConfig::get_env_bool_or_default("TEST_BOOL", false));
        assert!(!EnvironmentConfig::get_env_bool_or_default("NON_EXISTENT", false));
        std::env::remove_var("TEST_BOOL");
    }

    #[test]
    fn test_is_development() {
        // Save current environment
        let original = std::env::var("ENVIRONMENT").ok();

        // Test default (development)
        std::env::remove_var("ENVIRONMENT");
        assert!(
            EnvironmentConfig::is_development(),
            "Should be development when ENVIRONMENT is not set"
        );

        // Test explicit production
        std::env::set_var("ENVIRONMENT", "production");
        assert!(
            !EnvironmentConfig::is_development(),
            "Should not be development when ENVIRONMENT=production"
        );

        // Restore original environment
        if let Some(val) = original {
            std::env::set_var("ENVIRONMENT", val);
        } else {
            std::env::remove_var("ENVIRONMENT");
        }
    }

    #[test]
    fn test_is_production() {
        std::env::remove_var("ENVIRONMENT");
        assert!(!EnvironmentConfig::is_production());

        std::env::set_var("ENVIRONMENT", "production");
        assert!(EnvironmentConfig::is_production());

        std::env::remove_var("ENVIRONMENT");
    }

    #[test]
    fn test_bind_address() {
        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("BIND_ADDRESS");
        assert_eq!(EnvironmentConfig::bind_address(), crate::constants::network::DEFAULT_HOST);

        std::env::set_var("ENVIRONMENT", "production");
        assert_eq!(EnvironmentConfig::bind_address(), "0.0.0.0");

        std::env::remove_var("ENVIRONMENT");
        std::env::remove_var("BIND_ADDRESS");
    }

    #[test]
    fn test_port_getters() {
        std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
        assert_eq!(EnvironmentConfig::orchestrator_port(), 8080);

        std::env::remove_var("SONGBIRD_DISCOVERY_PORT");
        assert_eq!(EnvironmentConfig::discovery_port(), 8001);

        std::env::remove_var("SONGBIRD_REGISTRY_PORT");
        assert_eq!(EnvironmentConfig::registry_port(), 8002);

        std::env::remove_var("SONGBIRD_METRICS_PORT");
        assert_eq!(EnvironmentConfig::metrics_port(), 8004);

        std::env::remove_var("SONGBIRD_FEDERATION_PORT");
        assert_eq!(EnvironmentConfig::federation_port(), 8005);
    }
}
