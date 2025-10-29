//! Comprehensive Gaming Configuration Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//!
//! Tests for `songbird_types::config::gaming` module.

use songbird_types::config::gaming::*;
use std::time::Duration;

// ============================================================================
// GAMING MODE TESTS
// ============================================================================

#[test]
fn test_gaming_modes() {
    let perf = GamingMode::Performance;
    let balanced = GamingMode::Balanced;
    let power = GamingMode::PowerSaver;

    assert!(matches!(perf, GamingMode::Performance));
    assert!(matches!(balanced, GamingMode::Balanced));
    assert!(matches!(power, GamingMode::PowerSaver));
}

// ============================================================================
// GAME TYPE TESTS
// ============================================================================

#[test]
fn test_game_types() {
    let fps = GameType::Fps;
    let rts = GameType::Rts;
    let moba = GameType::Moba;
    let rpg = GameType::Rpg;
    let custom = GameType::Custom("MyGame".to_string());

    assert!(matches!(fps, GameType::Fps));
    assert!(matches!(rts, GameType::Rts));
    assert!(matches!(moba, GameType::Moba));
    assert!(matches!(rpg, GameType::Rpg));
    assert!(matches!(custom, GameType::Custom(_)));
}

// ============================================================================
// GAMING CONFIG TESTS
// ============================================================================

#[test]
fn test_gaming_config_default() {
    let config = CanonicalGamingConfig::default();
    assert!(config.core.enabled);
    assert!(matches!(config.core.mode, GamingMode::Performance));
}

// ============================================================================
// CORE CONFIG TESTS
// ============================================================================

#[test]
fn test_gaming_core_config_default() {
    let config = GamingCoreConfig::default();
    assert!(config.enabled);
    assert!(matches!(config.mode, GamingMode::Performance));
    assert!(matches!(config.default_game_type, GameType::Fps));
}

#[test]
fn test_gaming_core_config_custom() {
    let config = GamingCoreConfig {
        enabled: false,
        mode: GamingMode::PowerSaver,
        default_game_type: GameType::Rpg,
    };

    assert!(!config.enabled);
    assert!(matches!(config.mode, GamingMode::PowerSaver));
}

// ============================================================================
// NETWORK CONFIG TESTS
// ============================================================================

#[test]
fn test_gaming_network_config_default() {
    let config = GamingNetworkConfig::default();
    assert!(config.optimization.enabled);
}

#[test]
fn test_protocol_config_default() {
    let config = ProtocolConfig::default();
    assert_eq!(config.supported, vec!["udp".to_string(), "tcp".to_string()]);
    assert_eq!(config.default, "udp");
}

#[test]
fn test_gaming_port_config_default() {
    let config = GamingPortConfig::default();
    assert_eq!(config.base_port, 6112);
    assert_eq!(config.port_range, (6112, 6200));
    assert!(!config.reserved_ports.is_empty());
}

// ============================================================================
// SECURITY CONFIG TESTS
// ============================================================================

#[test]
fn test_gaming_security_config_default() {
    let config = GamingSecurityConfig::default();
    assert!(config.settings.enabled);
    assert!(config.auth.enabled);
}

#[test]
fn test_gaming_security_settings_default() {
    let settings = GamingSecuritySettings::default();
    assert!(settings.enabled);
    assert!(settings.anti_cheat);
    assert!(settings.encryption);
}

#[test]
fn test_gaming_auth_config_default() {
    let auth = GamingAuthConfig::default();
    assert!(auth.enabled);
    assert_eq!(auth.method, "jwt");
    assert_eq!(auth.session_timeout, 3600);
}

// ============================================================================
// PERFORMANCE CONFIG TESTS
// ============================================================================

#[test]
fn test_gaming_performance_config_default() {
    let config = GamingPerformanceConfig::default();
    assert!(config.settings.low_latency);
    assert!(config.optimization.enabled);
}

#[test]
fn test_gaming_performance_settings_default() {
    let settings = GamingPerformanceSettings::default();
    assert_eq!(settings.target_fps, 60);
    assert_eq!(settings.buffer_size, 8192);
    assert!(settings.low_latency);
}

