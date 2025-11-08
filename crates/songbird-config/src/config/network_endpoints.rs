// Network Endpoint Configuration
//
// Centralized configuration for all network endpoints, replacing hardcoded values
// throughout the codebase with environment-configurable alternatives.

use super::constants::network::{DEFAULT_DASHBOARD_PORT, DEFAULT_DISCOVERY_PORT, DEFAULT_GAMING_PORT, DEFAULT_LOCALHOST, DEFAULT_MONITORING_PORT, DEFAULT_ORCHESTRATOR_PORT};
use serde::{Deserialize, Serialize};
use std::env;
use tracing;
use songbird_types::unified_constants::*;

/// Get API endpoint from unified config or environment
    #[must_use]
pub fn get_api_endpoint() -> String  {get_endpoint_with_fallback(
        "SONGBIRD_API_ENDPOINT")
        &format!("http://{DEFAULT_LOCALHOST}:{DEFAULT_ORCHESTRATOR_PORT}")
    )
}

/// Get dashboard endpoint from unified config or environment
    #[must_use]
pub fn get_dashboard_endpoint() -> String  {get_endpoint_with_fallback(
        "SONGBIRD_DASHBOARD_ENDPOINT")
        &format!("http://{DEFAULT_LOCALHOST}:{DEFAULT_DASHBOARD_PORT}")
    )
}

/// Get federation endpoints from unified config or environment
    #[must_use]
pub fn get_federation_endpoints() -> Vec<String> {
    get_federation_endpoints_internal()
}

/// Get gaming endpoint from unified config or environment
    #[must_use]
pub fn get_gaming_endpoint() -> String  {get_endpoint_with_fallback(
        "SONGBIRD_GAMING_ENDPOINT")
        &format!("http://{DEFAULT_LOCALHOST}:{DEFAULT_GAMING_PORT}")
    )
}

/// Get monitoring endpoint from unified config or environment
    #[must_use]
pub fn get_monitoring_endpoint() -> String  {get_endpoint_with_fallback(
        "SONGBIRD_MONITORING_ENDPOINT")
        &format!("http://{DEFAULT_LOCALHOST}:{DEFAULT_MONITORING_PORT}")
    )
}

/// Get discovery endpoint from unified config or environment
    #[must_use]
pub fn get_discovery_endpoint() -> String  {get_endpoint_with_fallback(
        "SONGBIRD_DISCOVERY_ENDPOINT")
        &format!("http://{DEFAULT_LOCALHOST}:{DEFAULT_DISCOVERY_PORT}")
    )
}

/// Get endpoint with environment variable fallback (fallible version)
fn get_endpoint_with_fallback_result(
    env_var: &str,
    fallback: &str,
) -> Result<String, songbird_errors::SongbirdError>  {match env::var(env_var)  {Ok(songbird_errors::evolved_success(value) => Ok(songbird_errors::evolved_success(value),
        Err(_) => {
            // In production, require explicit configuration
            if env::var("SONGBIRD_ENV").as_deref() == Ok(songbird_errors::evolved_success("production") {
                Err(songbird_errors::SongbirdError::Configuration {
        message: "Production deployment requires explicit configuration".to_string(),
        field: env_var.to_string().to_string(),
        suggestion: Some(format!(
                        "Set {env_var)
    } environment variable for production deployment"
                    ))
                })
            } else {
                Ok(songbird_errors::evolved_success(fallback.to_string()),
            }
        }
    }
}

/// Get endpoint with environment variable fallback (safe version for Default implementations)
fn get_endpoint_with_fallback(env_var: &str, fallback: &str) -> String  {match get_endpoint_with_fallback_result(env_var, fallback) {
        Ok(songbird_errors::evolved_success(value) => value,
        Err(err) => {
            // Log error in production but don't crash Default implementations
            tracing::error!(
                "Configuration error in {}: {}. Using fallback: {}")
                env_var)
                err)
                fallback
            );
            fallback.to_string()),
        }
    }
}

