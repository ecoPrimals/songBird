//! Network Constants Module
//!
//! Network-specific constants for the Songbird configuration system.
//! Extracted from the main constants module for better organization.

use std::time::Duration;

// ============================================================================
// NETWORK CONSTANTS
// ============================================================================

/// Default network ports
pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;
pub const DEFAULT_DISCOVERY_PORT: u16 = 8001;
pub const DEFAULT_GAMING_PORT: u16 = 6112;
pub const DEFAULT_HEALTH_PORT: u16 = 8002;
pub const DEFAULT_DASHBOARD_PORT: u16 = 8003;
pub const DEFAULT_METRICS_PORT: u16 = 8004;
pub const DEFAULT_FEDERATION_PORT: u16 = 8005;
pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;

/// Port ranges
pub const MIN_DYNAMIC_PORT: u16 = 49152;
pub const MAX_DYNAMIC_PORT: u16 = 65535;
pub const GAMING_PORT_RANGE_START: u16 = 6112;
pub const GAMING_PORT_RANGE_END: u16 = 6200;

/// Network timeouts
pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Connection limits
pub const DEFAULT_MAX_CONNECTIONS: usize = 1000;
pub const DEFAULT_MAX_CONNECTIONS_PER_IP: usize = 10;
pub const DEFAULT_CONNECTION_BACKLOG: usize = 128;

/// Buffer sizes
pub const DEFAULT_BUFFER_SIZE: usize = 8192;
pub const MAX_BUFFER_SIZE: usize = 65536;
pub const MIN_BUFFER_SIZE: usize = 1024;
pub const LARGE_BUFFER_SIZE: usize = 32768;
pub const SMALL_BUFFER_SIZE: usize = 4096;

/// Performance and timeout constants
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 60;
pub const DEFAULT_SCAN_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_HEALTH_CHECK_TIMEOUT_MS: u64 = 30000;
pub const DEFAULT_DISCOVERY_TIMEOUT_MS: u64 = 10000;
pub const DEFAULT_CIRCUIT_BREAKER_TIMEOUT_MS: u64 = 30000;

/// Zero-copy optimization constants
pub const ZERO_COPY_THRESHOLD: usize = 8192;
pub const MEMORY_MAP_THRESHOLD: usize = 1048576; // 1MB
pub const VECTORED_IO_THRESHOLD: usize = 16384;

/// Load balancing constants
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;
pub const DEFAULT_JITTER_FACTOR: f64 = 0.1;

/// Default bind address for Songbird services
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// Default localhost address
pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

/// IPv4 localhost constant
pub const LOCALHOST_IPV4: &str = "127.0.0.1";

/// Protocol constants
pub const HTTP_SCHEME: &str = "http";
pub const HTTPS_SCHEME: &str = "https";
pub const WS_SCHEME: &str = "ws";
pub const WSS_SCHEME: &str = "wss";

/// HTTP/HTTPS ports for backward compatibility
pub const DEFAULT_HTTP_PORT: u16 = 8080;
pub const DEFAULT_HTTPS_PORT: u16 = 8443;

/// Gaming protocol constants
pub const STARCRAFT_DEFAULT_PORT: u16 = 6112;
pub const WARCRAFT_DEFAULT_PORT: u16 = 6112;
pub const DIABLO_DEFAULT_PORT: u16 = 6113;
pub const IPX_BROADCAST_PORT: u16 = 213;
pub const DIRECTPLAY_PORT: u16 = 2300;

/// Network validation constants
pub const MIN_PORT: u16 = 1;
pub const MAX_PORT: u16 = 65535;
pub const RESERVED_PORT_THRESHOLD: u16 = 1024;

/// Helper functions
#[must_use]
pub fn is_valid_port(port: u16) -> bool {
    port > 0
}

#[must_use]
pub fn is_reserved_port(port: u16) -> bool {
    port < RESERVED_PORT_THRESHOLD
}

#[must_use]
pub fn is_dynamic_port(port: u16) -> bool {
    port >= MIN_DYNAMIC_PORT
}

#[must_use]
pub fn get_default_port_for_service(service: &str) -> u16 {
    match service {
        "discovery" => DEFAULT_DISCOVERY_PORT,
        "gaming" => DEFAULT_GAMING_PORT,
        "health" => DEFAULT_HEALTH_PORT,
        "dashboard" => DEFAULT_DASHBOARD_PORT,
        "metrics" => DEFAULT_METRICS_PORT,
        "federation" => DEFAULT_FEDERATION_PORT,
        "websocket" => DEFAULT_WEBSOCKET_PORT,
        _ => DEFAULT_ORCHESTRATOR_PORT,
    }
}

/// Get default bind address
#[must_use]
pub fn default_bind_address() -> String {
    DEFAULT_BIND_ADDRESS.to_string()
}

/// Get external address from environment or use default
#[must_use]
pub fn external_address() -> String {
    std::env::var("SONGBIRD_EXTERNAL_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_validation() {
        assert!(is_valid_port(8080));
        assert!(!is_valid_port(0));
        assert!(is_valid_port(65535));
    }

    #[test]
    fn test_reserved_ports() {
        assert!(is_reserved_port(80));
        assert!(is_reserved_port(443));
        assert!(!is_reserved_port(8080));
    }

    #[test]
    fn test_dynamic_ports() {
        assert!(is_dynamic_port(49152));
        assert!(is_dynamic_port(65535));
        assert!(!is_dynamic_port(8080));
    }

    #[test]
    fn test_service_port_lookup() {
        assert_eq!(get_default_port_for_service("orchestrator"), 8080);
        assert_eq!(get_default_port_for_service("discovery"), 8001);
        assert_eq!(get_default_port_for_service("unknown"), 8080);
    }

    #[test]
    fn test_gaming_ports() {
        assert_eq!(STARCRAFT_DEFAULT_PORT, 6112);
        assert_eq!(WARCRAFT_DEFAULT_PORT, 6112);
        assert_eq!(DIABLO_DEFAULT_PORT, 6113);
    }
}
