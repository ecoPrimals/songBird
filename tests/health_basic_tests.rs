//! Basic Tests for Health Module

use songbird_lib::health::*;
use tokio::test;

/// Test HealthStatus enum values
#[test]
async fn test_health_status_enum() {
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
    ];

    for status in statuses {
        match status {
            HealthStatus::Healthy => assert_eq!(format!("{status:?}"), "Healthy"),
            HealthStatus::Degraded => assert_eq!(format!("{status:?}"), "Degraded"),
            HealthStatus::Unhealthy => assert_eq!(format!("{status:?}"), "Unhealthy"),
        }
    }
}

/// Test HealthCheckResult creation
#[test]
async fn test_health_check_result_creation() {
    let result = HealthCheckResult {
        name: "test-service".to_string(),
        status: HealthStatus::Healthy,
        message: "Service is running".to_string(),
        response_time_ms: 150,
    };

    assert_eq!(result.name, "test-service");
    assert_eq!(result.status, HealthStatus::Healthy);
    assert_eq!(result.message, "Service is running");
    assert_eq!(result.response_time_ms, 150);
}

/// Test HealthState enum values
#[test]
async fn test_health_state_enum() {
    let states = vec![
        HealthState::Healthy,
        HealthState::Degraded,
        HealthState::Unhealthy,
        HealthState::Critical,
        HealthState::Unknown,
        HealthState::Maintenance,
    ];

    for state in states {
        match state {
            HealthState::Healthy => assert_eq!(format!("{state:?}"), "Healthy"),
            HealthState::Degraded => assert_eq!(format!("{state:?}"), "Degraded"),
            HealthState::Unhealthy => assert_eq!(format!("{state:?}"), "Unhealthy"),
            HealthState::Critical => assert_eq!(format!("{state:?}"), "Critical"),
            HealthState::Unknown => assert_eq!(format!("{state:?}"), "Unknown"),
            HealthState::Maintenance => assert_eq!(format!("{state:?}"), "Maintenance"),
        }
    }
}

/// Test HealthChecker creation
#[test]
async fn test_health_checker_creation() {
    let _checker = HealthChecker::new();
    // Checker created successfully - no assertion needed
}

/// Test HealthChecker default
#[test]
async fn test_health_checker_default() {
    let _checker = HealthChecker::default();
    // Default checker created successfully - no assertion needed
}

/// Test HealthChecker check_all with no checks
#[test]
async fn test_health_checker_check_all_empty() {
    let checker = HealthChecker::new();
    let results = checker.check_all().await;
    assert_eq!(results.len(), 0);
}

/// Test HealthCheckResult cloning
#[test]
async fn test_health_check_result_cloning() {
    let result = HealthCheckResult {
        name: "test-service".to_string(),
        status: HealthStatus::Healthy,
        message: "OK".to_string(),
        response_time_ms: 100,
    };

    let cloned = result.clone();
    assert_eq!(cloned.name, result.name);
    assert_eq!(cloned.status, result.status);
    assert_eq!(cloned.message, result.message);
    assert_eq!(cloned.response_time_ms, result.response_time_ms);
}

/// Test HealthStatus equality
#[test]
async fn test_health_status_equality() {
    let status1 = HealthStatus::Healthy;
    let status2 = HealthStatus::Healthy;
    let status3 = HealthStatus::Degraded;

    assert_eq!(status1, status2);
    assert_ne!(status1, status3);
    assert_ne!(status2, status3);
}

/// Test HealthCheckResult with different response times
#[test]
async fn test_health_check_result_response_times() {
    let response_times = vec![0, 50, 100, 500, 1000, 5000];

    for time in response_times {
        let result = HealthCheckResult {
            name: "test-service".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            response_time_ms: time,
        };

        assert_eq!(result.response_time_ms, time);
    }
}

/// Test HealthCheckResult with different statuses
#[test]
async fn test_health_check_result_statuses() {
    let results = vec![
        HealthCheckResult {
            name: "healthy-service".to_string(),
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            response_time_ms: 100,
        },
        HealthCheckResult {
            name: "degraded-service".to_string(),
            status: HealthStatus::Degraded,
            message: "Slow response".to_string(),
            response_time_ms: 2000,
        },
        HealthCheckResult {
            name: "unhealthy-service".to_string(),
            status: HealthStatus::Unhealthy,
            message: "Service down".to_string(),
            response_time_ms: 0,
        },
    ];

    for result in results {
        assert!(!result.name.is_empty());
        assert!(!result.message.is_empty());
        assert!(result.response_time_ms < 10000); // Should be under 10 seconds
    }
}
