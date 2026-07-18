// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use super::assert_protocol_debug;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
    let adapter =
        StorageAdapter::new("http://storage-provider:8082".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {e}"))
        })?;
    assert_eq!(adapter.endpoint(), "http://storage-provider:8082");
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = StorageAdapter::new("http://storage-provider:8082".to_string())
        .await
        .map_err(|e| SongbirdError::configuration(format!("Adapter creation should succeed: {e}")))?
        .with_timeout(Duration::from_secs(10));
    assert_eq!(adapter.timeout, Duration::from_secs(10));
    Ok(())
}

#[tokio::test]
async fn test_adapter_default_timeout() -> SongbirdResult<()> {
    let adapter =
        StorageAdapter::new("http://storage-service:8082".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {e}"))
        })?;
    assert_eq!(adapter.timeout, Duration::from_secs(5));
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_access() -> SongbirdResult<()> {
    let adapter =
        StorageAdapter::new("http://test-storage:9000".to_string()).await.map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {e}"))
        })?;
    assert_eq!(adapter.endpoint(), "http://test-storage:9000");
    Ok(())
}

#[tokio::test]
async fn test_adapter_debug_format() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("http://storage:8082".to_string()).await.map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {e}"))
    })?;
    let debug_str = format!("{adapter:?}");
    assert!(debug_str.contains("StorageAdapter"));
    assert!(debug_str.contains("http://storage:8082"));
    Ok(())
}

// ========== Protocol detection and adapter construction ==========

#[tokio::test]
async fn test_new_selects_tarpc_protocol() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("tarpc://127.0.0.1:9200".to_string()).await?;
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9200");
    assert_protocol_debug(&adapter, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_jsonrpc_for_unix() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("unix:///tmp/songbird-storage-test.sock".to_string()).await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/songbird-storage-test.sock");
    assert_protocol_debug(&adapter, "JsonRpc");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_http_for_http() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("http://storage:8082".to_string()).await?;
    assert_protocol_debug(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_http_for_https() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("https://storage.example:8443".to_string()).await?;
    assert_protocol_debug(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_unknown_scheme_falls_back_to_http() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("ftp://example:21".to_string()).await?;
    assert_protocol_debug(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_protocol_debug_formatting() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("tarpc://localhost:9300".to_string()).await?;
    let dbg = format!("{adapter:?}");
    assert!(dbg.contains("StorageAdapter"));
    assert!(dbg.contains("tarpc://localhost:9300"));
    Ok(())
}

#[tokio::test]
async fn test_with_timeout_preserves_protocol() -> SongbirdResult<()> {
    let adapter = StorageAdapter::new("tarpc://127.0.0.1:9400".to_string())
        .await?
        .with_timeout(Duration::from_millis(500));
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9400");
    assert_eq!(adapter.timeout, Duration::from_millis(500));
    assert_protocol_debug(&adapter, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn test_storage_provider_trait_impl() -> SongbirdResult<()> {
    struct StaticStorage(StorageMetrics);

    impl StorageProvider for StaticStorage {
        async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics> {
            Ok(self.0.clone())
        }
    }

    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 250_000_000_000,
        available_bytes: 750_000_000_000,
        object_count: 1_500,
        avg_read_latency_ms: 15.0,
        avg_write_latency_ms: 25.0,
        timestamp: chrono::Utc::now(),
    };
    let provider = StaticStorage(metrics.clone());
    let health = provider.check_storage_health().await?;
    assert_eq!(health, metrics.health_status());
    Ok(())
}

// --- HTTP + discovery (mockito): exercise `collect_metrics` / `check_health` / trait on adapter ---

#[tokio::test]
async fn unit_http_collect_metrics_success() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .expect(2)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 3500000000,
            "available_bytes": 6500000000,
            "object_count": 5000,
            "avg_read_latency_ms": 12.5,
            "avg_write_latency_ms": 18.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;
    assert_eq!(metrics.total_capacity_bytes, 10_000_000_000);
    assert_eq!(metrics.object_count, 5000);
    assert!((metrics.avg_read_latency_ms - 12.5).abs() < 0.01);

    let again = StorageProvider::collect_storage_metrics(&adapter).await?;
    assert_eq!(again.object_count, metrics.object_count);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn unit_http_check_health_maps_from_metrics() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 8700000000,
            "available_bytes": 1300000000,
            "object_count": 8000,
            "avg_read_latency_ms": 15.0,
            "avg_write_latency_ms": 20.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await?;
    let health = adapter.check_health().await?;
    assert_eq!(health, StorageHealth::Warning);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn unit_http_collect_metrics_http_error_status() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("adapter");
    let err = adapter.collect_metrics().await.expect_err("expected HTTP error");
    assert!(err.to_string().contains("503"), "{}", err);

    mock.assert_async().await;
}

#[tokio::test]
async fn unit_http_collect_metrics_invalid_json_body() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("adapter");
    let err = adapter.collect_metrics().await.expect_err("parse error");
    assert!(err.to_string().contains("Failed to parse storage metrics"), "{}", err);

    mock.assert_async().await;
}

#[tokio::test]
async fn unit_http_collect_metrics_epoch_timestamp_is_replaced() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 5000000000,
            "used_bytes": 1000000000,
            "available_bytes": 4000000000,
            "object_count": 2000,
            "avg_read_latency_ms": 8.0,
            "avg_write_latency_ms": 10.0,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;
    let now = chrono::Utc::now();
    assert!((now - metrics.timestamp).num_seconds().abs() < 10);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn unit_from_discovery_with_resolver_injected_endpoint() -> SongbirdResult<()> {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();
    let mut m = HashMap::new();
    m.insert(CapabilityType::Storage, endpoint.clone());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let adapter = StorageAdapter::from_discovery_with_resolver(resolver).await?;
    assert_eq!(adapter.endpoint(), endpoint.as_str());
    Ok(())
}

/// `songbird-test-utils` mock storage primal: smoke test (metrics shape used in integration scenarios).
#[test]
fn storage_mock_provider_fixture_from_test_utils() {
    use songbird_test_utils::mocks::storage_provider::MockStorageProvider;

    let mock = MockStorageProvider::new();
    let m = mock.get_metrics();
    assert!(m.total_capacity_bytes > 0);
    mock.simulate_near_capacity();
    assert!(mock.get_metrics().available_bytes < 100_000_000_000);
}
