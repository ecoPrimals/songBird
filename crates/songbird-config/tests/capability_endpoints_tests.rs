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
    clippy::unnecessary_literal_unwrap
)]

//! Tests for the Capability Endpoints system
//!
//! Covers: CapabilityType parsing, env_var_name, as_str,
//! CapabilityEndpointResolver, caching, and discovery.

use songbird_config::capability_endpoints::*;
use std::sync::Mutex;

/// File-local mutex to serialize tests that modify process-wide env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ============================================================
// CapabilityType FromStr Tests
// ============================================================

#[test]
fn test_capability_type_from_str_security() {
    let variants = ["security", "auth", "authentication", "encryption"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Security, "'{v}' should parse to Security");
    }
}

#[test]
fn test_capability_type_from_str_storage() {
    let variants = ["storage", "database", "persistence", "cache"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Storage, "'{v}' should parse to Storage");
    }
}

#[test]
fn test_capability_type_from_str_compute() {
    let variants = ["compute", "execution", "runtime", "container"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Compute, "'{v}' should parse to Compute");
    }
}

#[test]
fn test_capability_type_from_str_ai() {
    let variants = ["ai", "ml", "inference", "intelligence"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Ai, "'{v}' should parse to Ai");
    }
}

#[test]
fn test_capability_type_from_str_orchestration() {
    let variants = ["orchestration", "coordination", "workflow"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Orchestration, "'{v}' should parse to Orchestration");
    }
}

#[test]
fn test_capability_type_from_str_observability() {
    let variants = ["observability", "logging", "metrics", "tracing"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Observability, "'{v}' should parse to Observability");
    }
}

#[test]
fn test_capability_type_from_str_networking() {
    let variants = ["networking", "mesh", "loadbalancing"];
    for v in &variants {
        let cap: CapabilityType = v.parse().unwrap();
        assert_eq!(cap, CapabilityType::Networking, "'{v}' should parse to Networking");
    }
}

#[test]
fn test_capability_type_from_str_custom() {
    let cap: CapabilityType = "my_custom_capability".parse().unwrap();
    match &cap {
        CapabilityType::Custom(name) => assert_eq!(name, "my_custom_capability"),
        other => panic!("Expected Custom, got {:?}", other),
    }
}

#[test]
fn test_capability_type_from_str_case_insensitive() {
    let cap: CapabilityType = "SECURITY".parse().unwrap();
    assert_eq!(cap, CapabilityType::Security);

    let cap: CapabilityType = "Storage".parse().unwrap();
    assert_eq!(cap, CapabilityType::Storage);

    let cap: CapabilityType = "AI".parse().unwrap();
    assert_eq!(cap, CapabilityType::Ai);
}

// ============================================================
// CapabilityType env_var_name Tests
// ============================================================

#[test]
fn test_env_var_name_known_types() {
    assert_eq!(CapabilityType::Security.env_var_name(), "CAPABILITY_SECURITY_ENDPOINT");
    assert_eq!(CapabilityType::Storage.env_var_name(), "CAPABILITY_STORAGE_ENDPOINT");
    assert_eq!(CapabilityType::Compute.env_var_name(), "CAPABILITY_COMPUTE_ENDPOINT");
    assert_eq!(CapabilityType::Ai.env_var_name(), "CAPABILITY_AI_ENDPOINT");
    assert_eq!(CapabilityType::Orchestration.env_var_name(), "CAPABILITY_ORCHESTRATION_ENDPOINT");
    assert_eq!(CapabilityType::Observability.env_var_name(), "CAPABILITY_OBSERVABILITY_ENDPOINT");
    assert_eq!(CapabilityType::Networking.env_var_name(), "CAPABILITY_NETWORKING_ENDPOINT");
}

#[test]
fn test_env_var_name_custom_type() {
    let cap = CapabilityType::Custom("blockchain".to_string());
    assert_eq!(cap.env_var_name(), "CAPABILITY_BLOCKCHAIN_ENDPOINT");
}

// ============================================================
// CapabilityType as_str Tests
// ============================================================

#[test]
fn test_as_str_known_types() {
    assert_eq!(CapabilityType::Security.as_str(), "security");
    assert_eq!(CapabilityType::Storage.as_str(), "storage");
    assert_eq!(CapabilityType::Compute.as_str(), "compute");
    assert_eq!(CapabilityType::Ai.as_str(), "ai");
    assert_eq!(CapabilityType::Orchestration.as_str(), "orchestration");
    assert_eq!(CapabilityType::Observability.as_str(), "observability");
    assert_eq!(CapabilityType::Networking.as_str(), "networking");
}

#[test]
fn test_as_str_custom() {
    let cap = CapabilityType::Custom("my_cap".to_string());
    assert_eq!(cap.as_str(), "my_cap");
}

// ============================================================
// Serialization Tests
// ============================================================

