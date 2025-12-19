//! # Federation Configuration Module
//!
//! **CANONICAL FEDERATION CONFIGURATION** ✅
//!
//! This module provides federation and clustering configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::env;

// ============================================================================
// FEDERATION CONFIGURATION - Zero-Trust Federation
// ============================================================================

/// **CANONICAL**: Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationConfig {
    /// Enable federation features (default: true)
    pub enabled: bool,
    
    /// Cluster name (default: auto-detected from hostname)
    pub cluster_name: String,
    
    /// Progressive trust escalation enabled (default: true)
    pub trust_escalation: bool,
    
    /// Initial trust level for new federation members (default: anonymous)
    pub initial_trust_level: String,
    
    /// Allow capability escalation (default: true)
    pub allow_capability_escalation: bool,
    
    /// Allow identity escalation (default: true)
    pub allow_identity_escalation: bool,
    
    /// Require hardware key for admin operations (default: true)
    pub require_hardware_for_admin: bool,
    
    /// Federated service discovery enabled (default: true)
    pub federated_discovery: bool,
    
    /// Auto-accept federation members from LAN (default: true)
    pub auto_accept_lan: bool,
    
    /// Auto-accept federation members from WAN (default: false, manual approval)
    pub auto_accept_wan: bool,
    
    /// Trust timeouts for different trust levels (in seconds)
    pub trust_timeouts: TrustTimeouts,
}

/// Trust timeouts for progressive escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustTimeouts {
    /// Anonymous sessions expire after this (default: 3600 = 1 hour)
    pub anonymous: u64,
    /// Capability sessions expire after this (default: 86400 = 24 hours)
    pub capability: u64,
    /// Identity sessions expire after this (default: 604800 = 7 days)
    pub identity: u64,
    /// Hardware sessions never expire (default: 0 = never)
    pub hardware: u64,
}

impl Default for CanonicalFederationConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_ENABLE_FEDERATION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true), // Federation enabled by default
            cluster_name: env::var("SONGBIRD_CLUSTER_NAME")
                .unwrap_or_else(|_| {
                    hostname::get()
                        .ok()
                        .and_then(|h| h.into_string().ok())
                        .unwrap_or_else(|| "songbird-cluster".to_string())
                }),
            trust_escalation: env::var("SONGBIRD_TRUST_ESCALATION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            initial_trust_level: "anonymous".to_string(),
            allow_capability_escalation: true,
            allow_identity_escalation: true,
            require_hardware_for_admin: true,
            federated_discovery: true,
            auto_accept_lan: true,  // Trust LAN by default
            auto_accept_wan: false, // Manual approval for WAN
            trust_timeouts: TrustTimeouts::default(),
        }
    }
}

impl Default for TrustTimeouts {
    fn default() -> Self {
        Self {
            anonymous: 3600,    // 1 hour
            capability: 86400,  // 24 hours
            identity: 604800,   // 7 days
            hardware: 0,        // Never expire
        }
    }
}
