// SPDX-License-Identifier: AGPL-3.0-only
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
    clippy::cast_possible_wrap
)]
// SPDX-License-Identifier: AGPL-3.0-only
//! Coverage tests for songbird_discovery::primal_self_knowledge
//!
//! Tests the self-knowledge discovery architecture with env-safe patterns.

use songbird_discovery::primal_self_knowledge::{
    DiscoveryMechanism, DnsSrvDiscovery, EnvironmentDiscovery, PrimalError, PrimalIdentity,
    PrimalInfo, PrimalSelfKnowledge,
};
use std::time::SystemTime;

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

// ═══════════════════════════════════════════════════════════════════════
// PrimalError tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_primal_error_display() {
    let err = PrimalError::DiscoveryFailed {
        reason: "no provider".to_string(),
    };
    assert!(err.to_string().contains("no provider"));

    let err2 = PrimalError::IntrospectionFailed("bad format".to_string());
    assert!(err2.to_string().contains("bad format"));
}

#[test]
fn test_primal_error_from_var_error() {
    let var_err = std::env::VarError::NotPresent;
    let err: PrimalError = var_err.into();
    assert!(err.to_string().contains("Environment variable error"));
}

// ═══════════════════════════════════════════════════════════════════════
// PrimalIdentity tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_primal_identity_serialization() {
    let identity = PrimalIdentity {
        name: "songbird".to_string(),
        capabilities: vec!["orchestration".to_string(), "discovery".to_string()],
    };

    let json = serde_json::to_string(&identity).unwrap();
    let deserialized: PrimalIdentity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "songbird");
    assert_eq!(deserialized.capabilities.len(), 2);
}

#[test]
fn test_primal_identity_clone_debug() {
    let identity = PrimalIdentity {
        name: "test".to_string(),
        capabilities: vec!["cap1".to_string()],
    };
    let cloned = identity.clone();
    assert_eq!(identity.name, cloned.name);
    let debug = format!("{:?}", identity);
    assert!(debug.contains("test"));
}

// ═══════════════════════════════════════════════════════════════════════
// PrimalInfo tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_primal_info_serialization() {
    let info = PrimalInfo {
        name: "beardog".to_string(),
        host: "localhost".to_string(),
        port: 9090,
        capabilities: vec!["security".to_string(), "crypto".to_string()],
        discovered_at: SystemTime::now(),
        discovery_method: "environment".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: PrimalInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "beardog");
    assert_eq!(deserialized.host, "localhost");
    assert_eq!(deserialized.port, 9090);
    assert_eq!(deserialized.capabilities.len(), 2);
}

#[test]
fn test_primal_info_clone_debug() {
    let info = PrimalInfo {
        name: "test-primal".to_string(),
        host: "10.0.0.1".to_string(),
        port: 8080,
        capabilities: vec!["compute".to_string()],
        discovered_at: SystemTime::now(),
        discovery_method: "dns-srv".to_string(),
    };
    let cloned = info.clone();
    assert_eq!(info.name, cloned.name);
    let debug = format!("{:?}", info);
    assert!(debug.contains("test-primal"));
}

// ═══════════════════════════════════════════════════════════════════════
// PrimalSelfKnowledge tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_discover_self() {
    let _guard = ENV_LOCK.lock().await;
    let result = PrimalSelfKnowledge::discover_self();
    assert!(result.is_ok());

    let self_knowledge = result.unwrap();
    let identity = self_knowledge.identity();
    assert!(!identity.name.is_empty());
}

#[tokio::test]
async fn test_discover_self_with_primal_name() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("PRIMAL_NAME", "test-songbird");
    let result = PrimalSelfKnowledge::discover_self();
    songbird_process_env::remove_var("PRIMAL_NAME");

    assert!(result.is_ok());
    let identity = result.unwrap().identity();
    assert_eq!(identity.name, "test-songbird");
}

#[tokio::test]
async fn test_discover_self_with_service_name() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::remove_var("PRIMAL_NAME");
    songbird_process_env::set_var("SERVICE_NAME", "my-service");
    let result = PrimalSelfKnowledge::discover_self();
    songbird_process_env::remove_var("SERVICE_NAME");

    assert!(result.is_ok());
    let identity = result.unwrap().identity();
    assert_eq!(identity.name, "my-service");
}

#[tokio::test]
async fn test_discovered_empty_initially() {
    let _guard = ENV_LOCK.lock().await;
    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let discovered = self_knowledge.discovered().await;
    assert!(discovered.is_empty());
}