#[test]
fn test_gaming_optimization_config_default() {
    let config = GamingOptimizationConfig::default();
    assert!(config.enabled);
    assert!(config.cpu_optimization);
    assert!(config.memory_optimization);
}

// ============================================================================
// AUTO CONFIG TESTS
// ============================================================================

#[test]
fn test_gaming_auto_config_default() {
    let config = GamingAutoConfig::default();
    assert!(config.enabled);
    assert!(config.detection.enabled);
    assert!(config.optimization.enabled);
}

#[test]
fn test_security_provider_integration_config_default() {
    let config = SecurityProviderIntegrationConfig::default();
    assert!(!config.enabled);
    assert!(config.endpoint.is_none());
}

#[test]
fn test_security_provider_monitoring_config_default() {
    let config = SecurityProviderMonitoringConfig::default();
    assert!(config.enabled);
    assert_eq!(config.metrics_interval, 60);
    assert_eq!(config.health_check_interval, 30);
}

#[test]
fn test_auto_detection_config_default() {
    let config = AutoDetectionConfig::default();
    assert!(config.enabled);
    assert_eq!(config.timeout_seconds, 10);
    assert_eq!(config.interval_seconds, 30);
}

#[test]
fn test_network_optimization_config_default() {
    let config = NetworkOptimizationConfig::default();
    assert!(config.enabled);
    assert!(config.buffer_optimization);
    assert!(config.connection_pooling);
}

// ============================================================================
// ONE-TOUCH CONFIG TESTS
// ============================================================================

#[test]
fn test_one_touch_config_default() {
    let config = OneTouchConfig::default();
    assert!(config.enabled);
    assert_eq!(config.default_profile.name, "default");
}

#[test]
fn test_gaming_profile_default() {
    let profile = GamingProfile::default();
    assert_eq!(profile.name, "default");
    assert!(!profile.protocol_preference.is_empty());
}

#[test]
fn test_gaming_template() {
    let template = GamingTemplate {
        name: "FPS Template".to_string(),
        ports: vec![6112, 6113],
        protocols: vec![GameProtocolClass::FirstPersonShooter],
    };

    assert_eq!(template.name, "FPS Template");
    assert_eq!(template.ports.len(), 2);
}

// ============================================================================
// ENCRYPTION CONFIG TESTS
// ============================================================================

#[test]
fn test_encryption_config_default() {
    let config = EncryptionConfig::default();
    assert!(config.enabled);
    assert_eq!(config.algorithm, "AES256");
    assert_eq!(config.key_size, 256);
}

// ============================================================================
// AUTHENTICATION CONFIG TESTS
// ============================================================================

#[test]
fn test_authentication_config_default() {
    let config = AuthenticationConfig::default();
    assert!(config.enabled);
    assert_eq!(config.method, "bearer_token");
    assert_eq!(config.token_lifetime, Duration::from_secs(3600));
}

// ============================================================================
// PRIVILEGE CONFIG TESTS
// ============================================================================

#[test]
fn test_privilege_config_default() {
    let config = PrivilegeConfig::default();
    assert!(config.enabled);
    assert_eq!(config.default_level, 1);
    assert_eq!(config.max_level, 10);
}

// ============================================================================
// PERFORMANCE MODE TESTS
// ============================================================================

#[test]
fn test_performance_modes() {
    let high = PerformanceMode::HighPerformance;
    let balanced = PerformanceMode::Balanced;
    let saver = PerformanceMode::PowerSaver;

    assert!(matches!(high, PerformanceMode::HighPerformance));
    assert!(matches!(balanced, PerformanceMode::Balanced));
    assert!(matches!(saver, PerformanceMode::PowerSaver));
}

// ============================================================================
// BENCHMARK CONFIG TESTS
// ============================================================================

#[test]
fn test_benchmark_config_default() {
    let config = BenchmarkConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.interval, Duration::from_secs(60));
    assert_eq!(config.iterations, 10);
}

