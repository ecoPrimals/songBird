// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use super::roundtrip;

#[test]
fn serde_roundtrip_gaming_mode() {
    roundtrip(&GamingMode::Balanced);
}

#[test]
fn serde_roundtrip_game_type_variants() {
    roundtrip(&GameType::Moba);
    roundtrip(&GameType::Custom("x".to_string()));
}

#[test]
fn serde_roundtrip_canonical_gaming_config() {
    roundtrip(&CanonicalGamingConfig::default());
}

#[test]
fn serde_roundtrip_gaming_core_config() {
    roundtrip(&GamingCoreConfig::default());
}

#[test]
fn serde_roundtrip_gaming_network_config() {
    roundtrip(&GamingNetworkConfig::default());
}

#[test]
fn serde_roundtrip_protocol_and_ports() {
    roundtrip(&ProtocolConfig::default());
    roundtrip(&GamingPortConfig::default());
}

#[test]
fn serde_roundtrip_security_stack() {
    roundtrip(&GamingSecurityConfig::default());
    roundtrip(&GamingSecuritySettings::default());
    roundtrip(&GamingAuthConfig::default());
}

#[test]
fn serde_roundtrip_performance_stack() {
    roundtrip(&GamingPerformanceConfig::default());
    roundtrip(&GamingPerformanceSettings::default());
    roundtrip(&GamingOptimizationConfig::default());
}

#[test]
fn serde_roundtrip_auto_and_provider() {
    roundtrip(&GamingAutoConfig::default());
    roundtrip(&SecurityProviderIntegrationConfig::default());
    roundtrip(&SecurityProviderMonitoringConfig::default());
    roundtrip(&AutoDetectionConfig::default());
    roundtrip(&NetworkOptimizationConfig::default());
}

#[test]
fn serde_roundtrip_one_touch_and_profile() {
    roundtrip(&OneTouchConfig::default());
    roundtrip(&GamingProfile::default());
}

#[test]
fn serde_roundtrip_gaming_template() {
    let t = GamingTemplate {
        name: "arena".to_string(),
        ports: vec![6112, 6113],
        protocols: vec![GameProtocolClass::FirstPersonShooter],
    };
    roundtrip(&t);
}

#[test]
fn serde_roundtrip_encryption_authentication_privilege() {
    roundtrip(&EncryptionConfig::default());
    roundtrip(&AuthenticationConfig::default());
    roundtrip(&PrivilegeConfig::default());
}

#[test]
fn serde_roundtrip_performance_mode() {
    roundtrip(&PerformanceMode::PowerSaver);
}

#[test]
fn serde_roundtrip_benchmark_qos_protocol_detection() {
    roundtrip(&BenchmarkConfig::default());
    roundtrip(&QoSConfig::default());
    roundtrip(&ProtocolDetectionConfig::default());
}

#[test]
fn serde_roundtrip_detection_rule() {
    let r = DetectionRule {
        name: "q3".to_string(),
        signature: Some(vec![0xFF, 0x00]),
        protocol_class: GameProtocolClass::Custom("c".to_string()),
    };
    roundtrip(&r);
}

#[test]
fn serde_roundtrip_nat_and_servers() {
    roundtrip(&NatTraversalConfig::default());
    roundtrip(&StunServerConfig {
        address: "stun:3478".to_string(),
        enabled: true,
    });
    roundtrip(&TurnServerConfig {
        address: "turn:3478".to_string(),
        username: Some("u".to_string()),
        password: None,
        enabled: true,
    });
}

#[test]
fn serde_roundtrip_game_protocol_class_variants() {
    roundtrip(&GameProtocolClass::Sports);
    roundtrip(&GameProtocolClass::Custom("mod".to_string()));
}
