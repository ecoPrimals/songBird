// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

//! Extended [`MockTransport`] coverage for compute, security, and AI adapters (constructors,
//! transport branches, traits, and edge cases).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::{SongbirdError, SongbirdResult};

use crate::adapters::AIAdapter;
use crate::adapters::ComputeAdapter;
use crate::adapters::SecurityAdapter;
use crate::adapters::ai::AIProvider;
use crate::adapters::compute::ComputeMetricsProvider;
use crate::adapters::compute::HealthStatus;
use crate::adapters::security::AuthResult;
use crate::adapters::security::SecurityProvider;
use crate::adapters::transport::{AdapterTransportKind, DelayTransport, MockTransport};
use crate::trust_types::TrustEvaluationRequest;

// --- helpers ----------------------------------------------------------------

fn security_metrics_json_zero_ts() -> serde_json::Value {
    json!({
        "active_sessions": 0,
        "failed_auth_attempts": 0,
        "blocked_ips": 0,
        "security_score": 0.95,
        "timestamp": "1970-01-01T00:00:00Z"
    })
}

fn compute_metrics_json_zero_ts() -> serde_json::Value {
    json!({
        "cpu_usage_percent": 0.0,
        "memory_usage_bytes": 0,
        "memory_available_bytes": 0,
        "active_containers": 0,
        "queued_jobs": 0,
        "performance_score": 0.0,
        "timestamp": "1970-01-01T00:00:00Z"
    })
}

fn ai_metrics_json_zero_ts() -> serde_json::Value {
    json!({
        "active_models": 0,
        "total_requests": 0,
        "avg_latency_ms": 0.0,
        "accuracy_score": 0.0,
        "gpu_utilization_percent": 0.0,
        "timestamp": "1970-01-01T00:00:00Z"
    })
}

// --- discovery + new() ------------------------------------------------------

#[tokio::test]
async fn compute_new_from_discovery_resolver_success() -> SongbirdResult<()> {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Compute, "http://127.0.0.1:5999".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let adapter = ComputeAdapter::new_from_discovery_with_resolver(resolver).await?;
    assert_eq!(adapter.endpoint(), "http://127.0.0.1:5999");
    Ok(())
}

#[tokio::test]
async fn compute_new_from_discovery_resolver_invalid_endpoint_fails() {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Compute, "tarpc://no-port".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let err = ComputeAdapter::new_from_discovery_with_resolver(resolver)
        .await
        .expect_err("invalid tarpc URL should fail client init");
    assert!(
        err.to_string().contains("port") || err.to_string().contains("configuration"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn compute_new_empty_url_succeeds() -> SongbirdResult<()> {
    let adapter = ComputeAdapter::new(String::new()).await?;
    assert_eq!(adapter.endpoint(), "");
    Ok(())
}

#[tokio::test]
async fn security_new_empty_url_succeeds() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new(String::new()).await?;
    assert_eq!(adapter.endpoint(), "");
    Ok(())
}

#[tokio::test]
async fn ai_new_empty_url_succeeds() -> SongbirdResult<()> {
    let adapter = AIAdapter::new(String::new()).await?;
    assert_eq!(adapter.endpoint(), "");
    Ok(())
}

#[tokio::test]
async fn security_from_discovery_resolver_success() -> SongbirdResult<()> {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Security, "http://127.0.0.1:5998".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let adapter = SecurityAdapter::from_discovery_with_resolver(resolver).await?;
    assert_eq!(adapter.endpoint(), "http://127.0.0.1:5998");
    Ok(())
}

#[tokio::test]
async fn security_from_discovery_resolver_invalid_endpoint_fails() {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Security, "tarpc://no-port".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let err = SecurityAdapter::from_discovery_with_resolver(resolver)
        .await
        .expect_err("invalid tarpc URL should fail client init");
    assert!(
        err.to_string().contains("port") || err.to_string().contains("configuration"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn ai_from_discovery_resolver_success() -> SongbirdResult<()> {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Ai, "http://127.0.0.1:5997".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let adapter = AIAdapter::from_discovery_with_resolver(resolver).await?;
    assert_eq!(adapter.endpoint(), "http://127.0.0.1:5997");
    Ok(())
}

#[tokio::test]
async fn ai_from_discovery_resolver_invalid_endpoint_fails() {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Ai, "tarpc://no-port".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let err = AIAdapter::from_discovery_with_resolver(resolver)
        .await
        .expect_err("invalid tarpc URL should fail client init");
    assert!(
        err.to_string().contains("port") || err.to_string().contains("configuration"),
        "unexpected: {err}"
    );
}

// --- Security adapter (mock) ------------------------------------------------

#[tokio::test]
async fn security_collect_metrics_transport_error_http() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("down"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("transport error");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_collect_metrics_transport_error_tarpc_branch() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("tarpc down"))]));
    let adapter = SecurityAdapter::with_transport(
        "tarpc://mock".into(),
        mock,
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("transport error");
    let msg = err.to_string();
    assert!(msg.contains("tarpc") || msg.contains("security provider"), "{msg}");
}

#[tokio::test]
async fn security_verify_auth_transport_error() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("auth rpc fail"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.verify_auth("t").await.expect_err("rpc fail");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn security_verify_auth_tarpc_error_message() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("x"))]));
    let adapter = SecurityAdapter::with_transport(
        "tarpc://127.0.0.1:1".into(),
        mock,
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let err = adapter.verify_auth("t").await.expect_err("fail");
    assert!(err.to_string().contains("tarpc"), "{}", err);
}

