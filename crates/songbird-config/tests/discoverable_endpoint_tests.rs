// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for the Discoverable Endpoint System
//!
//! Covers: DiscoverableEndpoint constructors, EndpointSpec methods,
//! parse_endpoint, discovery, resolve_named_port, and edge cases.

use songbird_config::discoverable_endpoint::*;
use std::sync::Mutex;

/// File-local mutex to serialize tests that modify process-wide env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ============================================================
// Constructor Tests
// ============================================================

#[test]
fn test_from_env_creates_valid_endpoint() {
    let ep = DiscoverableEndpoint::from_env("MY_SERVICE");
    assert!(ep.cache_discovery);
    assert!(ep.dev_fallback.is_some());
    let fallback = ep.dev_fallback.unwrap();
    assert_eq!(fallback.host, "localhost");
    assert_eq!(fallback.port, 8080);
}

#[test]
fn test_from_k8s_service() {
    let ep = DiscoverableEndpoint::from_k8s_service("my-service", "default", 8080);
    assert!(ep.cache_discovery);
    assert!(ep.dev_fallback.is_some());
    let fallback = ep.dev_fallback.unwrap();
    assert!(fallback.host.contains("my-service"));
    assert!(fallback.host.contains("default"));
    assert!(fallback.host.contains("svc.cluster.local"));
    assert_eq!(fallback.port, 8080);
}

#[test]
fn test_from_consul_service() {
    let ep = DiscoverableEndpoint::from_consul_service("my-consul-service");
    assert!(ep.cache_discovery);
    assert!(ep.dev_fallback.is_none()); // Consul services don't have dev fallbacks
}

#[test]
fn test_default_discoverable_endpoint() {
    let ep = DiscoverableEndpoint::default();
    assert!(ep.cache_discovery);
    assert!(ep.dev_fallback.is_some());
}

// ============================================================
// EndpointSpec Tests
// ============================================================

#[test]
fn test_endpoint_spec_to_url_with_all_fields() {
    let spec = EndpointSpec {
        host: "example.com".to_string(),
        port: 443,
        protocol: Some("https".to_string()),
        path: Some("/api/v2".to_string()),
    };
    assert_eq!(spec.to_url(), "https://example.com:443/api/v2");
}

#[test]
fn test_endpoint_spec_to_url_without_protocol() {
    let spec = EndpointSpec {
        host: "localhost".to_string(),
        port: 8080,
        protocol: None,
        path: None,
    };
    assert_eq!(spec.to_url(), "http://localhost:8080");
}

#[test]
fn test_endpoint_spec_to_url_without_path() {
    let spec = EndpointSpec {
        host: "10.0.0.1".to_string(),
        port: 3000,
        protocol: Some("grpc".to_string()),
        path: None,
    };
    assert_eq!(spec.to_url(), "grpc://10.0.0.1:3000");
}

#[test]
fn test_endpoint_spec_to_socket_addr_with_ip() {
    let spec = EndpointSpec {
        host: "127.0.0.1".to_string(),
        port: 8080,
        protocol: None,
        path: None,
    };
    let addr = spec.to_socket_addr().unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_endpoint_spec_to_socket_addr_with_ipv6() {
    let spec = EndpointSpec {
        host: "::1".to_string(),
        port: 9090,
        protocol: None,
        path: None,
    };
    let addr = spec.to_socket_addr().unwrap();
    assert!(addr.ip().is_loopback());
    assert_eq!(addr.port(), 9090);
}

#[test]
fn test_endpoint_spec_to_socket_addr_with_hostname_fails() {
    let spec = EndpointSpec {
        host: "example.com".to_string(),
        port: 80,
        protocol: None,
        path: None,
    };
    let result = spec.to_socket_addr();
    assert!(result.is_err());
}

#[test]
fn test_endpoint_spec_serialization_roundtrip() {
    let spec = EndpointSpec {
        host: "myhost".to_string(),
        port: 5000,
        protocol: Some("https".to_string()),
        path: Some("/v1".to_string()),
    };
    let json = serde_json::to_string(&spec).unwrap();
    let deserialized: EndpointSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(spec.host, deserialized.host);
    assert_eq!(spec.port, deserialized.port);
    assert_eq!(spec.protocol, deserialized.protocol);
    assert_eq!(spec.path, deserialized.path);
}

// ============================================================
// Discovery Tests (async)
// ============================================================

#[tokio::test]
async fn test_discover_from_env_var() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("TEST_DISCOVER_ENDPOINT", "http://discovered-host:9999/path");

    let ep = DiscoverableEndpoint::from_env("TEST_DISCOVER_ENDPOINT");
    let result = ep.discover().await;
    assert!(result.is_ok(), "Should discover from env var");
    let spec = result.unwrap();
    assert_eq!(spec.host, "discovered-host");
    assert_eq!(spec.port, 9999);

    std::env::remove_var("TEST_DISCOVER_ENDPOINT");
}

