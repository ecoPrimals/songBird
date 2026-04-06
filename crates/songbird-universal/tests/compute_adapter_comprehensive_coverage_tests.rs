// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive Compute Adapter Coverage Tests
//!
//! **Goal**: Raise coverage from 60.13% to 85%+
//!
//! This test suite focuses on:
//! - ComputeMetrics calculations and edge cases
//! - HealthStatus transitions and boundaries
//! - Adapter creation and configuration
//! - Error condition handling
//! - Serialization/deserialization
//! - Trait implementations
//!
//! **Modern Rust Patterns**:
//! - Comprehensive boundary testing
//! - Idiomatic error handling
//! - Zero unsafe code

use songbird_universal::adapters::compute::{
    ComputeAdapter, ComputeMetrics, HealthStatus as ComputeHealth,
};
use std::time::Duration;

// ============================================================================
// COMPUTE METRICS COMPREHENSIVE TESTS
// ============================================================================

#[tokio::test]
async fn test_compute_metrics_healthy_system() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 45.0,
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 10,
        queued_jobs: 2,
        performance_score: 0.92,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.total_memory_bytes(), 8_000_000_000);
    assert!((metrics.memory_usage_percent() - 25.0).abs() < 0.1);
    assert!(!metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Healthy);
}

#[tokio::test]
async fn test_compute_metrics_degraded_high_cpu() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 85.0, // Just above 80% threshold
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 15,
        queued_jobs: 5,
        performance_score: 0.65,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_degraded_high_memory() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 7_000_000_000, // 87.5% usage
        memory_available_bytes: 1_000_000_000,
        active_containers: 12,
        queued_jobs: 3,
        performance_score: 0.70,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_degraded_high_queue() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 8,
        queued_jobs: 12, // Above 10 threshold
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_unhealthy_extreme_cpu() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 98.0, // Above 95% threshold
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 25,
        queued_jobs: 8,
        performance_score: 0.30,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_compute_metrics_unhealthy_extreme_memory() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 60.0,
        memory_usage_bytes: 7_800_000_000, // 97.5% usage
        memory_available_bytes: 200_000_000,
        active_containers: 20,
        queued_jobs: 5,
        performance_score: 0.25,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_compute_metrics_boundary_cpu_80_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 80.0, // Exactly at boundary
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 10,
        queued_jobs: 5,
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high load (needs > 80)
    assert!(!metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Healthy);
}

#[tokio::test]
async fn test_compute_metrics_boundary_cpu_81_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 81.0, // Just over boundary
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 10,
        queued_jobs: 5,
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_boundary_memory_85_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 6_800_000_000, // Exactly 85%
        memory_available_bytes: 1_200_000_000,
        active_containers: 10,
        queued_jobs: 5,
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high load (needs > 85)
    assert!(!metrics.is_high_load());
}

#[tokio::test]
async fn test_compute_metrics_boundary_memory_86_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 6_880_000_000, // 86%
        memory_available_bytes: 1_120_000_000,
        active_containers: 10,
        queued_jobs: 5,
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_boundary_queue_10() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 10,
        queued_jobs: 10, // Exactly at boundary
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high load (needs > 10)
    assert!(!metrics.is_high_load());
}

#[tokio::test]
async fn test_compute_metrics_boundary_queue_11() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 10,
        queued_jobs: 11, // Just over boundary
        performance_score: 0.75,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_boundary_cpu_95_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 95.0, // Exactly at unhealthy boundary
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 15,
        queued_jobs: 5,
        performance_score: 0.50,
        timestamp: chrono::Utc::now(),
    };

    // Should be degraded, not unhealthy (needs > 95)
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_boundary_cpu_96_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 96.0, // Just over unhealthy boundary
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 15,
        queued_jobs: 5,
        performance_score: 0.50,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_compute_metrics_boundary_memory_95_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 7_600_000_000, // Exactly 95%
        memory_available_bytes: 400_000_000,
        active_containers: 15,
        queued_jobs: 5,
        performance_score: 0.50,
        timestamp: chrono::Utc::now(),
    };

    // Should be degraded, not unhealthy (needs > 95)
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);
}

#[tokio::test]
async fn test_compute_metrics_boundary_memory_96_percent() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 7_680_000_000, // 96%
        memory_available_bytes: 320_000_000,
        active_containers: 15,
        queued_jobs: 5,
        performance_score: 0.50,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_compute_metrics_zero_memory() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 0,
        memory_available_bytes: 0,
        active_containers: 0,
        queued_jobs: 0,
        performance_score: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.total_memory_bytes(), 0);
    assert_eq!(metrics.memory_usage_percent(), 0.0);
    assert!(!metrics.is_high_load());
}

