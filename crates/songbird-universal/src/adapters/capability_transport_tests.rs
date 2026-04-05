// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

//! Mock [`CapabilityTransport`] coverage for security, compute, and AI adapters.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;

use crate::adapters::AIAdapter;
use crate::adapters::compute::ComputeAdapter;
use crate::adapters::security::SecurityAdapter;
use crate::adapters::transport::{AdapterTransportKind, DelayTransport, MockTransport};
use crate::trust_types::{TrustEvaluationRequest, TrustEvaluationResponse};
use songbird_types::{SongbirdError, SongbirdResult, TrustLevel};

fn security_metrics_json() -> serde_json::Value {
    json!({
        "active_sessions": 1,
        "failed_auth_attempts": 0,
        "blocked_ips": 0,
        "security_score": 0.9,
        "timestamp": "2020-01-01T00:00:00Z"
    })
}

#[tokio::test]
async fn security_collect_metrics_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(security_metrics_json())]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_eq!(m.active_sessions, 1);
    assert!((m.security_score - 0.9).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn security_collect_metrics_parse_error() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("not-an-object"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("invalid metrics JSON should fail parse");
    assert!(matches!(err, SongbirdError::Security(_)));
}

#[tokio::test]
async fn security_collect_metrics_timeout() {
    let inner = Arc::new(MockTransport::new(vec![Ok(security_metrics_json())]));
    let slow = Arc::new(DelayTransport {
        inner,
        delay: Duration::from_secs(60),
    });
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        slow,
        AdapterTransportKind::Http,
        Duration::from_millis(5),
    );
    let err = adapter.collect_metrics().await.expect_err("should time out");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_verify_auth_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("Authorized"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let r = adapter.verify_auth("tok").await?;
    assert_eq!(r, crate::adapters::security::AuthResult::Authorized);
    Ok(())
}

#[tokio::test]
async fn security_verify_auth_unauthorized() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("Unauthorized"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let r = adapter.verify_auth("bad").await?;
    assert_eq!(r, crate::adapters::security::AuthResult::Unauthorized);
    Ok(())
}

#[tokio::test]
async fn security_verify_auth_parse_error() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!(42))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.verify_auth("x").await.expect_err("bad auth JSON");
    assert!(matches!(err, SongbirdError::Security(_)));
}

#[tokio::test]
async fn security_verify_auth_timeout() {
    let inner = Arc::new(MockTransport::new(vec![Ok(json!("Authorized"))]));
    let slow = Arc::new(DelayTransport {
        inner,
        delay: Duration::from_secs(60),
    });
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        slow,
        AdapterTransportKind::Http,
        Duration::from_millis(5),
    );
    let err = adapter.verify_auth("t").await.expect_err("timeout");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_call_generic_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({"ok": true}))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let v = adapter.call_generic("m", json!({})).await?;
    assert_eq!(v, json!({"ok": true}));
    Ok(())
}

#[tokio::test]
async fn security_call_generic_transport_error() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("boom"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.call_generic("m", json!({})).await.expect_err("transport error");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_call_generic_timeout() {
    let inner = Arc::new(MockTransport::new(vec![Ok(json!({}))]));
    let slow = Arc::new(DelayTransport {
        inner,
        delay: Duration::from_secs(60),
    });
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        slow,
        AdapterTransportKind::Http,
        Duration::from_millis(5),
    );
    let err = adapter.call_generic("m", json!({})).await.expect_err("timeout");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_evaluate_trust_success() -> SongbirdResult<()> {
    let resp = TrustEvaluationResponse {
        decision: "auto_accept".into(),
        trust_level: TrustLevel::Elevated,
        reason: "ok".into(),
        suggested_action: None,
        metadata: None,
    };
    let mock =
        Arc::new(MockTransport::new(vec![Ok(serde_json::to_value(&resp).expect("serialize"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let req = TrustEvaluationRequest::new("peer", vec![]);
    let out = adapter.evaluate_trust(&req).await?;
    assert_eq!(out.decision, "auto_accept");
    Ok(())
}

#[tokio::test]
async fn security_evaluate_trust_error() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("nope"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let req = TrustEvaluationRequest::new("peer", vec![]);
    let err = adapter.evaluate_trust(&req).await.expect_err("rpc fail");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_get_identity_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({
        "encryption_tag": "t:family:f:n",
        "capabilities": ["identity"]
    }))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let id = adapter.get_identity().await?;
    assert_eq!(id.encryption_tag, "t:family:f:n");
    Ok(())
}

#[tokio::test]
async fn security_get_identity_error() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("down"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.get_identity().await.expect_err("fail");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

fn compute_metrics_json() -> serde_json::Value {
    json!({
        "cpu_usage_percent": 10.0,
        "memory_usage_bytes": 100,
        "memory_available_bytes": 900,
        "active_containers": 1,
        "queued_jobs": 0,
        "performance_score": 0.9,
        "timestamp": "2020-01-01T00:00:00Z"
    })
}

#[tokio::test]
async fn compute_collect_metrics_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(compute_metrics_json())]));
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert!((m.cpu_usage_percent - 10.0).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn compute_collect_metrics_parse_error() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("nope"))]));
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("parse");
    assert!(matches!(err, SongbirdError::Service { .. }));
}

#[tokio::test]
async fn compute_collect_metrics_timeout() {
    let inner = Arc::new(MockTransport::new(vec![Ok(compute_metrics_json())]));
    let slow = Arc::new(DelayTransport {
        inner,
        delay: Duration::from_secs(60),
    });
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        slow,
        AdapterTransportKind::Http,
        Duration::from_millis(5),
    );
    let err = adapter.collect_metrics().await.expect_err("timeout");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

fn ai_metrics_json() -> serde_json::Value {
    json!({
        "active_models": 2,
        "total_requests": 10,
        "avg_latency_ms": 1.0,
        "accuracy_score": 0.9,
        "gpu_utilization_percent": 5.0,
        "timestamp": "2020-01-01T00:00:00Z"
    })
}

#[tokio::test]
async fn ai_collect_metrics_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(ai_metrics_json())]));
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_eq!(m.active_models, 2);
    Ok(())
}

#[tokio::test]
async fn ai_collect_metrics_parse_error() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!([]))]));
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("parse");
    assert!(matches!(err, SongbirdError::Service { .. }));
}

#[tokio::test]
async fn ai_collect_metrics_timeout() {
    let inner = Arc::new(MockTransport::new(vec![Ok(ai_metrics_json())]));
    let slow = Arc::new(DelayTransport {
        inner,
        delay: Duration::from_secs(60),
    });
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        slow,
        AdapterTransportKind::Http,
        Duration::from_millis(5),
    );
    let err = adapter.collect_metrics().await.expect_err("timeout");
    assert!(matches!(err, SongbirdError::Network { .. }));
}
