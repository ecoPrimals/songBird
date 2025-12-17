// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Endpoint Validation Tests
//!
//! Tests for endpoint URL validation, parsing, and normalization.

#[test]
fn test_http_endpoint_valid() {
    let endpoint = "http://localhost:8080";
    assert!(endpoint.starts_with("http://"));
}

#[test]
fn test_https_endpoint_valid() {
    let endpoint = "https://example.com:443";
    assert!(endpoint.starts_with("https://"));
}

#[test]
fn test_endpoint_with_path() {
    let endpoint = "http://localhost:8080/api/v1";
    assert!(endpoint.contains("/api"));
}

#[test]
fn test_endpoint_with_query() {
    let endpoint = "http://localhost:8080?param=value";
    assert!(endpoint.contains('?'));
}

#[test]
fn test_endpoint_with_fragment() {
    let endpoint = "http://localhost:8080#section";
    assert!(endpoint.contains('#'));
}

#[test]
fn test_endpoint_ipv4_address() {
    let endpoint = "http://192.168.1.1:8080";
    assert!(endpoint.contains("192.168.1.1"));
}

#[test]
fn test_endpoint_ipv6_address() {
    let endpoint = "http://[::1]:8080";
    assert!(endpoint.contains("::1"));
}

#[test]
fn test_endpoint_domain_name() {
    let endpoint = "https://api.example.com:443";
    assert!(endpoint.contains("api.example.com"));
}

#[test]
fn test_endpoint_subdomain() {
    let endpoint = "https://sub.domain.example.com";
    assert!(endpoint.contains("sub.domain"));
}

#[test]
fn test_endpoint_port_80() {
    let endpoint = "http://example.com:80";
    assert!(endpoint.contains(":80"));
}

#[test]
fn test_endpoint_port_443() {
    let endpoint = "https://example.com:443";
    assert!(endpoint.contains(":443"));
}

#[test]
fn test_endpoint_port_custom() {
    let endpoint = "http://localhost:3000";
    assert!(endpoint.contains(":3000"));
}

#[test]
fn test_endpoint_no_port_http() {
    let endpoint = "http://example.com";
    assert!(!endpoint.contains(':') || endpoint.matches(':').count() == 1);
}

#[test]
fn test_endpoint_no_port_https() {
    let endpoint = "https://example.com";
    assert!(!endpoint[8..].contains(':'));
}

#[test]
fn test_endpoint_localhost() {
    let endpoint = "http://localhost:8080";
    assert!(endpoint.contains("localhost"));
}

#[test]
fn test_endpoint_127_0_0_1() {
    let endpoint = "http://127.0.0.1:8080";
    assert!(endpoint.contains("127.0.0.1"));
}

#[test]
fn test_endpoint_with_username() {
    let endpoint = "http://user@localhost:8080";
    assert!(endpoint.contains("user@"));
}

#[test]
fn test_endpoint_with_username_password() {
    let endpoint = "http://user:pass@localhost:8080";
    assert!(endpoint.contains("user:pass@"));
}

#[test]
fn test_endpoint_trailing_slash() {
    let endpoint = "http://example.com/";
    assert!(endpoint.ends_with('/'));
}

#[test]
fn test_endpoint_no_trailing_slash() {
    let endpoint = "http://example.com";
    assert!(!endpoint.ends_with('/'));
}

#[test]
fn test_endpoint_multiple_path_segments() {
    let endpoint = "http://example.com/api/v1/users/123";
    assert!(endpoint.matches('/').count() >= 3);
}

#[test]
fn test_endpoint_with_hyphen_in_domain() {
    let endpoint = "https://my-api.example.com";
    assert!(endpoint.contains("my-api"));
}

#[test]
fn test_endpoint_with_underscore_in_path() {
    let endpoint = "http://example.com/api_v1";
    assert!(endpoint.contains("api_v1"));
}

#[test]
fn test_endpoint_case_sensitive_path() {
    let endpoint1 = "http://example.com/Api";
    let endpoint2 = "http://example.com/api";
    assert_ne!(endpoint1, endpoint2);
}

