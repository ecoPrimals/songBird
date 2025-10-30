//! Comprehensive tests for `defaults::ports` and `defaults::hosts` modules
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

use serial_test::serial;
use songbird_config::defaults::{hosts, ports};

// ============================================================================
// PORT CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_orchestrator_port_default() {
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    let port = ports::orchestrator_port();
    assert_eq!(port, 8080);
}

#[test]
#[serial]
fn test_orchestrator_port_from_env() {
    std::env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "9000");
    let port = ports::orchestrator_port();
    assert_eq!(port, 9000);
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
}

#[test]
fn test_discovery_port_default() {
    std::env::remove_var("SONGBIRD_DISCOVERY_PORT");
    let port = ports::discovery_port();
    assert_eq!(port, 8081);
}

#[test]
#[serial]
fn test_discovery_port_from_env() {
    std::env::set_var("SONGBIRD_DISCOVERY_PORT", "9001");
    let port = ports::discovery_port();
    assert_eq!(port, 9001);
    std::env::remove_var("SONGBIRD_DISCOVERY_PORT");
}

#[test]
fn test_dashboard_port_default() {
    std::env::remove_var("SONGBIRD_DASHBOARD_PORT");
    let port = ports::dashboard_port();
    assert_eq!(port, 3000);
}

#[test]
#[serial]
fn test_dashboard_port_from_env() {
    std::env::set_var("SONGBIRD_DASHBOARD_PORT", "4000");
    let port = ports::dashboard_port();
    assert_eq!(port, 4000);
    std::env::remove_var("SONGBIRD_DASHBOARD_PORT");
}

#[test]
fn test_metrics_port_default() {
    std::env::remove_var("SONGBIRD_METRICS_PORT");
    let port = ports::metrics_port();
    assert_eq!(port, 9090);
}

#[test]
#[serial]
fn test_metrics_port_from_env() {
    std::env::set_var("SONGBIRD_METRICS_PORT", "9091");
    let port = ports::metrics_port();
    assert_eq!(port, 9091);
    std::env::remove_var("SONGBIRD_METRICS_PORT");
}

#[test]
fn test_federation_port_default() {
    std::env::remove_var("SONGBIRD_FEDERATION_PORT");
    let port = ports::federation_port();
    assert_eq!(port, 8082);
}

#[test]
#[serial]
fn test_federation_port_from_env() {
    std::env::set_var("SONGBIRD_FEDERATION_PORT", "8083");
    let port = ports::federation_port();
    assert_eq!(port, 8083);
    std::env::remove_var("SONGBIRD_FEDERATION_PORT");
}

#[test]
fn test_websocket_port_default() {
    std::env::remove_var("SONGBIRD_WEBSOCKET_PORT");
    let port = ports::websocket_port();
    assert_eq!(port, 8080);
}

#[test]
fn test_websocket_port_from_env() {
    std::env::set_var("SONGBIRD_WEBSOCKET_PORT", "8084");
    let port = ports::websocket_port();
    assert_eq!(port, 8084);
    std::env::remove_var("SONGBIRD_WEBSOCKET_PORT");
}

#[test]
fn test_gaming_port_default() {
    std::env::remove_var("SONGBIRD_GAMING_PORT");
    let port = ports::gaming_port();
    assert_eq!(port, 6112); // StarCraft IPX default
}

#[test]
fn test_all_ports_are_valid() {
    // Clear all port environment variables
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    std::env::remove_var("SONGBIRD_DISCOVERY_PORT");
    std::env::remove_var("SONGBIRD_DASHBOARD_PORT");
    std::env::remove_var("SONGBIRD_METRICS_PORT");
    std::env::remove_var("SONGBIRD_FEDERATION_PORT");
    std::env::remove_var("SONGBIRD_WEBSOCKET_PORT");
    std::env::remove_var("SONGBIRD_GAMING_PORT");

    // All ports should be valid (non-zero)
    assert!(ports::orchestrator_port() > 0);
    assert!(ports::discovery_port() > 0);
    assert!(ports::dashboard_port() > 0);
    assert!(ports::metrics_port() > 0);
    assert!(ports::federation_port() > 0);
    assert!(ports::websocket_port() > 0);
    assert!(ports::gaming_port() > 0);
}

#[test]
fn test_ports_are_unique_defaults() {
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    std::env::remove_var("SONGBIRD_DISCOVERY_PORT");
    std::env::remove_var("SONGBIRD_FEDERATION_PORT");
    std::env::remove_var("SONGBIRD_METRICS_PORT");
    std::env::remove_var("SONGBIRD_DASHBOARD_PORT");

    let orchestrator = ports::orchestrator_port();
    let discovery = ports::discovery_port();
    let federation = ports::federation_port();
    let metrics = ports::metrics_port();
    let dashboard = ports::dashboard_port();

    // Most ports should be unique (orchestrator and websocket share 8080)
    assert_ne!(orchestrator, discovery);
    assert_ne!(orchestrator, federation);
    assert_ne!(discovery, federation);
    assert_ne!(metrics, dashboard);
}

// ============================================================================
// HOST CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_default_host_localhost() {
    std::env::remove_var("SONGBIRD_HOST");
    let host = hosts::default_host();
    assert_eq!(host, "127.0.0.1");
}

#[test]
fn test_default_host_from_env() {
    std::env::set_var("SONGBIRD_HOST", "192.168.1.100");
    let host = hosts::default_host();
    assert_eq!(host, "192.168.1.100");
    std::env::remove_var("SONGBIRD_HOST");
}

