// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for Discovery Factory
//!
//! Comprehensive tests for Universal Discovery Factory and adapters

use songbird_discovery::UniversalDiscoveryFactory;
use songbird_discovery::traits::discovery::{DiscoveryBackend, DiscoveryConfig};
use std::time::Duration;

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    assert!(matches!(config.backend, DiscoveryBackend::Static));
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.connection_timeout, Duration::from_secs(10));
    assert_eq!(config.retry_attempts, 3);
    assert_eq!(config.retry_delay, Duration::from_secs(1));
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        backend: DiscoveryBackend::Kubernetes {
            namespace: Some("default".to_string()),
            in_cluster: true,
            kubeconfig_path: None,
        },
        health_check_interval: Duration::from_secs(60),
        connection_timeout: Duration::from_secs(30),
        retry_attempts: 5,
        retry_delay: Duration::from_secs(2),
    };

    assert!(matches!(config.backend, DiscoveryBackend::Kubernetes { .. }));
    assert_eq!(config.health_check_interval, Duration::from_secs(60));
    assert_eq!(config.retry_attempts, 5);
}

#[test]
fn test_discovery_backend_static() {
    let backend = DiscoveryBackend::Static;

    assert!(matches!(backend, DiscoveryBackend::Static));
}

#[test]
fn test_discovery_backend_songbird() {
    let backend = DiscoveryBackend::Songbird {
        federation_enabled: true,
        trust_verification: true,
        attribution_tracking: true,
    };

    if let DiscoveryBackend::Songbird {
        federation_enabled,
        trust_verification,
        attribution_tracking,
    } = backend
    {
        assert!(federation_enabled);
        assert!(trust_verification);
        assert!(attribution_tracking);
    } else {
        panic!("Expected Songbird variant");
    }
}

#[test]
fn test_discovery_backend_kubernetes() {
    let backend = DiscoveryBackend::Kubernetes {
        namespace: Some("production".to_string()),
        in_cluster: true,
        kubeconfig_path: None,
    };

    if let DiscoveryBackend::Kubernetes {
        namespace,
        in_cluster,
        kubeconfig_path,
    } = backend
    {
        assert_eq!(namespace, Some("production".to_string()));
        assert!(in_cluster);
        assert!(kubeconfig_path.is_none());
    } else {
        panic!("Expected Kubernetes variant");
    }
}

#[test]
fn test_discovery_backend_kubernetes_with_kubeconfig() {
    let backend = DiscoveryBackend::Kubernetes {
        namespace: None,
        in_cluster: false,
        kubeconfig_path: Some("/home/user/.kube/config".to_string()),
    };

    if let DiscoveryBackend::Kubernetes {
        namespace,
        in_cluster,
        kubeconfig_path,
    } = backend
    {
        assert!(namespace.is_none());
        assert!(!in_cluster);
        assert_eq!(kubeconfig_path, Some("/home/user/.kube/config".to_string()));
    } else {
        panic!("Expected Kubernetes variant");
    }
}

#[test]
fn test_discovery_backend_etcd() {
    let backend = DiscoveryBackend::Etcd {
        endpoints: vec!["http://etcd1:2379".to_string(), "http://etcd2:2379".to_string()],
        username: Some("admin".to_string()),
        password: Some("secret".to_string()),
    };

    if let DiscoveryBackend::Etcd {
        endpoints,
        username,
        password,
    } = backend
    {
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0], "http://etcd1:2379");
        assert_eq!(username, Some("admin".to_string()));
        assert_eq!(password, Some("secret".to_string()));
    } else {
        panic!("Expected Etcd variant");
    }
}

#[test]
fn test_discovery_backend_etcd_no_auth() {
    let backend = DiscoveryBackend::Etcd {
        endpoints: vec!["http://localhost:2379".to_string()],
        username: None,
        password: None,
    };

    if let DiscoveryBackend::Etcd {
        endpoints,
        username,
        password,
    } = backend
    {
        assert_eq!(endpoints.len(), 1);
        assert!(username.is_none());
        assert!(password.is_none());
    } else {
        panic!("Expected Etcd variant");
    }
}

#[test]
fn test_discovery_config_serialization() {
    let config = DiscoveryConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    assert!(json.contains("backend"));
    assert!(json.contains("health_check_interval"));

    let deserialized: DiscoveryConfig = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(config.retry_attempts, deserialized.retry_attempts);
}

#[test]
fn test_discovery_config_clone() {
    let config = DiscoveryConfig::default();
    let cloned = config.clone();

    assert_eq!(config.retry_attempts, cloned.retry_attempts);
    assert_eq!(config.health_check_interval, cloned.health_check_interval);
}