// ============================================================================
// QOS CONFIG TESTS
// ============================================================================

#[test]
fn test_qos_config_default() {
    let config = QoSConfig::default();
    assert!(config.enabled);
    assert_eq!(config.priority_levels, 3);
    assert!(config.bandwidth_allocation.is_empty());
}

// ============================================================================
// PROTOCOL DETECTION TESTS
// ============================================================================

#[test]
fn test_protocol_detection_config_default() {
    let config = ProtocolDetectionConfig::default();
    assert!(config.enabled);
    assert_eq!(config.detection_timeout, Duration::from_secs(5));
    assert!(!config.supported_protocols.is_empty());
}

#[test]
fn test_detection_rule() {
    let rule = DetectionRule {
        name: "FPS Detection".to_string(),
        signature: Some(vec![0x01, 0x02, 0x03]),
        protocol_class: GameProtocolClass::FirstPersonShooter,
    };

    assert_eq!(rule.name, "FPS Detection");
    assert!(rule.signature.is_some());
}

// ============================================================================
// NAT TRAVERSAL TESTS
// ============================================================================

#[test]
fn test_nat_traversal_config_default() {
    let config = NatTraversalConfig::default();
    assert!(config.enabled);
    assert!(!config.stun_servers.is_empty());
    assert!(config.upnp_enabled);
}

#[test]
fn test_stun_server_config() {
    let stun = StunServerConfig {
        address: "stun.example.com:3478".to_string(),
        enabled: true,
    };

    assert_eq!(stun.address, "stun.example.com:3478");
    assert!(stun.enabled);
}

#[test]
fn test_turn_server_config() {
    let turn = TurnServerConfig {
        address: "turn.example.com:3478".to_string(),
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        enabled: true,
    };

    assert_eq!(turn.address, "turn.example.com:3478");
    assert!(turn.username.is_some());
    assert!(turn.enabled);
}

// ============================================================================
// GAME PROTOCOL CLASS TESTS
// ============================================================================

#[test]
fn test_game_protocol_classes() {
    let rts = GameProtocolClass::RealTimeStrategy;
    let fps = GameProtocolClass::FirstPersonShooter;
    let moba = GameProtocolClass::MultiplayerOnlineBattleArena;
    let mmo = GameProtocolClass::MassivelyMultiplayerOnline;
    let tbs = GameProtocolClass::TurnBasedStrategy;
    let racing = GameProtocolClass::Racing;
    let sports = GameProtocolClass::Sports;
    let custom = GameProtocolClass::Custom("Custom".to_string());

    assert_eq!(rts, GameProtocolClass::RealTimeStrategy);
    assert_eq!(fps, GameProtocolClass::FirstPersonShooter);
    assert_eq!(moba, GameProtocolClass::MultiplayerOnlineBattleArena);
    assert_eq!(mmo, GameProtocolClass::MassivelyMultiplayerOnline);
    assert_eq!(tbs, GameProtocolClass::TurnBasedStrategy);
    assert_eq!(racing, GameProtocolClass::Racing);
    assert_eq!(sports, GameProtocolClass::Sports);
    assert!(matches!(custom, GameProtocolClass::Custom(_)));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_gaming_config_serialization() {
    let config = CanonicalGamingConfig::default();
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: CanonicalGamingConfig =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.core.enabled, config.core.enabled);
}

#[test]
fn test_game_protocol_class_serialization() {
    let protocol = GameProtocolClass::FirstPersonShooter;
    let json = serde_json::to_string(&protocol).expect("Failed to serialize");
    let deserialized: GameProtocolClass =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, protocol);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_gaming_config_setup() {
    let mut config = CanonicalGamingConfig::default();

    config.core.mode = GamingMode::Performance;
    config.performance.settings.target_fps = 144;
    config.network.ports.base_port = 7000;

    assert!(matches!(config.core.mode, GamingMode::Performance));
    assert_eq!(config.performance.settings.target_fps, 144);
    assert_eq!(config.network.ports.base_port, 7000);
}