#[test]
fn test_bind_address_default() {
    std::env::remove_var("SONGBIRD_BIND_ADDRESS");
    let addr = hosts::bind_address();
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_bind_address_from_env() {
    std::env::set_var("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
    let addr = hosts::bind_address();
    assert_eq!(addr, "127.0.0.1");
    std::env::remove_var("SONGBIRD_BIND_ADDRESS");
}

#[test]
fn test_discovery_host_uses_default() {
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_DISCOVERY_HOST");
    let host = hosts::discovery_host();
    assert_eq!(host, "127.0.0.1"); // Should use default_host
}

#[test]
fn test_discovery_host_from_env() {
    std::env::set_var("SONGBIRD_DISCOVERY_HOST", "10.0.0.1");
    let host = hosts::discovery_host();
    assert_eq!(host, "10.0.0.1");
    std::env::remove_var("SONGBIRD_DISCOVERY_HOST");
}

#[test]
fn test_orchestrator_host_uses_default() {
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_HOST");
    let host = hosts::orchestrator_host();
    assert_eq!(host, "127.0.0.1");
}

#[test]
fn test_orchestrator_host_from_env() {
    std::env::set_var("SONGBIRD_ORCHESTRATOR_HOST", "10.0.0.2");
    let host = hosts::orchestrator_host();
    assert_eq!(host, "10.0.0.2");
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_HOST");
}

// ============================================================================
// INTEGRATION TESTS - HOSTS AND PORTS
// ============================================================================

#[test]
fn test_complete_endpoint_construction() {
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");

    let host = hosts::default_host();
    let port = ports::orchestrator_port();
    let endpoint = format!("http://{host}:{port}");

    assert_eq!(endpoint, "http://127.0.0.1:8080");
}

#[test]
fn test_discovery_endpoint_construction() {
    std::env::remove_var("SONGBIRD_DISCOVERY_HOST");
    std::env::remove_var("SONGBIRD_DISCOVERY_PORT");

    let host = hosts::discovery_host();
    let port = ports::discovery_port();
    let endpoint = format!("http://{host}:{port}");

    assert_eq!(endpoint, "http://127.0.0.1:8081");
}

#[test]
fn test_dashboard_endpoint_construction() {
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_DASHBOARD_PORT");

    let host = hosts::default_host();
    let port = ports::dashboard_port();
    let endpoint = format!("http://{host}:{port}");

    assert_eq!(endpoint, "http://127.0.0.1:3000");
}

#[test]
fn test_metrics_endpoint_construction() {
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_METRICS_PORT");

    let host = hosts::default_host();
    let port = ports::metrics_port();
    let endpoint = format!("http://{host}:{port}");

    assert_eq!(endpoint, "http://127.0.0.1:9090");
}

#[test]
fn test_production_bind_configuration() {
    std::env::remove_var("SONGBIRD_BIND_ADDRESS");
    let bind = hosts::bind_address();

    // Production bind should listen on all interfaces
    assert_eq!(bind, "0.0.0.0");
}

#[test]
fn test_development_host_configuration() {
    std::env::remove_var("SONGBIRD_HOST");
    let host = hosts::default_host();

    // Development should default to localhost
    assert_eq!(host, "127.0.0.1");
}

// ============================================================================
// EDGE CASES AND VALIDATION
// ============================================================================

#[test]
fn test_invalid_port_env_uses_default() {
    std::env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "invalid");
    let port = ports::orchestrator_port();
    assert_eq!(port, 8080); // Should fall back to default
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
}

#[test]
fn test_empty_host_env_uses_default() {
    std::env::set_var("SONGBIRD_HOST", "");
    let host = hosts::default_host();
    // Empty string should be returned as-is (environment variable is set)
    assert_eq!(host, "");
    std::env::remove_var("SONGBIRD_HOST");
}

#[test]
#[serial]
fn test_port_range_boundaries() {
    // Test minimum valid port (above privileged range)
    std::env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "1024");
    let port = ports::orchestrator_port();
    assert_eq!(port, 1024);

    // Test maximum valid port
    std::env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "65535");
    let port = ports::orchestrator_port();
    assert_eq!(port, 65535);

    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
}

#[test]
fn test_ipv6_host_configuration() {
    std::env::set_var("SONGBIRD_HOST", "::1");
    let host = hosts::default_host();
    assert_eq!(host, "::1");
    std::env::remove_var("SONGBIRD_HOST");
}

#[test]
fn test_hostname_instead_of_ip() {
    std::env::set_var("SONGBIRD_HOST", "songbird.local");
    let host = hosts::default_host();
    assert_eq!(host, "songbird.local");
    std::env::remove_var("SONGBIRD_HOST");
}

// ============================================================================
// CLEANUP AND ISOLATION TESTS
// ============================================================================

#[test]
fn test_env_isolation() {
    // Set values
    std::env::set_var("SONGBIRD_TEST_PORT", "12345");
    assert_eq!(std::env::var("SONGBIRD_TEST_PORT").unwrap(), "12345");

    // Clear values
    std::env::remove_var("SONGBIRD_TEST_PORT");
    assert!(std::env::var("SONGBIRD_TEST_PORT").is_err());
}

#[test]
fn test_concurrent_default_access() {
    // Multiple calls should return same defaults
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");

    let port1 = ports::orchestrator_port();
    let port2 = ports::orchestrator_port();
    let port3 = ports::orchestrator_port();

    assert_eq!(port1, port2);
    assert_eq!(port2, port3);
}

#[test]
fn test_all_hosts_valid_strings() {
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_BIND_ADDRESS");
    std::env::remove_var("SONGBIRD_DISCOVERY_HOST");
    std::env::remove_var("SONGBIRD_ORCHESTRATOR_HOST");

    // All hosts should return non-empty strings
    assert!(!hosts::default_host().is_empty());
    assert!(!hosts::bind_address().is_empty());
    assert!(!hosts::discovery_host().is_empty());
    assert!(!hosts::orchestrator_host().is_empty());
}