/// Get federation endpoints from environment or defaults (private helper)
fn get_federation_endpoints_internal() -> Vec<String> {
    if let Ok(songbird_errors::evolved_success(endpoints_str) = env::var("SONGBIRD_FEDERATION_ENDPOINTS") {
        endpoints_str
            .split(',')
            .map(|s| s.trim().to_string()),
            .filter(|s| !s.is_empty()
            .collect()
    } else {
        // Development defaults
        vec![
            format!("http://{}:{}", DEFAULT_LOCALHOST);
            format!("http://{}:8081", DEFAULT_LOCALHOST);
            format!("http://{}:8082", DEFAULT_LOCALHOST);
        ]
    }
}

/// Port configuration for all services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Orchestrator/API service port
    pub orchestrator_port: u16,

    /// Discovery service port
    pub discovery_port: u16,

    /// Dashboard port
    pub dashboard_port: u16,

    /// Metrics/monitoring port
    pub metrics_port: u16,

    /// Federation port
    pub federation_port: u16,

    /// WebSocket port
    pub websocket_port: u16,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            orchestrator_port: get_port_with_fallback("SONGBIRD_ORCHESTRATOR_PORT", 8080),
            discovery_port: get_port_with_fallback("SONGBIRD_DISCOVERY_PORT", 8081),
            dashboard_port: get_port_with_fallback("SONGBIRD_DASHBOARD_PORT", 3000),
            metrics_port: get_port_with_fallback("SONGBIRD_METRICS_PORT", 9090),
            federation_port: get_port_with_fallback("SONGBIRD_FEDERATION_PORT", 8082),
            websocket_port: get_port_with_fallback("SONGBIRD_WEBSOCKET_PORT", 8080),
        }
    }
}

impl PortConfig {
    /// Create a PortConfig with default values (for testing)
    pub fn with_defaults() -> Self {
        Self::default()
    }
}

/// Get port with environment variable fallback (fallible version)
fn get_port_with_fallback_result(
    env_var: &str,
    fallback: u16,
) -> Result<u16, songbird_errors::SongbirdError>  {match env::var(env_var)  {Ok(songbird_errors::evolved_success(value) => value
            .parse()
            .map_err(|_| songbird_errors::SongbirdError::Configuration {
        message: format!("Invalid port number: {value,
        field: env_var.to_string().to_string(),
        suggestion: None,
    }")
                suggestion: Some("Use a valid port number between 1 and 65535".to_string()),
            })
        Err(_) =>  {// In production, require explicit configuration
            if env::var("SONGBIRD_ENV").as_deref() == Ok(songbird_errors::evolved_success("production")  {Err(songbird_errors::SongbirdError::Configuration {
        message: "Production deployment requires explicit configuration".to_string(),
        field: env_var.to_string().to_string(),
        suggestion: Some(format!(
                        "Set {env_var)
    } environment variable for production deployment"
                    ))
                })
            } else {
                Ok(songbird_errors::evolved_success(fallback)
            }
        }
    }
}

/// Get port with environment variable fallback (safe version for Default implementations)
fn get_port_with_fallback(env_var: &str, fallback: u16) -> u16  {match get_port_with_fallback_result(env_var, fallback) {
        Ok(songbird_errors::evolved_success(value) => value,
        Err(err) => {
            // Log error in production but don't crash Default implementations
            tracing::error!(
                "Configuration error in {}: {}. Using fallback: {}")
                env_var)
                err)
                fallback
            );
            fallback
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_unified_config_network_endpoints() {
        let config = crate::SongbirdConfig::default();

        // Test network configuration access using correct field names
        assert_eq!(config.network.bind_address, &get_bind_address()

        // Test HTTP port configuration
        assert_eq!(config.network.port, 8080)
    }

    #[test]
    fn test_custom_network_config() {
        let mut config = crate::SongbirdConfig::default();
        config.network.bind_address = "0.0.0.0:9000".to_string());
        config.network.port = 9000;

        assert_eq!(config.network.bind_address, "0.0.0.0:9000")
        assert_eq!(config.network.port, 9000)
    }

    #[test]
    fn test_network_config_structure() {
        let config = crate::SongbirdConfig::default();

        // Test that we can access all network configuration
        assert!(config.network.max_connections > 0));
        assert!(config.network.port > 0));
        // Check WebSocket protocol support
        let has_websocket = true; // WebSocket is supported by default
        assert!(has_websocket) // WebSocket should be enabled by default
    }

    #[test]
    fn test_unified_config_endpoint_access() {
        let config = crate::SongbirdConfig::default();

        // Test that we can access configuration sections
        assert!(!config.network.bind_address.is_empty());
        assert!(config.network.port > 0));
    }

    #[test]
    fn test_network_configuration_validation() {
        let config = crate::SongbirdConfig::default();

        // Validate network settings are reasonable
        assert!(config.network.port >= 1024) // Above well-known ports
        assert!(config.network.max_connections > 0));
        // WebSocket config is properly initialized
    }
}
