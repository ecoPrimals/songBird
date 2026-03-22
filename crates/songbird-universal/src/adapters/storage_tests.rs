// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

#[test]
fn test_storage_metrics_calculations() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000, // 1TB
        used_bytes: 250_000_000_000,             // 250GB
        available_bytes: 750_000_000_000,        // 750GB
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };

    assert!((metrics.usage_percent() - 25.0).abs() < 0.1);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_nearly_full() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 960_000_000_000, // 96%
        available_bytes: 40_000_000_000,
        object_count: 50_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 600.0, // High write latency
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_nearly_full());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_storage_warning() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 870_000_000_000, // 87%
        available_bytes: 130_000_000_000,
        object_count: 25_000,
        avg_read_latency_ms: 120.0, // High read latency
        avg_write_latency_ms: 180.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_nearly_full());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
    let adapter =
        StorageAdapter::new("http://storage-provider:8082".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
    assert_eq!(adapter.endpoint(), "http://storage-provider:8082");
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = StorageAdapter::new("http://storage-provider:8082".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(10));
    assert_eq!(adapter.timeout, Duration::from_secs(10));
    Ok(())
}

#[test]
fn test_storage_health_equality() {
    assert_eq!(StorageHealth::Healthy, StorageHealth::Healthy);
    assert_eq!(StorageHealth::Warning, StorageHealth::Warning);
    assert_eq!(StorageHealth::Critical, StorageHealth::Critical);
    assert_ne!(StorageHealth::Healthy, StorageHealth::Warning);
    assert_ne!(StorageHealth::Warning, StorageHealth::Critical);
}

#[test]
fn test_storage_usage_zero_capacity() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        object_count: 0,
        avg_read_latency_ms: 10.0,
        avg_write_latency_ms: 20.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 0.0);
}

#[test]
fn test_nearly_full_boundary() {
    // Just below threshold (90%)
    let metrics_below = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 900_000_000_000,
        available_bytes: 100_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_nearly_full());

    // Just above threshold
    let metrics_above = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 901_000_000_000,
        available_bytes: 99_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_nearly_full());
}

#[test]
fn test_high_read_latency_boundary() {
    // Just below threshold
    let metrics_below = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 100.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_high_latency());

    // Just above threshold
    let metrics_above = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 100.1,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_high_latency());
}

#[test]
fn test_high_write_latency_boundary() {
    // Just below threshold
    let metrics_below = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 200.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_high_latency());

    // Just above threshold
    let metrics_above = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 200.1,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_high_latency());
}

#[test]
fn test_health_status_critical_high_usage() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 960_000_000_000, // 96%
        available_bytes: 40_000_000_000,
        object_count: 50_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_health_status_critical_high_write_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 501.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[test]
fn test_health_status_warning_high_usage() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 860_000_000_000, // 86%
        available_bytes: 140_000_000_000,
        object_count: 25_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_health_status_warning_high_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 120.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_health_status_boundary_95_usage() {
    // Exactly at critical threshold
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 950_000_000_000, // 95%
        available_bytes: 50_000_000_000,
        object_count: 40_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_health_status_boundary_500ms_write() {
    // Exactly at critical threshold
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 10_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 500.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_storage_metrics_serialization() -> SongbirdResult<()> {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;
    assert!(json.contains("total_capacity_bytes"));
    assert!(json.contains("object_count"));
    Ok(())
}

#[test]
fn test_storage_health_serialization() -> SongbirdResult<()> {
    assert_eq!(
        serde_json::to_string(&StorageHealth::Healthy).map_err(|e| {
            SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None,
            }
        })?,
        "\"Healthy\""
    );
    assert_eq!(
        serde_json::to_string(&StorageHealth::Warning).map_err(|e| {
            SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None,
            }
        })?,
        "\"Warning\""
    );
    assert_eq!(
        serde_json::to_string(&StorageHealth::Critical).map_err(|e| {
            SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {}", e),
                debug_info: None,
            }
        })?,
        "\"Critical\""
    );
    Ok(())
}

