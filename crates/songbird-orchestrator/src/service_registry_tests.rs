// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use songbird_types::defaults::ports::DEFAULT_PORT_RANGE_START;

#[tokio::test]
async fn test_service_registration() {
    let registry = ServiceRegistry::new();

    let request = RegistrationRequest {
        primal_name: "TestService".to_string(),
        primal_version: "1.0.0".to_string(),
        capabilities: vec![],
        protocols: vec!["https".to_string()],
        preferred_protocol: "https".to_string(),
        health_check_path: Some("/health".to_string()),
        metadata: None,
    };

    let response = registry.register(request).await.unwrap();

    assert_eq!(response.status, "registered");
    assert!(!response.service_id.is_empty());
    assert!(response.assigned_endpoint.port >= DEFAULT_PORT_RANGE_START);
}

#[tokio::test]
async fn test_heartbeat() {
    let registry = ServiceRegistry::new();

    // Register service
    let request = RegistrationRequest {
        primal_name: "TestService".to_string(),
        primal_version: "1.0.0".to_string(),
        capabilities: vec![],
        protocols: vec!["https".to_string()],
        preferred_protocol: "https".to_string(),
        health_check_path: None,
        metadata: None,
    };

    let registration = registry.register(request).await.unwrap();

    // Send heartbeat
    let heartbeat = HeartbeatRequest {
        service_id: registration.service_id.clone(),
        token: registration.registration_token.clone(),
        status: "operational".to_string(),
        current_load: None,
        capabilities_changed: false,
    };

    let response = registry.heartbeat(heartbeat).await.unwrap();
    assert_eq!(response.status, "acknowledged");
}

#[tokio::test]
async fn test_deregistration() {
    let registry = ServiceRegistry::new();

    // Register service
    let request = RegistrationRequest {
        primal_name: "TestService".to_string(),
        primal_version: "1.0.0".to_string(),
        capabilities: vec![],
        protocols: vec!["https".to_string()],
        preferred_protocol: "https".to_string(),
        health_check_path: None,
        metadata: None,
    };

    let registration = registry.register(request).await.unwrap();

    // Deregister
    let dereg = DeregistrationRequest {
        service_id: registration.service_id.clone(),
        token: registration.registration_token.clone(),
        reason: "test".to_string(),
    };

    registry.deregister(dereg).await.unwrap();

    // Verify service is gone
    assert!(registry.get_service(&registration.service_id).await.is_none());
}

