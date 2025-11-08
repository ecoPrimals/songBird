//! Comprehensive tests for Compute Capability Adapter
//!
//! Tests for compute metrics collection, error handling, and edge cases.
//! NOTE: These tests now use the capability-based `ComputeAdapter`.

use chrono::Utc;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::adapters::compute::{ComputeAdapter, ComputeMetrics, HealthStatus};
use std::time::Duration;

/// Helper to create test metrics
fn create_test_metrics() -> ComputeMetrics {
    ComputeMetrics {
        cpu_usage_percent: 45.5,
        memory_usage_bytes: 4_000_000_000,
        memory_available_bytes: 12_000_000_000,
        active_containers: 10,
        queued_jobs: 3,
        performance_score: 0.85,
        timestamp: Utc::now(),
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[test]
fn test_compute_adapter_new_success() -> SongbirdResult<()> {
    // Arrange & Act
    let adapter = ComputeAdapter::new(format!("http://example.com:{}", test_orchestrator_port()));

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(adapter.endpoint(), format!("http://example.com:{}", test_orchestrator_port()));
    Ok(())
}

#[test]
fn test_compute_adapter_endpoint_validation() -> SongbirdResult<()> {
    // Arrange & Act
    let adapter = ComputeAdapter::new("http://compute-service".to_string());

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(adapter.endpoint(), "http://compute-service");
    Ok(())
}

#[test]
fn test_compute_adapter_with_timeout() -> SongbirdResult<()> {
    // 🍼 MIGRATED: Renamed from test_toadstool_adapter_with_timeout
    // Arrange
    let custom_timeout = Duration::from_secs(30);

    // Act
    let adapter = ComputeAdapter::new("http://example.com".to_string())
        .ok_or_else(|| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?
        .with_timeout(custom_timeout);

    // Assert
    assert_eq!(adapter.endpoint(), "http://example.com");
    Ok(())
}

// ============================================================================
// METRICS COLLECTION SUCCESS TESTS
// ============================================================================

#[tokio::test]
async fn test_toadstool_collect_metrics_success() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 45.5,
                "memory_usage_bytes": 4000000000,
                "memory_available_bytes": 12000000000,
                "active_containers": 10,
                "queued_jobs": 3,
                "performance_score": 0.85,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_ok());
    let metrics = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert!((metrics.cpu_usage_percent - 45.5).abs() < 0.001);
    assert_eq!(metrics.memory_usage_bytes, 4_000_000_000);
    assert_eq!(metrics.memory_available_bytes, 12_000_000_000);
    assert_eq!(metrics.active_containers, 10);
    assert_eq!(metrics.queued_jobs, 3);
    assert!((metrics.performance_score - 0.85).abs() < 0.001);
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn test_toadstool_collect_metrics_url_formatting() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 30.0,
                "memory_usage_bytes": 2000000000,
                "memory_available_bytes": 14000000000,
                "active_containers": 5,
                "queued_jobs": 1,
                "performance_score": 0.90,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_ok());
    mock.assert_async().await;
    Ok(())
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_toadstool_collect_metrics_network_error() -> SongbirdResult<()> {
    // Arrange
    let adapter = ComputeAdapter::new("http://nonexistent-host-12345:9999".to_string())
        .ok_or_else(|| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?
        .with_timeout(Duration::from_millis(100));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Network { .. }));
    Ok(())
}

#[tokio::test]
async fn test_toadstool_collect_metrics_timeout() -> SongbirdResult<()> {
    // Arrange
    // Create adapter with very short timeout for non-responsive endpoint
    let adapter = ComputeAdapter::new("http://10.255.255.1:9999".to_string())
        .ok_or_else(|| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?
        .with_timeout(Duration::from_millis(50));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Network { .. }));
    Ok(())
}

#[tokio::test]
async fn test_toadstool_collect_metrics_server_error_500() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Service { .. }));
    assert!(error.to_string().contains("500"));
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn test_toadstool_collect_metrics_server_error_503() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Service { .. }));
    assert!(error.to_string().contains("503"));
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn test_toadstool_collect_metrics_invalid_json() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Service { .. }));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_toadstool_collect_metrics_missing_fields() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"cpu_usage_percent": 50.0}"#) // Missing required fields
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Service { .. }));
    mock.assert_async().await;
    Ok(())
}

// ============================================================================
// HEALTH CHECK TESTS
// ============================================================================

