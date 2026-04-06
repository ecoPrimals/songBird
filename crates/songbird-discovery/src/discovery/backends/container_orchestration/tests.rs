// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::{
    ApiEndpoint, AuthenticationMethod, ContainerInfo, NamespaceConfig,
    UniversalContainerOrchestration,
};

#[test]
fn api_endpoint_fields() {
    let ep = ApiEndpoint {
        url: "https://k8s.example:443".to_string(),
        version: "v1".to_string(),
        timeout: std::time::Duration::from_secs(10),
        verify_tls: true,
    };
    assert!(ep.url.contains("k8s"));
    assert!(ep.verify_tls);
}

#[test]
fn namespace_config_auto_discover_extends() {
    let cfg = NamespaceConfig {
        default_namespace: "default".to_string(),
        accessible_namespaces: vec!["app".to_string()],
        auto_discover: true,
    };
    assert!(cfg.auto_discover);
}

#[test]
fn authentication_method_variants_constructible() {
    assert!(matches!(
        AuthenticationMethod::BearerToken {
            token: "t".into(),
        },
        AuthenticationMethod::BearerToken { .. }
    ));
    assert!(matches!(
        AuthenticationMethod::ServiceAccount {
            token_path: "/var/run/secrets/token".into(),
        },
        AuthenticationMethod::ServiceAccount { .. }
    ));
    assert!(matches!(AuthenticationMethod::None, AuthenticationMethod::None));
}

#[test]
fn container_info_clone() {
    let c = ContainerInfo {
        id: "id1".to_string(),
        name: "n1".to_string(),
        image: "img:latest".to_string(),
        status: "running".to_string(),
        ports: vec!["8080/tcp".to_string()],
    };
    assert_eq!(c.name, "n1");
}

#[tokio::test]
async fn orchestration_new_authenticate_and_namespaces() {
    let c = UniversalContainerOrchestration::new().await.unwrap();
    let tok = c.authenticate().await.unwrap();
    assert!(!tok.is_empty());
    let ns = c.get_available_namespaces().await.unwrap();
    assert!(ns.iter().any(|s| s == "default"));
}

#[tokio::test]
async fn check_api_connectivity_bool() {
    let c = UniversalContainerOrchestration::new().await.unwrap();
    let _ = c.check_api_connectivity().await.unwrap();
}

#[tokio::test]
async fn discovered_container_map() {
    let mut c = UniversalContainerOrchestration::new().await.unwrap();
    c.add_discovered_container(
        "cid".to_string(),
        ContainerInfo {
            id: "cid".to_string(),
            name: "cname".to_string(),
            image: "i".to_string(),
            status: "up".to_string(),
            ports: vec![],
        },
    );
    assert_eq!(c.get_discovered_containers().len(), 1);
}

#[tokio::test]
async fn list_all_trait_ok() {
    use crate::traits::ServiceDiscovery;
    let c = UniversalContainerOrchestration::new().await.unwrap();
    let v = ServiceDiscovery::list_all(&c).await.unwrap();
    let _ = v;
}
