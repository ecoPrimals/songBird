// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Coverage tests for songbird-types config modules
//!
//! Tests gaming, adapters, communication, and other config types.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use songbird_types::config::adapters::*;
use songbird_types::config::communication::*;
use songbird_types::config::gaming::*;
use std::time::Duration;

// ==================== GAMING CONFIG ====================

#[test]
fn test_gaming_mode_serialization() {
    let modes = vec![GamingMode::Performance, GamingMode::Balanced, GamingMode::PowerSaver];
    for mode in &modes {
        let json = serde_json::to_string(mode).unwrap();
        let de: GamingMode = serde_json::from_str(&json).unwrap();
        let debug_orig = format!("{mode:?}");
        let debug_de = format!("{de:?}");
        assert_eq!(debug_orig, debug_de);
    }
}

#[test]
fn test_game_type_serialization() {
    let types = vec![
        GameType::Fps,
        GameType::Rts,
        GameType::Moba,
        GameType::Rpg,
        GameType::Custom("battle-royale".to_string()),
    ];
    for gt in &types {
        let json = serde_json::to_string(gt).unwrap();
        let de: GameType = serde_json::from_str(&json).unwrap();
        let debug_orig = format!("{gt:?}");
        let debug_de = format!("{de:?}");
        assert_eq!(debug_orig, debug_de);
    }
}

#[test]
fn test_canonical_gaming_config_default() {
    let config = CanonicalGamingConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalGamingConfig"));
}

#[test]
fn test_canonical_gaming_config_serialization() {
    let config = CanonicalGamingConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let de: CanonicalGamingConfig = serde_json::from_str(&json).unwrap();
    let debug = format!("{de:?}");
    assert!(debug.contains("CanonicalGamingConfig"));
}

#[test]
fn test_gaming_core_config_default() {
    let config = GamingCoreConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GamingCoreConfig"));
}

#[test]
fn test_gaming_network_config_default() {
    let config = GamingNetworkConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GamingNetworkConfig"));
}

#[test]
fn test_gaming_security_config_default() {
    let config = GamingSecurityConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GamingSecurityConfig"));
}

#[test]
fn test_gaming_performance_config_default() {
    let config = GamingPerformanceConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GamingPerformanceConfig"));
}

#[test]
fn test_gaming_auto_config_default() {
    let config = GamingAutoConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GamingAutoConfig"));
}

// ==================== ADAPTER CONFIG ====================

#[test]
fn test_adapter_config_default() {
    let config = CanonicalUniversalAdapterConfig::default();
    assert!(config.auto_discovery);
    assert!(config.primal_instances.is_empty());
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalUniversalAdapterConfig"));
}

#[test]
fn test_adapter_config_serialization() {
    let config = CanonicalUniversalAdapterConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let de: CanonicalUniversalAdapterConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(de.auto_discovery, config.auto_discovery);
}

#[test]
fn test_multi_instance_config_default() {
    let config = CanonicalMultiInstanceConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalMultiInstanceConfig"));
}

#[test]
fn test_lifecycle_config_default() {
    let config = CanonicalInstanceLifecycleConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalInstanceLifecycleConfig"));
}

#[test]
fn test_port_management_config_default() {
    let config = CanonicalPortManagementConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalPortManagementConfig"));
}

#[test]
fn test_adapter_security_config_default() {
    let config = CanonicalAdapterSecurityConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalAdapterSecurityConfig"));
}

#[test]
fn test_adapter_timeout_config_default() {
    let config = CanonicalTimeoutConfig::default();
    assert!(config.connection_timeout > Duration::ZERO);
    assert!(config.default_request_timeout > Duration::ZERO);
}

#[test]
fn test_adapter_monitoring_config_default() {
    let config = CanonicalAdapterMonitoringConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalAdapterMonitoringConfig"));
}

// ==================== COMMUNICATION CONFIG ====================

#[test]
fn test_communication_config_default() {
    let config = CanonicalCommunicationConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("CanonicalCommunicationConfig"));
}

#[test]
fn test_communication_config_serialization() {
    let config = CanonicalCommunicationConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let de: CanonicalCommunicationConfig = serde_json::from_str(&json).unwrap();
    let debug = format!("{de:?}");
    assert!(debug.contains("CanonicalCommunicationConfig"));
}

#[test]
fn test_http_client_config_default() {
    let config = HttpClientConfig::default();
    assert!(config.timeout > Duration::ZERO);
    assert!(config.max_connections_per_host > 0);
    assert!(!config.user_agent.is_empty());
}

#[test]
fn test_websocket_config_default() {
    let config = WebSocketConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("WebSocketConfig"));
}

#[test]
fn test_grpc_config_default() {
    let config = GrpcConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("GrpcConfig"));
}

#[test]
fn test_jsonrpc_config_default() {
    let config = JsonRpcConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("JsonRpcConfig"));
}

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("PerformanceConfig"));
}

#[test]
fn test_circuit_breaker_config_default() {
    let config = CircuitBreakerConfig::default();
    assert!(config.failure_threshold > 0);
    assert!(config.timeout > Duration::ZERO);
}

// ==================== SERIALIZATION ROUNDTRIPS ====================

#[test]
fn test_all_gaming_subconfigs_serialize() {
    let config = CanonicalGamingConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    assert!(json.contains("core"));
    assert!(json.contains("network"));
    assert!(json.contains("security"));
    assert!(json.contains("performance"));
}

#[test]
fn test_all_adapter_subconfigs_serialize() {
    let config = CanonicalUniversalAdapterConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    assert!(json.contains("auto_discovery"));
    assert!(json.contains("multi_instance"));
    assert!(json.contains("lifecycle"));
    assert!(json.contains("port_management"));
}

#[test]
fn test_all_communication_subconfigs_serialize() {
    let config = CanonicalCommunicationConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    assert!(json.contains("http"));
    assert!(json.contains("websocket"));
    assert!(json.contains("grpc"));
    assert!(json.contains("jsonrpc"));
}