#[tokio::test]
async fn test_discover_primal_via_environment() {
    let _guard = ENV_LOCK.lock().await;
    // Set up env vars for capability discovery
    songbird_process_env::set_var("COMPUTE_HOST", "10.0.0.5");
    songbird_process_env::set_var("COMPUTE_PORT", "7070");

    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let result = self_knowledge.discover_primal("compute").await;

    songbird_process_env::remove_var("COMPUTE_HOST");
    songbird_process_env::remove_var("COMPUTE_PORT");

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.host, "10.0.0.5");
    assert_eq!(info.port, 7070);
    assert_eq!(info.discovery_method, "environment");
}

#[tokio::test]
async fn test_discover_primal_caching() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("STORAGE_HOST", "10.0.0.10");
    songbird_process_env::set_var("STORAGE_PORT", "5050");

    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();

    // First discovery
    let result1 = self_knowledge.discover_primal("storage").await;
    assert!(result1.is_ok());

    // Second discovery should use cache
    let result2 = self_knowledge.discover_primal("storage").await;
    assert!(result2.is_ok());

    let discovered = self_knowledge.discovered().await;
    assert!(discovered.contains_key("storage"));

    songbird_process_env::remove_var("STORAGE_HOST");
    songbird_process_env::remove_var("STORAGE_PORT");
}

#[tokio::test]
async fn test_discover_primal_not_found() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::remove_var("NONEXISTENT_HOST");
    songbird_process_env::remove_var("NONEXISTENT_PORT");

    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let result = self_knowledge.discover_primal("nonexistent").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        PrimalError::DiscoveryFailed {
            reason,
        } => {
            assert!(reason.contains("nonexistent"));
        }
        e => panic!("Expected DiscoveryFailed, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_discover_primal_with_primal_prefix() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("PRIMAL_AI_HOST", "ai-server.local");
    songbird_process_env::set_var("PRIMAL_AI_PORT", "6060");

    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let result = self_knowledge.discover_primal("ai").await;

    songbird_process_env::remove_var("PRIMAL_AI_HOST");
    songbird_process_env::remove_var("PRIMAL_AI_PORT");

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.host, "ai-server.local");
    assert_eq!(info.port, 6060);
}

#[tokio::test]
async fn test_introspect_capabilities_with_env() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("ENABLE_SECURITY", "1");

    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let identity = self_knowledge.identity();
    assert!(identity.capabilities.contains(&"security".to_string()));

    songbird_process_env::remove_var("ENABLE_SECURITY");
}

#[tokio::test]
async fn test_introspect_capabilities_with_ai_env() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("ENABLE_AI", "1");

    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let identity = self_knowledge.identity();
    assert!(identity.capabilities.contains(&"ai".to_string()));

    songbird_process_env::remove_var("ENABLE_AI");
}

// ═══════════════════════════════════════════════════════════════════════
// EnvironmentDiscovery tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_environment_discovery_name() {
    let discovery = EnvironmentDiscovery::new();
    assert_eq!(discovery.name(), "environment");
}

#[tokio::test]
async fn test_environment_discovery_success() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("ORCHESTRATION_HOST", "orch.local");
    songbird_process_env::set_var("ORCHESTRATION_PORT", "3000");

    let discovery = EnvironmentDiscovery::new();
    let result = discovery.discover("orchestration").await;

    songbird_process_env::remove_var("ORCHESTRATION_HOST");
    songbird_process_env::remove_var("ORCHESTRATION_PORT");

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.host, "orch.local");
    assert_eq!(info.port, 3000);
    assert_eq!(info.name, "orchestration");
}

#[tokio::test]
async fn test_environment_discovery_missing_host() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::remove_var("MISSING_HOST");
    songbird_process_env::remove_var("PRIMAL_MISSING_HOST");

    let discovery = EnvironmentDiscovery::new();
    let result = discovery.discover("missing").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_environment_discovery_invalid_port() {
    let _guard = ENV_LOCK.lock().await;
    songbird_process_env::set_var("BADPORT_HOST", "localhost");
    songbird_process_env::set_var("BADPORT_PORT", "not_a_number");

    let discovery = EnvironmentDiscovery::new();
    let result = discovery.discover("badport").await;

    songbird_process_env::remove_var("BADPORT_HOST");
    songbird_process_env::remove_var("BADPORT_PORT");

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// DnsSrvDiscovery tests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_dns_srv_discovery_name() {
    let discovery = DnsSrvDiscovery::new();
    assert_eq!(discovery.name(), "dns-srv");
}

#[tokio::test]
async fn test_dns_srv_discovery_fails_gracefully() {
    // DNS SRV lookup is expected to fail for non-existent services
    let discovery = DnsSrvDiscovery::new();
    let result = discovery.discover("nonexistent_capability").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        PrimalError::DiscoveryFailed {
            reason,
        } => {
            assert!(reason.contains("DNS SRV"));
        }
        e => panic!("Expected DiscoveryFailed, got: {:?}", e),
    }
}
