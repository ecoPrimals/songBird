// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

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
