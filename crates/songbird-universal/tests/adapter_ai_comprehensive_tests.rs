//! Comprehensive tests for AI Capability Adapter
//!
//! Tests for AI metrics collection, error handling, and edge cases.
//! NOTE: These tests use the capability-based `AIAdapter` (primal-agnostic).

use chrono::Utc;
use songbird_types::SongbirdError;
use songbird_universal::adapters::ai::{AIAdapter, AIHealth, AIMetrics, ModelType};
use std::time::Duration;

/// Helper to create test metrics
fn create_test_metrics() -> AIMetrics {
    AIMetrics {
        active_models: 3,
        total_requests: 1_500,
        avg_latency_ms: 250.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 45.0,
        timestamp: Utc::now(),
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[test]
fn test_ai_adapter_new_success() {
    // Arrange & Act
    let adapter = AIAdapter::new("http://example.com:8083".to_string());

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.unwrap();
    assert_eq!(adapter.endpoint(), "http://example.com:8083");
}

#[test]
fn test_ai_adapter_endpoint_validation() {
    // Arrange & Act
    let adapter = AIAdapter::new("http://ai-service".to_string());

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.unwrap();
    assert_eq!(adapter.endpoint(), "http://ai-service");
}

#[test]
fn test_ai_adapter_with_timeout() {
    // Arrange
    let custom_timeout = Duration::from_secs(60);

    // Act
    let adapter =
        AIAdapter::new("http://example.com".to_string()).unwrap().with_timeout(custom_timeout);

    // Assert
    assert_eq!(adapter.endpoint(), "http://example.com");
}

// ============================================================================
// METRICS CALCULATION TESTS
// ============================================================================

#[test]
fn test_ai_metrics_normal_load() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert
    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[test]
fn test_ai_metrics_high_gpu_load() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 10,
        total_requests: 50_000,
        avg_latency_ms: 500.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 92.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_gpu_load());
}

#[test]
fn test_ai_metrics_high_latency() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 10_000,
        avg_latency_ms: 1200.0, // High latency
        accuracy_score: 0.88,
        gpu_utilization_percent: 60.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_latency());
}

#[test]
fn test_ai_metrics_zero_load() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 0,
        total_requests: 0,
        avg_latency_ms: 0.0,
        accuracy_score: 0.0,
        gpu_utilization_percent: 0.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_ai_health_healthy() {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, AIHealth::Healthy);
}

#[test]
fn test_ai_health_degraded_gpu() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 8,
        total_requests: 20_000,
        avg_latency_ms: 800.0,
        accuracy_score: 0.89,
        gpu_utilization_percent: 91.0, // High GPU usage
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, AIHealth::Degraded);
}

#[test]
fn test_ai_health_degraded_latency() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 15_000,
        avg_latency_ms: 1100.0, // High latency
        accuracy_score: 0.90,
        gpu_utilization_percent: 70.0,
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, AIHealth::Degraded);
}

#[test]
fn test_ai_health_overloaded_gpu() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 20,
        total_requests: 100_000,
        avg_latency_ms: 1500.0,
        accuracy_score: 0.85,
        gpu_utilization_percent: 99.0, // Critical GPU usage
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, AIHealth::Overloaded);
}

#[test]
fn test_ai_health_overloaded_latency() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 15,
        total_requests: 75_000,
        avg_latency_ms: 2500.0, // Critical latency
        accuracy_score: 0.87,
        gpu_utilization_percent: 85.0,
        timestamp: Utc::now(),
    };

    // Act
    let health = metrics.health_status();

    // Assert
    assert_eq!(health, AIHealth::Overloaded);
}

// ============================================================================
// HTTP CLIENT TESTS
// ============================================================================

#[tokio::test]
async fn test_ai_collect_metrics_success() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 3,
                "total_requests": 1500,
                "avg_latency_ms": 250.0,
                "accuracy_score": 0.92,
                "gpu_utilization_percent": 45.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.active_models, 3);
    assert_eq!(metrics.total_requests, 1_500);
    assert!((metrics.accuracy_score - 0.92).abs() < 0.01);
}

#[tokio::test]
async fn test_ai_collect_metrics_server_error_500() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Service { .. }));
}

