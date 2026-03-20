// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for health monitoring types
//!
//! Tests for `HealthStatus`, `HealthMetrics`, and related types.

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::float_cmp)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::unreadable_literal)]

    use super::super::{HealthCheckResult, HealthState, HealthStatus, HealthStatusDetails};
    use std::collections::HashMap;

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::Healthy;
        assert_eq!(format!("{:?}", status), "Healthy");
    }

    #[test]
    fn test_health_status_degraded() {
        let status = HealthStatus::Degraded;
        assert_eq!(format!("{:?}", status), "Degraded");
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::Unhealthy;
        assert_eq!(format!("{:?}", status), "Unhealthy");
    }

    #[test]
    fn test_health_status_all_variants() {
        let statuses = vec![HealthStatus::Healthy, HealthStatus::Degraded, HealthStatus::Unhealthy];
        assert_eq!(statuses.len(), 3);
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_clone() {
        let status1 = HealthStatus::Healthy;
        let status2 = status1.clone();
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_health_state_healthy() {
        let state = HealthState::Healthy;
        assert_eq!(format!("{:?}", state), "Healthy");
    }

    #[test]
    fn test_health_state_degraded() {
        let state = HealthState::Degraded;
        assert_eq!(format!("{:?}", state), "Degraded");
    }

    #[test]
    fn test_health_state_unhealthy() {
        let state = HealthState::Unhealthy;
        assert_eq!(format!("{:?}", state), "Unhealthy");
    }

    #[test]
    fn test_health_state_critical() {
        let state = HealthState::Critical;
        assert_eq!(format!("{:?}", state), "Critical");
    }

    #[test]
    fn test_health_state_unknown() {
        let state = HealthState::Unknown;
        assert_eq!(format!("{:?}", state), "Unknown");
    }

    #[test]
    fn test_health_state_maintenance() {
        let state = HealthState::Maintenance;
        assert_eq!(format!("{:?}", state), "Maintenance");
    }

    #[test]
    fn test_health_check_result_creation() {
        let result = HealthCheckResult {
            name: "database-check".to_string(),
            status: HealthStatus::Healthy,
            message: "All systems operational".to_string(),
            response_time_ms: 45,
        };

        assert_eq!(result.name, "database-check");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.response_time_ms, 45);
    }

    #[test]
    fn test_health_check_result_clone() {
        let result1 = HealthCheckResult {
            name: "test".to_string(),
            status: HealthStatus::Degraded,
            message: "Warning".to_string(),
            response_time_ms: 100,
        };

        let result2 = result1.clone();
        assert_eq!(result1.name, result2.name);
        assert_eq!(result1.status, result2.status);
    }

    #[test]
    fn test_health_status_details_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), serde_json::json!("1.0.0"));

        let details = HealthStatusDetails {
            state: HealthState::Healthy,
            score: 0.95,
            checks_passed: 10,
            checks_failed: 0,
            last_updated: std::time::SystemTime::now(),
            metadata,
        };

        assert_eq!(details.state, HealthState::Healthy);
        assert_eq!(details.score, 0.95);
        assert_eq!(details.checks_passed, 10);
        assert_eq!(details.checks_failed, 0);
    }

    #[test]
    fn test_health_status_details_with_failures() {
        let details = HealthStatusDetails {
            state: HealthState::Degraded,
            score: 0.75,
            checks_passed: 8,
            checks_failed: 2,
            last_updated: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(details.checks_failed, 2);
        assert!(details.score < 1.0);
    }

    #[test]
    fn test_multiple_health_check_results() {
        let results = vec![
            HealthCheckResult {
                name: "database".to_string(),
                status: HealthStatus::Healthy,
                message: "OK".to_string(),
                response_time_ms: 45,
            },
            HealthCheckResult {
                name: "cache".to_string(),
                status: HealthStatus::Degraded,
                message: "High latency".to_string(),
                response_time_ms: 200,
            },
            HealthCheckResult {
                name: "api".to_string(),
                status: HealthStatus::Unhealthy,
                message: "Connection refused".to_string(),
                response_time_ms: 5000,
            },
        ];

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].status, HealthStatus::Healthy);
        assert_eq!(results[1].status, HealthStatus::Degraded);
        assert_eq!(results[2].status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_result_with_long_message() {
        let long_message = "This is a very long error message that contains detailed information about what went wrong in the system and includes specific error codes and recovery suggestions".to_string();

        let result = HealthCheckResult {
            name: "verbose-check".to_string(),
            status: HealthStatus::Unhealthy,
            message: long_message.clone(),
            response_time_ms: 3000,
        };

        assert_eq!(result.message, long_message);
    }

    #[test]
    fn test_health_status_details_with_metadata() {
        let mut metadata = HashMap::new();
        for i in 0..20 {
            metadata.insert(format!("metric_{}", i), serde_json::json!(i));
        }

        let details = HealthStatusDetails {
            state: HealthState::Healthy,
            score: 0.98,
            checks_passed: 20,
            checks_failed: 0,
            last_updated: std::time::SystemTime::now(),
            metadata: metadata.clone(),
        };

        assert_eq!(details.metadata.len(), 20);
    }

    #[test]
    fn test_health_state_all_variants() {
        let states = vec![
            HealthState::Healthy,
            HealthState::Degraded,
            HealthState::Unhealthy,
            HealthState::Critical,
            HealthState::Unknown,
            HealthState::Maintenance,
        ];

        assert_eq!(states.len(), 6);
    }

    #[test]
    fn test_health_status_debug_output() {
        let status = HealthStatus::Degraded;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Degraded"));
    }

    #[test]
    fn test_health_check_result_debug_output() {
        let result = HealthCheckResult {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            response_time_ms: 50,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("Healthy"));
    }

    #[test]
    fn test_health_status_details_clone() {
        let details1 = HealthStatusDetails {
            state: HealthState::Healthy,
            score: 0.95,
            checks_passed: 10,
            checks_failed: 0,
            last_updated: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        };

        let details2 = details1.clone();
        assert_eq!(details1.checks_passed, details2.checks_passed);
        assert_eq!(details1.score, details2.score);
    }

    #[test]
    fn test_health_check_result_fast_response() {
        let result = HealthCheckResult {
            name: "fast-check".to_string(),
            status: HealthStatus::Healthy,
            message: "Quick response".to_string(),
            response_time_ms: 5,
        };

        assert!(result.response_time_ms < 10);
    }

    #[test]
    fn test_health_check_result_slow_response() {
        let result = HealthCheckResult {
            name: "slow-check".to_string(),
            status: HealthStatus::Degraded,
            message: "Slow response".to_string(),
            response_time_ms: 5000,
        };

        assert!(result.response_time_ms > 1000);
    }
}
