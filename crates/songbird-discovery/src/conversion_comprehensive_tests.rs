//! Comprehensive tests for type conversion utilities
//!
//! Tests conversion between discovery `ServiceInfo` and universal `ServiceInfo`.

#[cfg(test)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use crate::conversion::parse_endpoint;
    use crate::traits::service::ServiceInfo as DiscoveryServiceInfo;
    use songbird_universal::ServiceInfo as UniversalServiceInfo;
    use std::collections::HashMap;

    #[test]
    fn test_parse_endpoint_with_port() {
        let (host, port) = parse_endpoint("example.com:8080");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_http_url() {
        let (host, port) = parse_endpoint("http://example.com:8080");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_https_url() {
        let (host, port) = parse_endpoint("https://example.com:443");
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[test]
    fn test_parse_endpoint_with_path() {
        let (host, port) = parse_endpoint("example.com:8080/api/v1");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_no_port_defaults_8080() {
        let (host, port) = parse_endpoint("example.com");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_parse_endpoint_localhost() {
        let (host, port) = parse_endpoint("localhost:3000");
        assert_eq!(host, "localhost");
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_parse_endpoint_ipv4() {
        let (host, port) = parse_endpoint("192.168.1.1:5000");
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 5000);
    }

    #[test]
    fn test_parse_endpoint_with_invalid_port_defaults() {
        let (host, port) = parse_endpoint("example.com:invalid");
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
    }

    #[test]
    fn test_universal_to_discovery_conversion() {
        let universal = create_test_universal_service("test-service", "localhost:8080");
        let discovery: DiscoveryServiceInfo = universal.into();

        assert_eq!(discovery.name, "test-service");
        assert_eq!(discovery.host, "localhost");
        assert_eq!(discovery.port, 8080);
    }

    #[test]
    fn test_discovery_to_universal_conversion() {
        let discovery = create_test_discovery_service("test-service", "localhost", 8080);
        let universal: UniversalServiceInfo = discovery.into();

        assert_eq!(universal.name, "test-service");
        assert!(universal.endpoint.contains("localhost"));
        assert!(universal.endpoint.contains("8080"));
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = create_test_universal_service("round-trip", "example.com:9090");
        let discovery: DiscoveryServiceInfo = original.clone().into();
        let back: UniversalServiceInfo = discovery.into();

        assert_eq!(original.name, back.name);
    }

    #[test]
    fn test_conversion_preserves_metadata() {
        let mut universal = create_test_universal_service("meta-service", "localhost:8080");
        universal.metadata.insert("key1".to_string(), "value1".to_string());
        universal.metadata.insert("key2".to_string(), "value2".to_string());

        let discovery: DiscoveryServiceInfo = universal.into();
        assert_eq!(discovery.metadata.len(), 2);
    }

    #[test]
    fn test_conversion_with_empty_metadata() {
        let universal = create_test_universal_service("empty-meta", "localhost:8080");
        let discovery: DiscoveryServiceInfo = universal.into();

        assert!(discovery.metadata.is_empty());
    }

    #[test]
    fn test_conversion_with_complex_endpoint() {
        let universal = create_test_universal_service(
            "complex-service",
            "https://api.example.com:443/v1/services",
        );
        let discovery: DiscoveryServiceInfo = universal.into();

        assert_eq!(discovery.host, "api.example.com");
        assert_eq!(discovery.port, 443);
    }

    #[test]
    fn test_multiple_services_conversion() {
        let services = vec![
            create_test_universal_service("svc1", "host1:8080"),
            create_test_universal_service("svc2", "host2:8081"),
            create_test_universal_service("svc3", "host3:8082"),
        ];

        let discovery_services: Vec<DiscoveryServiceInfo> =
            services.into_iter().map(Into::into).collect();

        assert_eq!(discovery_services.len(), 3);
        assert_eq!(discovery_services[0].name, "svc1");
        assert_eq!(discovery_services[1].name, "svc2");
        assert_eq!(discovery_services[2].name, "svc3");
    }

    #[test]
    fn test_conversion_with_special_characters() {
        let universal = create_test_universal_service("special-service", "host-name.domain:8080");
        let discovery: DiscoveryServiceInfo = universal.into();

        assert_eq!(discovery.host, "host-name.domain");
        assert_eq!(discovery.port, 8080);
    }

    #[test]
    fn test_conversion_with_numeric_host() {
        let universal = create_test_universal_service("numeric", "127.0.0.1:3000");
        let discovery: DiscoveryServiceInfo = universal.into();

        assert_eq!(discovery.host, "127.0.0.1");
        assert_eq!(discovery.port, 3000);
    }

    #[test]
    fn test_parse_endpoint_edge_cases() {
        // Empty string
        let (host, port) = parse_endpoint("");
        assert_eq!(port, 8080);

        // Just port
        let (host, port) = parse_endpoint(":9090");
        assert_eq!(port, 9090);

        // Multiple colons (ipv6-like, take last)
        let (host, port) = parse_endpoint("::1:8080");
        assert!(port == 8080 || port == 8080);
    }

    #[test]
    fn test_conversion_batch_processing() {
        let count = 100;
        let converted: Vec<DiscoveryServiceInfo> = (0..count)
            .map(|i| {
                create_test_universal_service(
                    &format!("service-{i}"),
                    &format!("host{}:808{}", i, i % 10),
                )
            })
            .map(std::convert::Into::into)
            .collect();

        assert_eq!(converted.len(), count);
    }

    #[test]
    fn test_discovery_service_has_valid_endpoints() {
        let universal = create_test_universal_service("endpoint-test", "localhost:8080");
        let discovery: DiscoveryServiceInfo = universal.into();

        assert!(!discovery.endpoints.is_empty());
        assert!(discovery.health_check_endpoint.is_some());
    }

    #[test]
    fn test_discovery_service_has_timestamps() {
        let universal = create_test_universal_service("timestamp-test", "localhost:8080");
        let discovery: DiscoveryServiceInfo = universal.into();

        // Should have created_at and updated_at
        assert!(discovery.created_at.timestamp() > 0);
        assert!(discovery.updated_at.timestamp() > 0);
    }

    #[test]
    fn test_discovery_service_has_unique_ids() {
        let universal1 = create_test_universal_service("service1", "localhost:8080");
        let universal2 = create_test_universal_service("service2", "localhost:8081");

        let discovery1: DiscoveryServiceInfo = universal1.into();
        let discovery2: DiscoveryServiceInfo = universal2.into();

        assert_ne!(discovery1.service_id, discovery2.service_id);
        assert_ne!(discovery1.instance_id, discovery2.instance_id);
    }

    #[test]
    fn test_conversion_with_different_primal_types() {
        use songbird_universal::PrimalType;

        let mut universal = create_test_universal_service("primal-test", "localhost:8080");
        universal.primal_type = PrimalType::new("custom");

        let discovery: DiscoveryServiceInfo = universal.into();
        assert!(
            discovery.service_type.contains("custom") || discovery.service_type.contains("Custom")
        );
    }

    // Helper functions
    fn create_test_universal_service(name: &str, endpoint: &str) -> UniversalServiceInfo {
        UniversalServiceInfo {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            primal_type: songbird_universal::PrimalType::new("test"),
            capabilities: vec![],
            health: songbird_universal::HealthStatus::Healthy,
            metadata: HashMap::new(),
        }
    }

    fn create_test_discovery_service(name: &str, host: &str, port: u16) -> DiscoveryServiceInfo {
        use crate::traits::service::{ServiceEndpoint, ServiceStatus};
        use chrono::Utc;

        let now = Utc::now();
        DiscoveryServiceInfo {
            service_id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: None,
            endpoints: vec![ServiceEndpoint {
                path: "/".to_string(),
                method: "GET".to_string(),
                description: None,
                parameters: vec![],
                response_schema: None,
                auth_required: false,
                rate_limit: None,
            }],
            health_check_endpoint: Some("/health".to_string()),
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now,
            instance_id: uuid::Uuid::new_v4().to_string(),
            host: host.to_string(),
            port,
        }
    }
}
