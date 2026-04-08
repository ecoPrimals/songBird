// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

#[test]
fn test_model_type_serialization() -> SongbirdResult<()> {
    assert_eq!(
        serde_json::to_string(&ModelType::Llm).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None
        })?,
        "\"Llm\""
    );
    assert_eq!(
        serde_json::to_string(&ModelType::Vision).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?,
        "\"Vision\""
    );
    assert_eq!(
        serde_json::to_string(&ModelType::Audio).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None
        })?,
        "\"Audio\""
    );
    assert_eq!(
        serde_json::to_string(&ModelType::Embedding).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?,
        "\"Embedding\""
    );
    Ok(())
}

#[test]
fn test_model_type_all_variants() -> SongbirdResult<()> {
    assert_eq!(ModelType::Llm, ModelType::Llm);
    assert_eq!(ModelType::Vision, ModelType::Vision);
    assert_eq!(ModelType::Audio, ModelType::Audio);
    assert_eq!(ModelType::Embedding, ModelType::Embedding);
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_access() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://test-ai:8083".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;
    assert_eq!(adapter.endpoint(), "http://test-ai:8083");
    Ok(())
}

#[tokio::test]
async fn test_adapter_default_timeout() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://test-ai:8083".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;
    assert_eq!(adapter.timeout, Duration::from_secs(15));
    Ok(())
}

#[tokio::test]
async fn test_adapter_custom_timeout() -> SongbirdResult<()> {
    let custom_timeout = Duration::from_secs(45);
    let adapter = AIAdapter::new("http://test-ai:8083".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(custom_timeout);
    assert_eq!(adapter.timeout, custom_timeout);
    Ok(())
}

#[tokio::test]
async fn test_adapter_debug_format() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://test-ai:8083".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;
    let debug_str = format!("{:?}", adapter);
    assert!(debug_str.contains("AIAdapter"));
    assert!(debug_str.contains("http://test-ai:8083"));
    Ok(())
}

#[test]
fn test_ai_metrics_zero_values() {
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

#[test]
fn test_ai_metrics_edge_case_98_percent() {
    // Exactly at overload threshold
    let metrics = AIMetrics {
        active_models: 10,
        total_requests: 10000,
        avg_latency_ms: 500.0,
        accuracy_score: 0.9,
        gpu_utilization_percent: 98.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[test]
fn test_ai_metrics_edge_case_2000ms() {
    // Exactly at overload threshold
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 2000.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[test]
fn test_ai_metrics_perfect_conditions() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 5000,
        avg_latency_ms: 50.0,
        accuracy_score: 0.99,
        gpu_utilization_percent: 25.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}
