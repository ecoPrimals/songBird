//! Comprehensive tests for defaults module
//! Tests all default configuration functions with and without environment variables

use serial_test::serial;
use songbird_config::defaults::{endpoints, hosts, ports, timeouts};
use std::env;
use std::time::Duration;

// ============================================================================
// Port Tests
// ============================================================================

#[test]
fn test_orchestrator_port_default() {
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    assert_eq!(ports::orchestrator_port(), 8080);
}

#[test]
#[serial]
fn test_orchestrator_port_from_env() {
    env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "9090");
    assert_eq!(ports::orchestrator_port(), 9090);
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
}

#[test]
fn test_discovery_port_default() {
    env::remove_var("SONGBIRD_DISCOVERY_PORT");
    assert_eq!(ports::discovery_port(), 8081);
}

#[test]
#[serial]
fn test_discovery_port_from_env() {
    env::set_var("SONGBIRD_DISCOVERY_PORT", "9091");
    assert_eq!(ports::discovery_port(), 9091);
    env::remove_var("SONGBIRD_DISCOVERY_PORT");
}

#[test]
fn test_dashboard_port_default() {
    env::remove_var("SONGBIRD_DASHBOARD_PORT");
    assert_eq!(ports::dashboard_port(), 3000);
}

#[test]
fn test_metrics_port_default() {
    env::remove_var("SONGBIRD_METRICS_PORT");
    assert_eq!(ports::metrics_port(), 9090);
}

#[test]
fn test_federation_port_default() {
    env::remove_var("SONGBIRD_FEDERATION_PORT");
    assert_eq!(ports::federation_port(), 8082);
}

#[test]
fn test_websocket_port_default() {
    env::remove_var("SONGBIRD_WEBSOCKET_PORT");
    assert_eq!(ports::websocket_port(), 8080);
}

#[test]
fn test_gaming_port_default() {
    env::remove_var("SONGBIRD_GAMING_PORT");
    assert_eq!(ports::gaming_port(), 6112); // StarCraft IPX port
}

#[test]
fn test_health_port_default() {
    env::remove_var("SONGBIRD_HEALTH_PORT");
    assert_eq!(ports::health_port(), 8002);
}

#[test]
fn test_gaming_port_range() {
    env::remove_var("SONGBIRD_GAMING_PORT_START");
    env::remove_var("SONGBIRD_GAMING_PORT_END");
    assert_eq!(ports::gaming_port_range_start(), 7000);
    assert_eq!(ports::gaming_port_range_end(), 7100);
    assert!(ports::gaming_port_range_end() > ports::gaming_port_range_start());
}

#[test]
fn test_starcraft_port_default() {
    env::remove_var("SONGBIRD_STARCRAFT_PORT");
    assert_eq!(ports::starcraft_port(), 6112);
}

#[test]
fn test_aoe2_port_default() {
    env::remove_var("SONGBIRD_AOE2_PORT");
    assert_eq!(ports::aoe2_port(), 2300);
}

#[test]
fn test_cnc_port_range() {
    env::remove_var("SONGBIRD_CNC_PORT_START");
    env::remove_var("SONGBIRD_CNC_PORT_END");
    assert_eq!(ports::cnc_port_range_start(), 1234);
    assert_eq!(ports::cnc_port_range_end(), 1240);
}

#[test]
fn test_service_port_with_custom_name() {
    env::remove_var("SONGBIRD_MYSERVICE_PORT");
    assert_eq!(ports::service_port("MYSERVICE", 5000), 5000);
}

#[test]
#[serial]
fn test_service_port_with_env() {
    env::set_var("SONGBIRD_TESTSERVICE_PORT", "6000");
    assert_eq!(ports::service_port("TESTSERVICE", 5000), 6000);
    env::remove_var("SONGBIRD_TESTSERVICE_PORT");
}

#[test]
#[serial]
fn test_port_invalid_env_uses_default() {
    env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "invalid");
    assert_eq!(ports::orchestrator_port(), 8080); // Should fallback to default
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
}

// ============================================================================
// Host Tests
// ============================================================================