#[tokio::test]
async fn test_discover_static_method() {
    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::Static {
            endpoint: EndpointSpec {
                host: "static-host".to_string(),
                port: 1234,
                protocol: Some("grpc".to_string()),
                path: None,
            },
        },
        fallback_methods: vec![],
        dev_fallback: None,
        cache_discovery: false,
    };

    let result = ep.discover().await;
    assert!(result.is_ok());
    let spec = result.unwrap();
    assert_eq!(spec.host, "static-host");
    assert_eq!(spec.port, 1234);
}

#[tokio::test]
async fn test_discover_consul_returns_not_implemented() {
    let ep = DiscoverableEndpoint::from_consul_service("test-service");
    let result = ep.discover().await;
    // Consul is not yet implemented, should fall back to env var methods or fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_falls_back_to_dev_fallback() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Set dev mode
    std::env::set_var("SONGBIRD_ENV", "development");
    std::env::remove_var("NONEXISTENT_SERVICE_ENDPOINT");
    std::env::remove_var("NONEXISTENT_SERVICE_ENDPOINT_HOST");

    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::Environment {
            var_name: "NONEXISTENT_SERVICE_ENDPOINT".to_string(),
            parser: EndpointParser::Url,
        },
        fallback_methods: vec![],
        dev_fallback: Some(EndpointSpec {
            host: "dev-host".to_string(),
            port: 5555,
            protocol: Some("http".to_string()),
            path: None,
        }),
        cache_discovery: false,
    };

    let result = ep.discover().await;
    assert!(result.is_ok(), "Should fall back to dev endpoint");
    let spec = result.unwrap();
    assert_eq!(spec.host, "dev-host");
    assert_eq!(spec.port, 5555);

    std::env::remove_var("SONGBIRD_ENV");
}

#[tokio::test]
async fn test_discover_all_methods_fail_no_dev_fallback() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("SONGBIRD_ENV");
    std::env::remove_var("RUST_ENV");
    std::env::remove_var("TOTALLY_NONEXISTENT_VAR");

    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::Environment {
            var_name: "TOTALLY_NONEXISTENT_VAR".to_string(),
            parser: EndpointParser::Url,
        },
        fallback_methods: vec![],
        dev_fallback: None,
        cache_discovery: false,
    };

    let result = ep.discover().await;
    assert!(result.is_err(), "Should fail when all methods exhausted");
}

// ============================================================
// PortSpec Tests
// ============================================================

#[test]
fn test_port_spec_named_serialization() {
    let port = PortSpec::Named("http".to_string());
    let json = serde_json::to_string(&port).unwrap();
    let deserialized: PortSpec = serde_json::from_str(&json).unwrap();
    match deserialized {
        PortSpec::Named(name) => assert_eq!(name, "http"),
        _ => panic!("Expected Named variant"),
    }
}

#[test]
fn test_port_spec_number_serialization() {
    let port = PortSpec::Number(8080);
    let json = serde_json::to_string(&port).unwrap();
    let deserialized: PortSpec = serde_json::from_str(&json).unwrap();
    match deserialized {
        PortSpec::Number(n) => assert_eq!(n, 8080),
        _ => panic!("Expected Number variant"),
    }
}

#[test]
fn test_port_spec_environment_serialization() {
    let port = PortSpec::Environment("MY_PORT".to_string());
    let json = serde_json::to_string(&port).unwrap();
    let deserialized: PortSpec = serde_json::from_str(&json).unwrap();
    match deserialized {
        PortSpec::Environment(var) => assert_eq!(var, "MY_PORT"),
        _ => panic!("Expected Environment variant"),
    }
}

// ============================================================
// DiscoveryMethod Serialization Tests
// ============================================================

#[test]
fn test_discovery_method_environment_serialization() {
    let method = DiscoveryMethod::Environment {
        var_name: "MY_ENDPOINT".to_string(),
        parser: EndpointParser::Url,
    };
    let json = serde_json::to_string(&method).unwrap();
    let deserialized: DiscoveryMethod = serde_json::from_str(&json).unwrap();
    match deserialized {
        DiscoveryMethod::Environment {
            var_name,
            ..
        } => assert_eq!(var_name, "MY_ENDPOINT"),
        _ => panic!("Expected Environment variant"),
    }
}

#[test]
fn test_discovery_method_static_serialization() {
    let method = DiscoveryMethod::Static {
        endpoint: EndpointSpec {
            host: "host".to_string(),
            port: 80,
            protocol: None,
            path: None,
        },
    };
    let json = serde_json::to_string(&method).unwrap();
    assert!(json.contains("host"));
}

#[test]
fn test_discoverable_endpoint_debug() {
    let ep = DiscoverableEndpoint::from_env("TEST");
    let debug = format!("{:?}", ep);
    assert!(debug.contains("DiscoverableEndpoint"));
}
