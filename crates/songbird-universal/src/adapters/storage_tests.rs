// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
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

// ========== Protocol detection and adapter construction ==========

fn assert_protocol_debug(adapter: &StorageAdapter, expected: &str) {
    let dbg = format!("{adapter:?}");
    assert!(dbg.contains(expected), "expected Debug to contain {expected:?}, got {dbg}");
}

#[tokio::test]
async fn test_new_selects_tarpc_protocol() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("tarpc://127.0.0.1:9200".to_string()).await?;
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9200");
    assert_protocol_debug(&adapter, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_jsonrpc_for_unix() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("unix:///tmp/songbird-storage-test.sock".to_string()).await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/songbird-storage-test.sock");
    assert_protocol_debug(&adapter, "JsonRpc");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_http_for_http() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("http://storage:8082".to_string()).await?;
    assert_protocol_debug(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_http_for_https() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("https://storage.example:8443".to_string()).await?;
    assert_protocol_debug(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_unknown_scheme_falls_back_to_http() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("ftp://example:21".to_string()).await?;
    assert_protocol_debug(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_protocol_debug_formatting() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("tarpc://localhost:9300".to_string()).await?;
    let dbg = format!("{adapter:?}");
    assert!(dbg.contains("StorageAdapter"));
    assert!(dbg.contains("tarpc://localhost:9300"));
    Ok(())
}

#[tokio::test]
async fn test_with_timeout_preserves_protocol() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("tarpc://127.0.0.1:9400".to_string())
        .await?
        .with_timeout(Duration::from_millis(500));
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9400");
    assert_eq!(adapter.timeout, Duration::from_millis(500));
    assert_protocol_debug(&adapter, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn test_storage_provider_trait_impl() -> SongbirdResult<()> {
    struct StaticStorage(StorageMetrics);

    impl StorageProvider for StaticStorage {
        async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics> {
            Ok(self.0.clone())
        }
    }

    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    let provider = StaticStorage(metrics.clone());
    let health = provider.check_storage_health().await?;
    assert_eq!(health, metrics.health_status());
    Ok(())
}

// --- HTTP + discovery (mockito): exercise `collect_metrics` / `check_health` / trait on adapter ---

#[tokio::test]
async fn unit_http_collect_metrics_success() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .expect(2)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 3500000000,
            "available_bytes": 6500000000,
            "object_count": 5000,
            "avg_read_latency_ms": 12.5,
            "avg_write_latency_ms": 18.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;
    assert_eq!(metrics.total_capacity_bytes, 10_000_000_000);
    assert_eq!(metrics.object_count, 5000);
    assert!((metrics.avg_read_latency_ms - 12.5).abs() < 0.01);

    let again = StorageProvider::collect_storage_metrics(&adapter).await?;
    assert_eq!(again.object_count, metrics.object_count);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn unit_http_check_health_maps_from_metrics() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 8700000000,
            "available_bytes": 1300000000,
            "object_count": 8000,
            "avg_read_latency_ms": 15.0,
            "avg_write_latency_ms": 20.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await?;
    let health = adapter.check_health().await?;
    assert_eq!(health, StorageHealth::Warning);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn unit_http_collect_metrics_http_error_status() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("adapter");
    let err = adapter.collect_metrics().await.expect_err("expected HTTP error");
    assert!(err.to_string().contains("503"), "{}", err);

    mock.assert_async().await;
}

#[tokio::test]
async fn unit_http_collect_metrics_invalid_json_body() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("adapter");
    let err = adapter.collect_metrics().await.expect_err("parse error");
    assert!(err.to_string().contains("Failed to parse storage metrics"), "{}", err);

    mock.assert_async().await;
}

#[tokio::test]
async fn unit_http_collect_metrics_epoch_timestamp_is_replaced() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 5000000000,
            "used_bytes": 1000000000,
            "available_bytes": 4000000000,
            "object_count": 2000,
            "avg_read_latency_ms": 8.0,
            "avg_write_latency_ms": 10.0,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;
    let now = chrono::Utc::now();
    assert!((now - metrics.timestamp).num_seconds().abs() < 10);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn unit_from_discovery_with_resolver_injected_endpoint() -> SongbirdResult<()> {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();
    let mut m = HashMap::new();
    m.insert(CapabilityType::Storage, endpoint.clone());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let adapter = StorageAdapter::from_discovery_with_resolver(resolver).await?;
    assert_eq!(adapter.endpoint(), endpoint.as_str());
    Ok(())
}

/// `songbird-test-utils` mock storage primal: smoke test (metrics shape used in integration scenarios).
#[test]
fn storage_mock_provider_fixture_from_test_utils() {
    use songbird_test_utils::mocks::storage_provider::MockStorageProvider;

    let mock = MockStorageProvider::new();
    let m = mock.get_metrics();
    assert!(m.total_capacity_bytes > 0);
    mock.simulate_near_capacity();
    assert!(mock.get_metrics().available_bytes < 100_000_000_000);
}
