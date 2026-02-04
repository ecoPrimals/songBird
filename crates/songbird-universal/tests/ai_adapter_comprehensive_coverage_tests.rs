// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Comprehensive AI Adapter Coverage Tests
//!
//! **Goal**: Raise coverage from 64.62% to 85%+
//!
//! This test suite focuses on:
//! - AIMetrics GPU and latency calculations
//! - AIHealth status transitions (Healthy → Degraded → Overloaded)
//! - ModelType variants
//! - Adapter creation and configuration
//!
//! **Modern Rust Patterns**:
//! - Comprehensive boundary testing
//! - Zero unsafe code

use songbird_universal::adapters::ai::{AIAdapter, AIHealth, AIMetrics, ModelType};
use std::time::Duration;

// ============================================================================
// AI METRICS COMPREHENSIVE TESTS
// ============================================================================

#[tokio::test]
async fn test_ai_metrics_healthy_system() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 65.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_ai_metrics_degraded_high_gpu() {
    let metrics = AIMetrics {
        active_models: 4,
        total_requests: 1500,
        avg_latency_ms: 400.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 95.0, // > 90%
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_metrics_degraded_high_latency() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1200,
        avg_latency_ms: 1500.0, // > 1000ms
        accuracy_score: 0.88,
        gpu_utilization_percent: 70.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_high_gpu_load());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_metrics_overloaded_extreme_gpu() {
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 2000,
        avg_latency_ms: 800.0,
        accuracy_score: 0.85,
        gpu_utilization_percent: 99.0, // > 98%
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[tokio::test]
async fn test_ai_metrics_overloaded_extreme_latency() {
    let metrics = AIMetrics {
        active_models: 4,
        total_requests: 1800,
        avg_latency_ms: 2500.0, // > 2000ms
        accuracy_score: 0.80,
        gpu_utilization_percent: 75.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

// Boundary tests for GPU load
#[tokio::test]
async fn test_ai_metrics_boundary_gpu_90_percent() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 300.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 90.0, // Exactly at boundary
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high (needs > 90)
    assert!(!metrics.is_high_gpu_load());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_ai_metrics_boundary_gpu_91_percent() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 300.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 91.0, // Just over boundary
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_gpu_load());
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_metrics_boundary_gpu_98_percent() {
    let metrics = AIMetrics {
        active_models: 4,
        total_requests: 1500,
        avg_latency_ms: 400.0,
        accuracy_score: 0.88,
        gpu_utilization_percent: 98.0, // Exactly at overload boundary
        timestamp: chrono::Utc::now(),
    };

    // Should be Degraded, not Overloaded (needs > 98)
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_metrics_boundary_gpu_99_percent() {
    let metrics = AIMetrics {
        active_models: 4,
        total_requests: 1500,
        avg_latency_ms: 400.0,
        accuracy_score: 0.88,
        gpu_utilization_percent: 99.0, // Just over overload boundary
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

// Boundary tests for latency
#[tokio::test]
async fn test_ai_metrics_boundary_latency_1000ms() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 1000.0, // Exactly at boundary
        accuracy_score: 0.92,
        gpu_utilization_percent: 70.0,
        timestamp: chrono::Utc::now(),
    };

    // Should NOT be high (needs > 1000)
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_ai_metrics_boundary_latency_1001ms() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 1001.0, // Just over boundary
        accuracy_score: 0.92,
        gpu_utilization_percent: 70.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_metrics_boundary_latency_2000ms() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1200,
        avg_latency_ms: 2000.0, // Exactly at overload boundary
        accuracy_score: 0.85,
        gpu_utilization_percent: 75.0,
        timestamp: chrono::Utc::now(),
    };

    // Should be Degraded, not Overloaded (needs > 2000)
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_metrics_boundary_latency_2001ms() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1200,
        avg_latency_ms: 2001.0, // Just over overload boundary
        accuracy_score: 0.85,
        gpu_utilization_percent: 75.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[tokio::test]
async fn test_ai_metrics_zero_values() {
    let metrics = AIMetrics {
        active_models: 0,
        total_requests: 0,
        avg_latency_ms: 0.0,
        accuracy_score: 0.0,
        gpu_utilization_percent: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_ai_metrics_extreme_values() {
    let metrics = AIMetrics {
        active_models: 100,
        total_requests: u64::MAX,
        avg_latency_ms: 10000.0,
        accuracy_score: 1.0,
        gpu_utilization_percent: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_gpu_load());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[tokio::test]
async fn test_ai_metrics_serialization() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 65.0,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics);
    assert!(json.is_ok());
    let json_str = json.expect("test precondition");
    assert!(json_str.contains("active_models"));
    assert!(json_str.contains("1000"));
}

#[tokio::test]
async fn test_ai_metrics_deserialization() {
    let json = r#"{
        "active_models": 4,
        "total_requests": 1500,
        "avg_latency_ms": 300.5,
        "accuracy_score": 0.92,
        "gpu_utilization_percent": 72.5,
        "timestamp": "2025-11-18T12:00:00Z"
    }"#;

    let metrics: Result<AIMetrics, _> = serde_json::from_str(json);
    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.active_models, 4);
    assert_eq!(metrics.total_requests, 1500);
}

#[tokio::test]
async fn test_ai_metrics_clone() {
    let metrics = AIMetrics {
        active_models: 2,
        total_requests: 800,
        avg_latency_ms: 200.0,
        accuracy_score: 0.88,
        gpu_utilization_percent: 55.0,
        timestamp: chrono::Utc::now(),
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.active_models, metrics.active_models);
    assert_eq!(cloned.total_requests, metrics.total_requests);
}

#[tokio::test]
async fn test_ai_metrics_debug() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 65.0,
        timestamp: chrono::Utc::now(),
    };

    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("AIMetrics"));
}

