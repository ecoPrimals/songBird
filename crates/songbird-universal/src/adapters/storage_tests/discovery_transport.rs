// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery fallbacks, `build_default_transport` error paths, and [`MockTransport`] coverage.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::super::*;
use super::assert_protocol_debug;
use crate::adapters::transport::{
    AdapterTransportKind, DelayTransport, MockTransport, build_default_transport,
};
use serde_json::json;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use std::time::Duration;

use crate::adapters::discovery_test_sync::lock_discovery_env;

#[tokio::test]
async fn new_accepts_tarpc_ip_and_localhost() -> SongbirdResult<()> {
    let a = StorageAdapter::new("tarpc://127.0.0.1:9201".to_string()).await?;
    assert_eq!(a.endpoint(), "tarpc://127.0.0.1:9201");
    assert_protocol_debug(&a, "Tarpc");
    let b = StorageAdapter::new("tarpc://localhost:9202".to_string()).await?;
    assert_protocol_debug(&b, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn new_accepts_unix_and_http_urls() -> SongbirdResult<()> {
    let u = StorageAdapter::new("unix:///tmp/songbird-storage-discovery.sock".to_string()).await?;
    assert_protocol_debug(&u, "JsonRpc");
    let h = StorageAdapter::new("http://storage:8082".to_string()).await?;
    assert_protocol_debug(&h, "Http");
    let s = StorageAdapter::new("https://storage.example:8443".to_string()).await?;
    assert_protocol_debug(&s, "Http");
    Ok(())
}

#[tokio::test]
async fn new_propagates_build_default_transport_unix_empty_path_err() {
    let err = StorageAdapter::new("unix://".to_string()).await.expect_err("empty unix path");
    let msg = err.to_string();
    assert!(
        msg.contains("Empty socket path")
            || msg.contains("configuration")
            || msg.contains("JSON-RPC"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn new_propagates_build_default_transport_invalid_tarpc_err() {
    let err = StorageAdapter::new("tarpc://not-a-host:99999".to_string())
        .await
        .expect_err("invalid tarpc endpoint");
    let msg = err.to_string();
    assert!(msg.contains("tarpc") || msg.contains("configuration"), "unexpected: {msg}");
}

#[test]
fn build_default_transport_invalid_tarpc_configuration_error() {
    let err = build_default_transport("tarpc://bad:99999").expect_err("invalid");
    assert!(err.to_string().contains("configuration") || err.to_string().contains("tarpc"));
}

#[test]
fn build_default_transport_unix_empty_path_configuration_error() {
    let err = build_default_transport("unix://").expect_err("empty unix");
    assert!(err.to_string().contains("configuration") || err.to_string().contains("Empty"));
}

#[tokio::test]
async fn from_discovery_fallback_prefers_songbird_storage_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_STORAGE_ENDPOINT", "http://from-songbird-storage:8082");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("SONGBIRD_STORAGE_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-songbird-storage:8082");
    assert_protocol_debug(&adapter, "Http");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_storage_provider_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::set_var("STORAGE_PROVIDER_ENDPOINT", "http://from-provider:9001");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("STORAGE_PROVIDER_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-provider:9001");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_storage_endpoint_var() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::set_var("STORAGE_ENDPOINT", "http://from-storage-endpoint:9002");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("STORAGE_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-storage-endpoint:9002");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_nestgate_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::set_var("NESTGATE_ENDPOINT", "http://from-nestgate:9003");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("NESTGATE_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-nestgate:9003");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_prefers_songbird_over_other_legacy_envs() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_STORAGE_ENDPOINT", "http://songbird-wins:1111");
    songbird_process_env::set_var("STORAGE_PROVIDER_ENDPOINT", "http://legacy-loses:2222");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("SONGBIRD_STORAGE_ENDPOINT wins");
    assert_eq!(adapter.endpoint(), "http://songbird-wins:1111");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_default_songbird_host_and_storage_port() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
    songbird_process_env::remove_var("STORAGE_PROVIDER_ENDPOINT");
    songbird_process_env::remove_var("STORAGE_ENDPOINT");
    songbird_process_env::remove_var("NESTGATE_ENDPOINT");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("host+port fallback");
    assert_eq!(
        adapter.endpoint(),
        format!(
            "{}:{}",
            "http://localhost",
            songbird_config::defaults::ports::service_port("STORAGE", 8082)
        )
    );
    assert_protocol_debug(&adapter, "Http");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_custom_host_and_storage_port_env() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    songbird_process_env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
    songbird_process_env::remove_var("STORAGE_PROVIDER_ENDPOINT");
    songbird_process_env::remove_var("STORAGE_ENDPOINT");
    songbird_process_env::remove_var("NESTGATE_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_HOST", "http://custom-host");
    songbird_process_env::set_var("SONGBIRD_STORAGE_PORT", "7777");

    let adapter = StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("custom host+port");
    assert_eq!(adapter.endpoint(), "http://custom-host:7777");

    songbird_process_env::reset_overlay();
    Ok(())
}

fn sample_metrics_json() -> serde_json::Value {
    json!({
        "total_capacity_bytes": 10_000_000_000_u64,
        "used_bytes": 1_000_000_000_u64,
        "available_bytes": 9_000_000_000_u64,
        "object_count": 42_u64,
        "avg_read_latency_ms": 5.0,
        "avg_write_latency_ms": 7.0,
        "timestamp": "2025-11-18T12:00:00Z"
    })
}

#[tokio::test]
async fn collect_metrics_and_check_health_with_mock_transport() -> SongbirdResult<()> {
    let payload = sample_metrics_json();
    let adapter = StorageAdapter::with_transport(
        "http://mock-storage".to_string(),
        Arc::new(MockTransport::new(vec![Ok(payload.clone()), Ok(payload)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let metrics = adapter.collect_metrics().await?;
    assert_eq!(metrics.object_count, 42);
    assert_eq!(adapter.check_health().await?, StorageHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn collect_metrics_delay_transport_times_out() {
    let delayed = DelayTransport {
        inner: Arc::new(MockTransport::new(vec![])),
        delay: Duration::from_secs(30),
    };
    let adapter = StorageAdapter::with_transport(
        "http://mock-storage".to_string(),
        Arc::new(delayed),
        AdapterTransportKind::Http,
        Duration::from_millis(30),
    );
    let err = adapter.collect_metrics().await.expect_err("timeout");
    assert!(err.to_string().to_lowercase().contains("timeout"), "unexpected: {err}");
}

#[tokio::test]
async fn collect_metrics_mock_transport_error_passes_through() {
    let boom = SongbirdError::network("upstream transport failure");
    let adapter = StorageAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom.clone())])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("transport error");
    assert_eq!(err.to_string(), boom.to_string());
}

#[tokio::test]
async fn collect_metrics_parse_error_maps_to_storage_service() {
    let adapter = StorageAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(json!("not-metrics"))])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("bad shape");
    assert!(err.to_string().contains("storage") || err.to_string().contains("parse"), "{}", err);
}

#[tokio::test]
async fn collect_metrics_non_http_transport_error_is_wrapped() {
    let boom = SongbirdError::network("rpc down");
    let adapter = StorageAdapter::with_transport(
        "tarpc://127.0.0.1:1".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom)])),
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("wrapped");
    let s = err.to_string();
    assert!(s.contains("Failed to reach storage provider"), "{}", s);
}