#[test]
fn test_discovery_config_debug() {
    let config = DiscoveryConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("DiscoveryConfig"));
    assert!(debug_str.contains("backend"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_factory_create_for_static_config() {
    let config = DiscoveryConfig {
        backend: DiscoveryBackend::Static,
        health_check_interval: Duration::from_secs(30),
        connection_timeout: Duration::from_secs(10),
        retry_attempts: 3,
        retry_delay: Duration::from_secs(1),
    };

    let discovery = UniversalDiscoveryFactory::create_for_config(&config).await;
    assert!(discovery.is_ok(), "Static discovery should be created successfully");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_factory_create_auto_detect() {
    // Should create some form of discovery (likely static in test environment)
    let discovery = UniversalDiscoveryFactory::create_auto_detect().await;
    assert!(discovery.is_ok(), "Auto-detect should create a discovery instance");
}

#[test]
fn test_discovery_config_different_timeouts() {
    let fast = DiscoveryConfig {
        backend: DiscoveryBackend::Static,
        health_check_interval: Duration::from_secs(10),
        connection_timeout: Duration::from_secs(5),
        retry_attempts: 1,
        retry_delay: Duration::from_millis(500),
    };

    let slow = DiscoveryConfig {
        backend: DiscoveryBackend::Static,
        health_check_interval: Duration::from_secs(120),
        connection_timeout: Duration::from_secs(60),
        retry_attempts: 10,
        retry_delay: Duration::from_secs(5),
    };

    assert!(fast.health_check_interval < slow.health_check_interval);
    assert!(fast.retry_attempts < slow.retry_attempts);
}

#[test]
fn test_discovery_backend_songbird_variants() {
    let full_featured = DiscoveryBackend::Songbird {
        federation_enabled: true,
        trust_verification: true,
        attribution_tracking: true,
    };

    let minimal = DiscoveryBackend::Songbird {
        federation_enabled: false,
        trust_verification: false,
        attribution_tracking: false,
    };

    assert!(matches!(full_featured, DiscoveryBackend::Songbird { .. }));
    assert!(matches!(minimal, DiscoveryBackend::Songbird { .. }));
}

#[test]
fn test_discovery_config_extreme_values() {
    let aggressive = DiscoveryConfig {
        backend: DiscoveryBackend::Static,
        health_check_interval: Duration::from_secs(1),
        connection_timeout: Duration::from_secs(1),
        retry_attempts: 100,
        retry_delay: Duration::from_millis(10),
    };

    assert_eq!(aggressive.retry_attempts, 100);
    assert_eq!(aggressive.health_check_interval, Duration::from_secs(1));
}

#[test]
fn test_discovery_backend_etcd_multiple_endpoints() {
    let backend = DiscoveryBackend::Etcd {
        endpoints: vec![
            "http://etcd1:2379".to_string(),
            "http://etcd2:2379".to_string(),
            "http://etcd3:2379".to_string(),
        ],
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
    };

    if let DiscoveryBackend::Etcd {
        endpoints,
        ..
    } = backend
    {
        assert_eq!(endpoints.len(), 3);
    }
}

#[test]
fn test_discovery_backend_kubernetes_multi_namespace() {
    let namespaces = vec!["default", "production", "staging"];

    for ns in namespaces {
        let backend = DiscoveryBackend::Kubernetes {
            namespace: Some(ns.to_string()),
            in_cluster: true,
            kubeconfig_path: None,
        };

        if let DiscoveryBackend::Kubernetes {
            namespace,
            ..
        } = backend
        {
            assert_eq!(namespace, Some(ns.to_string()));
        }
    }
}

#[test]
fn test_discovery_config_serialization_all_backends() {
    let backends = vec![
        DiscoveryBackend::Static,
        DiscoveryBackend::Songbird {
            federation_enabled: true,
            trust_verification: true,
            attribution_tracking: true,
        },
        DiscoveryBackend::Kubernetes {
            namespace: Some("default".to_string()),
            in_cluster: true,
            kubeconfig_path: None,
        },
        DiscoveryBackend::Etcd {
            endpoints: vec!["http://localhost:2379".to_string()],
            username: None,
            password: None,
        },
    ];

    for backend in backends {
        let config = DiscoveryConfig {
            backend,
            health_check_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
        };

        let json = serde_json::to_string(&config).expect("Should serialize");
        let _deserialized: DiscoveryConfig =
            serde_json::from_str(&json).expect("Should deserialize");
    }
}

#[test]
fn test_discovery_config_zero_retries() {
    let config = DiscoveryConfig {
        backend: DiscoveryBackend::Static,
        health_check_interval: Duration::from_secs(30),
        connection_timeout: Duration::from_secs(10),
        retry_attempts: 0,
        retry_delay: Duration::from_secs(0),
    };

    assert_eq!(config.retry_attempts, 0);
    assert_eq!(config.retry_delay, Duration::from_secs(0));
}