// ============================================================================
// AI HEALTH TESTS
// ============================================================================

#[tokio::test]
async fn test_ai_health_all_variants() {
    let healthy = AIHealth::Healthy;
    let degraded = AIHealth::Degraded;
    let overloaded = AIHealth::Overloaded;

    assert_ne!(healthy, degraded);
    assert_ne!(healthy, overloaded);
    assert_ne!(degraded, overloaded);
}

#[tokio::test]
async fn test_ai_health_equality() {
    assert_eq!(AIHealth::Healthy, AIHealth::Healthy);
    assert_eq!(AIHealth::Degraded, AIHealth::Degraded);
    assert_eq!(AIHealth::Overloaded, AIHealth::Overloaded);
}

#[tokio::test]
async fn test_ai_health_clone() {
    let health = AIHealth::Degraded;
    let cloned = health;
    assert_eq!(health, cloned);
}

#[tokio::test]
async fn test_ai_health_copy() {
    let health = AIHealth::Healthy;
    let copied = health;
    assert_eq!(health, copied);
}

#[tokio::test]
async fn test_ai_health_debug() {
    let health = AIHealth::Overloaded;
    let debug_str = format!("{:?}", health);
    assert!(debug_str.contains("Overloaded"));
}

#[tokio::test]
async fn test_ai_health_serialization() {
    let states = vec![AIHealth::Healthy, AIHealth::Degraded, AIHealth::Overloaded];

    for state in states {
        let json = serde_json::to_string(&state);
        assert!(json.is_ok(), "Should serialize {:?}", state);
    }
}

