// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use crate::adapters::transport::{AdapterTransportKind, DelayTransport, MockTransport};
use serde_json::json;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use std::time::Duration;

// --- collect_metrics / check_health / AIProvider (MockTransport), legacy env deprecation ---

#[tokio::test]
async fn collect_metrics_mock_transport_valid_json_verifies_fields() -> SongbirdResult<()> {
    let body = json!({
        "active_models": 4,
        "total_requests": 9001,
        "avg_latency_ms": 12.5,
        "accuracy_score": 0.97,
        "gpu_utilization_percent": 42.0,
        "timestamp": "2024-06-01T12:00:00Z"
    });
    let adapter = AIAdapter::with_transport(
        "http://mock-ai".to_string(),
        Arc::new(MockTransport::new(vec![Ok(body)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_eq!(m.active_models, 4);
    assert_eq!(m.total_requests, 9001);
    assert!((m.avg_latency_ms - 12.5).abs() < 1e-9);
    assert!((m.accuracy_score - 0.97).abs() < 1e-9);
    assert!((m.gpu_utilization_percent - 42.0).abs() < 1e-9);
    Ok(())
}

#[tokio::test]
async fn collect_metrics_times_out_with_delay_transport() {
    let delayed = DelayTransport {
        inner: Arc::new(MockTransport::new(vec![])),
        delay: Duration::from_secs(30),
    };
    let adapter = AIAdapter::with_transport(
        "http://mock-ai".to_string(),
        Arc::new(delayed),
        AdapterTransportKind::Http,
        Duration::from_millis(20),
    );
    let err = adapter.collect_metrics().await.expect_err("should time out");
    assert!(err.to_string().to_lowercase().contains("timeout"), "unexpected: {err}");
}

#[tokio::test]
async fn collect_metrics_http_transport_error_passes_through() {
    let boom = SongbirdError::network("upstream http failure");
    let adapter = AIAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom.clone())])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("transport error");
    assert_eq!(err.to_string(), boom.to_string());
}

#[tokio::test]
async fn check_health_delegates_to_collect_metrics() -> SongbirdResult<()> {
    let body = serde_json::to_value(AIMetrics {
        active_models: 3,
        total_requests: 100,
        avg_latency_ms: 100.0,
        accuracy_score: 1.0,
        gpu_utilization_percent: 50.0,
        timestamp: chrono::Utc::now(),
    })?;
    let adapter = AIAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(body)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    assert_eq!(adapter.check_health().await?, AIHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn ai_provider_collect_ai_metrics_uses_mock_transport() -> SongbirdResult<()> {
    let body = json!({
        "active_models": 1,
        "total_requests": 2,
        "avg_latency_ms": 3.0,
        "accuracy_score": 0.5,
        "gpu_utilization_percent": 10.0,
        "timestamp": "2024-01-01T00:00:00Z"
    });
    let adapter = AIAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(body)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = AIProvider::collect_ai_metrics(&adapter).await?;
    assert_eq!(m.active_models, 1);
    assert_eq!(m.total_requests, 2);
    Ok(())
}

#[tokio::test]
async fn ai_provider_check_ai_health_uses_mock_transport() -> SongbirdResult<()> {
    let body = serde_json::to_value(AIMetrics {
        active_models: 2,
        total_requests: 10,
        avg_latency_ms: 50.0,
        accuracy_score: 0.99,
        gpu_utilization_percent: 30.0,
        timestamp: chrono::Utc::now(),
    })?;
    let adapter = AIAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(body)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let h = AIProvider::check_ai_health(&adapter).await?;
    assert_eq!(h, AIHealth::Healthy);
    Ok(())
}