#[tokio::test]
async fn test_compute_metrics_extreme_values() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 100.0,
        memory_usage_bytes: u64::MAX / 2,
        memory_available_bytes: 1000,
        active_containers: 1000,
        queued_jobs: 1000,
        performance_score: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_compute_metrics_serialization() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 45.0,
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 10,
        queued_jobs: 2,
        performance_score: 0.92,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics);
    assert!(json.is_ok(), "Should serialize successfully");

    let json_str = json.expect("test precondition");
    assert!(json_str.contains("cpu_usage_percent"));
    assert!(json_str.contains("45"));
}

#[tokio::test]
async fn test_compute_metrics_deserialization() {
    let json = r#"{
        "cpu_usage_percent": 55.5,
        "memory_usage_bytes": 3000000000,
        "memory_available_bytes": 5000000000,
        "active_containers": 15,
        "queued_jobs": 3,
        "performance_score": 0.88,
        "timestamp": "2025-11-18T12:00:00Z"
    }"#;

    let metrics: Result<ComputeMetrics, _> = serde_json::from_str(json);
    assert!(metrics.is_ok(), "Should deserialize successfully");

    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.cpu_usage_percent, 55.5);
    assert_eq!(metrics.active_containers, 15);
}

#[tokio::test]
async fn test_compute_metrics_clone() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 60.0,
        memory_usage_bytes: 4_000_000_000,
        memory_available_bytes: 4_000_000_000,
        active_containers: 12,
        queued_jobs: 4,
        performance_score: 0.85,
        timestamp: chrono::Utc::now(),
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.cpu_usage_percent, metrics.cpu_usage_percent);
    assert_eq!(cloned.active_containers, metrics.active_containers);
}

#[tokio::test]
async fn test_compute_metrics_debug() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 45.0,
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 10,
        queued_jobs: 2,
        performance_score: 0.92,
        timestamp: chrono::Utc::now(),
    };

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("ComputeMetrics"));
    assert!(debug_str.contains("45"));
}

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[tokio::test]
async fn test_health_status_all_variants() {
    let healthy = ComputeHealth::Healthy;
    let degraded = ComputeHealth::Degraded;
    let unhealthy = ComputeHealth::Unhealthy;

    assert_ne!(healthy, degraded);
    assert_ne!(healthy, unhealthy);
    assert_ne!(degraded, unhealthy);
}