#[tokio::test]
async fn test_ai_health_deserialization() {
    let test_cases = vec![
        (r#""Healthy""#, AIHealth::Healthy),
        (r#""Degraded""#, AIHealth::Degraded),
        (r#""Overloaded""#, AIHealth::Overloaded),
    ];

    for (json, expected) in test_cases {
        let health: Result<AIHealth, _> = serde_json::from_str(json);
        assert!(health.is_ok(), "Should deserialize: {}", json);
        assert_eq!(health.expect("test precondition"), expected);
    }
}

// ============================================================================
// MODEL TYPE TESTS
// ============================================================================

#[tokio::test]
async fn test_model_type_all_variants() {
    let llm = ModelType::Llm;
    let vision = ModelType::Vision;
    let audio = ModelType::Audio;
    let embedding = ModelType::Embedding;

    assert_ne!(llm, vision);
    assert_ne!(llm, audio);
    assert_ne!(llm, embedding);
    assert_ne!(vision, audio);
}

#[tokio::test]
async fn test_model_type_equality() {
    assert_eq!(ModelType::Llm, ModelType::Llm);
    assert_eq!(ModelType::Vision, ModelType::Vision);
    assert_eq!(ModelType::Audio, ModelType::Audio);
    assert_eq!(ModelType::Embedding, ModelType::Embedding);
}

#[tokio::test]
async fn test_model_type_clone() {
    let model = ModelType::Vision;
    let cloned = model;
    assert_eq!(model, cloned);
}

#[tokio::test]
async fn test_model_type_copy() {
    let model = ModelType::Audio;
    let copied = model;
    assert_eq!(model, copied);
}

#[tokio::test]
async fn test_model_type_debug() {
    let model = ModelType::Llm;
    let debug_str = format!("{:?}", model);
    assert!(debug_str.contains("Llm"));
}

#[tokio::test]
async fn test_model_type_serialization() {
    let types = vec![ModelType::Llm, ModelType::Vision, ModelType::Audio, ModelType::Embedding];

    for model_type in types {
        let json = serde_json::to_string(&model_type);
        assert!(json.is_ok(), "Should serialize {:?}", model_type);
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_new_success() {
    let endpoint = "http://localhost:8082".to_string();
    let adapter = AIAdapter::new(endpoint.clone()).await;

    assert!(adapter.is_ok());
    let adapter = adapter.expect("test precondition");
    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test]
async fn test_adapter_new_various_endpoints() {
    let endpoints = vec![
        "http://localhost:8082",
        "https://ai.example.com",
        "http://192.168.1.100:9000",
        "http://[::1]:8082",
    ];

    for endpoint in endpoints {
        let adapter = AIAdapter::new(endpoint.to_string()).await;
        assert!(adapter.is_ok(), "Should handle endpoint: {}", endpoint);
    }
}

#[tokio::test]
async fn test_adapter_with_timeout() {
    let endpoint = "http://localhost:8082".to_string();
    let adapter = AIAdapter::new(endpoint).await.expect("test precondition");

    let custom_timeout = Duration::from_secs(30);
    let _adapter_with_timeout = adapter.with_timeout(custom_timeout);
}

#[tokio::test]
async fn test_adapter_endpoint_getter() {
    let endpoint = "http://ai-service:8082".to_string();
    let adapter = AIAdapter::new(endpoint.clone()).await.expect("test precondition");

    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test]
async fn test_adapter_builder_pattern() {
    let adapter = AIAdapter::new("http://localhost:8082".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_secs(25));

    assert_eq!(adapter.endpoint(), "http://localhost:8082");
}

#[tokio::test]
async fn test_multiple_adapters_independent() {
    let adapter1 = AIAdapter::new("http://ai1:8082".to_string()).await.expect("test precondition");
    let adapter2 = AIAdapter::new("http://ai2:8083".to_string()).await.expect("test precondition");

    assert_eq!(adapter1.endpoint(), "http://ai1:8082");
    assert_eq!(adapter2.endpoint(), "http://ai2:8083");
    assert_ne!(adapter1.endpoint(), adapter2.endpoint());
}

// ============================================================================
// WORKFLOW TESTS
// ============================================================================

#[tokio::test]
async fn test_ai_workflow_normal_operation() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 65.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), AIHealth::Healthy);
    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
}

#[tokio::test]
async fn test_ai_workflow_degrading_system() {
    // System starts healthy
    let mut metrics = AIMetrics {
        active_models: 2,
        total_requests: 500,
        avg_latency_ms: 300.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 60.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), AIHealth::Healthy);

    // Load increases - degraded
    metrics.total_requests = 1000;
    metrics.avg_latency_ms = 1200.0;
    metrics.gpu_utilization_percent = 92.0;
    assert_eq!(metrics.health_status(), AIHealth::Degraded);

    // System becomes overloaded
    metrics.total_requests = 2000;
    metrics.avg_latency_ms = 2500.0;
    metrics.gpu_utilization_percent = 99.0;
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}
