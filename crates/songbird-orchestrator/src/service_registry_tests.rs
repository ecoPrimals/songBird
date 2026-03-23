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
