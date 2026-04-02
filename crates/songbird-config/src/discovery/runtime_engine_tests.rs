// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;

#[tokio::test]
async fn test_environment_discovery() {
    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment],
        Duration::from_secs(60),
        |k| {
            if k == "SECURITY_ENDPOINT" {
                Ok("127.0.0.1:8443".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        },
    );

    let services = engine.discover_by_capability("security").await;
    assert!(!services.is_empty(), "Should discover security service from environment");
}

#[tokio::test]
async fn test_cache_functionality() {
    const CAP: &str = "sb_rteng_cache_isolated";
    const ENV: &str = "SB_RTENG_CACHE_ISOLATED_ENDPOINT";

    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment],
        Duration::from_millis(10),
        move |k| {
            if k == ENV {
                Ok("127.0.0.1:9000".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        },
    );

    let services1 = engine.discover_by_capability(CAP).await;
    let services2 = engine.discover_by_capability(CAP).await;
    assert_eq!(services1, services2, "Cache should return same results");

    tokio::time::sleep(Duration::from_millis(15)).await;

    let services3 = engine.discover_by_capability(CAP).await;
    assert_eq!(services1, services3, "Should still find service after cache expiry");
}

#[test]
fn test_backend_detection() {
    let backends = CapabilityDiscoveryEngine::detect_backends();
    assert!(!backends.is_empty(), "Should detect at least environment backend");
    assert!(
        backends.contains(&DiscoveryBackend::Environment),
        "Should always include environment backend"
    );
}

#[test]
fn test_engine_new_empty_backends_still_runs() {
    let engine = CapabilityDiscoveryEngine::new(vec![], Duration::from_secs(60));
    assert_eq!(engine.cache_ttl, Duration::from_secs(60));
}

#[tokio::test]
async fn test_discover_with_no_backends_returns_empty() {
    let engine = CapabilityDiscoveryEngine::new(vec![], Duration::from_secs(60));
    let addrs = engine.discover_by_capability("anything").await;
    assert!(addrs.is_empty());
}

#[tokio::test]
async fn test_register_self_no_panic() {
    let engine = CapabilityDiscoveryEngine::new(
        vec![DiscoveryBackend::Environment],
        Duration::from_secs(60),
    );
    let addr: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
    engine.register_self(&["test".to_string()], addr).await.expect("register_self returns Ok");
}

#[test]
fn test_with_defaults_constructed() {
    let _ = CapabilityDiscoveryEngine::with_defaults();
}

#[test]
fn discovery_backend_equality_and_clone() {
    let a = DiscoveryBackend::Consul {
        endpoint: "http://127.0.0.1:8500".into(),
    };
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(a, DiscoveryBackend::Environment);
}

#[tokio::test]
async fn discover_from_environment_strips_https_prefix() {
    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment],
        Duration::from_secs(60),
        |k| {
            if k == "STORAGE_ENDPOINT" {
                Ok("https://127.0.0.1:9000".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        },
    );
    let addrs = engine.discover_by_capability("storage").await;
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].port(), 9000);
}

#[tokio::test]
async fn discover_deduplicates_same_address() {
    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment, DiscoveryBackend::Environment],
        Duration::from_secs(60),
        |k| {
            if k == "AI_ENDPOINT" {
                Ok("127.0.0.1:7777".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        },
    );
    let addrs = engine.discover_by_capability("ai").await;
    assert_eq!(addrs.len(), 1);
}

#[tokio::test]
async fn discover_returns_empty_when_env_endpoint_missing() {
    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment],
        Duration::from_secs(60),
        |_| Err(std::env::VarError::NotPresent),
    );
    let cap = format!("sb_missing_env_{}", std::process::id());
    let addrs = engine.discover_by_capability(&cap).await;
    assert!(addrs.is_empty());
}

#[tokio::test]
async fn discover_ignores_invalid_env_endpoint_without_panic() {
    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment],
        Duration::from_secs(60),
        |k| {
            if k == "BROKEN_ENDPOINT" {
                Ok("not-a-socket-addr".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        },
    );
    let addrs = engine.discover_by_capability("broken").await;
    assert!(addrs.is_empty());
}

#[test]
fn discover_by_capability_sort_key_orders_ip_then_port() {
    let mut addrs: Vec<SocketAddr> = vec![
        "10.0.0.2:9000".parse().expect("addr"),
        "10.0.0.1:1".parse().expect("addr"),
        "10.0.0.1:9000".parse().expect("addr"),
    ];
    addrs.sort_by_key(|addr| (addr.ip(), addr.port()));
    assert_eq!(addrs[0].to_string(), "10.0.0.1:1");
    assert_eq!(addrs[1].to_string(), "10.0.0.1:9000");
    assert_eq!(addrs[2].to_string(), "10.0.0.2:9000");
}

#[tokio::test]
async fn discover_strips_http_prefix_from_env_endpoint() {
    let engine = CapabilityDiscoveryEngine::new_with_env_reader(
        vec![DiscoveryBackend::Environment],
        Duration::from_secs(60),
        |k| {
            if k == "WEB_ENDPOINT" {
                Ok("http://192.0.2.1:4444".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        },
    );
    let addrs = engine.discover_by_capability("web").await;
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].port(), 4444);
}
