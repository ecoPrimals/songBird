// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;

#[test]
fn default_canonical_gaming_config() {
    let c = CanonicalGamingConfig::default();
    assert!(c.core.enabled);
    assert!(matches!(c.core.mode, GamingMode::Performance));
    assert!(matches!(c.core.default_game_type, GameType::Fps));
    assert!(c.network.optimization.enabled);
    assert!(c.security.settings.enabled);
    assert!(c.performance.settings.low_latency);
    assert!(c.auto.enabled);
    assert!(c.one_touch.enabled);
}

#[test]
fn default_gaming_core_config() {
    let c = GamingCoreConfig::default();
    assert!(c.enabled);
    assert!(matches!(c.mode, GamingMode::Performance));
    assert!(matches!(c.default_game_type, GameType::Fps));
}

#[test]
fn default_gaming_network_config() {
    let c = GamingNetworkConfig::default();
    assert!(c.optimization.enabled);
    assert!(c.protocols.supported.contains(&"udp".to_string()));
    assert_eq!(c.ports.base_port, 6112);
}

#[test]
fn default_protocol_config() {
    let c = ProtocolConfig::default();
    assert_eq!(c.default, "udp");
    assert_eq!(c.supported.len(), 2);
}

#[test]
fn default_gaming_port_config() {
    let c = GamingPortConfig::default();
    assert_eq!(c.port_range, (6112, 6200));
    assert_eq!(c.reserved_ports.len(), 3);
}

#[test]
fn default_gaming_security_config() {
    let c = GamingSecurityConfig::default();
    assert!(c.settings.anti_cheat);
    assert_eq!(c.auth.method, "jwt");
}

#[test]
fn default_gaming_security_settings() {
    let c = GamingSecuritySettings::default();
    assert!(c.enabled && c.encryption);
}

#[test]
fn default_gaming_auth_config() {
    let c = GamingAuthConfig::default();
    assert_eq!(c.session_timeout, 3600);
}

#[test]
fn default_gaming_performance_config() {
    let c = GamingPerformanceConfig::default();
    assert_eq!(c.settings.target_fps, 60);
    assert!(c.optimization.enabled);
}

#[test]
fn default_gaming_performance_settings() {
    let c = GamingPerformanceSettings::default();
    assert_eq!(c.buffer_size, 8192);
}

#[test]
fn default_gaming_optimization_config() {
    let c = GamingOptimizationConfig::default();
    assert!(c.cpu_optimization && c.memory_optimization);
}

#[test]
fn default_gaming_auto_config() {
    let c = GamingAutoConfig::default();
    assert!(c.detection.enabled);
    assert!(c.optimization.buffer_optimization);
}

#[test]
fn default_security_provider_integration_config() {
    let c = SecurityProviderIntegrationConfig::default();
    assert!(!c.enabled);
    assert_eq!(c.endpoint, None);
    assert!(c.auth.enabled);
}

#[test]
fn default_security_provider_monitoring_config() {
    let c = SecurityProviderMonitoringConfig::default();
    assert_eq!(c.metrics_interval, 60);
}

#[test]
fn default_auto_detection_config() {
    let c = AutoDetectionConfig::default();
    assert_eq!(c.timeout_seconds, 10);
}

#[test]
fn default_network_optimization_config() {
    let c = NetworkOptimizationConfig::default();
    assert!(c.connection_pooling);
}

#[test]
fn default_one_touch_config() {
    let c = OneTouchConfig::default();
    assert_eq!(c.default_profile.name, "default");
    assert!(c.templates.is_empty());
}

#[test]
fn default_gaming_profile() {
    let p = GamingProfile::default();
    assert_eq!(p.protocol_preference.len(), 1);
}

#[test]
fn default_encryption_config() {
    let c = EncryptionConfig::default();
    assert_eq!(c.key_size, 256);
}

#[test]
fn default_authentication_config() {
    let c = AuthenticationConfig::default();
    assert_eq!(c.token_lifetime, std::time::Duration::from_secs(3600));
}

#[test]
fn default_privilege_config() {
    let c = PrivilegeConfig::default();
    assert_eq!(c.max_level, 10);
}

#[test]
fn default_benchmark_config() {
    let c = BenchmarkConfig::default();
    assert!(!c.enabled);
    assert_eq!(c.iterations, 10);
}

#[test]
fn default_qos_config() {
    let c = QoSConfig::default();
    assert_eq!(c.priority_levels, 3);
}

#[test]
fn default_protocol_detection_config() {
    let c = ProtocolDetectionConfig::default();
    assert_eq!(c.detection_rules.len(), 0);
    assert_eq!(c.supported_protocols.len(), 2);
}

#[test]
fn default_nat_traversal_config() {
    let c = NatTraversalConfig::default();
    assert!(c.stun_servers[0].enabled);
}