#[tokio::test]
async fn test_toadstool_check_health_healthy() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 45.0,
                "memory_usage_bytes": 4000000000,
                "memory_available_bytes": 12000000000,
                "active_containers": 5,
                "queued_jobs": 2,
                "performance_score": 0.85,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.check_health().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(
        result.ok_or_else(|| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        HealthStatus::Healthy
    );
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn test_toadstool_check_health_degraded() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 85.0,
                "memory_usage_bytes": 6000000000,
                "memory_available_bytes": 2000000000,
                "active_containers": 15,
                "queued_jobs": 8,
                "performance_score": 0.60,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.check_health().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(
        result.ok_or_else(|| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        HealthStatus::Degraded
    );
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn test_toadstool_check_health_unhealthy() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 96.0,
                "memory_usage_bytes": 7600000000,
                "memory_available_bytes": 400000000,
                "active_containers": 25,
                "queued_jobs": 20,
                "performance_score": 0.30,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Act
    let result = adapter.check_health().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(
        result.ok_or_else(|| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        HealthStatus::Unhealthy
    );
    mock.assert_async().await;
    Ok(())
}

// ============================================================================
// METRICS CALCULATION TESTS
// ============================================================================

#[test]
fn test_compute_metrics_total_memory() {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let total = metrics.total_memory_bytes();

    // Assert
    assert_eq!(total, 16_000_000_000); // 4GB + 12GB
}

#[test]
fn test_compute_metrics_memory_percent() {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let percent = metrics.memory_usage_percent();

    // Assert
    assert!((percent - 25.0).abs() < 0.001); // 4GB / 16GB = 25%
}

#[test]
fn test_compute_metrics_memory_percent_zero_total() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 0.0,
        memory_usage_bytes: 0,
        memory_available_bytes: 0,
        active_containers: 0,
        queued_jobs: 0,
        performance_score: 0.0,
        timestamp: Utc::now(),
    };

    // Act
    let percent = metrics.memory_usage_percent();

    // Assert
    assert!((percent - 0.0).abs() < 0.001);
}

#[test]
fn test_compute_metrics_is_high_load_cpu() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 85.0, // > 80%
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 14_000_000_000,
        active_containers: 5,
        queued_jobs: 2,
        performance_score: 0.70,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_load());
}

#[test]
fn test_compute_metrics_is_high_load_memory() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 14_000_000_000, // > 85% of 16GB total
        memory_available_bytes: 2_000_000_000,
        active_containers: 5,
        queued_jobs: 2,
        performance_score: 0.70,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_load());
}

#[test]
fn test_compute_metrics_is_high_load_queued_jobs() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 4_000_000_000,
        memory_available_bytes: 12_000_000_000,
        active_containers: 5,
        queued_jobs: 15, // > 10
        performance_score: 0.70,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_load());
}

#[test]
fn test_compute_metrics_not_high_load() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert
    assert!(!metrics.is_high_load());
}

#[test]
fn test_compute_metrics_health_status_healthy() {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let status = metrics.health_status();

    // Assert
    assert_eq!(status, HealthStatus::Healthy);
}

#[test]
fn test_compute_metrics_health_status_degraded() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 85.0,
        memory_usage_bytes: 6_000_000_000,
        memory_available_bytes: 2_000_000_000,
        active_containers: 15,
        queued_jobs: 8,
        performance_score: 0.60,
        timestamp: Utc::now(),
    };

    // Act
    let status = metrics.health_status();

    // Assert
    assert_eq!(status, HealthStatus::Degraded);
}

#[test]
fn test_compute_metrics_health_status_unhealthy_cpu() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 96.0, // > 95%
        memory_usage_bytes: 4_000_000_000,
        memory_available_bytes: 12_000_000_000,
        active_containers: 10,
        queued_jobs: 3,
        performance_score: 0.50,
        timestamp: Utc::now(),
    };

    // Act
    let status = metrics.health_status();

    // Assert
    assert_eq!(status, HealthStatus::Unhealthy);
}

#[test]
fn test_compute_metrics_health_status_unhealthy_memory() {
    // Arrange
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 15_500_000_000, // > 95% of 16GB total
        memory_available_bytes: 500_000_000,
        active_containers: 10,
        queued_jobs: 3,
        performance_score: 0.50,
        timestamp: Utc::now(),
    };

    // Act
    let status = metrics.health_status();

    // Assert
    assert_eq!(status, HealthStatus::Unhealthy);
}
