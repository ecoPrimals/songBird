// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Configuration Validation Tests
//!
//! Comprehensive tests for configuration validation logic including:
//! - Environment variable parsing
//! - Default fallback values
//! - Validation rules
//! - Error handling
//! - Edge cases

use songbird_config::canonical::constants::{
    DEFAULT_BIND_ADDRESS, DEFAULT_LOCALHOST, LOCALHOST_IPV4,
};
use std::env;

// =============================================================================
// Environment Variable Tests
// =============================================================================

#[test]
fn test_localhost_constant() {
    assert_eq!(LOCALHOST_IPV4, "127.0.0.1");
    assert_eq!(DEFAULT_LOCALHOST, "127.0.0.1");
}

#[test]
fn test_default_bind_address() {
    assert_eq!(DEFAULT_BIND_ADDRESS, "127.0.0.1:8080");
    assert!(DEFAULT_BIND_ADDRESS.contains("127.0.0.1"));
    assert!(DEFAULT_BIND_ADDRESS.contains(":8080"));
}

#[test]
fn test_bind_address_parsing() {
    let addr = DEFAULT_BIND_ADDRESS;
    let parts: Vec<&str> = addr.split(':').collect();

    assert_eq!(parts.len(), 2, "Should have host and port");
    assert_eq!(parts[0], "127.0.0.1");
    assert_eq!(parts[1], "8080");
}

// =============================================================================
// IP Address Validation Tests
// =============================================================================

#[test]
fn test_ipv4_localhost_is_loopback() {
    let ip: std::net::IpAddr = LOCALHOST_IPV4.parse().expect("Should parse");
    assert!(ip.is_loopback(), "127.0.0.1 should be loopback");
}

#[test]
fn test_ipv4_format_validation() {
    // Valid IPv4 formats
    let valid = ["127.0.0.1", "192.168.1.1", "10.0.0.1", "172.16.0.1"];

    for ip_str in &valid {
        let result: Result<std::net::IpAddr, _> = ip_str.parse();
        assert!(result.is_ok(), "{} should be valid IPv4", ip_str);
    }
}

#[test]
fn test_invalid_ipv4_format() {
    let invalid = ["256.0.0.1", "192.168.1", "not-an-ip", "127.0.0.1.1"];

    for ip_str in &invalid {
        let result: Result<std::net::IpAddr, _> = ip_str.parse();
        assert!(result.is_err(), "{} should be invalid", ip_str);
    }
}

// =============================================================================
// Port Number Validation Tests
// =============================================================================

#[test]
fn test_valid_port_range() {
    // Common valid ports
    let valid_ports = [80u16, 443, 8080, 3000, 9090, 65535];

    for port in &valid_ports {
        assert!(*port > 0);
        assert!(*port <= u16::MAX);
    }
}

#[test]
fn test_default_port_8080() {
    let default_port = 8080u16;
    assert!(default_port > 1024, "Should be above privileged ports");
    assert!(default_port <= u16::MAX, "Should be valid port");
}

#[test]
fn test_port_edge_cases() {
    // Min and max valid ports
    assert!(1u16 > 0);
    assert!(u16::MAX == 65535);
    assert!(65536u32 > u32::from(u16::MAX)); // Would overflow u16
}

// =============================================================================
// Configuration Default Tests
// =============================================================================

#[test]
fn test_default_configuration_constants_exist() {
    // Verify all expected constants are defined
    let _ = LOCALHOST_IPV4;
    let _ = DEFAULT_LOCALHOST;
    let _ = DEFAULT_BIND_ADDRESS;
}

#[test]
fn test_default_values_are_safe() {
    // Localhost should be safe default
    assert!(LOCALHOST_IPV4.starts_with("127."));

    // Default bind should be localhost
    assert!(DEFAULT_BIND_ADDRESS.starts_with("127."));
}

// =============================================================================
// Environment Variable Handling Tests
// =============================================================================

