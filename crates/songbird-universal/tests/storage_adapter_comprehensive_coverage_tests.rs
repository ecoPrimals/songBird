// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Comprehensive Storage Adapter Coverage Tests
//!
//! **Goal**: Raise coverage from 66.50% to 85%+
//!
//! This test suite focuses on:
//! - StorageMetrics calculations (usage%, latency)
//! - StorageHealth status transitions (Healthy → Warning → Critical)
//! - Adapter creation and configuration
//! - Boundary testing for thresholds
//!
//! **Modern Rust Patterns**:
//! - Comprehensive boundary testing
//! - Zero unsafe code

use songbird_universal::adapters::storage::{StorageAdapter, StorageHealth, StorageMetrics};
use std::time::Duration;

// ============================================================================
// STORAGE METRICS COMPREHENSIVE TESTS
// ============================================================================

#[tokio::test]
async fn test_storage_metrics_healthy_system() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000, // 1TB
        used_bytes: 400_000_000_000,             // 400GB (40%)
        available_bytes: 600_000_000_000,
        object_count: 10000,
        avg_read_latency_ms: 25.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 40.0);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[tokio::test]
async fn test_storage_metrics_usage_calculation() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 750, // 75%
        available_bytes: 250,
        object_count: 100,
        avg_read_latency_ms: 30.0,
        avg_write_latency_ms: 60.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 75.0);
}

#[tokio::test]
async fn test_storage_metrics_warning_high_usage() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 870_000_000_000, // 87%
        available_bytes: 130_000_000_000,
        object_count: 50000,
        avg_read_latency_ms: 40.0,
        avg_write_latency_ms: 80.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_warning_high_read_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 400_000_000_000, // 40%
        available_bytes: 600_000_000_000,
        object_count: 10000,
        avg_read_latency_ms: 150.0, // > 100ms
        avg_write_latency_ms: 80.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_warning_high_write_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 400_000_000_000,
        available_bytes: 600_000_000_000,
        object_count: 10000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 250.0, // > 200ms
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_critical_usage() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 960_000_000_000, // 96%
        available_bytes: 40_000_000_000,
        object_count: 100000,
        avg_read_latency_ms: 45.0,
        avg_write_latency_ms: 90.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[tokio::test]
async fn test_storage_metrics_critical_write_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000, // 50%
        available_bytes: 500_000_000_000,
        object_count: 20000,
        avg_read_latency_ms: 60.0,
        avg_write_latency_ms: 600.0, // > 500ms
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

// Boundary tests for storage usage
#[tokio::test]
async fn test_storage_metrics_boundary_usage_85() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 850, // Exactly 85%
        available_bytes: 150,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    // Should be Healthy (needs > 85)
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[tokio::test]
async fn test_storage_metrics_boundary_usage_86() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 860, // 86%
        available_bytes: 140,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_boundary_usage_90() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 900, // Exactly 90%
        available_bytes: 100,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be nearly full (needs > 90)
    assert!(!metrics.is_nearly_full());
}

#[tokio::test]
async fn test_storage_metrics_boundary_usage_91() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 910, // 91%
        available_bytes: 90,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_nearly_full());
}

#[tokio::test]
async fn test_storage_metrics_boundary_usage_95() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 950, // Exactly 95%
        available_bytes: 50,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    // Should be Warning (needs > 95 for Critical)
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_boundary_usage_96() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 960, // 96%
        available_bytes: 40,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

// Boundary tests for read latency
#[tokio::test]
async fn test_storage_metrics_boundary_read_latency_100() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 500,
        available_bytes: 500,
        object_count: 100,
        avg_read_latency_ms: 100.0, // Exactly at boundary
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high (needs > 100)
    assert!(!metrics.is_high_latency());
}

#[tokio::test]
async fn test_storage_metrics_boundary_read_latency_101() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 500,
        available_bytes: 500,
        object_count: 100,
        avg_read_latency_ms: 101.0, // Just over boundary
        avg_write_latency_ms: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

// Boundary tests for write latency
#[tokio::test]
async fn test_storage_metrics_boundary_write_latency_200() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 500,
        available_bytes: 500,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 200.0, // Exactly at boundary
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high (needs > 200)
    assert!(!metrics.is_high_latency());
}

#[tokio::test]
async fn test_storage_metrics_boundary_write_latency_201() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 500,
        available_bytes: 500,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 201.0, // Just over boundary
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_boundary_write_latency_500() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 500,
        available_bytes: 500,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 500.0, // Exactly at Critical boundary
        timestamp: chrono::Utc::now(),
    };

    // Should be Warning (needs > 500 for Critical)
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_storage_metrics_boundary_write_latency_501() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 500,
        available_bytes: 500,
        object_count: 100,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 501.0, // Just over Critical boundary
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[tokio::test]
async fn test_storage_metrics_zero_capacity() {
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
}

#[tokio::test]
async fn test_storage_metrics_extreme_values() {
    let metrics = StorageMetrics {
        total_capacity_bytes: u64::MAX,
        used_bytes: u64::MAX - 1000,
        available_bytes: 1000,
        object_count: u64::MAX,
        avg_read_latency_ms: 10000.0,
        avg_write_latency_ms: 10000.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_nearly_full());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}

#[tokio::test]
async fn test_storage_metrics_serialization() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 400_000_000_000,
        available_bytes: 600_000_000_000,
        object_count: 10000,
        avg_read_latency_ms: 25.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics);
    assert!(json.is_ok());
    let json_str = json.expect("test precondition");
    assert!(json_str.contains("total_capacity_bytes"));
}

