// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use crate::traits::service::{ServiceInfo, ServiceStatus};
use chrono::Utc;

fn sample_service(name: &str) -> ServiceInfo {
    ServiceInfo {
        service_id: format!("{name}-id"),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        service_type: "test".to_string(),
        description: None,
        endpoints: vec![],
        health_check_endpoint: None,
        metadata: std::collections::HashMap::new(),
        tags: vec![],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        instance_id: format!("{name}-inst"),
        host: "127.0.0.1".to_string(),
        port: 8080,
    }
}

#[test]
fn cache_config_and_cache_stats_defaults() {
    let cfg = CacheConfig {
        default_ttl: std::time::Duration::from_secs(30),
        max_cache_size: 100,
        enabled: true,
    };
    assert!(cfg.enabled);
    assert_eq!(cfg.max_cache_size, 100);
}

#[test]
fn cached_service_info_holds_ttl() {
    let si = sample_service("alpha");
    let c = CachedServiceInfo {
        service_info: si,
        cached_at: std::time::Instant::now(),
        ttl: std::time::Duration::from_secs(60),
    };
    assert_eq!(c.service_info.name, "alpha");
    assert_eq!(c.ttl.as_secs(), 60);
}

#[tokio::test]
async fn universal_service_discovery_new_and_cache_roundtrip() {
    let mut d = UniversalServiceDiscovery::new().await.unwrap();
    d.cache_service("svc1", sample_service("svc1"));
    let got = d.get_cached_service("svc1");
    assert!(got.is_some());
    assert_eq!(got.unwrap().name, "svc1");
    let stats = d.get_cache_stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.valid_entries, 1);
    assert_eq!(stats.expired_entries, 0);
    assert_eq!(stats.max_capacity, 1000);
    d.cleanup_cache();
}

#[test]
fn service_info_serde_roundtrip() {
    let s = sample_service("serde");
    let json = serde_json::to_string(&s).unwrap();
    let back: ServiceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.service_id, s.service_id);
    assert_eq!(back.name, s.name);
}

#[tokio::test]
async fn list_all_via_trait_succeeds() {
    use crate::traits::ServiceDiscovery;
    let d = UniversalServiceDiscovery::new().await.unwrap();
    let list = ServiceDiscovery::list_all(&d).await.unwrap();
    let _ = list;
}

fn test_discovery() -> UniversalServiceDiscovery {
    UniversalServiceDiscovery {
        registry_endpoints: Vec::new(),
        service_cache: HashMap::new(),
        discovery_methods: Vec::new(),
        cache_config: CacheConfig {
            default_ttl: std::time::Duration::from_secs(30),
            max_cache_size: 1000,
            enabled: true,
        },
    }
}

fn test_discovery_with_cache(max_size: usize, enabled: bool) -> UniversalServiceDiscovery {
    UniversalServiceDiscovery {
        registry_endpoints: Vec::new(),
        service_cache: HashMap::new(),
        discovery_methods: Vec::new(),
        cache_config: CacheConfig {
            default_ttl: std::time::Duration::from_secs(30),
            max_cache_size: max_size,
            enabled,
        },
    }
}

#[tokio::test]
async fn new_constructs_with_auto_detected_methods() {
    let d = UniversalServiceDiscovery::new().await.unwrap();
    let stats = d.get_cache_stats();
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.max_capacity, 1000);
    assert_eq!(stats.hit_ratio, 0.0);
}

#[tokio::test]
async fn auto_detect_adds_environment_method_when_service_url_set() {
    let var = "SONGBIRD_TEST_DISCOVERY_SERVICE_URL";
    songbird_process_env::set_var(var, "http://127.0.0.1:8080");
    let mut d = test_discovery();
    d.auto_detect_discovery_methods().await.unwrap();
    assert!(d.discovery_methods.iter().any(|m| matches!(m, DiscoveryMethod::Environment)));
    songbird_process_env::remove_var(var);
}

#[test]
fn cache_service_respects_disabled_cache() {
    let mut d = test_discovery_with_cache(10, false);
    d.cache_service("ignored", sample_service("ignored"));
    assert!(d.get_cached_service("ignored").is_none());
    assert_eq!(d.get_cache_stats().total_entries, 0);
}

#[test]
fn get_cached_service_miss_when_not_cached() {
    let d = test_discovery();
    assert!(d.get_cached_service("missing").is_none());
}

#[test]
fn cache_ttl_expiry_makes_get_return_none() {
    let mut d = test_discovery();
    d.service_cache.insert(
        "expired".to_string(),
        CachedServiceInfo {
            service_info: sample_service("expired"),
            cached_at: std::time::Instant::now() - std::time::Duration::from_secs(3600),
            ttl: std::time::Duration::from_secs(1),
        },
    );
    assert!(d.get_cached_service("expired").is_none());
    let stats = d.get_cache_stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.valid_entries, 0);
    assert_eq!(stats.expired_entries, 1);
}

