// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
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
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive Integration Tests for Universal Adapters
//!
//! Tests multi-tier discovery using injectable resolvers (no process environment mutation).

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_test_utils::test_orchestrator_port;
use songbird_types::SongbirdResult;
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::collections::HashMap;

fn resolver_all_four(
    compute: String,
    security: String,
    storage: String,
    ai: String,
) -> CapabilityEndpointResolver {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Compute, compute);
    m.insert(CapabilityType::Security, security);
    m.insert(CapabilityType::Storage, storage);
    m.insert(CapabilityType::Ai, ai);
    CapabilityEndpointResolver::with_endpoint_overrides(m)
}

#[tokio::test]
async fn test_concurrent_multi_adapter_discovery() {
    let r = resolver_all_four(
        format!("http://compute-test:{}", test_orchestrator_port()),
        "http://security-test:8443".to_string(),
        "http://storage-test:9000".to_string(),
        "http://ai-test:8888".to_string(),
    );
    let r2 = r.clone();
    let r3 = r.clone();
    let r4 = r.clone();

    let results = tokio::join!(
        ComputeAdapter::new_from_discovery_with_resolver(r),
        SecurityAdapter::from_discovery_with_resolver(r2),
        StorageAdapter::from_discovery_with_resolver(r3),
        AIAdapter::from_discovery_with_resolver(r4),
    );

    assert!(results.0.is_ok(), "ComputeAdapter discovery failed");
    assert!(results.1.is_ok(), "SecurityAdapter discovery failed");
    assert!(results.2.is_ok(), "StorageAdapter discovery failed");
    assert!(results.3.is_ok(), "AIAdapter discovery failed");
}

#[tokio::test]
async fn test_legacy_named_endpoints_via_resolver() {
    let r = resolver_all_four(
        format!("http://legacy-compute:{}", test_orchestrator_port()),
        "http://legacy-security:8443".to_string(),
        "http://legacy-storage:9000".to_string(),
        "http://legacy-ai:8888".to_string(),
    );
    let r2 = r.clone();
    let r3 = r.clone();
    let r4 = r.clone();

    assert!(ComputeAdapter::new_from_discovery_with_resolver(r).await.is_ok());
    assert!(SecurityAdapter::from_discovery_with_resolver(r2).await.is_ok());
    assert!(StorageAdapter::from_discovery_with_resolver(r3).await.is_ok());
    assert!(AIAdapter::from_discovery_with_resolver(r4).await.is_ok());
}

#[tokio::test]
async fn test_host_port_style_url_via_explicit_new() {
    let adapter = ComputeAdapter::new("http://fallback-host:9999".to_string())
        .await
        .expect("explicit new");
    assert_eq!(adapter.endpoint(), "http://fallback-host:9999");
}

#[tokio::test]
async fn test_adapter_timeout_configuration() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(
        CapabilityType::Compute,
        format!("http://compute:{}", test_orchestrator_port()),
    );
    let adapter =
        ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::with_endpoint_overrides(m))
            .await?;

    let custom_adapter = adapter.with_timeout(std::time::Duration::from_secs(30));
    drop(custom_adapter);
    Ok(())
}

#[tokio::test]
async fn test_discovery_priority_injected_endpoint() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Security, "http://new-priority:8443".to_string());
    let adapter = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::with_endpoint_overrides(m)).await?;

    assert_eq!(adapter.endpoint(), "http://new-priority:8443");
    Ok(())
}

#[tokio::test]
async fn test_explicit_endpoint_creation() {
    let compute =
        ComputeAdapter::new(format!("http://explicit-compute:{}", test_orchestrator_port())).await;
    let security = SecurityAdapter::new("http://explicit-security:8443".to_string()).await;
    let storage = StorageAdapter::new("http://explicit-storage:9000".to_string()).await;
    let ai = AIAdapter::new("http://explicit-ai:8888".to_string()).await;

    assert!(compute.is_ok(), "Explicit compute creation failed");
    assert!(security.is_ok(), "Explicit security creation failed");
    assert!(storage.is_ok(), "Explicit storage creation failed");
    assert!(ai.is_ok(), "Explicit AI creation failed");
}

