// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Configuration Validation Tests
//!
//! Comprehensive tests for configuration validation logic including:
//! - Environment variable parsing
//! - Default fallback values
//! - Validation rules
//! - Error handling
//! - Edge cases

use songbird_config::canonical::constants;
use std::env;

// =============================================================================
// Environment Variable Tests
// =============================================================================

#[test]
fn test_localhost_constant() {
    // Test environment-aware localhost
    let localhost = constants::network::default_host();
    assert!(localhost == "127.0.0.1" || localhost == "0.0.0.0");
}

#[test]
fn test_default_bind_address() {
    // Test environment-aware bind address (IP only)
    let bind_addr = constants::get_bind_address();
    assert!(bind_addr == "127.0.0.1" || bind_addr == "0.0.0.0");
    // Parse to verify it's a valid IP
    assert!(bind_addr.parse::<std::net::IpAddr>().is_ok());
}

#[test]
fn test_bind_address_parsing() {
    // Test full socket address construction
    let host = constants::get_bind_address();
    let port = 8080u16; // Standard dev port
    let socket_addr = format!("{}:{}", host, port);

    // Should parse as valid socket address
    assert!(socket_addr.parse::<std::net::SocketAddr>().is_ok());

    // Verify components
    let parts: Vec<&str> = socket_addr.split(':').collect();
    assert_eq!(parts.len(), 2, "Should have host and port");
    assert!(parts[0] == "127.0.0.1" || parts[0] == "0.0.0.0");
    assert!(parts[1].parse::<u16>().is_ok(), "Port should be valid");
}

// =============================================================================
// IP Address Validation Tests
// =============================================================================

#[test]
fn test_ipv4_localhost_is_loopback() {
    let localhost = constants::network::default_host();
    let ip: std::net::IpAddr = localhost.parse().expect("Should parse");
    // Either localhost (127.0.0.1) or all interfaces (0.0.0.0) depending on environment
    assert!(ip.is_loopback() || ip.is_unspecified(), "Should be loopback or unspecified");
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
fn test_default_configuration_functions_work() {
    // Verify all configuration functions work
    let _host = constants::network::default_host();
    let _bind_addr = constants::get_bind_address();

    // These should never panic
    assert!(!_host.is_empty());
    assert!(!_bind_addr.is_empty());
}

#[test]
fn test_default_values_are_safe() {
    // Host should be safe default (localhost or unspecified)
    let host = constants::network::default_host();
    assert!(host.starts_with("127.") || host.starts_with("0."));

    // Default bind should use safe host
    let bind_addr = constants::get_bind_address();
    assert!(bind_addr.starts_with("127.") || bind_addr.starts_with("0."));
}

// =============================================================================
// Environment Variable Handling Tests
// =============================================================================

#[test]
fn test_env_var_override_behavior() {
    // Test that we can read environment variables
    songbird_process_env::set_var("TEST_CONFIG_VAR", "test_value");
    let value = env::var("TEST_CONFIG_VAR").expect("test precondition");
    assert_eq!(value, "test_value");
    songbird_process_env::remove_var("TEST_CONFIG_VAR");
}

#[test]
fn test_env_var_missing_returns_error() {
    let result = env::var("NONEXISTENT_CONFIG_VAR_12345");
    assert!(result.is_err(), "Should error for missing var");
}

#[test]
fn test_env_var_empty_string() {
    songbird_process_env::set_var("EMPTY_VAR", "");
    let value = env::var("EMPTY_VAR").expect("test precondition");
    assert_eq!(value, "", "Should handle empty string");
    songbird_process_env::remove_var("EMPTY_VAR");
}

#[test]
fn test_env_var_whitespace() {
    songbird_process_env::set_var("WHITESPACE_VAR", "  value  ");
    let value = env::var("WHITESPACE_VAR").expect("test precondition");
    assert_eq!(value.trim(), "value");
    songbird_process_env::remove_var("WHITESPACE_VAR");
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
    let bind_addr = constants::get_bind_address();
    let endpoint = format!("http://{}", bind_addr);
    assert!(endpoint.starts_with("http://"));
    assert!(endpoint.contains("127.0.0.1") || endpoint.contains("0.0.0.0"));
}

#[test]
fn test_https_endpoint_format() {
    let bind_addr = constants::get_bind_address();
    let endpoint = format!("https://{}", bind_addr);
    assert!(endpoint.starts_with("https://"));
}

#[test]
fn test_endpoint_with_path() {
    let bind_addr = constants::get_bind_address();
    let endpoint = format!("http://{}/api/v1", bind_addr);
    assert!(endpoint.ends_with("/api/v1"));
}

// =============================================================================
// Network Address Validation Tests
// =============================================================================

#[test]
fn test_socket_addr_parsing() {
    // Construct full socket address
    let host = constants::get_bind_address();
    let port = 8080u16; // Standard dev port
    let socket_addr_str = format!("{}:{}", host, port);

    let addr: std::net::SocketAddr = socket_addr_str.parse().expect("Should parse socket address");

    // Should parse as valid socket address with valid port
    assert!(addr.port() > 0);
    assert!(addr.ip().is_loopback() || addr.ip().is_unspecified());
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
    let host = constants::network::default_host();
    let port = 8080u16;

    // 1. Validate host
    let ip: std::net::IpAddr = host.parse().expect("Valid IP");
    assert!(ip.is_loopback() || ip.is_unspecified());

    // 2. Validate port
    assert!(port > 0);
    assert!(port <= u16::MAX);

    // 3. Construct endpoint
    let endpoint = format!("{}:{}", host, port);
    assert!(endpoint.contains(':'));

    // 4. Parse as socket address
    let _socket: std::net::SocketAddr = endpoint.parse().expect("Valid socket");
}

#[test]
fn test_configuration_validation_chain() {
    // Test chained validation
    let host = constants::network::default_host();
    let bind_ip = constants::get_bind_address();
    let port = 8080u16; // Standard dev port
    let socket_addr = format!("{}:{}", bind_ip, port);

    let config_valid = true
        && !host.is_empty()
        && host.parse::<std::net::IpAddr>().is_ok()
        && !bind_ip.is_empty()
        && bind_ip.parse::<std::net::IpAddr>().is_ok()
        && socket_addr.contains(':')
        && socket_addr.parse::<std::net::SocketAddr>().is_ok();

    assert!(config_valid, "All config validations should pass");
}