#[tokio::test]
async fn test_adapter_default_timeout() -> SongbirdResult<()> {
    let adapter =
        StorageAdapter::new("http://storage-service:8082".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
    assert_eq!(adapter.timeout, Duration::from_secs(5));
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_access() -> SongbirdResult<()> {
    let adapter =
        StorageAdapter::new("http://test-storage:9000".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
    assert_eq!(adapter.endpoint(), "http://test-storage:9000");
    Ok(())
}

#[tokio::test]
async fn test_adapter_debug_format() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("http://storage:8082".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;
    let debug_str = format!("{:?}", adapter);
    assert!(debug_str.contains("StorageAdapter"));
    assert!(debug_str.contains("http://storage:8082"));
    Ok(())
}

#[test]
fn test_storage_metrics_perfect_conditions() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 100_000_000_000, // 10%
        available_bytes: 900_000_000_000,
        object_count: 500,
        avg_read_latency_ms: 5.0,
        avg_write_latency_ms: 10.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_metrics_all_zero() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        object_count: 0,
        avg_read_latency_ms: 0.0,
        avg_write_latency_ms: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 0.0);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_100_percent_full() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 1_000_000_000_000, // 100%
        available_bytes: 0,
        object_count: 100_000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 100.0);
    assert!(metrics.is_nearly_full());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

// ========== NEW TESTS (10 tests to reach 85% coverage) ==========

#[test]
fn test_storage_metrics_clone() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    let cloned = metrics;
    assert_eq!(cloned.total_capacity_bytes, 1_000_000_000_000);
    assert_eq!(cloned.used_bytes, 250_000_000_000);
    assert_eq!(cloned.object_count, 1_500);
}

#[test]
fn test_storage_health_clone() {
    let health = StorageHealth::Warning;
    let cloned = health;
    assert_eq!(health, cloned);
}

#[test]
fn test_storage_metrics_deserialization() -> SongbirdResult<()> {
    let json = r#"{
        "total_capacity_bytes": 5000000000000,
        "used_bytes": 1000000000000,
        "available_bytes": 4000000000000,
        "object_count": 8500,
        "avg_read_latency_ms": 35.5,
        "avg_write_latency_ms": 75.2,
        "timestamp": "2024-01-01T00:00:00Z"
    }"#;

    let metrics: StorageMetrics =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(metrics.total_capacity_bytes, 5_000_000_000_000);
    assert_eq!(metrics.used_bytes, 1_000_000_000_000);
    assert_eq!(metrics.object_count, 8500);
    assert!((metrics.avg_read_latency_ms - 35.5).abs() < 0.001);
    Ok(())
}

#[test]
fn test_storage_health_deserialization() -> SongbirdResult<()> {
    let json = r#""Warning""#;
    let health: StorageHealth =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(health, StorageHealth::Warning);

    let json = r#""Critical""#;
    let health: StorageHealth =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(health, StorageHealth::Critical);
    Ok(())
}

#[test]
fn test_storage_metrics_debug() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("StorageMetrics"));
    assert!(debug_str.contains("object_count"));
}

#[test]
fn test_storage_health_debug() {
    let health = StorageHealth::Critical;
    let debug_str = format!("{:?}", health);
    assert!(debug_str.contains("Critical"));
}

#[tokio::test]
async fn test_adapter_chained_timeout() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("http://storage:8082".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(12));

    assert_eq!(adapter.timeout, Duration::from_secs(12), "Last timeout should be applied");
    Ok(())
}

#[test]
fn test_storage_metrics_edge_case_85_percent() {
    // Exactly at warning threshold
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 850_000_000_000, // 85%
        available_bytes: 150_000_000_000,
        object_count: 20_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[test]
fn test_storage_metrics_edge_case_86_percent() {
    // Just above warning threshold
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 860_000_000_000, // 86%
        available_bytes: 140_000_000_000,
        object_count: 20_000,
        avg_read_latency_ms: 20.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[test]
fn test_storage_metrics_max_values() {
    let metrics = StorageMetrics {
        total_capacity_bytes: u64::MAX,
        used_bytes: u64::MAX,
        available_bytes: 0,
        object_count: u64::MAX,
        avg_read_latency_ms: f64::MAX,
        avg_write_latency_ms: f64::MAX,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 100.0);
    assert!(metrics.is_nearly_full());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}
