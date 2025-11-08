//! Comprehensive Discoverable Endpoint Tests
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

//!
//! Tests for the zero-hardcoding endpoint discovery system.

use songbird_config::discoverable_endpoint::*;
use songbird_test_utils::test_bind_address;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// ENDPOINT SPEC TESTS
// ============================================================================

#[test]
fn test_endpoint_spec_creation() {
    let spec = EndpointSpec {
        host: "example.com".to_string(),
        port: 8080,
        protocol: Some("https".to_string()),
        path: Some("/api/v1".to_string()),
    };

    assert_eq!(spec.host, "example.com");
    assert_eq!(spec.port, 8080);
    assert_eq!(spec.protocol, Some("https".to_string()));
    assert_eq!(spec.path, Some("/api/v1".to_string()));
}

#[test]
fn test_endpoint_spec_minimal() {
    let spec = EndpointSpec {
        host: test_bind_address(),
        port: 3000,
        protocol: None,
        path: None,
    };

    assert_eq!(spec.host, test_bind_address());
    assert_eq!(spec.port, 3000);
    assert!(spec.protocol.is_none());
    assert!(spec.path.is_none());
}

#[test]
fn test_endpoint_spec_clone() -> SongbirdResult<()> {
    let spec1 = EndpointSpec {
        host: "test.local".to_string(),
        port: 9000,
        protocol: Some("http".to_string()),
        path: None,
    };

    let spec2 = spec1.clone();
    assert_eq!(spec1.host, spec2.host);
    assert_eq!(spec1.port, spec2.port);
    Ok(())
}

#[test]
fn test_endpoint_spec_debug() -> SongbirdResult<()> {
    let spec = EndpointSpec {
        host: "debug.com".to_string(),
        port: 443,
        protocol: Some("https".to_string()),
        path: Some("/".to_string()),
    };

    let debug_str = format!("{spec:?}");
    assert!(debug_str.contains("EndpointSpec"));
    Ok(())
}

#[test]
fn test_endpoint_spec_serialization() -> SongbirdResult<()> {
    let spec = EndpointSpec {
        host: "serialize.test".to_string(),
        port: 8443,
        protocol: Some("https".to_string()),
        path: Some("/v1".to_string()),
    };

    let json = serde_json::to_string(&spec)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: EndpointSpec =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(deserialized.host, spec.host);
    assert_eq!(deserialized.port, spec.port);
    Ok(())
}

// ============================================================================
// PORT SPEC TESTS
// ============================================================================

#[test]
fn test_port_spec_named() {
    let port = PortSpec::Named("http".to_string());

    if let PortSpec::Named(name) = port {
        assert_eq!(name, "http");
    } else {
        panic!("Expected Named port spec");
    }
}

#[test]
fn test_port_spec_number() {
    let port = PortSpec::Number(8080);

    if let PortSpec::Number(num) = port {
        assert_eq!(num, 8080);
    } else {
        panic!("Expected Number port spec");
    }
}

#[test]
fn test_port_spec_environment() -> SongbirdResult<()> {
    let port = PortSpec::Environment("SERVICE_PORT".to_string());

    if let PortSpec::Environment(var) = port {
        assert_eq!(var, "SERVICE_PORT");
    } else {
        panic!("Expected Environment port spec");
    }
    Ok(())
}

#[test]
fn test_port_spec_clone() -> SongbirdResult<()> {
    let port1 = PortSpec::Number(9090);
    let port2 = port1;

    assert!(matches!(port2, PortSpec::Number(9090)));
    Ok(())
}

#[test]
fn test_port_spec_debug() -> SongbirdResult<()> {
    let port = PortSpec::Named("https".to_string());
    let debug_str = format!("{port:?}");
    assert!(debug_str.contains("Named"));
    Ok(())
}

#[test]
fn test_port_spec_serialization() -> SongbirdResult<()> {
    let port = PortSpec::Number(3000);
    let json = serde_json::to_string(&port)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: PortSpec =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert!(matches!(deserialized, PortSpec::Number(3000)));
    Ok(())
}

// ============================================================================
// ENDPOINT PARSER TESTS
// ============================================================================

