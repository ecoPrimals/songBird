//! Default endpoint configuration with environment variable support
//!
//! Combines host and port information to create full endpoint URLs.
//!
//! # Environment Variables
//!
//! - `SONGBIRD_ORCHESTRATOR_URL` - Full orchestrator endpoint URL
//! - `SONGBIRD_DISCOVERY_URL` - Full discovery endpoint URL
//! - `SONGBIRD_DASHBOARD_URL` - Full dashboard URL
//! - `SONGBIRD_METRICS_URL` - Full metrics endpoint URL
//! - `SONGBIRD_CORS_ORIGINS` - Comma-separated CORS allowed origins

use super::{hosts, ports};
use std::env;

/// Get orchestrator endpoint URL from environment or construct from defaults
///
/// # Environment Variable
/// `SONGBIRD_ORCHESTRATOR_URL` (default: constructed from host and port)
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::endpoints::orchestrator_endpoint;
///
/// let url = orchestrator_endpoint();
/// // Returns "http://127.0.0.1:8080" or value from SONGBIRD_ORCHESTRATOR_URL
/// ```
#[must_use]
pub fn orchestrator_endpoint() -> String {
    env::var("SONGBIRD_ORCHESTRATOR_URL").unwrap_or_else(|_| {
        format!("http://{}:{}", hosts::orchestrator_host(), ports::orchestrator_port())
    })
}

/// Get discovery endpoint URL from environment or construct from defaults
///
/// # Environment Variable
/// `SONGBIRD_DISCOVERY_URL` (default: constructed from host and port)
#[must_use]
pub fn discovery_endpoint() -> String {
    env::var("SONGBIRD_DISCOVERY_URL").unwrap_or_else(|_| {
        format!("http://{}:{}", hosts::discovery_host(), ports::discovery_port())
    })
}

/// Get dashboard URL from environment or construct from defaults
///
/// # Environment Variable
/// `SONGBIRD_DASHBOARD_URL` (default: constructed from host and port)
#[must_use]
pub fn dashboard_url() -> String {
    env::var("SONGBIRD_DASHBOARD_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", hosts::default_host(), ports::dashboard_port()))
}

/// Get metrics endpoint URL from environment or construct from defaults
///
/// # Environment Variable
/// `SONGBIRD_METRICS_URL` (default: constructed from host and port)
#[must_use]
pub fn metrics_url() -> String {
    env::var("SONGBIRD_METRICS_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", hosts::default_host(), ports::metrics_port()))
}

/// Get WebSocket endpoint URL from environment or construct from defaults
///
/// # Environment Variable
/// `SONGBIRD_WEBSOCKET_URL` (default: constructed from host and port)
#[must_use]
pub fn websocket_url() -> String {
    env::var("SONGBIRD_WEBSOCKET_URL")
        .unwrap_or_else(|_| format!("ws://{}:{}", hosts::default_host(), ports::websocket_port()))
}

/// Get CORS allowed origins from environment or default
///
/// # Environment Variable
/// `SONGBIRD_CORS_ORIGINS` - Comma-separated list of origins
/// (default: <http://localhost:3000>)
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::endpoints::cors_origins;
///
/// let origins = cors_origins();
/// // Returns vec!["http://localhost:3000"] or parsed from env var
/// ```
#[must_use]
pub fn cors_origins() -> Vec<String> {
    env::var("SONGBIRD_CORS_ORIGINS").ok().map_or_else(
        // Test uses localhost - acceptable for unit tests
        || vec![format!("http://localhost:{}", ports::dashboard_port())],
        |s| s.split(',').map(|o| o.trim().to_string()).collect(),
    )
}

/// Get service endpoint by name from environment or construct from defaults
///
/// # Environment Variable Pattern
/// `SONGBIRD_{SERVICE}_URL` where SERVICE is uppercase service name
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::endpoints::service_endpoint;
///
/// let url = service_endpoint("CUSTOM");
/// ```
#[must_use]
pub fn service_endpoint(service_name: &str) -> String {
    let env_var = format!("SONGBIRD_{}_URL", service_name.to_uppercase());
    env::var(env_var).unwrap_or_else(|_| {
        let host = hosts::service_host(service_name);
        let port = ports::service_port(service_name, 8080);
        format!("http://{host}:{port}")
    })
}

/// Get full bind address (host:port) for service binding
///
/// Combines `bind_address()` and specified port
#[must_use]
pub fn bind_socket_addr(port: u16) -> String {
    format!("{}:{port}", hosts::bind_address())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_endpoint() {
        let url = orchestrator_endpoint();
        assert!(url.starts_with("http://"));
        assert!(url.contains(':'));
    }

    #[test]
    fn test_discovery_endpoint() {
        let url = discovery_endpoint();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_dashboard_url() {
        let url = dashboard_url();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_metrics_url() {
        let url = metrics_url();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_websocket_url() {
        let url = websocket_url();
        assert!(url.starts_with("ws://"));
    }

    #[test]
    fn test_cors_origins() {
        let origins = cors_origins();
        assert!(!origins.is_empty());
        assert!(origins[0].starts_with("http://"));
    }

    #[test]
    fn test_service_endpoint() {
        let url = service_endpoint("TEST");
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_bind_socket_addr() {
        let addr = bind_socket_addr(8080);
        assert!(addr.contains(':'));
        assert!(addr.contains("8080"));
    }
}