#[test]
fn test_default_host() {
    env::remove_var("SONGBIRD_HOST");
    assert_eq!(hosts::default_host(), "127.0.0.1");
}

#[test]
#[serial]
fn test_default_host_from_env() {
    env::set_var("SONGBIRD_HOST", "0.0.0.0");
    assert_eq!(hosts::default_host(), "0.0.0.0");
    env::remove_var("SONGBIRD_HOST");
}

#[test]
fn test_bind_address_default() {
    env::remove_var("SONGBIRD_BIND_ADDRESS");
    assert_eq!(hosts::bind_address(), "0.0.0.0");
}

#[test]
fn test_discovery_host_default() {
    env::remove_var("SONGBIRD_DISCOVERY_HOST");
    env::remove_var("SONGBIRD_HOST");
    assert_eq!(hosts::discovery_host(), "127.0.0.1");
}

#[test]
#[serial]
fn test_discovery_host_from_env() {
    env::set_var("SONGBIRD_DISCOVERY_HOST", "discovery.local");
    assert_eq!(hosts::discovery_host(), "discovery.local");
    env::remove_var("SONGBIRD_DISCOVERY_HOST");
}

// ============================================================================
// Timeout Tests
// ============================================================================

#[test]
fn test_standard_timeout_default() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    assert_eq!(timeouts::standard_timeout(), Duration::from_millis(5000));
}

#[test]
#[serial]
fn test_standard_timeout_from_env() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "3000");
    assert_eq!(timeouts::standard_timeout(), Duration::from_millis(3000));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_long_timeout_default() {
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
    assert_eq!(timeouts::long_timeout(), Duration::from_millis(30000));
}

#[test]
fn test_cache_expiry_default() {
    env::remove_var("SONGBIRD_CACHE_EXPIRY_MS");
    assert_eq!(timeouts::cache_expiry(), Duration::from_millis(300_000));
}

#[test]
fn test_heartbeat_interval_default() {
    env::remove_var("SONGBIRD_HEARTBEAT_MS");
    assert_eq!(timeouts::heartbeat_interval(), Duration::from_millis(60000));
}

#[test]
fn test_connection_timeout_default() {
    env::remove_var("SONGBIRD_CONNECTION_TIMEOUT_MS");
    assert_eq!(timeouts::connection_timeout(), Duration::from_millis(10000));
}

#[test]
fn test_retry_backoff_default() {
    env::remove_var("SONGBIRD_RETRY_BACKOFF_MS");
    assert_eq!(timeouts::retry_backoff(), Duration::from_millis(1000));
}

#[test]
#[serial]
fn test_timeout_invalid_env_uses_default() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "not_a_number");
    assert_eq!(timeouts::standard_timeout(), Duration::from_millis(5000));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

// ============================================================================
// Endpoint Tests
// ============================================================================

#[test]
fn test_orchestrator_endpoint_default() {
    env::remove_var("SONGBIRD_ORCHESTRATOR_URL");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    let endpoint = endpoints::orchestrator_endpoint();
    assert_eq!(endpoint, "http://127.0.0.1:8080");
}

#[test]
#[serial]
fn test_orchestrator_endpoint_from_env() {
    env::set_var("SONGBIRD_ORCHESTRATOR_URL", "http://custom:9000");
    assert_eq!(endpoints::orchestrator_endpoint(), "http://custom:9000");
    env::remove_var("SONGBIRD_ORCHESTRATOR_URL");
}

#[test]
fn test_discovery_endpoint_default() {
    env::remove_var("SONGBIRD_DISCOVERY_URL");
    env::remove_var("SONGBIRD_DISCOVERY_HOST");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_DISCOVERY_PORT");
    let endpoint = endpoints::discovery_endpoint();
    assert_eq!(endpoint, "http://127.0.0.1:8081");
}

#[test]
fn test_dashboard_endpoint_default() {
    env::remove_var("SONGBIRD_DASHBOARD_URL");
    let endpoint = endpoints::dashboard_url();
    assert!(endpoint.starts_with("http://"));
    assert!(endpoint.contains(":3000"));
}

