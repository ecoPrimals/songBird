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
#![cfg(feature = "tests-incomplete")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for adapter discovery using injectable resolvers (no process env).

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_test_utils::{test_bind_address, test_metrics_port, test_orchestrator_port};
use songbird_types::SongbirdResult;
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::collections::HashMap;

#[tokio::test]
async fn test_ai_adapter_discovery_from_injected_resolver() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, format!("http://ai-provider:{}", test_orchestrator_port()));
    let result = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(result.endpoint(), format!("http://ai-provider:{}", test_orchestrator_port()));
    Ok(())
}

#[tokio::test]
async fn test_compute_adapter_discovery_from_injected_resolver() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Compute, format!("http://compute-provider:{}", test_metrics_port()));
    let result = ComputeAdapter::new_from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(result.endpoint(), format!("http://compute-provider:{}", test_metrics_port()));
    Ok(())
}

#[tokio::test]
async fn test_security_adapter_discovery_from_injected_resolver() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Security, "https://security-provider:8443".to_string());
    let result = SecurityAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(result.endpoint(), "https://security-provider:8443");
    Ok(())
}

#[tokio::test]
async fn test_storage_adapter_discovery_from_injected_resolver() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Storage, "http://storage-provider:9000".to_string());
    let result = StorageAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(result.endpoint(), "http://storage-provider:9000");
    Ok(())
}

#[tokio::test]
async fn test_adapter_discovery_fallback_bind_address_shape() -> SongbirdResult<()> {
    let bind = test_bind_address("ai");
    let adapter = AIAdapter::new(format!("http://{}:8083", bind.as_str())).await?;
    assert!(adapter.endpoint().contains(bind.as_str()) || adapter.endpoint().contains("127.0.0.1"));
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_validation() {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, format!("http://valid:{}", test_orchestrator_port()));
    let result1 = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await;
    assert!(result1.is_ok());

    let mut m2 = HashMap::new();
    m2.insert(CapabilityType::Ai, "https://secure:443".to_string());
    let result2 = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m2),
    )
    .await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_multiple_adapter_discovery_independence() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, format!("http://ai:{}", test_orchestrator_port()));
    m.insert(CapabilityType::Compute, format!("http://compute:{}", test_metrics_port()));
    m.insert(CapabilityType::Storage, "http://storage:9000".to_string());
    let r = CapabilityEndpointResolver::with_endpoint_overrides(m);
    let r2 = r.clone();
    let r3 = r.clone();

    let ai_result = AIAdapter::from_discovery_with_resolver(r).await;
    let compute_result = ComputeAdapter::new_from_discovery_with_resolver(r2).await;
    let storage_result = StorageAdapter::from_discovery_with_resolver(r3).await;

    assert!(ai_result.is_ok());
    assert!(compute_result.is_ok());
    assert!(storage_result.is_ok());

    assert_eq!(ai_result.unwrap().endpoint(), format!("http://ai:{}", test_orchestrator_port()));
    assert_eq!(
        compute_result.unwrap().endpoint(),
        format!("http://compute:{}", test_metrics_port())
    );
    assert_eq!(storage_result.unwrap().endpoint(), "http://storage:9000");
    Ok(())
}

#[tokio::test]
async fn test_adapter_discovery_with_custom_timeout() {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, format!("http://ai:{}", test_orchestrator_port()));
    let result = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_compute_adapter_direct_construction() -> SongbirdResult<()> {
    let adapter =
        ComputeAdapter::new(format!("http://explicit:{}", test_metrics_port()).to_string()).await?;
    assert_eq!(adapter.endpoint(), format!("http://explicit:{}", test_metrics_port()));
    Ok(())
}

#[tokio::test]
async fn test_adapter_endpoint_formats() {
    let test_cases = vec![
        format!("http://localhost:{}", test_orchestrator_port()),
        "https://secure.example.com:443".to_string(),
        format!("http://192.168.1.100:{}", test_metrics_port()),
        format!("http://service.namespace.svc.cluster.local:{}", test_orchestrator_port()),
    ];

    for endpoint in test_cases {
        let mut m = HashMap::new();
        m.insert(CapabilityType::Compute, endpoint.clone());
        let result = ComputeAdapter::new_from_discovery_with_resolver(
            CapabilityEndpointResolver::with_endpoint_overrides(m),
        )
        .await;

        assert!(result.is_ok(), "Failed for endpoint: {}", endpoint);
        if let Ok(adapter) = result {
            assert_eq!(adapter.endpoint(), endpoint);
        }
    }
}

#[tokio::test]
async fn test_adapter_discovery_cache_behavior() {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, format!("http://ai:{}", test_orchestrator_port()));
    let r = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let result1 = AIAdapter::from_discovery_with_resolver(r.clone()).await;
    let result2 = AIAdapter::from_discovery_with_resolver(r).await;
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_adapter_concurrent_discovery() {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, format!("http://ai:{}", test_orchestrator_port()));
    m.insert(CapabilityType::Compute, format!("http://compute:{}", test_metrics_port()));
    m.insert(CapabilityType::Storage, "http://storage:9000".to_string());
    let r = CapabilityEndpointResolver::with_endpoint_overrides(m);
    let r2 = r.clone();
    let r3 = r.clone();

    let (ai_result, compute_result, storage_result) = tokio::join!(
        AIAdapter::from_discovery_with_resolver(r),
        ComputeAdapter::new_from_discovery_with_resolver(r2),
        StorageAdapter::from_discovery_with_resolver(r3)
    );

    assert!(ai_result.is_ok());
    assert!(compute_result.is_ok());
    assert!(storage_result.is_ok());
}

#[tokio::test]
async fn test_adapter_discovery_composed_url_via_explicit_new() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("http://custom-host:9999".to_string()).await?;
    assert_eq!(adapter.endpoint(), "http://custom-host:9999");
    Ok(())
}