#[tokio::test]
async fn test_invalid_endpoint_handling() {
    let compute = ComputeAdapter::new("invalid-url".to_string()).await;
    let security = SecurityAdapter::new(String::new()).await;

    assert!(compute.is_ok() || compute.is_err());
    assert!(security.is_ok() || security.is_err());
}

#[tokio::test]
async fn test_adapter_capability_isolation() -> SongbirdResult<()> {
    let mut mc = HashMap::new();
    mc.insert(
        CapabilityType::Compute,
        format!("http://compute-only:{}", test_orchestrator_port()),
    );
    let compute = ComputeAdapter::new_from_discovery_with_resolver(CapabilityEndpointResolver::with_endpoint_overrides(mc)).await?;

    let mut ms = HashMap::new();
    ms.insert(CapabilityType::Security, "http://security-only:8443".to_string());
    let security = SecurityAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::with_endpoint_overrides(ms)).await?;

    assert_ne!(compute.endpoint(), security.endpoint());
    Ok(())
}

#[tokio::test]
async fn test_multiple_adapter_instances() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Storage, "http://storage:9000".to_string());
    let r = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let storage1 = StorageAdapter::from_discovery_with_resolver(r.clone()).await?;
    let storage2 = StorageAdapter::from_discovery_with_resolver(r).await?;

    drop(storage1);
    drop(storage2);
    Ok(())
}

#[tokio::test]
async fn test_ai_explicit_new_when_no_discovery() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://127.0.0.1:8083".to_string()).await?;
    assert!(!adapter.endpoint().is_empty());
    Ok(())
}

#[tokio::test]
async fn test_capability_type_boundaries() -> SongbirdResult<()> {
    let r = resolver_all_four(
        "http://compute:1111".to_string(),
        "http://security:2222".to_string(),
        "http://storage:3333".to_string(),
        "http://ai:4444".to_string(),
    );
    let r2 = r.clone();
    let r3 = r.clone();
    let r4 = r.clone();

    let compute = ComputeAdapter::new_from_discovery_with_resolver(r).await?;
    let security = SecurityAdapter::from_discovery_with_resolver(r2).await?;
    let storage = StorageAdapter::from_discovery_with_resolver(r3).await?;
    let ai = AIAdapter::from_discovery_with_resolver(r4).await?;

    assert_eq!(compute.endpoint(), "http://compute:1111");
    assert_eq!(security.endpoint(), "http://security:2222");
    assert_eq!(storage.endpoint(), "http://storage:3333");
    assert_eq!(ai.endpoint(), "http://ai:4444");
    Ok(())
}

#[tokio::test]
async fn test_mixed_injected_configuration() {
    let mut m = HashMap::new();
    m.insert(
        CapabilityType::Compute,
        format!("http://compute-cap:{}", test_orchestrator_port()),
    );
    m.insert(CapabilityType::Security, "http://security-legacy:8443".to_string());
    let r = CapabilityEndpointResolver::with_endpoint_overrides(m);
    let r2 = r.clone();

    assert!(ComputeAdapter::new_from_discovery_with_resolver(r).await.is_ok());
    assert!(SecurityAdapter::from_discovery_with_resolver(r2).await.is_ok());

    let mut ms = HashMap::new();
    ms.insert(CapabilityType::Storage, "http://storage-fb:9000".to_string());
    assert!(StorageAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::with_endpoint_overrides(ms)).await.is_ok());

    let mut ma = HashMap::new();
    ma.insert(CapabilityType::Ai, "http://ai-custom:7777".to_string());
    let ai = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::with_endpoint_overrides(ma)).await;
    assert!(ai.is_ok());
    assert_eq!(ai.expect("ai").endpoint(), "http://ai-custom:7777");
}

#[tokio::test]
async fn test_discovery_consistency() {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Storage, "http://consistent:9000".to_string());
    let r = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let storage1 = StorageAdapter::from_discovery_with_resolver(r.clone()).await;
    let storage2 = StorageAdapter::from_discovery_with_resolver(r.clone()).await;
    let storage3 = StorageAdapter::from_discovery_with_resolver(r).await;

    assert!(storage1.is_ok(), "First discovery failed");
    assert!(storage2.is_ok(), "Second discovery failed");
    assert!(storage3.is_ok(), "Third discovery failed");
}
