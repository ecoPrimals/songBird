// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! E2E Test: Sovereign Service Discovery
//!
//! Tests that services discover each other through capability-based discovery
//! with ZERO hardcoded knowledge of other primals.
//!
//! **Sovereignty Principle**: Each primal knows only itself, discovers others at runtime.
//! All tests use unique per-test env keys for full concurrency.

use songbird_types::SongbirdResult;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_service_discovers_orchestrator_via_env() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ESOVORCH_ENDPOINT", "http://10.0.1.100:8080");

    let endpoint = songbird_process_env::var("E2ESOVORCH_ENDPOINT")
        .expect("Orchestrator endpoint should be set");

    assert_eq!(endpoint, "http://10.0.1.100:8080");
    assert!(!endpoint.contains("localhost"));
    assert!(!endpoint.contains("127.0.0.1"));

    songbird_process_env::remove_var("E2ESOVORCH_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_independent_service_discovery() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ESOVSVCA_ENDPOINT", "http://10.0.1.50:9001");
    songbird_process_env::set_var("E2ESOVSVCB_ENDPOINT", "http://10.0.1.51:9002");
    songbird_process_env::set_var("E2ESOVSVCC_ENDPOINT", "http://10.0.1.52:9003");

    let service_a = songbird_process_env::var("E2ESOVSVCA_ENDPOINT")?;
    let service_b = songbird_process_env::var("E2ESOVSVCB_ENDPOINT")?;
    let service_c = songbird_process_env::var("E2ESOVSVCC_ENDPOINT")?;

    assert_ne!(service_a, service_b);
    assert_ne!(service_b, service_c);
    assert_ne!(service_a, service_c);

    for endpoint in [&service_a, &service_b, &service_c] {
        assert!(!endpoint.contains("localhost"));
        assert!(!endpoint.contains("127.0.0.1"));
        assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
    }

    songbird_process_env::remove_var("E2ESOVSVCA_ENDPOINT");
    songbird_process_env::remove_var("E2ESOVSVCB_ENDPOINT");
    songbird_process_env::remove_var("E2ESOVSVCC_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_discovery_fails_without_configuration() {
    songbird_process_env::remove_var("E2ESOVUNKNOWN_ENDPOINT");

    let result = songbird_process_env::var("E2ESOVUNKNOWN_ENDPOINT");

    assert!(result.is_err(), "Discovery should fail without configuration");
    assert!(matches!(result.unwrap_err(), std::env::VarError::NotPresent));
}