#[tokio::test]
async fn test_ai_collect_metrics_server_error_503() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(503)
        .with_header("content-type", "text/plain")
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_collect_metrics_network_error() {
    // Arrange
    let adapter = AIAdapter::new("http://nonexistent-host-12345:9999".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn test_ai_collect_metrics_timeout() {
    // Arrange
    // Create adapter with very short timeout for non-responsive endpoint
    let adapter = AIAdapter::new("http://10.255.255.1:9999".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(50));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_collect_metrics_invalid_json() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{ invalid json }")
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_collect_metrics_missing_fields() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"active_models": 3}"#) // Missing required fields
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ai_check_health_healthy() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 3,
                "total_requests": 1500,
                "avg_latency_ms": 250.0,
                "accuracy_score": 0.92,
                "gpu_utilization_percent": 45.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.check_health().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_ai_check_health_degraded() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 8,
                "total_requests": 20000,
                "avg_latency_ms": 1100.0,
                "accuracy_score": 0.89,
                "gpu_utilization_percent": 91.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.check_health().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_ai_check_health_overloaded() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 20,
                "total_requests": 100000,
                "avg_latency_ms": 2500.0,
                "accuracy_score": 0.85,
                "gpu_utilization_percent": 99.0,
                "timestamp": "2025-10-27T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.check_health().await;

    // Assert
    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AIHealth::Overloaded);
}

// ============================================================================
// EDGE CASES AND BOUNDARY CONDITIONS
// ============================================================================

#[test]
fn test_ai_metrics_extreme_gpu_utilization() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 50,
        total_requests: 500_000,
        avg_latency_ms: 3000.0,
        accuracy_score: 0.80,
        gpu_utilization_percent: 100.0, // Maximum GPU
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_gpu_load());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[test]
fn test_ai_metrics_minimal_latency() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 1,
        total_requests: 100,
        avg_latency_ms: 10.0, // Very fast
        accuracy_score: 0.95,
        gpu_utilization_percent: 10.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[test]
fn test_ai_health_enum_equality() {
    // Test enum equality
    assert_eq!(AIHealth::Healthy, AIHealth::Healthy);
    assert_ne!(AIHealth::Healthy, AIHealth::Degraded);
    assert_ne!(AIHealth::Degraded, AIHealth::Overloaded);
    assert_eq!(AIHealth::Overloaded, AIHealth::Overloaded);
}

#[test]
fn test_model_type_enum_equality() {
    // Test enum equality
    assert_eq!(ModelType::Llm, ModelType::Llm);
    assert_ne!(ModelType::Llm, ModelType::Vision);
    assert_ne!(ModelType::Vision, ModelType::Audio);
    assert_eq!(ModelType::Embedding, ModelType::Embedding);
}

#[test]
fn test_ai_metrics_serialization() {
    // Arrange
    let metrics = create_test_metrics();

    // Act
    let json = serde_json::to_string(&metrics);

    // Assert
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("active_models"));
    assert!(json_str.contains("total_requests"));
    assert!(json_str.contains("avg_latency_ms"));
    assert!(json_str.contains("gpu_utilization_percent"));
}

#[test]
fn test_ai_metrics_deserialization() {
    // Arrange
    let json = r#"{
        "active_models": 3,
        "total_requests": 1500,
        "avg_latency_ms": 250.0,
        "accuracy_score": 0.92,
        "gpu_utilization_percent": 45.0,
        "timestamp": "2025-10-27T12:00:00Z"
    }"#;

    // Act
    let metrics: Result<AIMetrics, _> = serde_json::from_str(json);

    // Assert
    assert!(metrics.is_ok());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.active_models, 3);
    assert_eq!(metrics.total_requests, 1_500);
    assert!((metrics.avg_latency_ms - 250.0).abs() < 0.1);
}

#[test]
fn test_ai_metrics_high_accuracy() {
    // Arrange
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 10_000,
        avg_latency_ms: 200.0,
        accuracy_score: 0.99, // Very high accuracy
        gpu_utilization_percent: 50.0,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!((metrics.accuracy_score - 0.99).abs() < f64::EPSILON);
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[test]
fn test_ai_metrics_boundary_gpu_90_percent() {
    // Arrange - exactly at 90% threshold
    let metrics = AIMetrics {
        active_models: 8,
        total_requests: 20_000,
        avg_latency_ms: 500.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 90.0, // Exactly at threshold
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(!metrics.is_high_gpu_load()); // Should be false at exactly 90%
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[test]
fn test_ai_metrics_boundary_gpu_91_percent() {
    // Arrange - just over 90% threshold
    let metrics = AIMetrics {
        active_models: 8,
        total_requests: 20_000,
        avg_latency_ms: 500.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 90.1, // Just over threshold
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_high_gpu_load()); // Should be true just over 90%
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}