#[test]
fn cleanup_cache_removes_expired_entries() {
    let mut d = test_discovery();
    d.service_cache.insert(
        "expired".to_string(),
        CachedServiceInfo {
            service_info: sample_service("expired"),
            cached_at: std::time::Instant::now() - std::time::Duration::from_secs(3600),
            ttl: std::time::Duration::from_secs(1),
        },
    );
    d.service_cache.insert(
        "fresh".to_string(),
        CachedServiceInfo {
            service_info: sample_service("fresh"),
            cached_at: std::time::Instant::now(),
            ttl: std::time::Duration::from_secs(3600),
        },
    );
    d.cleanup_cache();
    assert!(!d.service_cache.contains_key("expired"));
    assert!(d.service_cache.contains_key("fresh"));
    let stats = d.get_cache_stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.valid_entries, 1);
}

#[test]
fn cache_evicts_oldest_when_at_capacity() {
    let mut d = test_discovery_with_cache(2, true);
    d.cache_service("first", sample_service("first"));
    std::thread::sleep(std::time::Duration::from_millis(5));
    d.cache_service("second", sample_service("second"));
    std::thread::sleep(std::time::Duration::from_millis(5));
    d.cache_service("third", sample_service("third"));
    assert!(d.get_cached_service("first").is_none());
    assert!(d.get_cached_service("second").is_some());
    assert!(d.get_cached_service("third").is_some());
}

#[test]
fn get_cache_stats_after_multiple_caches() {
    let mut d = test_discovery();
    d.cache_service("a", sample_service("a"));
    d.cache_service("b", sample_service("b"));
    let stats = d.get_cache_stats();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.valid_entries, 2);
    assert_eq!(stats.expired_entries, 0);
}

#[test]
fn parse_universal_consul_format() {
    let d = test_discovery();
    let data = serde_json::json!({ "web": [], "api": [], "cache": [] });
    let services = d.parse_universal_service_response(&data).unwrap();
    assert_eq!(services.len(), 3);
    assert!(services.iter().any(|s| s.name == "web"));
    assert!(services.iter().all(|s| s.service_type == "http-registry"));
}

#[test]
fn parse_universal_object_with_nested_applications_uses_consul_branch() {
    let d = test_discovery();
    let data = serde_json::json!({
        "applications": {
            "application": [
                { "name": "USER-SERVICE" },
                { "name": "ORDER-SERVICE" }
            ]
        }
    });
    let services = d.parse_universal_service_response(&data).unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "applications");
    assert_eq!(services[0].service_type, "http-registry");
}

#[test]
fn parse_universal_generic_array_format() {
    let d = test_discovery();
    let data = serde_json::json!([
        { "name": "alpha" },
        { "name": "beta" },
        { "other": "ignored" }
    ]);
    let services = d.parse_universal_service_response(&data).unwrap();
    assert_eq!(services.len(), 2);
    assert!(services.iter().all(|s| s.service_type == "generic"));
}

#[test]
fn parse_universal_invalid_json_shape_returns_empty() {
    let d = test_discovery();
    let data = serde_json::json!("not-an-object-or-array");
    let services = d.parse_universal_service_response(&data).unwrap();
    assert!(services.is_empty());
}

#[tokio::test]
async fn discover_from_environment_finds_service_urls() {
    let var = "MYAPP_SERVICE_URL";
    songbird_process_env::set_var(var, "http://127.0.0.1:9000");
    let d = test_discovery();
    let services = d.discover_from_environment(&ServiceQuery::new()).await.unwrap();
    assert!(services.iter().any(|s| s.name == "myapp"));
    assert!(services.iter().any(|s| s.service_type == "environment"));
    songbird_process_env::remove_var(var);
}

#[tokio::test]
async fn discover_from_environment_finds_endpoint_suffix() {
    let var = "PAYMENTS_ENDPOINT";
    songbird_process_env::set_var(var, "http://127.0.0.1:9001");
    let d = test_discovery();
    let services = d.discover_from_environment(&ServiceQuery::new()).await.unwrap();
    assert!(services.iter().any(|s| s.name == "payments"));
    songbird_process_env::remove_var(var);
}

#[tokio::test]
async fn discover_from_environment_filters_by_name() {
    let url_var = "FILTERME_SERVICE_URL";
    let other_var = "OTHER_SERVICE_URL";
    songbird_process_env::set_var(url_var, "http://127.0.0.1:9002");
    songbird_process_env::set_var(other_var, "http://127.0.0.1:9003");
    let d = test_discovery();
    let mut query = ServiceQuery::new();
    query.name = Some("filterme".into());
    let services = d.discover_from_environment(&query).await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "filterme");
    songbird_process_env::remove_var(url_var);
    songbird_process_env::remove_var(other_var);
}