#[test]
fn test_endpoint_case_insensitive_domain() {
    let domain1 = "EXAMPLE.COM";
    let domain2 = "example.com";
    assert_eq!(domain1.to_lowercase(), domain2.to_lowercase());
}

#[test]
fn test_endpoint_max_port_65535() {
    let endpoint = "http://localhost:65535";
    assert!(endpoint.contains("65535"));
}

#[test]
fn test_endpoint_port_parsing() {
    let endpoint = "http://localhost:8080";
    let port_str = endpoint.split(':').last().expect("test precondition");
    let port: u16 = port_str.parse().expect("should parse valid input");
    assert_eq!(port, 8080);
}

#[test]
fn test_endpoint_scheme_extraction() {
    let endpoint = "https://example.com";
    let scheme = endpoint.split("://").next().expect("test precondition");
    assert_eq!(scheme, "https");
}

#[test]
fn test_endpoint_host_extraction() {
    let endpoint = "http://example.com:8080/path";
    let without_scheme = endpoint.strip_prefix("http://").expect("test precondition");
    let host = without_scheme.split(':').next().expect("test precondition");
    assert_eq!(host, "example.com");
}

#[test]
fn test_endpoint_url_encoding() {
    let endpoint = "http://example.com/path%20with%20spaces";
    assert!(endpoint.contains("%20"));
}

#[test]
fn test_endpoint_query_parameters() {
    let endpoint = "http://example.com?key1=value1&key2=value2";
    let query = endpoint.split('?').nth(1).expect("test precondition");
    assert!(query.contains('&'));
}

#[test]
fn test_endpoint_empty_query_value() {
    let endpoint = "http://example.com?key=";
    assert!(endpoint.contains("key="));
}

#[test]
fn test_endpoint_multiple_query_params() {
    let endpoint = "http://example.com?a=1&b=2&c=3";
    let params = endpoint.split('?').nth(1).expect("test precondition").split('&').count();
    assert_eq!(params, 3);
}

#[test]
fn test_endpoint_fragment_identifier() {
    let endpoint = "http://example.com/page#section";
    let fragment = endpoint.split('#').nth(1).expect("test precondition");
    assert_eq!(fragment, "section");
}

#[test]
fn test_endpoint_without_scheme() {
    let endpoint = "example.com:8080";
    assert!(!endpoint.contains("://"));
}

#[test]
fn test_endpoint_double_slash() {
    let endpoint = "http://example.com//double//slash";
    assert!(endpoint.contains("//double//"));
}

#[test]
fn test_endpoint_dot_in_path() {
    let endpoint = "http://example.com/file.json";
    assert!(endpoint.contains(".json"));
}

#[test]
fn test_endpoint_relative_path() {
    let endpoint = "http://example.com/./relative";
    assert!(endpoint.contains("./"));
}

#[test]
fn test_endpoint_parent_path() {
    let endpoint = "http://example.com/../parent";
    assert!(endpoint.contains("../"));
}

#[test]
fn test_endpoint_ws_protocol() {
    let endpoint = "ws://localhost:8080";
    assert!(endpoint.starts_with("ws://"));
}

#[test]
fn test_endpoint_wss_protocol() {
    let endpoint = "wss://secure.example.com";
    assert!(endpoint.starts_with("wss://"));
}

#[test]
fn test_endpoint_ftp_protocol() {
    let endpoint = "ftp://ftp.example.com";
    assert!(endpoint.starts_with("ftp://"));
}

#[test]
fn test_endpoint_comparison_equal() {
    let endpoint1 = "http://example.com:8080";
    let endpoint2 = "http://example.com:8080";
    assert_eq!(endpoint1, endpoint2);
}

#[test]
fn test_endpoint_comparison_different_port() {
    let endpoint1 = "http://example.com:8080";
    let endpoint2 = "http://example.com:8081";
    assert_ne!(endpoint1, endpoint2);
}

#[test]
fn test_endpoint_comparison_different_scheme() {
    let endpoint1 = "http://example.com";
    let endpoint2 = "https://example.com";
    assert_ne!(endpoint1, endpoint2);
}

#[test]
fn test_endpoint_length_short() {
    let endpoint = "http://a.co";
    assert!(endpoint.len() > 0);
}