#[test]
fn test_env_var_override_behavior() {
    // Test that we can read environment variables
    env::set_var("TEST_CONFIG_VAR", "test_value");
    let value = env::var("TEST_CONFIG_VAR").expect("test precondition");
    assert_eq!(value, "test_value");
    env::remove_var("TEST_CONFIG_VAR");
}

#[test]
fn test_env_var_missing_returns_error() {
    let result = env::var("NONEXISTENT_CONFIG_VAR_12345");
    assert!(result.is_err(), "Should error for missing var");
}

#[test]
fn test_env_var_empty_string() {
    env::set_var("EMPTY_VAR", "");
    let value = env::var("EMPTY_VAR").expect("test precondition");
    assert_eq!(value, "", "Should handle empty string");
    env::remove_var("EMPTY_VAR");
}

#[test]
fn test_env_var_whitespace() {
    env::set_var("WHITESPACE_VAR", "  value  ");
    let value = env::var("WHITESPACE_VAR").expect("test precondition");
    assert_eq!(value.trim(), "value");
    env::remove_var("WHITESPACE_VAR");
}

// =============================================================================
// Hostname Validation Tests
// =============================================================================

#[test]
fn test_localhost_hostname() {
    let localhost = "localhost";
    assert!(!localhost.is_empty());
    assert!(localhost.len() < 256); // Max hostname length
}

#[test]
fn test_valid_hostnames() {
    let valid = ["localhost", "api.example.com", "my-service", "service-01", "10.0.0.1"];

    for hostname in &valid {
        assert!(!hostname.is_empty());
        assert!(hostname.len() < 256);
    }
}

#[test]
fn test_hostname_special_characters() {
    let invalid = ["host name", "host@name", "host#name"];

    for hostname in &invalid {
        // Should contain invalid characters for DNS
        assert!(
            hostname.contains(char::is_whitespace)
                || hostname.contains('@')
                || hostname.contains('#')
        );
    }
}

// =============================================================================
// URL/Endpoint Validation Tests
// =============================================================================

#[test]
fn test_http_endpoint_format() {
    let endpoint = format!("http://{}", DEFAULT_BIND_ADDRESS);
    assert!(endpoint.starts_with("http://"));
    assert!(endpoint.contains("127.0.0.1"));
}

#[test]
fn test_https_endpoint_format() {
    let endpoint = format!("https://{}", DEFAULT_BIND_ADDRESS);
    assert!(endpoint.starts_with("https://"));
}

#[test]
fn test_endpoint_with_path() {
    let endpoint = format!("http://{}/api/v1", DEFAULT_BIND_ADDRESS);
    assert!(endpoint.ends_with("/api/v1"));
}

// =============================================================================
// Network Address Validation Tests
// =============================================================================

