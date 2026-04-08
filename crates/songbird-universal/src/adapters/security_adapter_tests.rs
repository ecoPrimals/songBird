// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

//! Tests for SecurityAdapter protocol detection, discovery fallback chain,
//! and SecurityProvider trait default implementation.

use super::*;
use crate::adapters::transport::{AdapterTransportKind, DelayTransport, MockTransport};
use serde_json::json;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;

static DISCOVERY_ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_discovery_env() -> std::sync::MutexGuard<'static, ()> {
    DISCOVERY_ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn assert_adapter_debug_protocol(adapter: &SecurityAdapter, expected: &str) {
    let dbg = format!("{adapter:?}");
    assert!(dbg.contains(expected), "expected Debug output to contain {expected:?}, got {dbg}");
}

#[tokio::test]
async fn test_new_selects_tarpc_protocol() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("tarpc://127.0.0.1:9001".to_string()).await?;
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9001");
    assert_adapter_debug_protocol(&adapter, "Tarpc");
    Ok(())
}

/// Explicit `localhost` + port form (tarpc hostname resolution path).
#[tokio::test]
async fn test_security_adapter_new_tarpc_localhost_1234() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("tarpc://localhost:1234".to_string()).await?;
    assert_eq!(adapter.endpoint(), "tarpc://localhost:1234");
    assert_adapter_debug_protocol(&adapter, "Tarpc");
    Ok(())
}

/// User-facing example shape: `/tmp/test.sock` (protocol detection for unix).
#[tokio::test]
async fn test_security_adapter_new_unix_tmp_test_sock() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("unix:///tmp/test.sock".to_string()).await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/test.sock");
    assert_adapter_debug_protocol(&adapter, "JsonRpc");
    Ok(())
}

#[tokio::test]
async fn test_security_adapter_new_tarpc_invalid_hostname_err() {
    let err = SecurityAdapter::new("tarpc://test:1234".to_string())
        .await
        .expect_err("non-localhost non-IP hostname should fail tarpc parse");
    let msg = err.to_string();
    assert!(
        msg.contains("Invalid hostname") || msg.contains("configuration"),
        "unexpected error: {msg}"
    );
}

