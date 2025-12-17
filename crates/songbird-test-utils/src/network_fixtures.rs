//! Network Test Fixtures
//!
//! Provides environment-configurable test fixtures for network configuration.
//! These replace hardcoded localhost/127.0.0.1/ports throughout test code.

use std::net::IpAddr;

/// Get test bind address from environment or use safe default
///
/// Respects `SONGBIRD_TEST_BIND_ADDRESS` environment variable.
/// Defaults to 127.0.0.1 for test isolation.
#[must_use]
pub fn test_bind_address() -> String {
    std::env::var("SONGBIRD_TEST_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Get test bind address as `IpAddr` type
#[must_use]
pub fn test_bind_ip() -> IpAddr {
    test_bind_address().parse().unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
}

/// Get test port from environment or use safe default
///
/// Respects `SONGBIRD_TEST_PORT` environment variable.
/// Defaults to 8080.
#[must_use]
pub fn test_port() -> u16 {
    std::env::var("SONGBIRD_TEST_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080)
}

/// Get orchestrator test port
#[must_use]
pub fn test_orchestrator_port() -> u16 {
    std::env::var("SONGBIRD_TEST_ORCHESTRATOR_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

/// Get discovery test port
#[must_use]
pub fn test_discovery_port() -> u16 {
    std::env::var("SONGBIRD_TEST_DISCOVERY_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8081)
}

/// Get health check test port
#[must_use]
pub fn test_health_port() -> u16 {
    std::env::var("SONGBIRD_TEST_HEALTH_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8082)
}

/// Get metrics test port
#[must_use]
pub fn test_metrics_port() -> u16 {
    std::env::var("SONGBIRD_TEST_METRICS_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(9090)
}

/// Get dashboard test port
#[must_use]
pub fn test_dashboard_port() -> u16 {
    std::env::var("SONGBIRD_TEST_DASHBOARD_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3000)
}

/// Get federation test port
#[must_use]
pub fn test_federation_port() -> u16 {
    std::env::var("SONGBIRD_TEST_FEDERATION_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8083)
}

/// Create a test service URL
///
/// # Arguments
/// * `service` - Service name (e.g., "orchestrator", "discovery")
///
/// # Returns
/// URL in format: `http://{bind_address}:{port}/{service}`
#[must_use]
pub fn test_service_url(service: &str) -> String {
    let env_var = format!("SONGBIRD_TEST_{}_URL", service.to_uppercase());
    std::env::var(&env_var)
        .unwrap_or_else(|_| format!("http://{}:{}/{}", test_bind_address(), test_port(), service))
}

/// Create a test endpoint URL with custom port
///
/// # Arguments
/// * `path` - Path component (e.g., "api/v1/health")
/// * `port` - Port number
#[must_use]
pub fn test_endpoint_url(path: &str, port: u16) -> String {
    format!("http://{}:{}/{}", test_bind_address(), port, path.trim_start_matches('/'))
}

/// Get test host:port pair
#[must_use]
pub fn test_host_port() -> String {
    format!("{}:{}", test_bind_address(), test_port())
}

/// Get test HTTP base URL
#[must_use]
pub fn test_http_url() -> String {
    format!("http://{}:{}", test_bind_address(), test_port())
}

/// Get test WebSocket URL
#[must_use]
pub fn test_websocket_url() -> String {
    format!("ws://{}:{}", test_bind_address(), test_port())
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_address_default() {
        let addr = test_bind_address();
        assert!(!addr.is_empty());
        assert!(addr.contains("127.0.0.1") || addr.parse::<IpAddr>().is_ok());
    }

    #[test]
    fn test_port_is_valid() {
        let port = test_port();
        assert!(port > 0);
        // Port is u16, so it's always < 65536
    }

    #[test]
    fn test_service_url_format() {
        let url = test_service_url("test");
        assert!(url.starts_with("http://"));
        assert!(url.contains("/test"));
    }

    #[test]
    fn test_endpoint_url_format() {
        let url = test_endpoint_url("api/health", 8080);
        assert!(url.starts_with("http://"));
        assert!(url.contains("api/health"));
        assert!(url.contains("8080"));
    }

    #[test]
    fn test_websocket_url_format() {
        let url = test_websocket_url();
        assert!(url.starts_with("ws://"));
    }
}