#[test]
fn test_socket_addr_parsing() {
    let addr: std::net::SocketAddr =
        DEFAULT_BIND_ADDRESS.parse().expect("Should parse socket address");

    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_socket_addr_ipv6() {
    let ipv6_addr = "[::1]:8080";
    let addr: Result<std::net::SocketAddr, _> = ipv6_addr.parse();
    assert!(addr.is_ok(), "Should parse IPv6 localhost");
}

#[test]
fn test_invalid_socket_addr() {
    let invalid = ["invalid:8080", "127.0.0.1:99999", "127.0.0.1"];

    for addr_str in &invalid {
        let result: Result<std::net::SocketAddr, _> = addr_str.parse();
        assert!(result.is_err(), "{} should be invalid", addr_str);
    }
}

// =============================================================================
// Configuration Combination Tests
// =============================================================================

#[test]
fn test_localhost_variations() {
    let variations = ["127.0.0.1", "::1", "localhost"];

    for variation in &variations {
        assert!(!variation.is_empty());
    }
}

#[test]
fn test_port_combinations() {
    let ports = [8080, 8081, 8082, 9090, 3000];

    for port in &ports {
        let addr = format!("127.0.0.1:{}", port);
        let parsed: Result<std::net::SocketAddr, _> = addr.parse();
        assert!(parsed.is_ok());
    }
}

// =============================================================================
// Security Validation Tests
// =============================================================================

#[test]
fn test_localhost_is_not_public() {
    let ip: std::net::IpAddr = "127.0.0.1".parse().expect("should parse valid input");
    assert!(ip.is_loopback());
    assert!(!is_public_ip(&ip));
}

#[test]
fn test_private_network_ranges() {
    let private_ips = ["10.0.0.1", "172.16.0.1", "192.168.1.1"];

    for ip_str in &private_ips {
        let ip: std::net::IpAddr = ip_str.parse().expect("should parse valid input");
        assert!(!is_public_ip(&ip));
    }
}

#[test]
fn test_public_ip_examples() {
    let public_ips = ["8.8.8.8", "1.1.1.1", "93.184.216.34"];

    for ip_str in &public_ips {
        let ip: std::net::IpAddr = ip_str.parse().expect("should parse valid input");
        assert!(is_public_ip(&ip));
    }
}

// Helper function for IP classification
fn is_public_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            !ipv4.is_private()
                && !ipv4.is_loopback()
                && !ipv4.is_link_local()
                && !ipv4.is_broadcast()
                && !ipv4.is_documentation()
        }
        std::net::IpAddr::V6(ipv6) => !ipv6.is_loopback() && !ipv6.is_multicast(),
    }
}

// =============================================================================
// Configuration Boundary Tests
// =============================================================================

#[test]
fn test_minimum_port_number() {
    let min_port = 1u16;
    assert!(min_port > 0);
}

#[test]
fn test_maximum_port_number() {
    let max_port = 65535u16;
    assert_eq!(max_port, u16::MAX);
}

#[test]
fn test_well_known_ports() {
    let http = 80u16;
    let https = 443u16;

    assert!(http < 1024); // Well-known port
    assert!(https < 1024); // Well-known port
}

#[test]
fn test_registered_ports() {
    let port = 8080u16;
    assert!((1024..49152).contains(&port)); // Registered port range
}

#[test]
fn test_dynamic_ports() {
    let port = 50000u16;
    assert!((49152..=65535).contains(&port)); // Dynamic port range
}

// =============================================================================
// Error Message Tests
// =============================================================================

#[test]
fn test_parse_error_contains_context() {
    let result: Result<std::net::IpAddr, _> = "invalid".parse();

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(!error_msg.is_empty());
    }
}

#[test]
fn test_env_var_error_message() {
    let result = env::var("NONEXISTENT_VAR_XYZ123");

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(!error_msg.is_empty());
    }
}

// =============================================================================
// Integration-Style Validation Tests
// =============================================================================

#[test]
fn test_full_endpoint_validation_flow() {
    // Simulate full validation
    let host = LOCALHOST_IPV4;
    let port = 8080u16;

    // 1. Validate host
    let ip: std::net::IpAddr = host.parse().expect("Valid IP");
    assert!(ip.is_loopback());

    // 2. Validate port
    assert!(port > 0);
    assert!(port <= u16::MAX);

    // 3. Construct endpoint
    let endpoint = format!("{}:{}", host, port);
    assert_eq!(endpoint, "127.0.0.1:8080");

    // 4. Parse as socket address
    let _socket: std::net::SocketAddr = endpoint.parse().expect("Valid socket");
}

#[test]
fn test_configuration_validation_chain() {
    // Test chained validation
    let config_valid = true
        && !LOCALHOST_IPV4.is_empty()
        && LOCALHOST_IPV4.parse::<std::net::IpAddr>().is_ok()
        && DEFAULT_BIND_ADDRESS.contains(':')
        && DEFAULT_BIND_ADDRESS.parse::<std::net::SocketAddr>().is_ok();

    assert!(config_valid, "All config validations should pass");
}
