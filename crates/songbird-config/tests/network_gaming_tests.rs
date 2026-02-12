//! Tests for gaming network configuration

use songbird_config::canonical::network::gaming::*;

#[test]
fn test_gaming_network_config_default() {
    let config = GamingNetworkConfig::default();
    assert_eq!(config.starcraft_port, 6112);
    assert_eq!(config.aoe2_port, 6113);
    assert_eq!(config.ipx_port, 6112);
    assert_eq!(config.udp_port, 6114);
    assert!(config.enable_lan_discovery);
    assert_eq!(config.max_players_per_game, 8);
}

#[test]
fn test_gaming_network_config_clone() {
    let config = GamingNetworkConfig::default();
    let cloned = config.clone();
    assert_eq!(config.starcraft_port, cloned.starcraft_port);
    assert_eq!(config.enable_lan_discovery, cloned.enable_lan_discovery);
}

#[test]
fn test_gaming_network_config_debug() {
    let config = GamingNetworkConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("GamingNetworkConfig"));
    assert!(debug_str.contains("starcraft_port"));
}

#[test]
fn test_gaming_network_config_serialization() {
    let config = GamingNetworkConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("starcraft_port"));
    assert!(json.contains("6112"));
}

#[test]
fn test_gaming_network_config_deserialization() {
    let json = r#"{"starcraft_port":7000,"aoe2_port":7001,"ipx_port":7002,"udp_port":7003,"enable_lan_discovery":false,"max_players_per_game":16}"#;
    let config: GamingNetworkConfig = serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(config.starcraft_port, 7000);
    assert_eq!(config.aoe2_port, 7001);
    assert!(!config.enable_lan_discovery);
    assert_eq!(config.max_players_per_game, 16);
}

#[test]
fn test_gaming_scale_default() {
    let scale = GamingScale::default();
    assert_eq!(scale, GamingScale::Home);
}

#[test]
fn test_gaming_scale_max_players() {
    assert_eq!(GamingScale::Home.max_players(), 4);
    assert_eq!(GamingScale::LanParty.max_players(), 16);
    assert_eq!(GamingScale::Tournament.max_players(), 64);
    assert_eq!(GamingScale::Professional.max_players(), 256);
}

#[test]
fn test_gaming_scale_recommended_bandwidth() {
    assert_eq!(GamingScale::Home.recommended_bandwidth_mbps(), 10);
    assert_eq!(GamingScale::LanParty.recommended_bandwidth_mbps(), 50);
    assert_eq!(GamingScale::Tournament.recommended_bandwidth_mbps(), 200);
    assert_eq!(GamingScale::Professional.recommended_bandwidth_mbps(), 1000);
}

#[test]
fn test_gaming_scale_recommended_connections() {
    assert_eq!(GamingScale::Home.recommended_connections(), 10);
    assert_eq!(GamingScale::LanParty.recommended_connections(), 50);
    assert_eq!(GamingScale::Tournament.recommended_connections(), 200);
    assert_eq!(GamingScale::Professional.recommended_connections(), 1000);
}

#[test]
fn test_gaming_scale_display() {
    assert_eq!(format!("{}", GamingScale::Home), "home");
    assert_eq!(format!("{}", GamingScale::LanParty), "lan-party");
    assert_eq!(format!("{}", GamingScale::Tournament), "tournament");
    assert_eq!(format!("{}", GamingScale::Professional), "professional");
}

#[test]
fn test_gaming_scale_clone() {
    let scale = GamingScale::Tournament;
    let cloned = scale;
    assert_eq!(scale, cloned);
}

#[test]
fn test_gaming_scale_copy() {
    let scale = GamingScale::LanParty;
    let copied = scale;
    assert_eq!(scale, copied);
}

#[test]
fn test_gaming_scale_equality() {
    assert_eq!(GamingScale::Home, GamingScale::Home);
    assert_ne!(GamingScale::Home, GamingScale::LanParty);
    assert_ne!(GamingScale::Tournament, GamingScale::Professional);
}

#[test]
fn test_gaming_scale_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(GamingScale::Home);
    set.insert(GamingScale::LanParty);
    set.insert(GamingScale::Home); // Duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_gaming_scale_debug() {
    let scale = GamingScale::Professional;
    let debug_str = format!("{:?}", scale);
    assert!(debug_str.contains("Professional"));
}

#[test]
fn test_gaming_scale_serialization() {
    let scale = GamingScale::Tournament;
    let json = serde_json::to_string(&scale).expect("Serialization should succeed");
    assert!(json.contains("Tournament"));
}

#[test]
fn test_gaming_scale_deserialization() {
    let json = r#""LanParty""#;
    let scale: GamingScale = serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(scale, GamingScale::LanParty);
}

