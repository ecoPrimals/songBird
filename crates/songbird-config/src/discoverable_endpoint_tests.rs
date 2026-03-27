// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;

#[test]
fn test_parse_url() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let spec = parse_endpoint("http://example.com:8080/api", &EndpointParser::Url)
        .map_err(|e| SongbirdError::configuration(format!("Test: URL should parse: {e}")))?;
    assert_eq!(spec.host, "example.com");
    assert_eq!(spec.port, 8080);
    assert_eq!(spec.protocol, Some("http".to_string()));
    Ok(())
}

#[test]
fn test_parse_host_port() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let spec = parse_endpoint("localhost:3000", &EndpointParser::HostPort).map_err(|e| {
        SongbirdError::configuration(format!("Test: localhost:3000 should parse: {e}"))
    })?;
    assert_eq!(spec.host, "localhost");
    assert_eq!(spec.port, 3000);
    Ok(())
}

#[test]
fn test_parse_hostname() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let spec = parse_endpoint("myservice", &EndpointParser::Hostname)
        .map_err(|e| SongbirdError::configuration(format!("Test: myservice should parse: {e}")))?;
    assert_eq!(spec.host, "myservice");
    assert_eq!(spec.port, 8080);
    Ok(())
}

#[test]
fn test_endpoint_to_url() {
    let spec = EndpointSpec {
        host: "localhost".to_string(),
        port: 8080,
        protocol: Some("https".to_string()),
        path: Some("/api/v1".to_string()),
    };
    assert_eq!(spec.to_url(), "https://localhost:8080/api/v1");
}

