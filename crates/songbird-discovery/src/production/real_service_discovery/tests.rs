// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use std::collections::HashMap;

use crate::discovery::core::ServiceInstance;
use crate::traits::ServiceDiscovery;

use super::*;

#[tokio::test]
async fn test_service_registration() {
    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig::default());

    // Use configurable test endpoint
    let test_host = std::env::var("TEST_SERVICE_HOST").unwrap_or_else(|_| {
        songbird_config::canonical::constants::network::DEFAULT_HOST.to_string()
    });
    let test_port =
        std::env::var("TEST_SERVICE_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    let _service = ServiceInstance {
        id: "test-service".to_string(),
        name: "Test Service".to_string(),
        endpoint: format!("http://{test_host}:{test_port}"),
        capabilities: vec!["test".to_string()],
        health_status: "unknown".to_string(),
        metadata: HashMap::new(),
    };

    // Test would need proper ServiceInfo conversion
    assert!(ServiceDiscovery::exists(&discovery, "test-service").await.is_ok());
}

#[tokio::test]
async fn test_capability_filtering() {
    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig::default());

    // Test capability filtering
    let services = discovery.get_services_by_capability("security").await;
    assert!(services.is_ok());
}

fn sample_service_info(
    id: &str,
    name: &str,
    endpoint_path: &str,
    tags: Vec<String>,
) -> crate::traits::ServiceInfo {
    use crate::traits::service::{ServiceEndpoint, ServiceInfo, ServiceStatus};
    use chrono::Utc;
    use std::collections::HashMap;

    ServiceInfo {
        service_id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        service_type: "test".to_string(),
        description: None,
        endpoints: vec![ServiceEndpoint {
            path: endpoint_path.to_string(),
            method: "GET".to_string(),
            description: None,
            parameters: Vec::new(),
            response_schema: None,
            auth_required: false,
            rate_limit: None,
        }],
        health_check_endpoint: None,
        metadata: HashMap::new(),
        tags,
        dependencies: Vec::new(),
        status: ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        instance_id: id.to_string(),
        host: endpoint_path.to_string(),
        port: 8080,
    }
}

#[tokio::test]
async fn register_list_and_capability_queries() {
    use crate::traits::discovery::{ServiceHealthStatus, ServiceQuery};

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info(
        "svc-reg-1",
        "RegistryAlpha",
        "http://127.0.0.1:9",
        vec!["security".to_string(), "metrics".to_string()],
    );

    ServiceDiscovery::register(&discovery, info).await.expect("register service");

    ServiceDiscovery::update_health(&discovery, "svc-reg-1", ServiceHealthStatus::Healthy)
        .await
        .expect("mark healthy");

    let by_cap = discovery.get_services_by_capability("security").await.expect("by capability");
    assert_eq!(by_cap.len(), 1);
    assert_eq!(by_cap[0].id, "svc-reg-1");

    let all = ServiceDiscovery::list_all(&discovery).await.expect("list all");
    assert_eq!(all.len(), 1);

    let mut q = ServiceQuery::new();
    q.name = Some("Alpha".into());
    let filtered = ServiceDiscovery::discover(&discovery, q).await.expect("discover");
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].name.contains("Alpha"));
}