#[tokio::test]
async fn test_security_adapter_new_unix_empty_path_err() {
    let err = SecurityAdapter::new("unix://".to_string())
        .await
        .expect_err("empty unix path should fail JSON-RPC client init");
    assert!(
        err.to_string().to_lowercase().contains("empty")
            || err.to_string().contains("configuration"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn test_new_selects_jsonrpc_protocol_for_unix() -> SongbirdResult<()> {
    let adapter =
        SecurityAdapter::new("unix:///tmp/songbird-security-test.sock".to_string()).await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/songbird-security-test.sock");
    assert_adapter_debug_protocol(&adapter, "JsonRpc");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_http_protocol_for_http() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("http://security:8080".to_string()).await?;
    assert_adapter_debug_protocol(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_selects_http_protocol_for_https() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("https://security.example:8443".to_string()).await?;
    assert_adapter_debug_protocol(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_unknown_scheme_uses_http_fallback_client() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("ftp://example:21".to_string()).await?;
    assert_adapter_debug_protocol(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_new_uppercase_tarpc_scheme_uses_http_fallback() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("TARPC://127.0.0.1:9001".to_string()).await?;
    assert_adapter_debug_protocol(&adapter, "Http");
    Ok(())
}

#[tokio::test]
async fn test_with_timeout_preserves_endpoint_and_protocol() -> SongbirdResult<()> {
    let adapter = SecurityAdapter::new("tarpc://127.0.0.1:9002".to_string())
        .await?
        .with_timeout(Duration::from_millis(250));
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9002");
    assert_eq!(adapter.timeout, Duration::from_millis(250));
    assert_adapter_debug_protocol(&adapter, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn test_from_discovery_resolver_injected_security_endpoint_tarpc() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Security, "tarpc://127.0.0.1:9100".to_string());
    let adapter = SecurityAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9100");
    assert_adapter_debug_protocol(&adapter, "Tarpc");
    Ok(())
}

#[tokio::test]
async fn test_from_discovery_resolver_injected_security_endpoint_unix() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Security, "unix:///tmp/injected-security.sock".to_string());
    let adapter = SecurityAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/injected-security.sock");
    assert_adapter_debug_protocol(&adapter, "JsonRpc");
    Ok(())
}

#[tokio::test]
async fn test_from_discovery_fallback_songbird_security_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_SECURITY_ENDPOINT", "http://from-songbird-env:7777");

    let adapter = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter should build from SONGBIRD_SECURITY_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-songbird-env:7777");
    assert_adapter_debug_protocol(&adapter, "Http");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_from_discovery_fallback_security_provider_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    songbird_process_env::set_var("SECURITY_PROVIDER_ENDPOINT", "http://from-legacy-security:7666");

    let adapter = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter should build from SECURITY_PROVIDER_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-legacy-security:7666");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_from_discovery_fallback_legacy_beardog_env_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    songbird_process_env::set_var("BEARDOG_ENDPOINT", "http://from-beardog:7555");

    let adapter = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter should build from BEARDOG_ENDPOINT fallback");
    assert_eq!(adapter.endpoint(), "http://from-beardog:7555");

    songbird_process_env::reset_overlay();
    Ok(())
}

/// When multiple legacy env vars are set, `SONGBIRD_SECURITY_ENDPOINT` wins (first in the chain).
#[tokio::test]
async fn test_from_discovery_fallback_prefers_songbird_security_over_legacy_provider()
-> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_SECURITY_ENDPOINT", "http://songbird-wins:1111");
    songbird_process_env::set_var("SECURITY_PROVIDER_ENDPOINT", "http://legacy-should-lose:2222");

    let adapter = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter should build from SONGBIRD_SECURITY_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://songbird-wins:1111");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_from_discovery_fallback_host_and_port_env() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_HOST", "http://custom-host");
    songbird_process_env::set_var("SONGBIRD_SECURITY_PORT", "9999");

    let adapter = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter should build from host+port fallback");
    assert_eq!(adapter.endpoint(), "http://custom-host:9999");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_security_provider_default_check_security_health() -> SongbirdResult<()> {
    struct StaticMetrics(SecurityMetrics);

    impl SecurityProvider for StaticMetrics {
        async fn collect_security_metrics(&self) -> SongbirdResult<SecurityMetrics> {
            Ok(self.0.clone())
        }

        async fn verify_authentication(&self, _token: &str) -> SongbirdResult<AuthResult> {
            Ok(AuthResult::Authorized)
        }
    }

    let metrics = SecurityMetrics {
        active_sessions: 12,
        failed_auth_attempts: 8,
        blocked_ips: 1,
        security_score: 0.92,
        timestamp: chrono::Utc::now(),
    };
    let provider = StaticMetrics(metrics.clone());
    let health = provider.check_security_health().await?;
    assert_eq!(health, metrics.health_status());
    Ok(())
}

#[test]
fn test_auth_result_json_roundtrip_all_string_forms() -> SongbirdResult<()> {
    for (json, expected) in [
        (r#""Authorized""#, AuthResult::Authorized),
        (r#""Unauthorized""#, AuthResult::Unauthorized),
        (r#""Expired""#, AuthResult::Expired),
        (r#""Invalid""#, AuthResult::Invalid),
    ] {
        let parsed: AuthResult = serde_json::from_str(json).map_err(|e| {
            SongbirdError::configuration(format!("AuthResult JSON parse should succeed: {e}"))
        })?;
        assert_eq!(parsed, expected);
        let again = serde_json::to_string(&parsed).map_err(|e| {
            SongbirdError::configuration(format!("AuthResult JSON serialize should succeed: {e}"))
        })?;
        let round: AuthResult = serde_json::from_str(&again).map_err(|e| {
            SongbirdError::configuration(format!("AuthResult JSON roundtrip should succeed: {e}"))
        })?;
        assert_eq!(round, expected);
    }
    Ok(())
}

#[test]
fn test_security_health_json_roundtrip_all_variants() -> SongbirdResult<()> {
    for health in [SecurityHealth::Healthy, SecurityHealth::Warning, SecurityHealth::Critical] {
        let json = serde_json::to_string(&health).map_err(|e| {
            SongbirdError::configuration(format!("SecurityHealth serialize should succeed: {e}"))
        })?;
        let parsed: SecurityHealth = serde_json::from_str(&json).map_err(|e| {
            SongbirdError::configuration(format!("SecurityHealth deserialize should succeed: {e}"))
        })?;
        assert_eq!(parsed, health);
    }
    Ok(())
}

#[test]
fn test_security_metrics_health_status_critical_when_under_attack_despite_high_score() {
    let metrics = SecurityMetrics {
        active_sessions: 20,
        failed_auth_attempts: 200,
        blocked_ips: 5,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_health_status_warning_failed_attempts_only() {
    let metrics = SecurityMetrics {
        active_sessions: 20,
        failed_auth_attempts: 75,
        blocked_ips: 5,
        security_score: 0.85,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

// --- collect_metrics / check_health / transport errors (MockTransport), discovery edge cases ---

#[tokio::test]
async fn collect_metrics_times_out_with_delay_transport() {
    let delayed = DelayTransport {
        inner: Arc::new(MockTransport::new(vec![])),
        delay: Duration::from_secs(30),
    };
    let adapter = SecurityAdapter::with_transport(
        "http://mock-security".to_string(),
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
    let adapter = SecurityAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom.clone())])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("transport error");
    assert_eq!(err.to_string(), boom.to_string());
}

#[tokio::test]
async fn collect_metrics_tarpc_transport_error_is_wrapped() {
    let boom = SongbirdError::network("rpc down");
    let adapter = SecurityAdapter::with_transport(
        "tarpc://127.0.0.1:1".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom)])),
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("wrapped");
    let s = err.to_string();
    assert!(s.contains("tarpc"), "{}", s);
}

#[tokio::test]
async fn collect_metrics_jsonrpc_transport_error_is_wrapped() {
    let boom = SongbirdError::network("uds down");
    let adapter = SecurityAdapter::with_transport(
        "unix:///tmp/songbird-security-mock.sock".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom)])),
        AdapterTransportKind::JsonRpc,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("wrapped");
    let s = err.to_string();
    assert!(s.contains("Failed to reach security provider"), "{}", s);
}

#[tokio::test]
async fn collect_metrics_parse_error_maps_to_security() {
    let adapter = SecurityAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(json!("not-metrics"))])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let err = adapter.collect_metrics().await.expect_err("bad json shape");
    let s = err.to_string();
    assert!(s.to_lowercase().contains("security") || s.contains("parse"), "{}", s);
}

#[tokio::test]
async fn collect_metrics_sets_timestamp_when_unix_epoch() -> SongbirdResult<()> {
    let body = json!({
        "active_sessions": 1,
        "failed_auth_attempts": 0,
        "blocked_ips": 0,
        "security_score": 1.0,
        "timestamp": "1970-01-01T00:00:00Z"
    });
    let adapter = SecurityAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(body)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    let m = adapter.collect_metrics().await?;
    assert_ne!(m.timestamp.timestamp(), 0);
    Ok(())
}

#[tokio::test]
async fn check_health_delegates_to_collect_metrics() -> SongbirdResult<()> {
    let body = serde_json::to_value(SecurityMetrics {
        active_sessions: 3,
        failed_auth_attempts: 1,
        blocked_ips: 0,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    })?;
    let adapter = SecurityAdapter::with_transport(
        "http://mock".to_string(),
        Arc::new(MockTransport::new(vec![Ok(body)])),
        AdapterTransportKind::Http,
        Duration::from_secs(5),
    );
    assert_eq!(adapter.check_health().await?, SecurityHealth::Healthy);
    Ok(())
}

#[tokio::test]
async fn verify_auth_tarpc_error_includes_tarpc_hint() {
    let boom = SongbirdError::network("no peer");
    let adapter = SecurityAdapter::with_transport(
        "tarpc://127.0.0.1:1".to_string(),
        Arc::new(MockTransport::new(vec![Err(boom)])),
        AdapterTransportKind::Tarpc,
        Duration::from_secs(5),
    );
    let err = adapter.verify_auth("tok").await.expect_err("auth fails");
    assert!(err.to_string().contains("tarpc"), "{}", err);
}

#[tokio::test]
async fn from_discovery_fallback_security_endpoint_env_only() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    for key in ["SONGBIRD_SECURITY_ENDPOINT", "SECURITY_PROVIDER_ENDPOINT", "BEARDOG_ENDPOINT"] {
        songbird_process_env::remove_var(key);
    }
    songbird_process_env::set_var("SECURITY_ENDPOINT", "http://from-security-endpoint-only:8800");

    let adapter =
        SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new()).await?;
    assert_eq!(adapter.endpoint(), "http://from-security-endpoint-only:8800");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_default_bind_address_and_security_port() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    for key in [
        "CAPABILITY_SECURITY_ENDPOINT",
        "SONGBIRD_SECURITY_ENDPOINT",
        "SECURITY_ENDPOINT",
        "SECURITY_PROVIDER_ENDPOINT",
        "BEARDOG_ENDPOINT",
        "SONGBIRD_HOST",
        "SONGBIRD_SECURITY_PORT",
    ] {
        songbird_process_env::remove_var(key);
    }

    let expected =
        format!("http://{}:8081", songbird_config::canonical::constants::get_bind_address());
    let adapter =
        SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new()).await?;
    assert_eq!(adapter.endpoint(), expected);

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn from_discovery_fallback_propagates_new_error_from_bad_env_endpoint() {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_SECURITY_ENDPOINT", "unix://");

    let err = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect_err("adapter new should fail");
    assert!(
        err.to_string().to_lowercase().contains("empty")
            || err.to_string().contains("configuration"),
        "unexpected: {err}"
    );

    songbird_process_env::reset_overlay();
}