#[tokio::test]
async fn test_port_allocation() {
    let config = RegistryConfig {
        port_range_start: 9000,
        port_range_end: 9002,
        ..Default::default()
    };

    let registry = ServiceRegistry::with_config(config);

    // Register 3 services (should fill the range)
    for i in 0..3 {
        let request = RegistrationRequest {
            primal_name: format!("Service{i}"),
            primal_version: "1.0.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        };

        registry.register(request).await.unwrap();
    }

    // 4th service should fail (no ports available)
    let request = RegistrationRequest {
        primal_name: "Service3".to_string(),
        primal_version: "1.0.0".to_string(),
        capabilities: vec![],
        protocols: vec!["https".to_string()],
        preferred_protocol: "https".to_string(),
        health_check_path: None,
        metadata: None,
    };

    assert!(registry.register(request).await.is_err());
}

#[tokio::test]
async fn get_service_after_register() {
    let registry = ServiceRegistry::new();
    let reg = registry
        .register(RegistrationRequest {
            primal_name: "P".to_string(),
            primal_version: "1".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();
    let svc = registry.get_service(&reg.service_id).await.unwrap();
    assert_eq!(svc.service_id, reg.service_id);
}

#[tokio::test]
async fn list_and_query_by_capability() {
    let registry = ServiceRegistry::new();
    let cap = ServiceCapability {
        name: "ml.infer".to_string(),
        capability_type: "compute".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    registry
        .register(RegistrationRequest {
            primal_name: "Infer".to_string(),
            primal_version: "1".to_string(),
            capabilities: vec![cap],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();
    let list = registry.list_services().await;
    assert_eq!(list.len(), 1);
    let q = registry.query_by_capability("ml.infer").await;
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].service_name, "Infer");
}

#[tokio::test]
async fn heartbeat_rejects_bad_token() {
    let registry = ServiceRegistry::new();
    let reg = registry
        .register(RegistrationRequest {
            primal_name: "P".to_string(),
            primal_version: "1".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();
    let err = registry
        .heartbeat(HeartbeatRequest {
            service_id: reg.service_id,
            token: "wrong".to_string(),
            status: "operational".to_string(),
            current_load: None,
            capabilities_changed: false,
        })
        .await;
    assert!(err.is_err());
}

#[tokio::test]
async fn deregister_wrong_token_fails() {
    let registry = ServiceRegistry::new();
    let reg = registry
        .register(RegistrationRequest {
            primal_name: "P".to_string(),
            primal_version: "1".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();
    let r = registry
        .deregister(DeregistrationRequest {
            service_id: reg.service_id,
            token: "bad".to_string(),
            reason: "x".to_string(),
        })
        .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn get_stats_reflects_registration() {
    let registry = ServiceRegistry::new();
    registry
        .register(RegistrationRequest {
            primal_name: "P".to_string(),
            primal_version: "1".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();
    let s = registry.get_stats().await;
    assert_eq!(s.total_services, 1);
    assert_eq!(s.active_services, 1);
    assert!(s.allocated_ports >= 1);
}

#[test]
fn service_endpoint_new_url() {
    let e = ServiceEndpoint::new("https", "127.0.0.1", 8443);
    assert_eq!(e.full_url, "https://127.0.0.1:8443");
    assert_eq!(e.port, 8443);
}

#[tokio::test]
async fn heartbeat_unknown_service_returns_error() {
    let registry = ServiceRegistry::new();
    let result = registry
        .heartbeat(HeartbeatRequest {
            service_id: "nonexistent-service-id".to_string(),
            token: "some-token".to_string(),
            status: "operational".to_string(),
            current_load: None,
            capabilities_changed: false,
        })
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test(start_paused = true)]
async fn cleanup_stale_services_removes_expired() {
    let config = RegistryConfig {
        service_ttl_sec: 0,
        ..RegistryConfig::default()
    };
    let registry = ServiceRegistry::with_config(config);

    registry
        .register(RegistrationRequest {
            primal_name: "Stale".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(registry.list_services().await.len(), 1);

    tokio::time::advance(std::time::Duration::from_millis(50)).await;
    let removed = registry.cleanup_stale_services().await;
    assert_eq!(removed, 1);
    assert!(registry.list_services().await.is_empty());
}

#[tokio::test]
async fn cleanup_stale_services_no_removal_when_fresh() {
    let registry = ServiceRegistry::new();

    registry
        .register(RegistrationRequest {
            primal_name: "Fresh".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    let removed = registry.cleanup_stale_services().await;
    assert_eq!(removed, 0);
    assert_eq!(registry.list_services().await.len(), 1);
}

#[test]
fn registry_config_default_port_range_valid() {
    let config = RegistryConfig::default();
    assert!(config.port_range_start < config.port_range_end);
    assert!(config.max_missed_heartbeats > 0);
    assert!(config.service_ttl_sec > 0);
    assert!(config.default_heartbeat_interval > 0);
}

#[tokio::test]
async fn query_by_capability_empty_when_no_match() {
    let registry = ServiceRegistry::new();

    registry
        .register(RegistrationRequest {
            primal_name: "NoMatch".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![ServiceCapability {
                name: "crypto".to_string(),
                capability_type: "provider".to_string(),
                metadata: std::collections::HashMap::new(),
            }],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    let result = registry.query_by_capability("nonexistent").await;
    assert!(result.is_empty());
}

#[tokio::test]
async fn query_by_capability_returns_matching() {
    let registry = ServiceRegistry::new();

    registry
        .register(RegistrationRequest {
            primal_name: "CryptoService".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![ServiceCapability {
                name: "crypto.encrypt".to_string(),
                capability_type: "provider".to_string(),
                metadata: std::collections::HashMap::new(),
            }],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    let result = registry.query_by_capability("crypto.encrypt").await;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].service_name, "CryptoService");
}

#[tokio::test(start_paused = true)]
async fn cleanup_releases_ports() {
    let config = RegistryConfig {
        service_ttl_sec: 0,
        ..RegistryConfig::default()
    };
    let registry = ServiceRegistry::with_config(config);

    registry
        .register(RegistrationRequest {
            primal_name: "PortUser".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    let stats_before = registry.get_stats().await;
    assert!(stats_before.allocated_ports >= 1);

    tokio::time::advance(std::time::Duration::from_millis(50)).await;
    registry.cleanup_stale_services().await;

    let stats_after = registry.get_stats().await;
    assert!(stats_after.allocated_ports < stats_before.allocated_ports);
}

#[tokio::test]
async fn default_trait_creates_new_registry() {
    let registry = ServiceRegistry::default();
    let stats = registry.get_stats().await;
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.allocated_ports, 0);
}

#[tokio::test]
async fn multi_protocol_registration_allocates_fallback_port() {
    let registry = ServiceRegistry::new();

    let request = RegistrationRequest {
        primal_name: "DualProto".to_string(),
        primal_version: "1.0.0".to_string(),
        capabilities: vec![],
        protocols: vec!["https".to_string(), "tarpc".to_string()],
        preferred_protocol: "https".to_string(),
        health_check_path: None,
        metadata: None,
    };

    let response = registry.register(request).await.unwrap();
    assert!(response.fallback_endpoint.is_some());
    let fallback = response.fallback_endpoint.unwrap();
    assert_ne!(response.assigned_endpoint.port, fallback.port);

    let stats = registry.get_stats().await;
    assert_eq!(stats.allocated_ports, 2);
}

#[tokio::test]
async fn single_protocol_registration_has_no_fallback() {
    let registry = ServiceRegistry::new();

    let request = RegistrationRequest {
        primal_name: "SingleProto".to_string(),
        primal_version: "1.0.0".to_string(),
        capabilities: vec![],
        protocols: vec!["https".to_string()],
        preferred_protocol: "https".to_string(),
        health_check_path: None,
        metadata: None,
    };

    let response = registry.register(request).await.unwrap();
    assert!(response.fallback_endpoint.is_none());
    assert_eq!(registry.get_stats().await.allocated_ports, 1);
}

#[tokio::test]
async fn deregister_releases_ports_for_reuse() {
    let config = RegistryConfig {
        port_range_start: 9500,
        port_range_end: 9502,
        ..Default::default()
    };
    let registry = ServiceRegistry::with_config(config);

    let reg1 = registry
        .register(RegistrationRequest {
            primal_name: "First".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    registry
        .register(RegistrationRequest {
            primal_name: "Second".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    registry
        .register(RegistrationRequest {
            primal_name: "Third".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    // Range exhausted
    assert!(
        registry
            .register(RegistrationRequest {
                primal_name: "Fourth".to_string(),
                primal_version: "1.0".to_string(),
                capabilities: vec![],
                protocols: vec!["https".to_string()],
                preferred_protocol: "https".to_string(),
                health_check_path: None,
                metadata: None,
            })
            .await
            .is_err()
    );

    // Deregister first service, freeing a port
    registry
        .deregister(DeregistrationRequest {
            service_id: reg1.service_id,
            token: reg1.registration_token,
            reason: "test".to_string(),
        })
        .await
        .unwrap();

    // Now re-registration should succeed
    let reg_new = registry
        .register(RegistrationRequest {
            primal_name: "Replacement".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await;
    assert!(reg_new.is_ok());
}

#[tokio::test]
async fn heartbeat_operational_sets_active() {
    let registry = ServiceRegistry::new();

    let reg = registry
        .register(RegistrationRequest {
            primal_name: "StatusTest".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    let response = registry
        .heartbeat(HeartbeatRequest {
            service_id: reg.service_id.clone(),
            token: reg.registration_token.clone(),
            status: "operational".to_string(),
            current_load: Some(serde_json::json!(0.85)),
            capabilities_changed: false,
        })
        .await
        .unwrap();

    assert_eq!(response.status, "acknowledged");
    assert!(response.next_heartbeat_sec > 0);

    let service = registry.get_service(&reg.service_id).await.unwrap();
    assert_eq!(service.status, ServiceStatus::Active);
    assert_eq!(service.missed_heartbeats, 0);
}

#[tokio::test]
async fn heartbeat_non_operational_preserves_active_status() {
    let registry = ServiceRegistry::new();

    let reg = registry
        .register(RegistrationRequest {
            primal_name: "StatusPreserve".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    // Non-"operational" status string does not change status from Active
    registry
        .heartbeat(HeartbeatRequest {
            service_id: reg.service_id.clone(),
            token: reg.registration_token,
            status: "degraded".to_string(),
            current_load: None,
            capabilities_changed: false,
        })
        .await
        .unwrap();

    let service = registry.get_service(&reg.service_id).await.unwrap();
    assert_eq!(service.status, ServiceStatus::Active);
}

#[tokio::test]
async fn deregister_nonexistent_service_fails() {
    let registry = ServiceRegistry::new();
    let result = registry
        .deregister(DeregistrationRequest {
            service_id: "does-not-exist".to_string(),
            token: "any-token".to_string(),
            reason: "test".to_string(),
        })
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn get_service_returns_none_for_unknown() {
    let registry = ServiceRegistry::new();
    assert!(registry.get_service("nonexistent").await.is_none());
}

#[tokio::test]
async fn list_services_empty_by_default() {
    let registry = ServiceRegistry::new();
    assert!(registry.list_services().await.is_empty());
}

#[tokio::test]
async fn registration_with_metadata() {
    let registry = ServiceRegistry::new();
    let mut meta: HashMap<String, serde_json::Value> = HashMap::new();
    meta.insert("region".to_string(), serde_json::json!("us-east"));
    meta.insert("tier".to_string(), serde_json::json!("production"));

    let reg = registry
        .register(RegistrationRequest {
            primal_name: "MetaService".to_string(),
            primal_version: "2.0".to_string(),
            capabilities: vec![ServiceCapability {
                name: "storage.put".to_string(),
                capability_type: "storage".to_string(),
                metadata: HashMap::new(),
            }],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: Some("/ready".to_string()),
            metadata: Some(meta),
        })
        .await
        .unwrap();

    let svc = registry.get_service(&reg.service_id).await.unwrap();
    assert_eq!(svc.metadata.get("region"), Some(&serde_json::json!("us-east")));
    assert_eq!(svc.metadata.get("tier"), Some(&serde_json::json!("production")));
}

#[test]
fn port_allocator_wraps_around() {
    let mut alloc = PortAllocator::new(9000, 9002);
    let p1 = alloc.allocate("svc1").unwrap();
    let p2 = alloc.allocate("svc2").unwrap();
    let p3 = alloc.allocate("svc3").unwrap();
    assert_eq!(p1, 9000);
    assert_eq!(p2, 9001);
    assert_eq!(p3, 9002);
    assert!(alloc.allocate("svc4").is_err());

    alloc.release(9001);
    let p_reuse = alloc.allocate("svc5").unwrap();
    assert_eq!(p_reuse, 9001);
}

#[test]
fn port_allocator_release_noop_for_unallocated() {
    let mut alloc = PortAllocator::new(9000, 9002);
    alloc.release(9999); // should not panic
    assert!(!alloc.is_allocated(9999));
}

#[test]
fn port_allocator_is_allocated_tracks_state() {
    let mut alloc = PortAllocator::new(9000, 9002);
    assert!(!alloc.is_allocated(9000));
    alloc.allocate("svc1").unwrap();
    assert!(alloc.is_allocated(9000));
    alloc.release(9000);
    assert!(!alloc.is_allocated(9000));
}

#[tokio::test]
async fn stats_shows_zero_degraded_for_active_services() {
    let registry = ServiceRegistry::new();

    registry
        .register(RegistrationRequest {
            primal_name: "HealthyService".to_string(),
            primal_version: "1.0".to_string(),
            capabilities: vec![],
            protocols: vec!["https".to_string()],
            preferred_protocol: "https".to_string(),
            health_check_path: None,
            metadata: None,
        })
        .await
        .unwrap();

    let stats = registry.get_stats().await;
    assert_eq!(stats.total_services, 1);
    assert_eq!(stats.active_services, 1);
    assert_eq!(stats.degraded_services, 0);
    assert_eq!(stats.inactive_services, 0);
}