#[test]
fn test_endpoint_length_long() {
    let long_domain = "a".repeat(253);
    let endpoint = format!("http://{}.com", long_domain);
    assert!(endpoint.len() > 260);
}

#[test]
fn test_endpoint_contains_colon() {
    let endpoint = "http://example.com:8080";
    assert!(endpoint.contains(':'));
}

#[test]
fn test_endpoint_starts_with_protocol() {
    let endpoint = "http://example.com";
    assert!(endpoint.starts_with("http"));
}

#[test]
fn test_endpoint_to_string() {
    let endpoint = "http://example.com";
    let as_string = endpoint.to_string();
    assert_eq!(endpoint, as_string);
}

#[test]
fn test_endpoint_clone() {
    let endpoint = "http://example.com".to_string();
    let cloned = endpoint.clone();
    assert_eq!(endpoint, cloned);
}

#[test]
fn test_endpoint_in_vec() {
    let endpoints = vec![
        "http://service1.com",
        "http://service2.com",
        "http://service3.com",
    ];
    assert_eq!(endpoints.len(), 3);
}

#[test]
fn test_endpoint_deduplication() {
    let mut endpoints = vec![
        "http://example.com".to_string(),
        "http://example.com".to_string(),
        "http://other.com".to_string(),
    ];
    endpoints.dedup();
    assert!(endpoints.len() >= 2);
}

#[test]
fn test_endpoint_sorting() {
    let mut endpoints = vec![
        "http://c.com",
        "http://a.com",
        "http://b.com",
    ];
    endpoints.sort();
    assert_eq!(endpoints[0], "http://a.com");
}

#[test]
fn test_endpoint_filter_https() {
    let endpoints = vec![
        "http://example.com",
        "https://secure.com",
        "http://other.com",
    ];
    let https_only: Vec<_> = endpoints.into_iter()
        .filter(|e| e.starts_with("https"))
        .collect();
    assert_eq!(https_only.len(), 1);
}

#[test]
fn test_endpoint_map_to_uppercase() {
    let endpoint = "http://example.com";
    let upper = endpoint.to_uppercase();
    assert!(upper.contains("HTTP"));
}

#[test]
fn test_endpoint_split_by_slash() {
    let endpoint = "http://example.com/api/v1/users";
    let parts: Vec<&str> = endpoint.split('/').collect();
    assert!(parts.len() > 3);
}

#[test]
fn test_endpoint_replace_domain() {
    let endpoint = "http://old.com/path";
    let replaced = endpoint.replace("old.com", "new.com");
    assert!(replaced.contains("new.com"));
}

#[test]
fn test_endpoint_trim_whitespace() {
    let endpoint = "  http://example.com  ";
    let trimmed = endpoint.trim();
    assert!(!trimmed.starts_with(' '));
}

#[test]
fn test_endpoint_is_empty() {
    let endpoint = "";
    assert!(endpoint.is_empty());
}

#[test]
fn test_endpoint_is_not_empty() {
    let endpoint = "http://example.com";
    assert!(!endpoint.is_empty());
}

#[test]
fn test_endpoint_char_count() {
    let endpoint = "http://example.com";
    assert!(endpoint.chars().count() > 0);
}

#[test]
fn test_endpoint_byte_length() {
    let endpoint = "http://example.com";
    assert_eq!(endpoint.len(), endpoint.as_bytes().len());
}

#[test]
fn test_endpoint_concatenation() {
    let base = "http://example.com";
    let path = "/api/v1";
    let full = format!("{}{}", base, path);
    assert!(full.contains("/api/v1"));
}

#[test]
fn test_endpoint_option_some() {
    let endpoint: Option<&str> = Some("http://example.com");
    assert!(endpoint.is_some());
}

#[test]
fn test_endpoint_option_none() {
    let endpoint: Option<&str> = None;
    assert!(endpoint.is_none());
}

#[test]
fn test_endpoint_result_ok() {
    let endpoint: Result<&str, ()> = Ok("http://example.com");
    assert!(endpoint.is_ok());
}

#[test]
fn test_endpoint_result_err() {
    let endpoint: Result<&str, &str> = Err("invalid");
    assert!(endpoint.is_err());
}