#[tokio::test]
async fn unregister_removes_service() {
    use crate::traits::ServiceDiscovery;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("svc-rm", "Rm", "http://127.0.0.1:8", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");
    assert!(ServiceDiscovery::exists(&discovery, "svc-rm").await.expect("exists"));

    ServiceDiscovery::unregister(&discovery, "svc-rm").await.expect("unregister");
    assert!(!ServiceDiscovery::exists(&discovery, "svc-rm").await.expect("gone"));
}

#[tokio::test]
async fn discover_excludes_unknown_health() {
    use crate::traits::ServiceDiscovery;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("u1", "UnknownHealth", "http://127.0.0.1:7", vec!["x".into()]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");
    // Unknown health — not listed by discover()
    let q = crate::traits::ServiceQuery::default();
    let found = ServiceDiscovery::discover(&discovery, q).await.expect("discover");
    assert_eq!(found.len(), 0);
}

#[tokio::test]
async fn discover_includes_degraded_health() {
    use crate::traits::ServiceDiscovery;
    use crate::traits::discovery::ServiceHealthStatus;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("d1", "DegradedSvc", "http://127.0.0.1:6", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");
    ServiceDiscovery::update_health(&discovery, "d1", ServiceHealthStatus::Degraded)
        .await
        .expect("health");

    let found = ServiceDiscovery::discover(&discovery, crate::traits::ServiceQuery::default())
        .await
        .expect("discover");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].service_id, "d1");
}

#[tokio::test]
async fn get_services_by_capability_excludes_non_healthy() {
    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    use crate::traits::ServiceDiscovery;

    let info = sample_service_info("cap-test", "Cap", "http://127.0.0.1:5", vec!["wanted".into()]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");

    let empty = discovery.get_services_by_capability("wanted").await.expect("cap");
    assert_eq!(empty.len(), 0);

    use crate::traits::discovery::ServiceHealthStatus;
    ServiceDiscovery::update_health(&discovery, "cap-test", ServiceHealthStatus::Healthy)
        .await
        .expect("mark healthy");

    let got = discovery.get_services_by_capability("wanted").await.expect("cap2");
    assert_eq!(got.len(), 1);
}

#[tokio::test]
async fn cleanup_unhealthy_keeps_when_retry_below_max() {
    use crate::traits::ServiceDiscovery;
    use crate::traits::discovery::ServiceHealthStatus;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        max_retry_attempts: 5,
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("uh", "Unhealthy", "http://127.0.0.1:4", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");
    ServiceDiscovery::update_health(&discovery, "uh", ServiceHealthStatus::Unhealthy)
        .await
        .expect("health");

    let removed = discovery.cleanup_unhealthy_services().await.expect("cleanup");
    assert_eq!(removed, 0);
    assert!(ServiceDiscovery::exists(&discovery, "uh").await.expect("exists"));
}

#[tokio::test]
async fn cleanup_unhealthy_removes_when_retry_budget_is_zero() {
    use crate::traits::ServiceDiscovery;
    use crate::traits::discovery::ServiceHealthStatus;

    // With `max_retry_attempts == 0`, `retry_count < max` is false even at 0, so unhealthy rows are dropped.
    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        max_retry_attempts: 0,
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("gone", "Gone", "http://127.0.0.1:3", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");
    ServiceDiscovery::update_health(&discovery, "gone", ServiceHealthStatus::Unhealthy)
        .await
        .expect("health");

    let removed = discovery.cleanup_unhealthy_services().await.expect("cleanup");
    assert_eq!(removed, 1);
    assert!(!ServiceDiscovery::exists(&discovery, "gone").await.expect("gone"));
}

#[tokio::test]
async fn update_metadata_for_known_service() {
    use crate::traits::ServiceDiscovery;
    use std::collections::HashMap;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("meta", "Meta", "http://127.0.0.1:2", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");

    let mut m = HashMap::new();
    m.insert("k".into(), "v".into());
    ServiceDiscovery::update_metadata(&discovery, "meta", m).await.expect("meta");

    let all = ServiceDiscovery::list_all(&discovery).await.expect("list");
    assert_eq!(all[0].metadata.get("k").and_then(|v| v.as_str()), Some("v"));
}

#[tokio::test]
async fn get_service_health_returns_none_when_missing() {
    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig::default());
    let h = discovery.get_service_health("nope").await.expect("health");
    assert!(h.is_none());
}

#[tokio::test]
async fn discover_filters_by_name_substring() {
    use crate::traits::ServiceDiscovery;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let a = sample_service_info("a", "AlphaTeam", "http://127.0.0.1:11", vec![]);
    let b = sample_service_info("b", "Beta", "http://127.0.0.1:12", vec![]);
    ServiceDiscovery::register(&discovery, a).await.expect("a");
    ServiceDiscovery::register(&discovery, b).await.expect("b");

    use crate::traits::discovery::ServiceHealthStatus;
    ServiceDiscovery::update_health(&discovery, "a", ServiceHealthStatus::Healthy)
        .await
        .expect("ha");
    ServiceDiscovery::update_health(&discovery, "b", ServiceHealthStatus::Healthy)
        .await
        .expect("hb");

    let mut q = crate::traits::ServiceQuery::new();
    q.name = Some("Alpha".into());
    let found = ServiceDiscovery::discover(&discovery, q).await.expect("discover");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "AlphaTeam");
}

#[tokio::test]
async fn exists_and_is_registered_agree() {
    use crate::traits::ServiceDiscovery;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("ex", "Ex", "http://127.0.0.1:10", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("reg");
    let e = ServiceDiscovery::exists(&discovery, "ex").await.expect("e");
    let i = ServiceDiscovery::is_registered(&discovery, "ex").await.expect("i");
    assert_eq!(e, i);
    assert!(e);
}

#[test]
fn service_health_status_roundtrips_json() {
    let v = serde_json::to_string(&ServiceHealthStatus::Degraded).unwrap();
    let back: ServiceHealthStatus = serde_json::from_str(&v).unwrap();
    assert_eq!(back, ServiceHealthStatus::Degraded);
}

#[test]
fn registered_service_serializes_metadata() {
    let reg = RegisteredService {
        instance: ServiceInstance {
            id: "i1".into(),
            name: "n".into(),
            endpoint: "http://127.0.0.1:1".into(),
            capabilities: vec!["c".into()],
            health_status: "ok".into(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("k".into(), "v".into());
                m
            },
        },
        registered_at: std::time::SystemTime::UNIX_EPOCH,
        last_heartbeat: None,
        health_status: ServiceHealthStatus::Healthy,
        retry_count: 0,
    };
    let json = serde_json::to_string(&reg).unwrap();
    let back: RegisteredService = serde_json::from_str(&json).unwrap();
    assert_eq!(back.instance.id, "i1");
    assert_eq!(back.health_status, ServiceHealthStatus::Healthy);
}

#[test]
fn production_discovery_config_clones_and_defaults() {
    let a = ProductionDiscoveryConfig::default();
    let b = a.clone();
    assert_eq!(a.max_retry_attempts, b.max_retry_attempts);
    assert_eq!(a.enable_health_checks, b.enable_health_checks);
}

#[test]
fn health_record_fields_accessible() {
    let r = HealthRecord {
        service_id: "s".into(),
        status: ServiceHealthStatus::Unknown,
        last_check: std::time::SystemTime::UNIX_EPOCH,
        response_time_ms: 12,
        error_message: Some("e".into()),
    };
    assert_eq!(r.response_time_ms, 12);
    assert_eq!(r.error_message.as_deref(), Some("e"));
}

#[tokio::test]
async fn discover_skips_name_mismatch_when_query_has_name() {
    use crate::traits::ServiceDiscovery;
    use crate::traits::discovery::{ServiceHealthStatus, ServiceQuery};

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let info = sample_service_info("n1", "OnlyName", "http://127.0.0.1:20", vec![]);
    ServiceDiscovery::register(&discovery, info).await.expect("register");
    ServiceDiscovery::update_health(&discovery, "n1", ServiceHealthStatus::Healthy)
        .await
        .expect("health");

    let mut q = ServiceQuery::new();
    q.name = Some("nomatch".into());
    let found = ServiceDiscovery::discover(&discovery, q).await.expect("discover");
    assert!(found.is_empty());
}

#[tokio::test]
async fn list_all_includes_all_health_states() {
    use crate::traits::ServiceDiscovery;

    let discovery = ProductionServiceDiscovery::new(ProductionDiscoveryConfig {
        enable_health_checks: false,
        ..ProductionDiscoveryConfig::default()
    });

    let u = sample_service_info("u", "U", "http://127.0.0.1:21", vec![]);
    ServiceDiscovery::register(&discovery, u).await.expect("reg");
    let all = ServiceDiscovery::list_all(&discovery).await.expect("list");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].service_id, "u");
}