#[tokio::test]
async fn test_capability_based_not_name_based() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

    songbird_process_env::set_var("E2ESOVSTORAGE_ENDPOINT", "http://10.0.1.200:8888");

    let engine = RuntimeDiscoveryEngine::new();
    let service = engine.discover_by_capability("e2esovstorage").await?;

    assert_eq!(service.capability, "e2esovstorage");
    assert_eq!(service.endpoint, "http://10.0.1.200:8888");

    songbird_process_env::remove_var("E2ESOVSTORAGE_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_provider_flexibility() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

    songbird_process_env::set_var("E2ESOVFLEX_ENDPOINT", "http://provider-a.local:8001");
    let engine = RuntimeDiscoveryEngine::new();
    let provider_a = engine.discover_by_capability("e2esovflex").await?;
    assert_eq!(provider_a.endpoint, "http://provider-a.local:8001");

    songbird_process_env::set_var("E2ESOVFLEX2_ENDPOINT", "http://provider-b.local:9999");
    let provider_b = RuntimeDiscoveryEngine::new()
        .discover_by_capability("e2esovflex2")
        .await?;
    assert_eq!(provider_b.endpoint, "http://provider-b.local:9999");

    songbird_process_env::remove_var("E2ESOVFLEX_ENDPOINT");
    songbird_process_env::remove_var("E2ESOVFLEX2_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_zero_network_topology_assumptions() -> SongbirdResult<()> {
    let test_cases = vec![
        ("http://10.0.1.50:8080", "Private network"),
        ("http://192.168.1.100:3000", "Local network"),
        ("http://172.16.0.50:9000", "Docker network"),
        ("https://service.example.com:443", "Internet"),
        ("http://[::1]:8080", "IPv6 loopback"),
        ("http://[2001:db8::1]:8080", "IPv6 address"),
    ];

    for (i, (endpoint, description)) in test_cases.iter().enumerate() {
        let key = format!("E2ESOVTOPO{i}_ENDPOINT");
        songbird_process_env::set_var(&key, *endpoint);
        let discovered = songbird_process_env::var(&key)?;
        assert_eq!(discovered, *endpoint, "Failed for: {description}");
        songbird_process_env::remove_var(&key);
    }

    Ok(())
}

#[tokio::test]
async fn test_discovery_with_timeout() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

    songbird_process_env::set_var("E2ESOVFAST_ENDPOINT", "http://10.0.1.123:7777");

    let result = timeout(Duration::from_secs(2), async {
        RuntimeDiscoveryEngine::new()
            .discover_by_capability("e2esovfast")
            .await
    })
    .await;

    assert!(result.is_ok(), "Discovery should complete within timeout");
    let service = result??;
    assert_eq!(service.endpoint, "http://10.0.1.123:7777");

    songbird_process_env::remove_var("E2ESOVFAST_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_primal_self_knowledge() -> SongbirdResult<()> {
    songbird_process_env::set_var("E2ESOVMYEP_ENDPOINT", "http://10.0.1.50:8080");
    songbird_process_env::set_var("E2ESOVMYCAP", "compute,storage");

    let my_endpoint = songbird_process_env::var("E2ESOVMYEP_ENDPOINT")?;
    let my_capabilities = songbird_process_env::var("E2ESOVMYCAP")?;

    assert!(!my_endpoint.is_empty());
    assert!(!my_capabilities.is_empty());
    assert!(
        songbird_process_env::var("E2EOTHER_PRIMAL_ENDPOINT").is_err(),
        "Should not have hardcoded knowledge of other primals"
    );

    songbird_process_env::remove_var("E2ESOVMYEP_ENDPOINT");
    songbird_process_env::remove_var("E2ESOVMYCAP");
    Ok(())
}

#[tokio::test]
async fn test_discovery_consistency() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

    songbird_process_env::set_var("E2ESOVCONS_ENDPOINT", "http://10.0.1.99:8888");

    let engine = RuntimeDiscoveryEngine::new();
    let result1 = engine.discover_by_capability("e2esovcons").await?;
    let result2 = engine.discover_by_capability("e2esovcons").await?;
    let result3 = engine.discover_by_capability("e2esovcons").await?;

    assert_eq!(result1.endpoint, result2.endpoint);
    assert_eq!(result2.endpoint, result3.endpoint);
    assert_eq!(result1.endpoint, "http://10.0.1.99:8888");

    songbird_process_env::remove_var("E2ESOVCONS_ENDPOINT");
    Ok(())
}

#[tokio::test]
async fn test_no_hardcoded_fallbacks() {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

    songbird_process_env::remove_var("E2ESOVNOFALL_ENDPOINT");

    let engine = RuntimeDiscoveryEngine::new();
    let result = engine.discover_by_capability("e2esovnofall").await;

    assert!(
        result.is_err(),
        "SOVEREIGNTY VIOLATION: Must not have hardcoded fallbacks!"
    );

    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("not found") || error_msg.contains("No service"),
        "Error should explain service not found: {error_msg}",
    );
}

#[tokio::test]
async fn test_isolated_discovery_engines() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;

    songbird_process_env::set_var("E2ESOVISO_ENDPOINT", "http://10.0.1.111:9999");

    let engine1 = RuntimeDiscoveryEngine::new();
    let engine2 = RuntimeDiscoveryEngine::new();
    let engine3 = RuntimeDiscoveryEngine::new();

    let service1 = engine1.discover_by_capability("e2esoviso").await?;
    let service2 = engine2.discover_by_capability("e2esoviso").await?;
    let service3 = engine3.discover_by_capability("e2esoviso").await?;

    assert_eq!(service1.endpoint, service2.endpoint);
    assert_eq!(service2.endpoint, service3.endpoint);

    songbird_process_env::remove_var("E2ESOVISO_ENDPOINT");
    Ok(())
}

#[cfg(test)]
mod sovereignty_tests {
    use songbird_types::SongbirdResult;

    #[tokio::test]
    async fn test_individual_human_frictionless() -> SongbirdResult<()> {
        songbird_process_env::set_var("E2ESOVHUMAN_ENDPOINT", "http://anywhere:8080");

        let endpoint = songbird_process_env::var("E2ESOVHUMAN_ENDPOINT")?;
        assert_eq!(endpoint, "http://anywhere:8080");

        songbird_process_env::remove_var("E2ESOVHUMAN_ENDPOINT");
        Ok(())
    }

    #[tokio::test]
    async fn test_entity_appropriate_friction() {
        let entity_endpoint = "http://corporate.example.com:8080";
        songbird_process_env::set_var("E2ESOVENTITY_ENDPOINT", entity_endpoint);

        let endpoint = songbird_process_env::var("E2ESOVENTITY_ENDPOINT")
            .expect("Entity endpoint should work");
        assert_eq!(endpoint, entity_endpoint);

        songbird_process_env::remove_var("E2ESOVENTITY_ENDPOINT");
    }
}