#[test]
fn test_endpoint_parser_all_variants() -> SongbirdResult<()> {
    let url_parser = EndpointParser::Url;
    let host_port_parser = EndpointParser::HostPort;
    let hostname_parser = EndpointParser::Hostname;
    let pattern_parser = EndpointParser::Pattern("custom".to_string());

    assert!(matches!(url_parser, EndpointParser::Url));
    assert!(matches!(host_port_parser, EndpointParser::HostPort));
    assert!(matches!(hostname_parser, EndpointParser::Hostname));
    assert!(matches!(pattern_parser, EndpointParser::Pattern(_)));
    Ok(())
}

#[test]
fn test_endpoint_parser_clone() -> SongbirdResult<()> {
    let parser1 = EndpointParser::Url;
    let parser2 = parser1;
    assert!(matches!(parser2, EndpointParser::Url));
    Ok(())
}

#[test]
fn test_endpoint_parser_debug() -> SongbirdResult<()> {
    let parser = EndpointParser::HostPort;
    let debug_str = format!("{parser:?}");
    assert!(debug_str.contains("HostPort"));
    Ok(())
}

#[test]
fn test_endpoint_parser_serialization() -> SongbirdResult<()> {
    let parser = EndpointParser::Url;
    let json = serde_json::to_string(&parser)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: EndpointParser =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert!(matches!(deserialized, EndpointParser::Url));
    Ok(())
}

// ============================================================================
// DISCOVERY METHOD TESTS
// ============================================================================

#[test]
fn test_discovery_method_environment() {
    let method = DiscoveryMethod::Environment {
        var_name: "API_ENDPOINT".to_string(),
        parser: EndpointParser::Url,
    };

    if let DiscoveryMethod::Environment {
        var_name,
        parser,
    } = method
    {
        assert_eq!(var_name, "API_ENDPOINT");
        assert!(matches!(parser, EndpointParser::Url));
    } else {
        panic!("Expected Environment discovery method");
    }
}

#[test]
fn test_discovery_method_dns() {
    let method = DiscoveryMethod::DnsServiceDiscovery {
        service_name: "_http._tcp.local".to_string(),
    };

    if let DiscoveryMethod::DnsServiceDiscovery {
        service_name,
    } = method
    {
        assert_eq!(service_name, "_http._tcp.local");
    } else {
        panic!("Expected DnsServiceDiscovery method");
    }
}

#[test]
fn test_discovery_method_network_probe() {
    let method = DiscoveryMethod::NetworkProbe {
        host_patterns: vec!["localhost".to_string()],
        port_range: (8000, 9000),
        health_path: "/health".to_string(),
    };

    if let DiscoveryMethod::NetworkProbe {
        host_patterns,
        port_range,
        health_path,
    } = method
    {
        assert_eq!(host_patterns.len(), 1);
        assert_eq!(port_range, (8000, 9000));
        assert_eq!(health_path, "/health");
    } else {
        panic!("Expected NetworkProbe method");
    }
}

#[test]
fn test_discovery_method_kubernetes() {
    let method = DiscoveryMethod::KubernetesService {
        service_name: "api-service".to_string(),
        namespace: "default".to_string(),
        port: PortSpec::Named("http".to_string()),
    };

    if let DiscoveryMethod::KubernetesService {
        service_name,
        namespace,
        port,
    } = method
    {
        assert_eq!(service_name, "api-service");
        assert_eq!(namespace, "default");
        assert!(matches!(port, PortSpec::Named(_)));
    } else {
        panic!("Expected KubernetesService method");
    }
}

#[test]
fn test_discovery_method_consul() {
    let method = DiscoveryMethod::ConsulService {
        service_name: "my-service".to_string(),
        consul_addr: Some("http://consul:8500".to_string()),
    };

    if let DiscoveryMethod::ConsulService {
        service_name,
        consul_addr,
    } = method
    {
        assert_eq!(service_name, "my-service");
        assert_eq!(consul_addr, Some("http://consul:8500".to_string()));
    } else {
        panic!("Expected ConsulService method");
    }
}