#[tokio::test]
async fn test_health_status_equality() {
    assert_eq!(ComputeHealth::Healthy, ComputeHealth::Healthy);
    assert_eq!(ComputeHealth::Degraded, ComputeHealth::Degraded);
    assert_eq!(ComputeHealth::Unhealthy, ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_health_status_clone() {
    let health = ComputeHealth::Degraded;
    let cloned = health;
    assert_eq!(health, cloned);
}

#[tokio::test]
async fn test_health_status_copy() {
    let health = ComputeHealth::Healthy;
    let copied = health; // Copy trait
    assert_eq!(health, copied);
}

#[tokio::test]
async fn test_health_status_debug() {
    let health = ComputeHealth::Unhealthy;
    let debug_str = format!("{:?}", health);
    assert!(debug_str.contains("Unhealthy"));
}

#[tokio::test]
async fn test_health_status_serialization() {
    let states = vec![ComputeHealth::Healthy, ComputeHealth::Degraded, ComputeHealth::Unhealthy];

    for state in states {
        let json = serde_json::to_string(&state);
        assert!(json.is_ok(), "Should serialize {:?}", state);
    }
}

#[tokio::test]
async fn test_health_status_deserialization() {
    let test_cases = vec![
        (r#""Healthy""#, ComputeHealth::Healthy),
        (r#""Degraded""#, ComputeHealth::Degraded),
        (r#""Unhealthy""#, ComputeHealth::Unhealthy),
    ];

    for (json, expected) in test_cases {
        let health: Result<ComputeHealth, _> = serde_json::from_str(json);
        assert!(health.is_ok(), "Should deserialize: {}", json);
        assert_eq!(health.expect("test precondition"), expected);
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_new_success() {
    let endpoint = "http://localhost:8080".to_string();
    let adapter = ComputeAdapter::new(endpoint.clone()).await;

    assert!(adapter.is_ok(), "Should create adapter successfully");
    let adapter = adapter.expect("test precondition");
    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test]
async fn test_adapter_new_various_endpoints() {
    let endpoints = vec![
        "http://localhost:8080",
        "https://compute.example.com",
        "http://192.168.1.100:9000",
        "http://[::1]:8080",
    ];

    for endpoint in endpoints {
        let adapter = ComputeAdapter::new(endpoint.to_string()).await;
        assert!(adapter.is_ok(), "Should handle endpoint: {}", endpoint);
    }
}

#[tokio::test]
async fn test_adapter_with_timeout() {
    let endpoint = "http://localhost:8080".to_string();
    let adapter = ComputeAdapter::new(endpoint).await.expect("test precondition");

    let custom_timeout = Duration::from_secs(20);
    let _adapter_with_timeout = adapter.with_timeout(custom_timeout);
    // Adapter should be created successfully with custom timeout
}

#[tokio::test]
async fn test_adapter_with_various_timeouts() {
    let endpoint = "http://localhost:8080".to_string();

    let timeouts = vec![
        Duration::from_millis(500),
        Duration::from_secs(5),
        Duration::from_secs(30),
        Duration::from_secs(120),
    ];

    for timeout in timeouts {
        let adapter = ComputeAdapter::new(endpoint.clone())
            .await
            .expect("test precondition")
            .with_timeout(timeout);
        assert_eq!(adapter.endpoint(), "http://localhost:8080");
    }
}

#[tokio::test]
async fn test_adapter_endpoint_getter() {
    let endpoint = "http://compute-service:8080".to_string();
    let adapter = ComputeAdapter::new(endpoint.clone()).await.expect("test precondition");

    assert_eq!(adapter.endpoint(), &endpoint);
    assert_eq!(adapter.endpoint(), "http://compute-service:8080");
}

#[tokio::test]
async fn test_adapter_builder_pattern() {
    let adapter = ComputeAdapter::new("http://localhost:8080".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_secs(15));

    assert_eq!(adapter.endpoint(), "http://localhost:8080");
}

#[tokio::test]
async fn test_multiple_adapters_independent() {
    let adapter1 =
        ComputeAdapter::new("http://compute1:8080".to_string()).await.expect("test precondition");
    let adapter2 =
        ComputeAdapter::new("http://compute2:8081".to_string()).await.expect("test precondition");

    assert_eq!(adapter1.endpoint(), "http://compute1:8080");
    assert_eq!(adapter2.endpoint(), "http://compute2:8081");
    assert_ne!(adapter1.endpoint(), adapter2.endpoint());
}

// ============================================================================
// WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_compute_workflow_normal_operation() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 45.0,
        memory_usage_bytes: 2_000_000_000,
        memory_available_bytes: 6_000_000_000,
        active_containers: 8,
        queued_jobs: 2,
        performance_score: 0.92,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), ComputeHealth::Healthy);
    assert!(!metrics.is_high_load());

    // Simulate normal operations
    let health = metrics.health_status();
    match health {
        ComputeHealth::Healthy => {
            // Can accept more work
            assert!(metrics.cpu_usage_percent < 80.0);
            assert!(metrics.queued_jobs <= 10);
        }
        _ => panic!("Expected healthy status"),
    }
}

#[tokio::test]
async fn test_compute_workflow_degrading_system() {
    // System starts healthy
    let mut metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 3_000_000_000,
        memory_available_bytes: 5_000_000_000,
        active_containers: 10,
        queued_jobs: 3,
        performance_score: 0.90,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), ComputeHealth::Healthy);

    // Load increases
    metrics.cpu_usage_percent = 85.0;
    metrics.queued_jobs = 8;
    assert_eq!(metrics.health_status(), ComputeHealth::Degraded);

    // System becomes critical
    metrics.cpu_usage_percent = 98.0;
    metrics.memory_usage_bytes = 7_800_000_000;
    metrics.memory_available_bytes = 200_000_000;
    metrics.queued_jobs = 25;
    assert_eq!(metrics.health_status(), ComputeHealth::Unhealthy);
}

#[tokio::test]
async fn test_adapter_endpoint_special_characters() {
    let endpoints = vec![
        "http://compute:8080/api/v1",
        "https://compute.example.com:443",
        "http://192.168.1.1:8080",
        "http://[::1]:8080",
    ];

    for endpoint in endpoints {
        let adapter = ComputeAdapter::new(endpoint.to_string()).await;
        assert!(adapter.is_ok(), "Should handle: {}", endpoint);
    }
}
