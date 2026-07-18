// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

// ========== NEW TESTS (10 tests to reach 85% coverage) ==========

#[test]
fn test_ai_metrics_clone() {
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 45.0,
        timestamp: chrono::Utc::now(),
    };
    let cloned = metrics;
    assert_eq!(cloned.active_models, 5);
    assert_eq!(cloned.total_requests, 1000);
    assert!((cloned.avg_latency_ms - 250.0).abs() < 0.001);
}

#[test]
fn test_ai_health_clone() {
    let health = AIHealth::Degraded;
    let cloned = health;
    assert_eq!(health, cloned);
}

#[test]
fn test_model_type_clone() {
    let model = ModelType::Llm;
    let cloned = model;
    assert_eq!(model, cloned);
}

#[test]
fn test_ai_metrics_deserialization() -> SongbirdResult<()> {
    let json = r#"{
            "active_models": 8,
            "total_requests": 15000,
            "avg_latency_ms": 350.5,
            "accuracy_score": 0.94,
            "gpu_utilization_percent": 75.5,
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

    let metrics: AIMetrics =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {e}"),
            debug_info: None,
        })?;
    assert_eq!(metrics.active_models, 8);
    assert_eq!(metrics.total_requests, 15000);
    assert!((metrics.avg_latency_ms - 350.5).abs() < 0.001);
    assert!((metrics.accuracy_score - 0.94).abs() < 0.001);
    assert!((metrics.gpu_utilization_percent - 75.5).abs() < 0.001);
    Ok(())
}

#[test]
fn test_ai_health_deserialization() -> SongbirdResult<()> {
    let json = r#""Degraded""#;
    let health: AIHealth =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {e}"),
            debug_info: None,
        })?;
    assert_eq!(health, AIHealth::Degraded);

    let json = r#""Overloaded""#;
    let health: AIHealth =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {e}"),
            debug_info: None,
        })?;
    assert_eq!(health, AIHealth::Overloaded);
    Ok(())
}

#[test]
fn test_model_type_deserialization() -> SongbirdResult<()> {
    let json = r#""Vision""#;
    let model: ModelType =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {e}"),
            debug_info: None,
        })?;
    assert_eq!(model, ModelType::Vision);

    let json = r#""Embedding""#;
    let model: ModelType =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {e}"),
            debug_info: None,
        })?;
    assert_eq!(model, ModelType::Embedding);
    Ok(())
}

#[test]
fn test_ai_metrics_debug_format() -> SongbirdResult<()> {
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 45.0,
        timestamp: chrono::Utc::now(),
    };
    let debug_str = format!("{metrics:?}");
    assert!(debug_str.contains("AIMetrics"));
    assert!(debug_str.contains("active_models"));
    Ok(())
}

#[test]
fn test_ai_metrics_max_values() -> SongbirdResult<()> {
    let metrics = AIMetrics {
        active_models: u32::MAX,
        total_requests: u64::MAX,
        avg_latency_ms: f64::MAX,
        accuracy_score: 1.0,
        gpu_utilization_percent: 100.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_gpu_load());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
    Ok(())
}

#[test]
fn test_model_type_debug() -> SongbirdResult<()> {
    let model = ModelType::Vision;
    let debug_str = format!("{model:?}");
    assert!(debug_str.contains("Vision"));

    let model = ModelType::Audio;
    let debug_str = format!("{model:?}");
    assert!(debug_str.contains("Audio"));
    Ok(())
}

#[tokio::test]
async fn test_adapter_chained_timeout() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://test:8083".to_string())
        .await
        .map_err(|e| SongbirdError::configuration(format!("Adapter creation should succeed: {e}")))?
        .with_timeout(Duration::from_secs(10))
        .with_timeout(Duration::from_secs(25));

    assert_eq!(adapter.timeout, Duration::from_secs(25), "Last timeout should be applied");
    Ok(())
}