#[tokio::test]
async fn security_evaluate_trust_parse_error() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("not-an-object"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let req = TrustEvaluationRequest::new("p", vec![]);
    let err = adapter.evaluate_trust(&req).await.expect_err("bad json");
    assert!(matches!(err, SongbirdError::Security(_)));
}

#[tokio::test]
async fn security_get_identity_parse_error() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({"capabilities": []}))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.get_identity().await.expect_err("missing encryption_tag");
    assert!(matches!(err, SongbirdError::Security(_)));
}

#[tokio::test]
async fn security_check_health_success() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(security_metrics_json_zero_ts())]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let h = adapter.check_health().await?;
    assert_eq!(h, crate::adapters::SecurityHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn security_call_generic_jsonrpc_pass_through_ok() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({"raw": true}))]));
    let adapter = SecurityAdapter::with_transport(
        "unix:///tmp/x.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let v = adapter.call_generic("btsp/x", json!({})).await?;
    assert_eq!(v, json!({"raw": true}));
    Ok(())
}

#[tokio::test]
async fn security_call_generic_http_transport_error_wrapped_network() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::service("x", "y"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.call_generic("m", json!({})).await.expect_err("inner");
    assert!(matches!(err, SongbirdError::Network { .. }));
    assert!(err.to_string().contains("HTTP request failed"), "{}", err);
}

#[tokio::test]
async fn security_provider_trait_default_health() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(security_metrics_json_zero_ts())]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let h = SecurityProvider::check_security_health(&adapter).await?;
    assert_eq!(h, crate::adapters::SecurityHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn security_provider_collect_and_verify() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![
        Ok(security_metrics_json_zero_ts()),
        Ok(json!("Authorized")),
    ]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = SecurityProvider::collect_security_metrics(&adapter).await?;
    assert_eq!(m.active_sessions, 0);
    let a = SecurityProvider::verify_authentication(&adapter, "tok").await?;
    assert_eq!(a, AuthResult::Authorized);
    Ok(())
}

#[tokio::test]
async fn security_metrics_timestamp_zero_filled() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(security_metrics_json_zero_ts())]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_ne!(m.timestamp.timestamp(), 0);
    Ok(())
}

#[tokio::test]
async fn security_evaluate_trust_jsonrpc_error_branch() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("jr"))]));
    let adapter = SecurityAdapter::with_transport(
        "unix:///tmp/s.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let req = TrustEvaluationRequest::new("p", vec![]);
    let err = adapter.evaluate_trust(&req).await.expect_err("fail");
    assert!(err.to_string().contains("JSON-RPC"), "{}", err);
}

#[tokio::test]
async fn security_get_identity_jsonrpc_error_branch() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("jr"))]));
    let adapter = SecurityAdapter::with_transport(
        "unix:///tmp/s.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let err = adapter.get_identity().await.expect_err("fail");
    assert!(err.to_string().contains("JSON-RPC"), "{}", err);
}

// --- Compute adapter (mock) -----------------------------------------------

#[tokio::test]
async fn compute_collect_metrics_transport_error_jsonrpc() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("ipc"))]));
    let adapter = ComputeAdapter::with_transport(
        "unix:///tmp/c.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("down");
    let msg = err.to_string();
    assert!(msg.contains("compute service") || msg.contains("Failed to reach"), "{msg}");
}

#[tokio::test]
async fn compute_collect_metrics_parse_error_non_http() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("bad"))]));
    let adapter = ComputeAdapter::with_transport(
        "unix:///tmp/c.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("parse");
    assert!(matches!(err, SongbirdError::Serialization { .. }));
}

#[tokio::test]
async fn compute_check_health_degraded() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({
        "cpu_usage_percent": 85.0,
        "memory_usage_bytes": 850,
        "memory_available_bytes": 150,
        "active_containers": 1,
        "queued_jobs": 11,
        "performance_score": 0.5,
        "timestamp": "2020-01-01T00:00:00Z"
    }))]));
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let h = adapter.check_health().await?;
    assert_eq!(h, HealthStatus::Degraded);
    Ok(())
}

#[tokio::test]
async fn compute_metrics_provider_trait() -> SongbirdResult<()> {
    let payload = compute_metrics_json_zero_ts();
    let mock = Arc::new(MockTransport::new(vec![Ok(payload.clone()), Ok(payload)]));
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = ComputeMetricsProvider::collect_compute_metrics(&adapter).await?;
    assert_eq!(m.queued_jobs, 0);
    let h = ComputeMetricsProvider::check_compute_health(&adapter).await?;
    assert_eq!(h, HealthStatus::Healthy);
    Ok(())
}

