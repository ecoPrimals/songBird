//! # Discovery Configuration Module
//!
//! **CANONICAL DISCOVERY CONFIGURATION** ✅
//!
//! This module provides service discovery configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::env;

// ============================================================================
// DISCOVERY CONFIGURATION - Secure Anonymous Discovery
// ============================================================================

/// **CANONICAL**: Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {
    /// Enable service discovery (default: true)
    pub enabled: bool,
    
    /// Discovery backend (default: "universal")
    pub backend: String,
    
    /// Anonymous discovery enabled (default: true, secure)
    pub anonymous: bool,
    
    /// Discovery port for UDP broadcast (default: 2300)
    pub port: u16,
    
    /// Broadcast addresses for discovery
    pub broadcast_addresses: Vec<String>,
    
    /// Discovery protocol version (default: "2.0")
    pub protocol_version: String,
    
    /// Session rotation interval in seconds (default: 3600 = 1 hour)
    pub session_rotation_interval: u64,
    
    /// Share capabilities in discovery (default: true)
    pub share_capabilities: bool,
    
    /// Share identity in discovery (default: false, anonymous first)
    pub share_identity: bool,
}

impl Default for CanonicalDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_ENABLE_DISCOVERY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            backend: env::var("SONGBIRD_DISCOVERY_BACKEND")
                .unwrap_or_else(|_| "universal".to_string()),
            anonymous: env::var("SONGBIRD_ANONYMOUS_DISCOVERY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true), // Anonymous by default (secure)
            port: env::var("SONGBIRD_DISCOVERY_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2300),
            broadcast_addresses: env::var("SONGBIRD_BROADCAST_ADDRESSES")
                .ok()
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|| vec![
                    "255.255.255.255:2300".to_string(),
                    "192.168.1.255:2300".to_string(),
                ]),
            protocol_version: "2.0".to_string(),
            session_rotation_interval: 3600, // 1 hour
            share_capabilities: true,
            share_identity: false, // Anonymous first
        }
    }
}