#[test]
fn test_discovery_method_static() {
    let spec = EndpointSpec {
        host: "static.example.com".to_string(),
        port: 443,
        protocol: Some("https".to_string()),
        path: None,
    };

    let method = DiscoveryMethod::Static {
        endpoint: spec.clone(),
    };

    if let DiscoveryMethod::Static {
        endpoint,
    } = method
    {
        assert_eq!(endpoint.host, spec.host);
    } else {
        panic!("Expected Static method");
    }
}

#[test]
fn test_discovery_method_clone() -> SongbirdResult<()> {
    let method1 = DiscoveryMethod::Environment {
        var_name: "TEST_VAR".to_string(),
        parser: EndpointParser::Url,
    };

    let method2 = method1;
    assert!(matches!(method2, DiscoveryMethod::Environment { .. }));
    Ok(())
}

#[test]
fn test_discovery_method_debug() -> SongbirdResult<()> {
    let method = DiscoveryMethod::DnsServiceDiscovery {
        service_name: "test".to_string(),
    };

    let debug_str = format!("{method:?}");
    assert!(debug_str.contains("DnsServiceDiscovery"));
    Ok(())
}

#[test]
fn test_discovery_method_serialization() -> SongbirdResult<()> {
    let method = DiscoveryMethod::Environment {
        var_name: "ENDPOINT".to_string(),
        parser: EndpointParser::Url,
    };

    let json = serde_json::to_string(&method)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: DiscoveryMethod =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert!(matches!(deserialized, DiscoveryMethod::Environment { .. }));
    Ok(())
}

// ============================================================================
// DISCOVERABLE ENDPOINT TESTS
// ============================================================================

#[test]
fn test_discoverable_endpoint_from_env() {
    let endpoint = DiscoverableEndpoint::from_env("SERVICE_ENDPOINT");

    assert!(matches!(endpoint.discovery_method, DiscoveryMethod::Environment { .. }));
    assert_eq!(endpoint.fallback_methods.len(), 2);
    assert!(endpoint.dev_fallback.is_some());
    assert!(endpoint.cache_discovery);
}

#[test]
fn test_discoverable_endpoint_from_kubernetes() {
    let endpoint = DiscoverableEndpoint::from_k8s_service("api-service", "production", 8080);

    assert!(matches!(endpoint.discovery_method, DiscoveryMethod::KubernetesService { .. }));
}

#[test]
fn test_discoverable_endpoint_from_consul() {
    let endpoint = DiscoverableEndpoint::from_consul_service("my-service");

    assert!(matches!(endpoint.discovery_method, DiscoveryMethod::ConsulService { .. }));
}

#[test]
fn test_discoverable_endpoint_with_fallbacks() {
    let endpoint = DiscoverableEndpoint::from_env("API_ENDPOINT");

    // Should have network probe fallback
    assert!(endpoint
        .fallback_methods
        .iter()
        .any(|m| matches!(m, DiscoveryMethod::NetworkProbe { .. })));
}

#[test]
fn test_discoverable_endpoint_custom() {
    let primary = DiscoveryMethod::Environment {
        var_name: "CUSTOM_ENDPOINT".to_string(),
        parser: EndpointParser::HostPort,
    };

    let fallback = DiscoveryMethod::Static {
        endpoint: EndpointSpec {
            host: "fallback.local".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            path: None,
        },
    };

    let endpoint = DiscoverableEndpoint {
        discovery_method: primary,
        fallback_methods: vec![fallback],
        dev_fallback: None,
        cache_discovery: false,
    };

    assert!(!endpoint.cache_discovery);
    assert!(endpoint.dev_fallback.is_none());
    assert_eq!(endpoint.fallback_methods.len(), 1);
}

#[test]
fn test_discoverable_endpoint_clone() -> SongbirdResult<()> {
    let endpoint1 = DiscoverableEndpoint::from_env("TEST_ENDPOINT");
    let endpoint2 = endpoint1;

    assert!(endpoint2.cache_discovery);
    Ok(())
}

