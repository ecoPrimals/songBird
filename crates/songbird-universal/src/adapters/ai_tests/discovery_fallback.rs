// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use crate::adapters::discovery_test_sync::lock_discovery_env;
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::SongbirdResult;
use std::collections::HashMap;

// --- AIAdapter protocol detection & discovery (no live services) ---

#[tokio::test]
async fn test_ai_adapter_new_tarpc_localhost_port() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("tarpc://localhost:1234".to_string()).await?;
    assert_eq!(adapter.endpoint(), "tarpc://localhost:1234");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_new_unix_tmp_test_sock() -> SongbirdResult<()> {
    let adapter = AIAdapter::new("unix:///tmp/test.sock".to_string()).await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/test.sock");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_new_tarpc_invalid_hostname_err() {
    let err = AIAdapter::new("tarpc://test:1234".to_string())
        .await
        .expect_err("tarpc hostname must be localhost or IP");
    assert!(
        err.to_string().contains("Invalid hostname") || err.to_string().contains("configuration"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_resolver_injected_tarpc() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, "tarpc://127.0.0.1:9101".to_string());
    let adapter = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(adapter.endpoint(), "tarpc://127.0.0.1:9101");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_resolver_injected_unix() -> SongbirdResult<()> {
    let mut m = HashMap::new();
    m.insert(CapabilityType::Ai, "unix:///tmp/injected-ai.sock".to_string());
    let adapter = AIAdapter::from_discovery_with_resolver(
        CapabilityEndpointResolver::with_endpoint_overrides(m),
    )
    .await?;
    assert_eq!(adapter.endpoint(), "unix:///tmp/injected-ai.sock");
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_songbird_ai_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_AI_ENDPOINT", "http://from-songbird-ai:7788");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from SONGBIRD_AI_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://from-songbird-ai:7788");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_ai_provider_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("AI_PROVIDER_ENDPOINT", "http://from-legacy-ai:7799");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from AI_PROVIDER_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://from-legacy-ai:7799");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_squirrel_endpoint() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SQUIRREL_ENDPOINT", "http://from-squirrel:7700");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from SQUIRREL_ENDPOINT");
    assert_eq!(adapter.endpoint(), "http://from-squirrel:7700");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_host_and_port_env() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_HOST", "http://custom-ai-host");
    songbird_process_env::set_var("SONGBIRD_AI_PORT", "8811");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter from host+port fallback");
    assert_eq!(adapter.endpoint(), "http://custom-ai-host:8811");

    songbird_process_env::reset_overlay();
    Ok(())
}

#[tokio::test]
async fn test_ai_adapter_from_discovery_fallback_prefers_songbird_ai_env() -> SongbirdResult<()> {
    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    songbird_process_env::set_var("SONGBIRD_AI_ENDPOINT", "http://songbird-wins:1111");
    songbird_process_env::set_var("AI_PROVIDER_ENDPOINT", "http://legacy-loses:2222");

    let adapter = AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new())
        .await
        .expect("adapter");
    assert_eq!(adapter.endpoint(), "http://songbird-wins:1111");

    songbird_process_env::reset_overlay();
    Ok(())
}