#[tokio::test]
async fn compute_collect_metrics_transport_error_http() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("http down"))]));
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("down");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn compute_metrics_timestamp_zero_filled() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(compute_metrics_json_zero_ts())]));
    let adapter = ComputeAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_ne!(m.timestamp.timestamp(), 0);
    Ok(())
}

#[tokio::test]
async fn compute_with_timeout_and_endpoint() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(compute_metrics_json_zero_ts())]));
    let adapter = ComputeAdapter::with_transport(
        "http://x".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(30),
    )
    .with_timeout(Duration::from_secs(60));
    assert_eq!(adapter.endpoint(), "http://x");
    let _ = adapter.collect_metrics().await?;
    Ok(())
}

// --- AI adapter (mock) ------------------------------------------------------

#[tokio::test]
async fn ai_collect_metrics_transport_error_jsonrpc() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("ipc"))]));
    let adapter = AIAdapter::with_transport(
        "unix:///tmp/a.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("down");
    let msg = err.to_string();
    assert!(msg.contains("AI provider") || msg.contains("Failed to reach"), "{msg}");
}

#[tokio::test]
async fn ai_collect_metrics_parse_error_non_http() {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!(null))]));
    let adapter = AIAdapter::with_transport(
        "unix:///tmp/a.sock".into(),
        mock,
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("parse");
    assert!(matches!(err, SongbirdError::Serialization { .. }));
}

#[tokio::test]
async fn ai_check_health_overloaded() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({
        "active_models": 1,
        "total_requests": 1,
        "avg_latency_ms": 3000.0,
        "accuracy_score": 0.1,
        "gpu_utilization_percent": 99.5,
        "timestamp": "2020-01-01T00:00:00Z"
    }))]));
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let h = adapter.check_health().await?;
    assert_eq!(h, crate::adapters::AIHealth::Overloaded);
    Ok(())
}

#[tokio::test]
async fn ai_provider_trait() -> SongbirdResult<()> {
    let payload = ai_metrics_json_zero_ts();
    let mock = Arc::new(MockTransport::new(vec![Ok(payload.clone()), Ok(payload)]));
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = AIProvider::collect_ai_metrics(&adapter).await?;
    assert_eq!(m.total_requests, 0);
    let h = AIProvider::check_ai_health(&adapter).await?;
    assert_eq!(h, crate::adapters::AIHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn ai_collect_metrics_transport_error_http() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("http"))]));
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("down");
    assert!(matches!(err, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn ai_metrics_timestamp_zero_filled() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(ai_metrics_json_zero_ts())]));
    let adapter = AIAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_ne!(m.timestamp.timestamp(), 0);
    Ok(())
}

#[tokio::test]
async fn ai_with_timeout_endpoint_debug() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(ai_metrics_json_zero_ts())]));
    let adapter = AIAdapter::with_transport(
        "http://ai".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    )
    .with_timeout(Duration::from_secs(120));
    assert_eq!(adapter.endpoint(), "http://ai");
    let dbg = format!("{:?}", adapter);
    assert!(dbg.contains("http://ai"));
    let _ = adapter.collect_metrics().await?;
    Ok(())
}

#[tokio::test]
async fn ai_collect_metrics_timeout_instrumented() {
    let inner = Arc::new(MockTransport::new(vec![Ok(ai_metrics_json_zero_ts())]));
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

#[tokio::test]
async fn security_verify_auth_expired_variant() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!("Expired"))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    assert_eq!(adapter.verify_auth("x").await?, AuthResult::Expired);
    Ok(())
}

#[tokio::test]
async fn security_evaluate_trust_tarpc_error_branch() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("t"))]));
    let adapter = SecurityAdapter::with_transport(
        "tarpc://127.0.0.1:1".into(),
        mock,
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let req = TrustEvaluationRequest::new("p", vec![]);
    let err = adapter.evaluate_trust(&req).await.expect_err("fail");
    assert!(err.to_string().contains("tarpc"), "{}", err);
}

#[tokio::test]
async fn security_get_identity_tarpc_error_branch() {
    let mock = Arc::new(MockTransport::new(vec![Err(SongbirdError::network("t"))]));
    let adapter = SecurityAdapter::with_transport(
        "tarpc://127.0.0.1:1".into(),
        mock,
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let err = adapter.get_identity().await.expect_err("fail");
    assert!(err.to_string().contains("tarpc"), "{}", err);
}

#[tokio::test]
async fn security_evaluate_trust_success_with_null_optionals() -> SongbirdResult<()> {
    let mock = Arc::new(MockTransport::new(vec![Ok(json!({
        "decision": "reject",
        "trust_level": "none",
        "reason": "nope",
        "suggested_action": null,
        "metadata": null
    }))]));
    let adapter = SecurityAdapter::with_transport(
        "http://mock".into(),
        mock,
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let req = TrustEvaluationRequest::new("peer", vec![]);
    let out = adapter.evaluate_trust(&req).await?;
    assert_eq!(out.decision, "reject");
    assert!(out.suggested_action.is_none());
    assert!(out.metadata.is_none());
    Ok(())
}