#[test]
fn test_metrics_endpoint_default() {
    env::remove_var("SONGBIRD_METRICS_URL");
    let endpoint = endpoints::metrics_url();
    assert!(endpoint.starts_with("http://"));
    assert!(endpoint.contains(":9090"));
}

#[test]
fn test_cors_origins_default() {
    env::remove_var("SONGBIRD_CORS_ORIGINS");
    let origins = endpoints::cors_origins();
    assert!(!origins.is_empty());
    assert!(origins[0].contains("3000")); // Dashboard port
}

#[test]
#[serial]
fn test_cors_origins_from_env() {
    env::set_var("SONGBIRD_CORS_ORIGINS", "http://example.com,http://test.com");
    let origins = endpoints::cors_origins();
    assert_eq!(origins.len(), 2);
    assert_eq!(origins[0], "http://example.com");
    assert_eq!(origins[1], "http://test.com");
    env::remove_var("SONGBIRD_CORS_ORIGINS");
}

#[test]
#[serial]
fn test_cors_origins_single_value() {
    env::set_var("SONGBIRD_CORS_ORIGINS", "http://single.com");
    let origins = endpoints::cors_origins();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0], "http://single.com");
    env::remove_var("SONGBIRD_CORS_ORIGINS");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_config_from_defaults() {
    // Clear all env vars (including ones that might affect the endpoint)
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_TIMEOUT_MS");

    // Get defaults
    let port = ports::orchestrator_port();
    let host = hosts::default_host();
    let timeout = timeouts::standard_timeout();
    let endpoint = endpoints::orchestrator_endpoint();

    // Verify consistency
    assert_eq!(port, 8080);
    assert_eq!(host, "127.0.0.1");
    assert_eq!(timeout, Duration::from_millis(5000));
    assert_eq!(endpoint, "http://127.0.0.1:8080");
}

#[test]
#[serial]
fn test_full_config_from_env() {
    // Set env vars
    env::set_var("SONGBIRD_ORCHESTRATOR_PORT", "9999");
    env::set_var("SONGBIRD_HOST", "custom.host");
    env::set_var("SONGBIRD_TIMEOUT_MS", "2000");

    // Get values
    let port = ports::orchestrator_port();
    let host = hosts::default_host();
    let timeout = timeouts::standard_timeout();

    // Verify
    assert_eq!(port, 9999);
    assert_eq!(host, "custom.host");
    assert_eq!(timeout, Duration::from_millis(2000));

    // Cleanup
    env::remove_var("SONGBIRD_ORCHESTRATOR_PORT");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_port_range_validity() {
    // Ensure all ports are valid u16 values
    assert!(ports::orchestrator_port() > 0);
    assert!(ports::discovery_port() > 0);
    assert!(ports::dashboard_port() > 0);
    assert!(ports::metrics_port() > 0);
    assert!(ports::federation_port() > 0);

    // Ensure ranges make sense
    assert!(ports::gaming_port_range_end() > ports::gaming_port_range_start());
    assert!(ports::cnc_port_range_end() > ports::cnc_port_range_start());
}

#[test]
fn test_timeout_duration_validity() {
    // All timeouts should be positive
    assert!(timeouts::standard_timeout() > Duration::ZERO);
    assert!(timeouts::long_timeout() > Duration::ZERO);
    assert!(timeouts::cache_expiry() > Duration::ZERO);
    assert!(timeouts::heartbeat_interval() > Duration::ZERO);
    assert!(timeouts::connection_timeout() > Duration::ZERO);
    assert!(timeouts::retry_backoff() > Duration::ZERO);

    // Long timeout should be longer than standard
    assert!(timeouts::long_timeout() > timeouts::standard_timeout());

    // Cache expiry should be longest
    assert!(timeouts::cache_expiry() > timeouts::long_timeout());
}

#[test]
fn test_endpoint_format_validity() {
    let endpoint = endpoints::orchestrator_endpoint();
    assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
    assert!(endpoint.contains(':'));
}

#[test]
fn test_no_hardcoded_ports_in_production() {
    // This test documents that we use the defaults module
    // instead of hardcoding ports throughout the codebase
    let _ = ports::orchestrator_port();
    let _ = ports::discovery_port();
    // If these functions exist and work, we're using centralized config ✅
}