#[tokio::test]
async fn beardog_endpoint_logs_deprecation_warning() -> SongbirdResult<()> {
    #[derive(Clone)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);
    impl Write for BufWriter {
        fn write(&mut self, d: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(d);
            Ok(d.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    for key in ["SONGBIRD_SECURITY_ENDPOINT", "SECURITY_ENDPOINT", "SECURITY_PROVIDER_ENDPOINT"] {
        songbird_process_env::remove_var(key);
    }
    songbird_process_env::set_var("BEARDOG_ENDPOINT", "http://from-beardog-warn:7711");

    let log_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let w = Arc::clone(&log_buf);
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .without_time()
            .with_target(false)
            .with_level(false)
            .with_ansi(false)
            .with_writer(move || BufWriter(Arc::clone(&w))),
    );
    let _trace_guard = tracing::subscriber::set_default(subscriber);

    let adapter =
        SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new()).await?;
    assert_eq!(adapter.endpoint(), "http://from-beardog-warn:7711");
    drop(_trace_guard);
    let logs = String::from_utf8_lossy(
        &log_buf.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(),
    )
    .into_owned();
    assert!(logs.contains("BEARDOG_ENDPOINT") && logs.contains("deprecated"), "logs were: {logs}");

    songbird_process_env::reset_overlay();
    Ok(())
}
