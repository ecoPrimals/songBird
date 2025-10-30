//! Unit tests for discovery conversion utilities

#[cfg(test)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::super::conversion::*;

    #[test]
    fn test_parse_endpoint_with_port() {
        let (host, port) = parse_endpoint("localhost:8080");
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_with_http() {
        let (host, port) = parse_endpoint("http://example.com:9000");
        assert_eq!(host, "example.com");
        assert_eq!(port, 9000);
    }

    #[test]
    fn test_parse_endpoint_with_https() {
        let (host, port) = parse_endpoint("https://api.example.com:443");
        assert_eq!(host, "api.example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_parse_endpoint_with_path() {
        let (host, port) = parse_endpoint("http://service:8080/api/v1");
        assert_eq!(host, "service");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_default_http_port() {
        let (host, port) = parse_endpoint("http://example.com");
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
    }

    #[test]
    fn test_parse_endpoint_default_https_port() {
        let (host, port) = parse_endpoint("https://secure.example.com");
        assert_eq!(host, "secure.example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_parse_endpoint_ip_address() {
        let (host, port) = parse_endpoint("192.168.1.1:3000");
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_parse_endpoint_ipv6() {
        let (host, port) = parse_endpoint("[::1]:8080");
        assert!(host.contains("::1"));
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_normalize_service_name() {
        let normalized = normalize_service_name("My-Service_123");
        assert_eq!(normalized, "my-service-123");
    }

    #[test]
    fn test_normalize_service_name_uppercase() {
        let normalized = normalize_service_name("UPPERCASE");
        assert_eq!(normalized, "uppercase");
    }

    #[test]
    fn test_normalize_service_name_with_spaces() {
        let normalized = normalize_service_name("my service");
        assert_eq!(normalized, "my-service");
    }

    #[test]
    fn test_normalize_service_name_underscores() {
        let normalized = normalize_service_name("test_service_name");
        assert_eq!(normalized, "test-service-name");
    }

    #[test]
    fn test_normalize_service_name_already_normalized() {
        let normalized = normalize_service_name("already-normalized");
        assert_eq!(normalized, "already-normalized");
    }

    #[test]
    fn test_normalize_service_name_empty() {
        let normalized = normalize_service_name("");
        assert_eq!(normalized, "");
    }

    #[test]
    fn test_normalize_service_name_special_chars() {
        let normalized = normalize_service_name("service@#$%name");
        // Should remove special chars
        assert!(!normalized.contains('@'));
        assert!(!normalized.contains('#'));
    }

    #[test]
    fn test_parse_endpoint_no_port() {
        let (host, port) = parse_endpoint("localhost");
        assert_eq!(host, "localhost");
        assert_eq!(port, 80); // Default HTTP port
    }

    #[test]
    fn test_parse_endpoint_high_port() {
        let (host, port) = parse_endpoint("service:65535");
        assert_eq!(host, "service");
        assert_eq!(port, 65535);
    }

    #[test]
    fn test_parse_endpoint_with_subdomain() {
        let (host, port) = parse_endpoint("api.v2.example.com:8080");
        assert_eq!(host, "api.v2.example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_localhost_variations() {
        let (host1, port1) = parse_endpoint("localhost:8080");
        let (host2, port2) = parse_endpoint("127.0.0.1:8080");
        
        assert_eq!(port1, port2);
        assert_eq!(port1, 8080);
    }

    #[test]
    fn test_normalize_service_name_numbers_only() {
        let normalized = normalize_service_name("12345");
        assert_eq!(normalized, "12345");
    }

    #[test]
    fn test_normalize_service_name_hyphens() {
        let normalized = normalize_service_name("my--service");
        // Should normalize multiple hyphens
        assert!(normalized.contains("my"));
        assert!(normalized.contains("service"));
    }

    #[test]
    fn test_parse_endpoint_with_username() {
        let (host, port) = parse_endpoint("http://user@service:8080");
        // Should extract host properly even with username
        assert!(host.contains("service") || host.contains("user@service"));
        assert_eq!(port, 8080);
    }
}

