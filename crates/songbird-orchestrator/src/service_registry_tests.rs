// SPDX-License-Identifier: AGPL-3.0-only
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