#[test]
fn test_discoverable_endpoint_debug() -> SongbirdResult<()> {
    let endpoint = DiscoverableEndpoint::from_env("DEBUG_ENDPOINT");
    let debug_str = format!("{endpoint:?}");
    assert!(debug_str.contains("DiscoverableEndpoint"));
    Ok(())
}

#[test]
fn test_discoverable_endpoint_serialization() -> SongbirdResult<()> {
    let endpoint = DiscoverableEndpoint::from_env("SERIALIZE_ENDPOINT");

    let json = serde_json::to_string(&endpoint)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: DiscoverableEndpoint =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert!(deserialized.cache_discovery);
    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_discovery_workflow() {
    // Create endpoint with full configuration
    let endpoint = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::Environment {
            var_name: "PROD_API_ENDPOINT".to_string(),
            parser: EndpointParser::Url,
        },
        fallback_methods: vec![
            DiscoveryMethod::KubernetesService {
                service_name: "api".to_string(),
                namespace: "production".to_string(),
                port: PortSpec::Named("https".to_string()),
            },
            DiscoveryMethod::ConsulService {
                service_name: "api-service".to_string(),
                consul_addr: None,
            },
        ],
        dev_fallback: Some(EndpointSpec {
            host: test_bind_address(),
            port: 8080,
            protocol: Some("http".to_string()),
            path: Some("/api".to_string()),
        }),
        cache_discovery: true,
    };

    // Verify configuration
    assert!(matches!(endpoint.discovery_method, DiscoveryMethod::Environment { .. }));
    assert_eq!(endpoint.fallback_methods.len(), 2);
    assert!(endpoint.dev_fallback.is_some());

    // Verify dev fallback
    let dev_fb = endpoint.dev_fallback.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(dev_fb.host, test_bind_address());
    assert_eq!(dev_fb.port, 8080);
}

#[test]
fn test_network_probe_configuration() {
    let method = DiscoveryMethod::NetworkProbe {
        host_patterns: vec![
            "api.internal".to_string(),
            "api.local".to_string(),
            "localhost".to_string(),
        ],
        port_range: (3000, 4000),
        health_path: "/api/health".to_string(),
    };

    if let DiscoveryMethod::NetworkProbe {
        host_patterns,
        port_range,
        health_path,
    } = method
    {
        assert_eq!(host_patterns.len(), 3);
        assert_eq!(port_range.0, 3000);
        assert_eq!(port_range.1, 4000);
        assert!(health_path.starts_with("/api"));
    }
}

#[test]
fn test_multi_parser_strategy() {
    let parsers = [
        EndpointParser::Url,
        EndpointParser::HostPort,
        EndpointParser::Hostname,
        EndpointParser::Pattern("regex".to_string()),
    ];

    assert_eq!(parsers.len(), 4);
}

#[test]
fn test_endpoint_spec_variants() {
    let specs = vec![
        EndpointSpec {
            host: "http://example.com".to_string(),
            port: 80,
            protocol: Some("http".to_string()),
            path: None,
        },
        EndpointSpec {
            host: "https://secure.example.com".to_string(),
            port: 443,
            protocol: Some("https".to_string()),
            path: Some("/v2".to_string()),
        },
        EndpointSpec {
            host: "grpc.service.local".to_string(),
            port: 50051,
            protocol: Some("grpc".to_string()),
            path: None,
        },
    ];

    assert_eq!(specs.len(), 3);
    assert!(specs.iter().all(|s| s.port > 0));
}

#[test]
fn test_kubernetes_service_discovery_variations() {
    let k8s_methods = vec![
        DiscoverableEndpoint::from_k8s_service("frontend", "default", 80),
        DiscoverableEndpoint::from_k8s_service("backend", "production", 8080),
        DiscoverableEndpoint::from_k8s_service("database", "data", 5432),
    ];

    assert_eq!(k8s_methods.len(), 3);
}

#[test]
fn test_consul_service_discovery_variations() {
    let consul_methods = vec![
        DiscoverableEndpoint::from_consul_service("web"),
        DiscoverableEndpoint::from_consul_service("api"),
    ];

    assert_eq!(consul_methods.len(), 2);
}
