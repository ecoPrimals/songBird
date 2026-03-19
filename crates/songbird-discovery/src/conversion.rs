//! Type conversion utilities for songbird-discovery
//!
//! This module provides conversions between discovery-specific types
//! and universal types used across the Songbird ecosystem.

use chrono::Utc;
use std::collections::HashMap;

use crate::traits::service::{ServiceEndpoint, ServiceInfo as DiscoveryServiceInfo, ServiceStatus};
use songbird_universal::ServiceInfo as UniversalServiceInfo;

#[cfg(test)]
#[path = "conversion_comprehensive_tests.rs"]
mod conversion_comprehensive_tests;

/// Convert from discovery `ServiceInfo` to universal `ServiceInfo`
///
/// This conversion loses some information (version, timestamps, etc.)
/// as the universal `ServiceInfo` is a simpler representation.
impl From<DiscoveryServiceInfo> for UniversalServiceInfo {
    fn from(discovery: DiscoveryServiceInfo) -> Self {
        // Convert metadata from HashMap<String, Value> to HashMap<String, String>
        let metadata: HashMap<String, String> =
            discovery.metadata.into_iter().map(|(k, v)| (k, v.to_string())).collect();

        // Construct endpoint from host:port (endpoints are API paths, not network endpoints)
        let endpoint = format!("{}:{}", discovery.host, discovery.port);

        Self {
            name: discovery.name,
            primal_type: songbird_universal::PrimalType::new("generic"), // Default, could be in metadata
            endpoint,
            capabilities: Vec::new(), // Could extract from metadata
            health: songbird_universal::HealthStatus::Unknown, // Would need conversion
            metadata,
        }
    }
}

/// Convert from universal `ServiceInfo` to discovery `ServiceInfo`
///
/// This conversion requires sensible defaults for fields not present
/// in the universal `ServiceInfo`.
impl From<UniversalServiceInfo> for DiscoveryServiceInfo {
    fn from(universal: UniversalServiceInfo) -> Self {
        // Convert metadata from HashMap<String, String> to HashMap<String, Value>
        let metadata: HashMap<String, serde_json::Value> = universal
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Parse endpoint to extract host and port
        let (host, port) = parse_endpoint(&universal.endpoint);

        // Note: endpoints in discovery are API paths, not network endpoints
        // Create a minimal health check endpoint
        let endpoints = vec![ServiceEndpoint {
            path: "/".to_string(),
            method: "GET".to_string(),
            description: Some("Root endpoint".to_string()),
            parameters: Vec::new(),
            response_schema: None,
            auth_required: false,
            rate_limit: None,
        }];

        let now = Utc::now();

        Self {
            service_id: uuid::Uuid::new_v4().to_string(),
            name: universal.name,
            version: "unknown".to_string(),
            service_type: format!("{:?}", universal.primal_type),
            description: None,
            endpoints,
            health_check_endpoint: Some("/health".to_string()),
            metadata,
            tags: Vec::new(),
            dependencies: Vec::new(),
            status: ServiceStatus::Running, // Default to running
            created_at: now,
            updated_at: now,
            instance_id: uuid::Uuid::new_v4().to_string(),
            host,
            port,
        }
    }
}

/// Helper function to parse endpoint string into host and port
#[must_use]
pub fn parse_endpoint(endpoint: &str) -> (String, u16) {
    // Handle various endpoint formats:
    // - "host:port"
    // - "http://host:port"
    // - "https://host:port"
    // - "host" (default port 8080)

    let endpoint = endpoint.trim_start_matches("http://").trim_start_matches("https://");

    endpoint.find(':').map_or_else(
        || {
            // No port specified, use default
            let host = endpoint.split('/').next().unwrap_or(endpoint).to_string();
            (host, 8080)
        },
        |idx| {
            let host = endpoint[..idx].to_string();
            let port_str = &endpoint[idx + 1..];
            // Handle paths after port: "host:port/path"
            let port = port_str.find('/').map_or_else(
                || port_str.parse().unwrap_or(8080),
                |slash| port_str[..slash].parse().unwrap_or(8080),
            );
            (host, port)
        },
    )
}

/// Extension trait for `DiscoveryServiceInfo` to add utility methods
pub trait ServiceInfoExt {
    /// Create a minimal `DiscoveryServiceInfo` with sensible defaults
    fn minimal(name: String, host: String, port: u16) -> Self;

    /// Update from a universal `ServiceInfo`, preserving existing fields
    fn update_from_universal(&mut self, universal: UniversalServiceInfo);
}

impl ServiceInfoExt for DiscoveryServiceInfo {
    fn minimal(name: String, host: String, port: u16) -> Self {
        let now = Utc::now();
        let _endpoint = format!("http://{host}:{port}");

        Self {
            service_id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "unknown".to_string(),
            service_type: "service".to_string(),
            description: None,
            endpoints: vec![ServiceEndpoint {
                path: "/".to_string(),
                method: "GET".to_string(),
                description: Some("Root endpoint".to_string()),
                parameters: Vec::new(),
                response_schema: None,
                auth_required: false,
                rate_limit: None,
            }],
            health_check_endpoint: Some("/health".to_string()),
            metadata: HashMap::new(),
            tags: Vec::new(),
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now,
            instance_id: uuid::Uuid::new_v4().to_string(),
            host,
            port,
        }
    }

    fn update_from_universal(&mut self, universal: UniversalServiceInfo) {
        // Update fields that exist in both
        self.name = universal.name;
        self.service_type = format!("{:?}", universal.primal_type);
        self.updated_at = Utc::now();

        // Convert metadata
        self.metadata = universal
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Parse and update endpoint
        let (host, port) = parse_endpoint(&universal.endpoint);
        self.host = host;
        self.port = port;

        // Update first endpoint description or add new one
        if let Some(endpoint) = self.endpoints.first_mut() {
            endpoint.description = Some(format!("Updated from {}", universal.endpoint));
        } else {
            self.endpoints.push(ServiceEndpoint {
                path: "/".to_string(),
                method: "GET".to_string(),
                description: Some(format!("Added from {}", universal.endpoint)),
                parameters: Vec::new(),
                response_schema: None,
                auth_required: false,
                rate_limit: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_endpoint() {
        assert_eq!(parse_endpoint("localhost:8080"), ("localhost".to_string(), 8080));
        assert_eq!(parse_endpoint("http://localhost:8080"), ("localhost".to_string(), 8080));
        assert_eq!(
            parse_endpoint("https://api.example.com:443"),
            ("api.example.com".to_string(), 443)
        );
        assert_eq!(parse_endpoint("localhost"), ("localhost".to_string(), 8080));
        assert_eq!(parse_endpoint("localhost:8080/api"), ("localhost".to_string(), 8080));
    }

    #[test]
    fn test_minimal_service_info() {
        let info = DiscoveryServiceInfo::minimal(
            "test-service".to_string(),
            "localhost".to_string(),
            8080,
        );

        assert_eq!(info.name, "test-service");
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 8080);
        assert_eq!(info.endpoints.len(), 1);
        assert_eq!(info.status, ServiceStatus::Running);
    }
}
