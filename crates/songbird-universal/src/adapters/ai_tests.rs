// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

static DISCOVERY_ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_discovery_env() -> std::sync::MutexGuard<'static, ()> {
    DISCOVERY_ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

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

#[tokio::test]
async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = AIAdapter::new("http://ai-provider:8083".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;
    assert_eq!(adapter.endpoint(), "http://ai-provider:8083");
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = AIAdapter::new("http://ai-provider:8083".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(20));
    assert_eq!(adapter.timeout, Duration::from_secs(20));
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
            message: format!("Deserialization should succeed: {}", e),
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
            message: format!("Deserialization should succeed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(health, AIHealth::Degraded);

    let json = r#""Overloaded""#;
    let health: AIHealth =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {}", e),
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
            message: format!("Deserialization should succeed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(model, ModelType::Vision);

    let json = r#""Embedding""#;
    let model: ModelType =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Deserialization should succeed: {}", e),
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
    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("AIMetrics"));
    assert!(debug_str.contains("active_models"));
    Ok(())
}

#[tokio::test]
async fn test_adapter_chained_timeout() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://test:8083".to_string())
        .await
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(10))
        .with_timeout(Duration::from_secs(25));

    assert_eq!(adapter.timeout, Duration::from_secs(25), "Last timeout should be applied");
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
    let debug_str = format!("{:?}", model);
    assert!(debug_str.contains("Vision"));

    let model = ModelType::Audio;
    let debug_str = format!("{:?}", model);
    assert!(debug_str.contains("Audio"));
    Ok(())
}

// --- AIAdapter protocol detection & discovery (no live services) ---

#[tokio::test]
async fn test_ai_adapter_new_tarpc_localhost_port() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("tarpc://localhost:1234".to_string()).await?;
    assert_eq!(adapter.endpoint(), "tarpc://localhost:1234");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_new_unix_tmp_test_sock() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("unix:///tmp/test.sock".to_string()).await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/test.sock");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_new_tarpc_invalid_hostname_err() {
    let err = AIAdapter::new("tarpc://test:1234".to_string())
        .await
        .expect_err("tarpc hostname must be localhost or IP");
    assert!(
        err.to_string().contains("Invalid hostname") || err.to_string().contains("configuration"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_resolver_injected_tarpc() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, "tarpc://127.0.0.1:9101".to_string());
    let adapter = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9101");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_resolver_injected_unix() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, "unix:///tmp/injected-ai.sock".to_string());
    let adapter = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/injected-ai.sock");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_songbird_ai_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_AI_ENDPOINT", "http://from-songbird-ai:7788");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from SONGBIRD_AI_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://from-songbird-ai:7788");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_ai_provider_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("AI_PROVIDER_ENDPOINT", "http://from-legacy-ai:7799");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from AI_PROVIDER_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://from-legacy-ai:7799");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_squirrel_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SQUIRREL_ENDPOINT", "http://from-squirrel:7700");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from SQUIRREL_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://from-squirrel:7700");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_host_and_port_env() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_HOST", "http://custom-ai-host");
    songbird_process_env::set_var("SONGBIRD_AI_PORT", "8811");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from host+port fallback");
    assert_eq!(adapter.endpoint(), "http://custom-ai-host:8811");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_prefers_songbird_ai_env() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_AI_ENDPOINT", "http://songbird-wins:1111");
    songbird_process_env::set_var("AI_PROVIDER_ENDPOINT", "http://legacy-loses:2222");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter");
    assert_eq!(adapter.endpoint(), "http://songbird-wins:1111");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_with_timeout_and_endpoint_tarpc() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("tarpc://127.0.0.1:9000".to_string())
        .await?
        .with_timeout(Duration::from_millis(400));
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9000");
    assert_eq!(adapter.timeout, Duration::from_millis(400));
    Ok(())
}
