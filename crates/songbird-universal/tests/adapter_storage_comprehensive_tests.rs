//! Comprehensive tests for Storage Capability Adapter
//!
//! Tests for storage metrics collection, error handling, and edge cases.
//! NOTE: These tests use the capability-based `StorageAdapter` (primal-agnostic).

use chrono::Utc;
use songbird_test_utils::test_health_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::adapters::storage::{StorageAdapter, StorageHealth, StorageMetrics};
use std::time::Duration;

/// Helper to create test metrics
fn create_test_metrics() -> StorageMetrics {
    StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000, // 1TB
        used_bytes: 250_000_000_000,             // 250GB
        available_bytes: 750_000_000_000,        // 750GB
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: Utc::now(),
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[test]
fn test_storage_adapter_new_success() -> SongbirdResult<()> {
    // Arrange & Act
    let adapter = StorageAdapter::new(format!("http://example.com:{}", test_health_port()));

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(adapter.endpoint(), format!("http://example.com:{}", test_health_port()));
    Ok(())
}

#[test]
fn test_storage_adapter_endpoint_validation() -> SongbirdResult<()> {
    // Arrange & Act
    let adapter = StorageAdapter::new("http://storage-service".to_string());

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(adapter.endpoint(), "http://storage-service");
    Ok(())
}

#[test]
fn test_storage_adapter_with_timeout() -> SongbirdResult<()> {
    // Arrange
    let custom_timeout = Duration::from_secs(30);

    // Act
    let adapter = StorageAdapter::new("http://example.com".to_string())
        .ok_or_else(|| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?
        .with_timeout(custom_timeout);

    // Assert
    assert_eq!(adapter.endpoint(), "http://example.com");
    Ok(())
}

// ============================================================================
// METRICS CALCULATION TESTS
// ============================================================================

#[test]
fn test_storage_metrics_usage_percent() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: Utc::now(),
    };

    // Act
    let usage = metrics.usage_percent();

    // Assert
    assert!((usage - 25.0).abs() < 0.1);
}

#[test]
fn test_storage_metrics_usage_percent_zero_capacity() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        object_count: 0,
        avg_read_latency_ms: 0.0,
        avg_write_latency_ms: 0.0,
        timestamp: Utc::now(),
    };

    // Act
    let usage = metrics.usage_percent();

    // Assert
    assert!((usage - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_storage_metrics_nearly_full() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 950_000_000_000, // 95%
        available_bytes: 50_000_000_000,
        object_count: 50_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 30.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_nearly_full());
}

#[test]
fn test_storage_metrics_not_nearly_full() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert
    assert!(!metrics.is_nearly_full());
}

#[test]
fn test_storage_metrics_high_latency_read() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 150.0, // High read latency
        avg_write_latency_ms: 25.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_latency());
}

#[test]
fn test_storage_metrics_high_latency_write() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 250.0, // High write latency
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_latency());
}

#[test]
fn test_storage_metrics_normal_latency() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert
    assert!(!metrics.is_high_latency());
}

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_storage_health_healthy() {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, StorageHealth::Healthy);
}

#[test]
fn test_storage_health_warning_high_usage() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 870_000_000_000, // 87%
        available_bytes: 130_000_000_000,
        object_count: 25_000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 80.0,
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, StorageHealth::Warning);
}

#[test]
fn test_storage_health_warning_high_latency() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000, // 50%
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 120.0, // High read latency
        avg_write_latency_ms: 180.0,
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, StorageHealth::Warning);
}

#[test]
fn test_storage_health_critical_full() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 970_000_000_000, // 97%
        available_bytes: 30_000_000_000,
        object_count: 100_000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, StorageHealth::Critical);
}

#[test]
fn test_storage_health_critical_write_latency() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000, // 50%
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 600.0, // Critical write latency
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, StorageHealth::Critical);
}

// ============================================================================
// HTTP CLIENT TESTS
// ============================================================================

#[tokio::test]
async fn test_storage_collect_metrics_success() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 250000000000,
                "available_bytes": 750000000000,
                "object_count": 1500,
                "avg_read_latency_ms": 15.0,
                "avg_write_latency_ms": 25.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    let metrics = result.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(metrics.total_capacity_bytes, 1_000_000_000_000);
    assert_eq!(metrics.used_bytes, 250_000_000_000);
    assert_eq!(metrics.object_count, 1_500);
    Ok(())
}

#[tokio::test]
async fn test_storage_collect_metrics_server_error_500() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Service { .. }));
    Ok(())
}