#[tokio::test]
async fn test_storage_metrics_deserialization() {
    let json = r#"{
        "total_capacity_bytes": 2000000000000,
        "used_bytes": 800000000000,
        "available_bytes": 1200000000000,
        "object_count": 50000,
        "avg_read_latency_ms": 35.5,
        "avg_write_latency_ms": 75.2,
        "timestamp": "2025-11-18T12:00:00Z"
    }"#;

    let metrics: Result<StorageMetrics, _> = serde_json::from_str(json);
    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.object_count, 50000);
}

#[tokio::test]
async fn test_storage_metrics_clone() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 400,
        available_bytes: 600,
        object_count: 100,
        avg_read_latency_ms: 25.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.total_capacity_bytes, metrics.total_capacity_bytes);
    assert_eq!(cloned.object_count, metrics.object_count);
}

#[tokio::test]
async fn test_storage_metrics_debug() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1000,
        used_bytes: 400,
        available_bytes: 600,
        object_count: 100,
        avg_read_latency_ms: 25.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("StorageMetrics"));
}

// ============================================================================
// STORAGE HEALTH TESTS
// ============================================================================

#[tokio::test]
async fn test_storage_health_all_variants() {
    let healthy = StorageHealth::Healthy;
    let warning = StorageHealth::Warning;
    let critical = StorageHealth::Critical;

    assert_ne!(healthy, warning);
    assert_ne!(healthy, critical);
    assert_ne!(warning, critical);
}

#[tokio::test]
async fn test_storage_health_equality() {
    assert_eq!(StorageHealth::Healthy, StorageHealth::Healthy);
    assert_eq!(StorageHealth::Warning, StorageHealth::Warning);
    assert_eq!(StorageHealth::Critical, StorageHealth::Critical);
}

#[tokio::test]
async fn test_storage_health_clone() {
    let health = StorageHealth::Warning;
    let cloned = health;
    assert_eq!(health, cloned);
}

#[tokio::test]
async fn test_storage_health_copy() {
    let health = StorageHealth::Healthy;
    let copied = health;
    assert_eq!(health, copied);
}

#[tokio::test]
async fn test_storage_health_debug() {
    let health = StorageHealth::Critical;
    let debug_str = format!("{:?}", health);
    assert!(debug_str.contains("Critical"));
}

#[tokio::test]
async fn test_storage_health_serialization() {
    let states = vec![StorageHealth::Healthy, StorageHealth::Warning, StorageHealth::Critical];

    for state in states {
        let json = serde_json::to_string(&state);
        assert!(json.is_ok(), "Should serialize {:?}", state);
    }
}

#[tokio::test]
async fn test_storage_health_deserialization() {
    let test_cases = vec![
        (r#""Healthy""#, StorageHealth::Healthy),
        (r#""Warning""#, StorageHealth::Warning),
        (r#""Critical""#, StorageHealth::Critical),
    ];

    for (json, expected) in test_cases {
        let health: Result<StorageHealth, _> = serde_json::from_str(json);
        assert!(health.is_ok(), "Should deserialize: {}", json);
        assert_eq!(health.expect("test precondition"), expected);
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_new_success() {
    let endpoint = "http://localhost:8084".to_string();
    let adapter = StorageAdapter::new(endpoint.clone()).await;

    assert!(adapter.is_ok());
    let adapter = adapter.expect("test precondition");
    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test]
async fn test_adapter_new_various_endpoints() {
    let endpoints = vec![
        "http://localhost:8084",
        "https://storage.example.com",
        "http://192.168.1.100:9000",
        "http://[::1]:8084",
    ];

    for endpoint in endpoints {
        let adapter = StorageAdapter::new(endpoint.to_string()).await;
        assert!(adapter.is_ok(), "Should handle endpoint: {}", endpoint);
    }
}

#[tokio::test]
async fn test_adapter_with_timeout() {
    let endpoint = "http://localhost:8084".to_string();
    let adapter = StorageAdapter::new(endpoint).await.expect("test precondition").await;

    let custom_timeout = Duration::from_secs(45);
    let _adapter_with_timeout = adapter.with_timeout(custom_timeout);
}

#[tokio::test]
async fn test_adapter_endpoint_getter() {
    let endpoint = "http://storage-service:8084".to_string();
    let adapter = StorageAdapter::new(endpoint.clone()).expect("test precondition").await;

    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test]
async fn test_adapter_builder_pattern() {
    let adapter = StorageAdapter::new("http://localhost:8084".to_string())
        .expect("test precondition")
        .with_timeout(Duration::from_secs(60));

    assert_eq!(adapter.endpoint(), "http://localhost:8084");
}

// ============================================================================
// WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_storage_workflow_normal_operation() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 400_000_000_000,
        available_bytes: 600_000_000_000,
        object_count: 10000,
        avg_read_latency_ms: 25.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_high_latency());
}

#[tokio::test]
async fn test_storage_workflow_degrading_system() {
    // System starts healthy
    let mut metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000, // 50%
        available_bytes: 500_000_000_000,
        object_count: 10000,
        avg_read_latency_ms: 30.0,
        avg_write_latency_ms: 60.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), StorageHealth::Healthy);

    // Storage fills up - warning
    metrics.used_bytes = 880_000_000_000; // 88%
    metrics.available_bytes = 120_000_000_000;
    metrics.object_count = 50000;
    metrics.avg_write_latency_ms = 250.0; // High latency
    assert_eq!(metrics.health_status(), StorageHealth::Warning);

    // Storage becomes critical
    metrics.used_bytes = 970_000_000_000; // 97%
    metrics.available_bytes = 30_000_000_000;
    metrics.object_count = 100000;
    metrics.avg_write_latency_ms = 600.0;
    assert_eq!(metrics.health_status(), StorageHealth::Critical);
}
