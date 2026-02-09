//! Gaming Network Module
//!
//! Provides network bridging and protocol translation for legacy gaming applications.

// Module declarations
pub mod auto_config;
pub mod bridge;
pub mod bridge_manager; // Generated for pedantic completion
pub mod gaming;
pub mod nat_manager; // Generated for pedantic completion
pub mod nat_traversal;
pub mod production_lan;
pub mod protocol_detector; // Generated for pedantic completion
pub mod security;
pub mod translators; // New modular translator system
pub mod types;

// Individual files
pub mod advanced_tunnel_system;
pub mod bstp_handshake;
pub mod canonical_gaming_manager;
pub mod canonical_types;
pub mod native_wireguard;
pub mod performance;
pub mod privilege_manager;
pub mod production_lan_manager;
pub mod production_protocol_detector;
pub mod production_tunnel_manager;
pub mod protocol_translators;
pub mod real_bridge_manager;
pub mod real_ipx_bridge;
pub mod real_protocol_detector;
pub mod security_provider;
pub mod traffic_classifier;
pub mod universal_bridge;
pub mod universal_detector;
pub mod wireguard_integration;

// Imports for the main gaming service
use self::{
    production_lan_manager::ProductionLanManager,
    real_bridge_manager::RealBridgeManager,
    types::*,
    universal_bridge::UniversalBridge,
    universal_detector::UniversalDetector,
};
use songbird_errors::SongbirdError;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn}; 

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryMessage {
    pub session_code: String,
    pub game_name: Option<String>,
    pub host_address: String,
    pub protocol_type: String,
    pub player_count: u32,
    pub max_players: u32,
    pub game_mode: Option<String>,
    pub server_info: Option<String>,
} 