#[tokio::test]
async fn test_storage_collect_metrics_server_error_503() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(503)
        .with_header("content-type", "text/plain")
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_storage_collect_metrics_network_error() -> SongbirdResult<()> {
    // Arrange
    let adapter = StorageAdapter::new("http://nonexistent-host-12345:9999".to_string())
        .ok_or_else(|| {
            SongbirdError::configuration("Missing performance configuration".to_string())
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
async fn test_storage_collect_metrics_timeout() -> SongbirdResult<()> {
    // Arrange
    // Create adapter with very short timeout for non-responsive endpoint
    let adapter = StorageAdapter::new("http://10.255.255.1:9999".to_string())
        .ok_or_else(|| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?
        .with_timeout(Duration::from_millis(50));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_storage_collect_metrics_invalid_json() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{ invalid json }")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_storage_collect_metrics_missing_fields() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"total_capacity_bytes": 1000}"#) // Missing required fields
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_storage_check_health_healthy() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 250000000000,
                "available_bytes": 750000000000,
                "object_count": 1500,
                "avg_read_latency_ms": 15.0,
                "avg_write_latency_ms": 25.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url())
        .or_else(|_| SongbirdError::configuration("Failed health check".to_string()))?;

    // Act
    let result = adapter.check_health().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result?, StorageHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn test_storage_check_health_warning() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 870000000000,
                "available_bytes": 130000000000,
                "object_count": 25000,
                "avg_read_latency_ms": 120.0,
                "avg_write_latency_ms": 180.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url())
        .or_else(|_| SongbirdError::configuration("Failed health check".to_string()))?;

    // Act
    let result = adapter.check_health().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result?, StorageHealth::Warning);
    Ok(())
}

#[tokio::test]
async fn test_storage_check_health_critical() -> SongbirdResult<()> {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 970000000000,
                "available_bytes": 30000000000,
                "object_count": 100000,
                "avg_read_latency_ms": 50.0,
                "avg_write_latency_ms": 600.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url())
        .or_else(|_| SongbirdError::configuration("Failed health check".to_string()))?;

    // Act
    let result = adapter.check_health().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result?, StorageHealth::Critical);
    Ok(())
}

// ============================================================================
// EDGE CASES AND BOUNDARY CONDITIONS
// ============================================================================

#[test]
fn test_storage_metrics_100_percent_full() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 1_000_000_000_000, // 100% full
        available_bytes: 0,
        object_count: 100_000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!((metrics.usage_percent() - 100.0).abs() < f64::EPSILON);
    assert!(metrics.is_nearly_full());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_storage_metrics_empty() {
    // Arrange
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 0, // Empty
        available_bytes: 1_000_000_000_000,
        object_count: 0,
        avg_read_latency_ms: 5.0,
        avg_write_latency_ms: 10.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!((metrics.usage_percent() - 0.0).abs() < f64::EPSILON);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_health_enum_equality() -> SongbirdResult<()> {
    // Test enum equality
    assert_eq!(StorageHealth::Healthy, StorageHealth::Healthy);
    assert_ne!(StorageHealth::Healthy, StorageHealth::Warning);
    assert_ne!(StorageHealth::Warning, StorageHealth::Critical);
    assert_eq!(StorageHealth::Critical, StorageHealth::Critical);
    Ok(())
}

#[test]
fn test_storage_metrics_serialization() -> SongbirdResult<()> {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let json = serde_json::to_string(&metrics);

    // Assert
    assert!(json.is_ok());
    let json_str = json.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert!(json_str.contains("total_capacity_bytes"));
    assert!(json_str.contains("used_bytes"));
    assert!(json_str.contains("object_count"));
    Ok(())
}

#[test]
fn test_storage_metrics_deserialization() -> SongbirdResult<()> {
    // Arrange
    let json = r#"{
        "total_capacity_bytes": 1000000000000,
        "used_bytes": 250000000000,
        "available_bytes": 750000000000,
        "object_count": 1500,
        "avg_read_latency_ms": 15.0,
        "avg_write_latency_ms": 25.0,
        "timestamp": "2025-10-27T12:00:00Z"
    }"#;

    // Act
    let metrics: Result<StorageMetrics, _> = serde_json::from_str(json);

    // Assert
    assert!(metrics.is_ok());
    let metrics = metrics.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(metrics.total_capacity_bytes, 1_000_000_000_000);
    assert_eq!(metrics.used_bytes, 250_000_000_000);
    assert_eq!(metrics.object_count, 1_500);
    Ok(())
}