#[test]
fn parse_url_rejects_bad_url() {
    let err = parse_endpoint("not a url", &EndpointParser::Url).expect_err("invalid url");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[test]
fn parse_url_rejects_unparseable_value() {
    let err = parse_endpoint(":::not-a-url", &EndpointParser::Url).expect_err("invalid url");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[test]
fn parse_host_port_rejects_wrong_segment_count() {
    let err = parse_endpoint("a:b:c", &EndpointParser::HostPort).expect_err("segments");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[test]
fn parse_host_port_rejects_invalid_port() {
    let err = parse_endpoint("host:99999", &EndpointParser::HostPort).expect_err("port");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[test]
fn parse_pattern_returns_not_implemented() {
    let err = parse_endpoint("x", &EndpointParser::Pattern("p".into())).expect_err("pattern");
    assert!(
        matches!(err, SongbirdError::NotImplemented { ref feature, .. } if feature == "endpoint_parser_pattern"),
        "{err:?}"
    );
}

#[test]
fn endpoint_to_url_defaults_protocol_and_path() {
    let spec = EndpointSpec {
        host: "127.0.0.1".to_string(),
        port: 80,
        protocol: None,
        path: None,
    };
    assert_eq!(spec.to_url(), "http://127.0.0.1:80");
}

#[test]
fn to_socket_addr_accepts_ip_literal() {
    let spec = EndpointSpec {
        host: "203.0.113.1".to_string(),
        port: 9000,
        protocol: None,
        path: None,
    };
    let sa = spec.to_socket_addr().expect("socket addr");
    assert_eq!(sa.port(), 9000);
}

#[test]
fn to_socket_addr_rejects_hostname() {
    let spec = EndpointSpec {
        host: "example.com".to_string(),
        port: 443,
        protocol: None,
        path: None,
    };
    assert!(spec.to_socket_addr().is_err());
}

#[test]
fn resolve_named_port_http_https_grpc() {
    assert_eq!(resolve_named_port("http").expect("http"), 80);
    assert_eq!(resolve_named_port("https").expect("https"), 443);
    assert_eq!(resolve_named_port("grpc").expect("grpc"), 9090);
}

#[test]
fn resolve_named_port_unknown_errors() {
    assert!(resolve_named_port("unknown_port_name").is_err());
}

#[tokio::test]
async fn discover_static_succeeds_without_env() {
    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::Static {
            endpoint: EndpointSpec {
                host: "10.0.0.1".to_string(),
                port: 1111,
                protocol: Some("https".to_string()),
                path: None,
            },
        },
        fallback_methods: vec![],
        dev_fallback: None,
        cache_discovery: false,
    };
    let got = ep.discover_with(|_| Err(std::env::VarError::NotPresent)).await.expect("static");
    assert_eq!(got.host, "10.0.0.1");
    assert_eq!(got.port, 1111);
}

#[tokio::test]
async fn discover_dev_fallback_when_development_env() {
    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::ConsulService {
            service_name: "x".into(),
            consul_addr: None,
        },
        fallback_methods: vec![],
        dev_fallback: Some(EndpointSpec {
            host: "dev.local".into(),
            port: 4000,
            protocol: Some("http".into()),
            path: None,
        }),
        cache_discovery: false,
    };
    let got = ep
        .discover_with(|k| {
            if k == "SONGBIRD_ENV" {
                Ok("development".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .expect("dev fallback");
    assert_eq!(got.host, "dev.local");
    assert_eq!(got.port, 4000);
}

#[tokio::test]
async fn discover_fails_when_no_method_and_no_dev_fallback() {
    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::ConsulService {
            service_name: "n".into(),
            consul_addr: None,
        },
        fallback_methods: vec![],
        dev_fallback: None,
        cache_discovery: false,
    };
    let err =
        ep.discover_with(|_| Err(std::env::VarError::NotPresent)).await.expect_err("no discovery");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[tokio::test]
async fn from_k8s_service_errors_outside_cluster() {
    let ep = DiscoverableEndpoint::from_k8s_service("api", "ns", 8080);
    let err =
        ep.discover_with(|_| Err(std::env::VarError::NotPresent)).await.expect_err("not in k8s");
    assert!(matches!(err, SongbirdError::Configuration { .. }), "{err:?}");
}

#[test]
fn from_env_sets_environment_discovery_and_var_name() {
    let ep = DiscoverableEndpoint::from_env("MY_SERVICE_URL");
    assert!(matches!(
        ep.discovery_method,
        DiscoveryMethod::Environment {
            ref var_name,
            parser: EndpointParser::Url,
        } if var_name == "MY_SERVICE_URL"
    ));
    assert!(ep.cache_discovery);
    assert!(ep.dev_fallback.is_some());
}

#[test]
fn from_k8s_service_sets_cluster_host_and_port() {
    let ep = DiscoverableEndpoint::from_k8s_service("payments", "prod", 9090);
    match &ep.discovery_method {
        DiscoveryMethod::KubernetesService {
            service_name,
            namespace,
            port: PortSpec::Number(p),
        } => {
            assert_eq!(service_name, "payments");
            assert_eq!(namespace, "prod");
            assert_eq!(*p, 9090);
        }
        _ => panic!("expected KubernetesService"),
    }
    let fb = ep.dev_fallback.as_ref().expect("dev fallback");
    assert_eq!(fb.host, "payments.prod.svc.cluster.local");
    assert_eq!(fb.port, 9090);
}

#[test]
fn from_consul_service_has_no_dev_fallback() {
    let ep = DiscoverableEndpoint::from_consul_service("auth");
    assert!(matches!(
        ep.discovery_method,
        DiscoveryMethod::ConsulService {
            ref service_name,
            consul_addr: None,
        } if service_name == "auth"
    ));
    assert!(ep.dev_fallback.is_none());
}

#[test]
fn default_uses_service_endpoint_var() {
    let ep = DiscoverableEndpoint::default();
    assert!(matches!(
        ep.discovery_method,
        DiscoveryMethod::Environment {
            ref var_name,
            ..
        } if var_name == "SERVICE_ENDPOINT"
    ));
}

#[test]
fn parse_url_default_port_80_when_omitted() {
    let spec = parse_endpoint("http://example.com/path", &EndpointParser::Url).expect("url");
    assert_eq!(spec.port, 80);
    assert_eq!(spec.path.as_deref(), Some("/path"));
}

#[test]
fn parse_url_https_default_port_443_when_omitted() {
    let spec = parse_endpoint("https://secure.example.com/", &EndpointParser::Url).expect("url");
    assert_eq!(spec.port, 443);
    assert_eq!(spec.protocol.as_deref(), Some("https"));
}

#[tokio::test]
async fn discover_dev_fallback_when_rust_env_dev() {
    let ep = DiscoverableEndpoint {
        discovery_method: DiscoveryMethod::ConsulService {
            service_name: "x".into(),
            consul_addr: None,
        },
        fallback_methods: vec![],
        dev_fallback: Some(EndpointSpec {
            host: "rust-dev.local".into(),
            port: 5000,
            protocol: Some("http".into()),
            path: None,
        }),
        cache_discovery: false,
    };
    let got = ep
        .discover_with(|k| {
            if k == "RUST_ENV" {
                Ok("dev".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .await
        .expect("dev fallback via RUST_ENV");
    assert_eq!(got.host, "rust-dev.local");
}

#[test]
fn to_socket_addr_accepts_ipv6_literal() {
    let spec = EndpointSpec {
        host: "::1".to_string(),
        port: 8080,
        protocol: None,
        path: None,
    };
    let sa = spec.to_socket_addr().expect("ipv6 loopback");
    assert!(sa.ip().is_loopback());
    assert_eq!(sa.port(), 8080);
}
