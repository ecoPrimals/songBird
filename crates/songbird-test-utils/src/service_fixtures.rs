// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service Test Fixtures
//!
//! Provides test fixtures for service-related testing without hardcoded values.

/// Create a test service name
#[must_use]
pub fn test_service_name(base: &str, index: Option<usize>) -> String {
    index.map_or_else(|| base.to_string(), |i| format!("{base}-{i}"))
}

/// Create test service endpoint URL
#[must_use]
pub fn test_service_endpoint(service_name: &str) -> String {
    format!(
        "http://{}:{}/{}",
        crate::network_fixtures::test_bind_ip_str(),
        crate::network_fixtures::test_port(),
        service_name
    )
}

/// Create test health endpoint URL
#[must_use]
pub fn test_health_endpoint(service_name: &str) -> String {
    format!(
        "http://{}:{}/{}/health",
        crate::network_fixtures::test_bind_ip_str(),
        crate::network_fixtures::test_port(),
        service_name
    )
}

#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_name_without_index() {
        let name = test_service_name("myservice", None);
        assert_eq!(name, "myservice");
    }

    #[test]
    fn test_service_name_with_index() {
        let name = test_service_name("service", Some(5));
        assert_eq!(name, "service-5");
    }

    #[test]
    fn test_service_endpoint_format() {
        let endpoint = test_service_endpoint("api");
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains("/api"));
    }

    #[test]
    fn test_health_endpoint_format() {
        let endpoint = test_health_endpoint("myservice");
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains("/health"));
        assert!(endpoint.contains("/myservice"));
    }
}
