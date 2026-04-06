// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! E2E Test: Runtime Capability-Based Discovery
//!
//! Tests the `RuntimeDiscoveryEngine` with zero hardcoding.
//! Every test uses unique per-test env keys for full concurrency.

use songbird_config::runtime_discovery::{RuntimeDiscoveryEngine, DiscoveryMethod};
use songbird_types::SongbirdResult;
use std::time::Duration;

#[tokio::test]
async fn test_discover_compute_via_environment() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ERTCOMP_ENDPOINT", "http://10.0.1.50:8001");

    let engine = RuntimeDiscoveryEngine::new();
    let service = engine.discover_by_capability("e2ertcomp").await?;

    assert_eq!(service.capability, "e2ertcomp");
    assert_eq!(service.endpoint, "http://10.0.1.50:8001");
    assert_eq!(service.discovered_via, DiscoveryMethod::Environment);
    assert_eq!(service.health_score, 1.0);

    songbird_process_env::remove_var("E2ERTCOMP_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_discover_multiple_capabilities() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2EMULTICOMP_ENDPOINT", "http://10.0.1.50:8001");
    songbird_process_env::set_var("E2EMULTIAI_ENDPOINT", "http://10.0.1.51:8002");
    songbird_process_env::set_var("E2EMULTISTO_ENDPOINT", "http://10.0.1.52:8003");
    songbird_process_env::set_var("E2EMULTISEC_ENDPOINT", "http://10.0.1.53:8004");

    let engine = RuntimeDiscoveryEngine::new();

    let compute = engine.discover_by_capability("e2emulticomp").await?;
    let ai = engine.discover_by_capability("e2emultiai").await?;
    let storage = engine.discover_by_capability("e2emultisto").await?;
    let security = engine.discover_by_capability("e2emultisec").await?;

    assert_eq!(compute.endpoint, "http://10.0.1.50:8001");
    assert_eq!(ai.endpoint, "http://10.0.1.51:8002");
    assert_eq!(storage.endpoint, "http://10.0.1.52:8003");
    assert_eq!(security.endpoint, "http://10.0.1.53:8004");

    songbird_process_env::remove_var("E2EMULTICOMP_ENDPOINT");
    songbird_process_env::remove_var("E2EMULTIAI_ENDPOINT");
    songbird_process_env::remove_var("E2EMULTISTO_ENDPOINT");
    songbird_process_env::remove_var("E2EMULTISEC_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_discovery_caching() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ECACHE_ENDPOINT", "http://10.0.1.100:9000");

    let engine = RuntimeDiscoveryEngine::new();
    let service1 = engine.discover_by_capability("e2ecache").await?;
    let service2 = engine.discover_by_capability("e2ecache").await?;

    assert_eq!(service1.endpoint, service2.endpoint);
    assert_eq!(service1.capability, service2.capability);

    songbird_process_env::remove_var("E2ECACHE_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_discovery_not_found() {
    songbird_process_env::remove_var("E2ENOEXIST_ENDPOINT");

    let engine = RuntimeDiscoveryEngine::new();
    let result = engine.discover_by_capability("e2enoexist").await;

    assert!(result.is_err());
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("e2enoexist") || error_msg.contains("not found"));
}

#[tokio::test]
async fn test_convenience_functions_via_engine() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ECONVCOMP_ENDPOINT", "http://10.0.1.50:8001");
    songbird_process_env::set_var("E2ECONVAI_ENDPOINT", "http://10.0.1.51:8002");
    songbird_process_env::set_var("E2ECONVSTO_ENDPOINT", "http://10.0.1.52:8003");
    songbird_process_env::set_var("E2ECONVSEC_ENDPOINT", "http://10.0.1.53:8004");

    let engine = RuntimeDiscoveryEngine::new();
    let compute = engine.discover_by_capability("e2econvcomp").await?;
    let ai = engine.discover_by_capability("e2econvai").await?;
    let storage = engine.discover_by_capability("e2econvsto").await?;
    let security = engine.discover_by_capability("e2econvsec").await?;

    assert_eq!(compute.endpoint, "http://10.0.1.50:8001");
    assert_eq!(ai.endpoint, "http://10.0.1.51:8002");
    assert_eq!(storage.endpoint, "http://10.0.1.52:8003");
    assert_eq!(security.endpoint, "http://10.0.1.53:8004");

    songbird_process_env::remove_var("E2ECONVCOMP_ENDPOINT");
    songbird_process_env::remove_var("E2ECONVAI_ENDPOINT");
    songbird_process_env::remove_var("E2ECONVSTO_ENDPOINT");
    songbird_process_env::remove_var("E2ECONVSEC_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_discovery_with_timeout() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2EQUICK_ENDPOINT", "http://10.0.1.200:9999");

    let engine = RuntimeDiscoveryEngine::with_capabilities(vec!["e2equick".to_string()]);
    let service = tokio::time::timeout(
        Duration::from_secs(1),
        engine.discover_by_capability("e2equick"),
    )
    .await??;

    assert_eq!(service.endpoint, "http://10.0.1.200:9999");

    songbird_process_env::remove_var("E2EQUICK_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_no_hardcoded_fallbacks() {
    songbird_process_env::remove_var("E2ENOFALLBACK_ENDPOINT");
    songbird_process_env::remove_var("DEFAULT_E2ENOFALLBACK_ENDPOINT");
    songbird_process_env::remove_var("FALLBACK_ENDPOINT");

    let engine = RuntimeDiscoveryEngine::new();
    let result = engine.discover_by_capability("e2enofallback").await;

    assert!(
        result.is_err(),
        "Discovery must fail without configuration — no hardcoded fallbacks allowed"
    );
}

#[tokio::test]
async fn test_case_insensitive_env_vars() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ECASE_ENDPOINT", "http://10.0.1.123:7777");

    let engine = RuntimeDiscoveryEngine::new();
    let service = engine.discover_by_capability("e2ecase").await?;

    assert_eq!(service.endpoint, "http://10.0.1.123:7777");

    songbird_process_env::remove_var("E2ECASE_ENDPOINT");
    Ok(())
}
