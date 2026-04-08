// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_ai_metrics_calculations() {
    let metrics = AIMetrics {
        active_models: 3,
        total_requests: 1_500,
        avg_latency_ms: 250.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 45.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[test]
fn test_ai_overloaded() {
    let metrics = AIMetrics {
        active_models: 20,
        total_requests: 50_000,
        avg_latency_ms: 2500.0,
        accuracy_score: 0.88,
        gpu_utilization_percent: 99.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_gpu_load());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[test]
fn test_ai_degraded() -> SongbirdResult<()> {
    let metrics = AIMetrics {
        active_models: 8,
        total_requests: 10_000,
        avg_latency_ms: 1200.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 92.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_high_gpu_load());
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
    Ok(())
}

#[test]
fn test_model_type_equality() {
    assert_eq!(ModelType::Llm, ModelType::Llm);
    assert_ne!(ModelType::Llm, ModelType::Vision);
    assert_eq!(ModelType::Audio, ModelType::Audio);
}

#[test]
fn test_ai_health_equality() {
    assert_eq!(AIHealth::Healthy, AIHealth::Healthy);
    assert_eq!(AIHealth::Degraded, AIHealth::Degraded);
    assert_eq!(AIHealth::Overloaded, AIHealth::Overloaded);
    assert_ne!(AIHealth::Healthy, AIHealth::Degraded);
    assert_ne!(AIHealth::Degraded, AIHealth::Overloaded);
}

#[test]
fn test_ai_metrics_high_gpu_boundary() {
    // Just below threshold
    let metrics_below = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 100.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 90.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_high_gpu_load());

    // Just above threshold
    let metrics_above = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 100.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 90.1,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_high_gpu_load());
}

#[test]
fn test_ai_metrics_high_latency_boundary() {
    // Just below threshold
    let metrics_below = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 1000.0,
        accuracy_score: 0.95,
        gpu_utilization_percent: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_below.is_high_latency());

    // Just above threshold
    let metrics_above = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 1000.1,
        accuracy_score: 0.95,
        gpu_utilization_percent: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_above.is_high_latency());
}

#[test]
fn test_health_status_overloaded_high_gpu() {
    let metrics = AIMetrics {
        active_models: 10,
        total_requests: 20000,
        avg_latency_ms: 500.0,
        accuracy_score: 0.9,
        gpu_utilization_percent: 98.1,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[test]
fn test_health_status_overloaded_high_latency() {
    let metrics = AIMetrics {
        active_models: 10,
        total_requests: 20000,
        avg_latency_ms: 2001.0,
        accuracy_score: 0.9,
        gpu_utilization_percent: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), AIHealth::Overloaded);
}

#[test]
fn test_health_status_degraded_gpu() {
    let metrics = AIMetrics {
        active_models: 8,
        total_requests: 10000,
        avg_latency_ms: 800.0,
        accuracy_score: 0.9,
        gpu_utilization_percent: 91.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[test]
fn test_health_status_degraded_latency() {
    let metrics = AIMetrics {
        active_models: 8,
        total_requests: 10000,
        avg_latency_ms: 1100.0,
        accuracy_score: 0.9,
        gpu_utilization_percent: 50.0,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), AIHealth::Degraded);
}

#[test]
fn test_ai_metrics_serialization() -> SongbirdResult<()> {
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 1000,
        avg_latency_ms: 250.0,
        accuracy_score: 0.92,
        gpu_utilization_percent: 45.0,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;
    assert!(json.contains("active_models"));
    assert!(json.contains(":5"));
    Ok(())
}

#[test]
fn test_ai_health_serialization() -> SongbirdResult<()> {
    let health = AIHealth::Healthy;
    let json = serde_json::to_string(&health).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;
    assert!(json.contains("Healthy"));

    let degraded = AIHealth::Degraded;
    let json = serde_json::to_string(&degraded).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;
    assert!(json.contains("Degraded"));

    let overloaded = AIHealth::Overloaded;
    let json = serde_json::to_string(&overloaded).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;
    assert!(json.contains("Overloaded"));
    Ok(())
}
