//! Additional tests for endpoint conversion functionality
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
//! Extends coverage for the conversion module.

use songbird_discovery::conversion::parse_endpoint;

#[test]
fn test_parse_endpoint_with_ipv4() {
    let (host, port) = parse_endpoint("192.168.1.100:3000");
    assert_eq!(host, "192.168.1.100");
    assert_eq!(port, 3000);
}

#[test]
fn test_parse_endpoint_with_domain() {
    let (host, port) = parse_endpoint("example.com:9090");
    assert_eq!(host, "example.com");
    assert_eq!(port, 9090);
}

#[test]
fn test_parse_endpoint_https_with_port() {
    let (host, port) = parse_endpoint("https://secure.example.com:443");
    assert_eq!(host, "secure.example.com");
    assert_eq!(port, 443);
}

#[test]
fn test_parse_endpoint_http_default_port() {
    let (host, port) = parse_endpoint("http://example.com");
    assert_eq!(host, "example.com");
    assert_eq!(port, 8080); // Default port
}

#[test]
fn test_parse_endpoint_with_path() {
    let (host, port) = parse_endpoint("example.com:8080/api/v1/services");
    assert_eq!(host, "example.com");
    assert_eq!(port, 8080);
}

#[test]
fn test_parse_endpoint_subdomain() {
    let (host, port) = parse_endpoint("api.services.example.com:8080");
    assert_eq!(host, "api.services.example.com");
    assert_eq!(port, 8080);
}

#[test]
fn test_parse_endpoint_localhost_variants() {
    let (host1, port1) = parse_endpoint("localhost:8080");
    assert_eq!(host1, "localhost");
    assert_eq!(port1, 8080);

    let (host2, port2) = parse_endpoint("127.0.0.1:8080");
    assert_eq!(host2, "127.0.0.1");
    assert_eq!(port2, 8080);
}

#[test]
fn test_parse_endpoint_high_port() {
    let (host, port) = parse_endpoint("example.com:65535");
    assert_eq!(host, "example.com");
    assert_eq!(port, 65535);
}

#[test]
fn test_parse_endpoint_low_port() {
    let (host, port) = parse_endpoint("example.com:80");
    assert_eq!(host, "example.com");
    assert_eq!(port, 80);
}

#[test]
fn test_parse_endpoint_with_underscore() {
    let (host, port) = parse_endpoint("my_service.example.com:9000");
    assert_eq!(host, "my_service.example.com");
    assert_eq!(port, 9000);
}

#[test]
fn test_parse_endpoint_with_hyphen() {
    let (host, port) = parse_endpoint("my-service.example.com:9000");
    assert_eq!(host, "my-service.example.com");
    assert_eq!(port, 9000);
}

#[test]
fn test_parse_endpoint_with_numbers() {
    let (host, port) = parse_endpoint("service123.example.com:8080");
    assert_eq!(host, "service123.example.com");
    assert_eq!(port, 8080);
}

#[test]
fn test_parse_endpoint_multiple_subdomains() {
    let (host, port) = parse_endpoint("a.b.c.example.com:8080");
    assert_eq!(host, "a.b.c.example.com");
    assert_eq!(port, 8080);
}

#[test]
fn test_parse_endpoint_http_with_custom_port() {
    let (host, port) = parse_endpoint("http://example.com:3000");
    assert_eq!(host, "example.com");
    assert_eq!(port, 3000);
}

#[test]
fn test_parse_endpoint_https_custom_port() {
    let (host, port) = parse_endpoint("https://example.com:8443");
    assert_eq!(host, "example.com");
    assert_eq!(port, 8443);
}

#[test]
fn test_parse_endpoint_with_query_string() {
    let (host, port) = parse_endpoint("example.com:8080/api?key=value");
    assert_eq!(host, "example.com");
    assert_eq!(port, 8080);
}

#[test]
fn test_parse_endpoint_consistency() {
    // Multiple calls should return same result
    let (host1, port1) = parse_endpoint("example.com:8080");
    let (host2, port2) = parse_endpoint("example.com:8080");

    assert_eq!(host1, host2);
    assert_eq!(port1, port2);
}

#[test]
fn test_parse_endpoint_different_ports_same_host() {
    let (host1, port1) = parse_endpoint("example.com:8080");
    let (host2, port2) = parse_endpoint("example.com:9090");

    assert_eq!(host1, host2);
    assert_ne!(port1, port2);
}

#[test]
fn test_parse_endpoint_same_port_different_hosts() {
    let (host1, port1) = parse_endpoint("example1.com:8080");
    let (host2, port2) = parse_endpoint("example2.com:8080");

    assert_ne!(host1, host2);
    assert_eq!(port1, port2);
}

#[test]
fn test_parse_endpoint_simple_hostname() {
    let (host, port) = parse_endpoint("simple:8080");
    assert_eq!(host, "simple");
    assert_eq!(port, 8080);
}
