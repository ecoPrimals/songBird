// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Custom test assertions for E2E testing
//!
//! Provides fluent assertion APIs for testing Songbird components.

use songbird_types::{HealthStatus, ServiceInfo};

/// Test assertion builder for fluent testing
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a service is healthy
    pub fn assert_healthy(status: HealthStatus) {
        assert_eq!(
            status,
            HealthStatus::Healthy,
            "Expected service to be healthy, but was {:?}",
            status
        );
    }

    /// Assert that a service is unhealthy
    pub fn assert_unhealthy(status: HealthStatus) {
        assert_eq!(
            status,
            HealthStatus::Unhealthy,
            "Expected service to be unhealthy, but was {:?}",
            status
        );
    }

    /// Assert that a service is degraded
    pub fn assert_degraded(status: HealthStatus) {
        assert_eq!(
            status,
            HealthStatus::Degraded,
            "Expected service to be degraded, but was {:?}",
            status
        );
    }

    /// Assert that a service list is not empty
    pub fn assert_services_found(services: &[ServiceInfo]) {
        assert!(
            !services.is_empty(),
            "Expected to find services, but list was empty"
        );
    }

    /// Assert that a service list contains a specific service
    pub fn assert_service_present(services: &[ServiceInfo], name: &str) {
        assert!(
            services.iter().any(|s| s.name == name),
            "Expected to find service '{}' in list, but it was not present. Found services: {:?}",
            name,
            services.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
    }

    /// Assert that a service has a specific capability
    pub fn assert_has_capability(service: &ServiceInfo, capability: &str) {
        assert!(
            service.capabilities.iter().any(|c| c == capability),
            "Expected service '{}' to have capability '{}', but it only has: {:?}",
            service.name,
            capability,
            service.capabilities
        );
    }

    /// Assert that a value is within a range
    pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: T, min: T, max: T) {
        assert!(
            value >= min && value <= max,
            "Expected value {:?} to be in range [{:?}, {:?}]",
            value,
            min,
            max
        );
    }

    /// Assert that a duration is within acceptable bounds
    pub fn assert_response_time_acceptable(response_time_ms: f64, max_acceptable_ms: f64) {
        assert!(
            response_time_ms <= max_acceptable_ms,
            "Response time {}ms exceeds acceptable limit of {}ms",
            response_time_ms,
            max_acceptable_ms
        );
    }

    /// Assert load balancing is roughly even across providers
    pub fn assert_load_balanced(
        request_counts: &std::collections::HashMap<String, usize>,
        total_requests: usize,
        variance_threshold: f64,
    ) {
        let provider_count = request_counts.len();
        if provider_count == 0 {
            panic!("Cannot assert load balancing with no providers");
        }

        let expected_per_provider = total_requests as f64 / provider_count as f64;
        let min_expected = (expected_per_provider * (1.0 - variance_threshold)) as usize;
        let max_expected = (expected_per_provider * (1.0 + variance_threshold)) as usize;

        for (provider, count) in request_counts {
            assert!(
                *count >= min_expected && *count <= max_expected,
                "Provider '{}' received {} requests, expected between {} and {} (±{}% of {})",
                provider,
                count,
                min_expected,
                max_expected,
                variance_threshold * 100.0,
                expected_per_provider
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_assertions() {
        TestAssertions::assert_healthy(HealthStatus::Healthy);
        TestAssertions::assert_unhealthy(HealthStatus::Unhealthy);
        TestAssertions::assert_degraded(HealthStatus::Degraded);
    }

    #[test]
    #[should_panic(expected = "Expected service to be healthy")]
    fn test_health_assertion_failure() {
        TestAssertions::assert_healthy(HealthStatus::Unhealthy);
    }

    #[test]
    fn test_range_assertion() {
        TestAssertions::assert_in_range(5, 1, 10);
        TestAssertions::assert_in_range(1, 1, 10);
        TestAssertions::assert_in_range(10, 1, 10);
    }

    #[test]
    #[should_panic]
    fn test_range_assertion_failure() {
        TestAssertions::assert_in_range(11, 1, 10);
    }

    #[test]
    fn test_response_time_assertion() {
        TestAssertions::assert_response_time_acceptable(50.0, 100.0);
    }

    #[test]
    #[should_panic(expected = "exceeds acceptable limit")]
    fn test_response_time_assertion_failure() {
        TestAssertions::assert_response_time_acceptable(150.0, 100.0);
    }
}