#[test]
fn test_capability_type_serialization() {
    let cap = CapabilityType::Security;
    let json = serde_json::to_string(&cap).unwrap();
    let deserialized: CapabilityType = serde_json::from_str(&json).unwrap();
    assert_eq!(cap, deserialized);
}

#[test]
fn test_capability_type_custom_serialization() {
    let cap = CapabilityType::Custom("gaming".to_string());
    let json = serde_json::to_string(&cap).unwrap();
    let deserialized: CapabilityType = serde_json::from_str(&json).unwrap();
    assert_eq!(cap, deserialized);
}

#[test]
fn test_capability_endpoint_serialization() {
    let ep = CapabilityEndpoint {
        capability: CapabilityType::Compute,
        endpoint: "http://localhost:8080".to_string(),
        provider_id: Some("toadstool".to_string()),
        discovery_method: DiscoveryMethod::Environment,
        confidence: 0.95,
        discovered_at: std::time::SystemTime::now(),
    };
    let json = serde_json::to_string(&ep).unwrap();
    let deserialized: CapabilityEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(ep.capability, deserialized.capability);
    assert_eq!(ep.endpoint, deserialized.endpoint);
    assert_eq!(ep.provider_id, deserialized.provider_id);
}

#[test]
fn test_discovery_method_serialization() {
    let methods = [
        DiscoveryMethod::Environment,
        DiscoveryMethod::ServiceRegistry,
        DiscoveryMethod::ContainerMetadata,
        DiscoveryMethod::Dns,
        DiscoveryMethod::NetworkScan,
        DiscoveryMethod::ConfigFile,
    ];
    for method in &methods {
        let json = serde_json::to_string(method).unwrap();
        let deserialized: DiscoveryMethod = serde_json::from_str(&json).unwrap();
        let debug1 = format!("{:?}", method);
        let debug2 = format!("{:?}", deserialized);
        assert_eq!(debug1, debug2);
    }
}

// ============================================================
// CapabilityEndpointResolver Tests
// ============================================================

#[tokio::test]
async fn test_resolver_creation() {
    let _guard = ENV_LOCK.lock().unwrap();
    let resolver = CapabilityEndpointResolver::new();
    let debug = format!("{:?}", resolver);
    assert!(debug.contains("CapabilityEndpointResolver"));
}

#[tokio::test]
async fn test_resolver_discovers_from_env() {
    let _guard = ENV_LOCK.lock().unwrap();
    songbird_process_env::set_var("CAPABILITY_COMPUTE_ENDPOINT", "http://compute:8080");

    let resolver = CapabilityEndpointResolver::new();
    let result = resolver.get_endpoint(CapabilityType::Compute).await;
    assert!(result.is_ok(), "Should discover endpoint from env");
    assert_eq!(result.unwrap(), "http://compute:8080");

    songbird_process_env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
}

#[tokio::test]
async fn test_resolver_caches_result() {
    let _guard = ENV_LOCK.lock().unwrap();
    songbird_process_env::set_var("CAPABILITY_AI_ENDPOINT", "http://ai:9090");

    let resolver = CapabilityEndpointResolver::new();

    // First call should discover
    let result1 = resolver.get_endpoint(CapabilityType::Ai).await.unwrap();
    assert_eq!(result1, "http://ai:9090");

    // Second call should use cache (remove env var to prove it)
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    let result2 = resolver.get_endpoint(CapabilityType::Ai).await.unwrap();
    assert_eq!(result2, "http://ai:9090"); // Still cached

    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
}

#[tokio::test]
async fn test_resolver_custom_capability_from_env() {
    let _guard = ENV_LOCK.lock().unwrap();
    songbird_process_env::set_var("CAPABILITY_BLOCKCHAIN_ENDPOINT", "http://chain:3000");

    let resolver = CapabilityEndpointResolver::new();
    let result = resolver.get_endpoint(CapabilityType::Custom("blockchain".to_string())).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "http://chain:3000");

    songbird_process_env::remove_var("CAPABILITY_BLOCKCHAIN_ENDPOINT");
}

// ============================================================
// Hash / Eq Tests
// ============================================================

#[test]
fn test_capability_type_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(CapabilityType::Security);
    set.insert(CapabilityType::Storage);
    set.insert(CapabilityType::Security); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&CapabilityType::Security));
    assert!(set.contains(&CapabilityType::Storage));
}

#[test]
fn test_capability_type_equality() {
    assert_eq!(CapabilityType::Security, CapabilityType::Security);
    assert_ne!(CapabilityType::Security, CapabilityType::Storage);
    assert_eq!(CapabilityType::Custom("x".to_string()), CapabilityType::Custom("x".to_string()));
    assert_ne!(CapabilityType::Custom("x".to_string()), CapabilityType::Custom("y".to_string()));
}
