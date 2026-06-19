// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Discovery Configuration Module
//!
//! **CANONICAL DISCOVERY CONFIGURATION** ✅
//!
//! This module provides service discovery configuration structures for the Songbird ecosystem.
//! Uses idiomatic Rust patterns: enums for state, bitflags for features.

use serde::{Deserialize, Serialize};

// ============================================================================
// DISCOVERY CONFIGURATION - Secure Anonymous Discovery
// ============================================================================

/// Discovery mode - uses enum instead of multiple bools
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DiscoveryMode {
    /// Discovery disabled
    Disabled,
    /// Anonymous discovery only (default, most secure)
    #[default]
    Anonymous,
    /// Share capabilities, but not identity
    CapabilityAware,
    /// Full disclosure (identity + capabilities)
    FullDisclosure,
}

impl DiscoveryMode {
    /// Check if discovery is enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Check if anonymous
    #[must_use]
    pub const fn is_anonymous(&self) -> bool {
        matches!(self, Self::Anonymous)
    }

    /// Check if sharing capabilities
    #[must_use]
    pub const fn shares_capabilities(&self) -> bool {
        matches!(self, Self::CapabilityAware | Self::FullDisclosure)
    }

    /// Check if sharing identity
    #[must_use]
    pub const fn shares_identity(&self) -> bool {
        matches!(self, Self::FullDisclosure)
    }
}

/// **CANONICAL**: Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {
    /// Discovery mode (replaces multiple bools)
    pub mode: DiscoveryMode,

    /// Discovery backend (default: "universal")
    pub backend: String,

    /// Discovery port for UDP broadcast (default: 2300)
    pub port: u16,

    /// Broadcast addresses for discovery (supports multicast)
    pub broadcast_addresses: Vec<String>,

    /// Known peer addresses for direct discovery (bypasses multicast)
    pub known_peers: Vec<String>,

    /// Discovery protocol version (default: "2.0")
    pub protocol_version: String,

    /// Session rotation interval in seconds (default: 3600 = 1 hour)
    pub session_rotation_interval: u64,
}

impl Default for CanonicalDiscoveryConfig {
    fn default() -> Self {
        // Parse discovery mode from environment
        let mode = songbird_process_env::var("SONGBIRD_DISCOVERY_MODE")
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "disabled" => Some(DiscoveryMode::Disabled),
                "anonymous" => Some(DiscoveryMode::Anonymous),
                "capability" => Some(DiscoveryMode::CapabilityAware),
                "full" => Some(DiscoveryMode::FullDisclosure),
                _ => None,
            })
            .unwrap_or_default(); // Anonymous by default

        Self {
            mode,
            backend: songbird_process_env::var("SONGBIRD_DISCOVERY_BACKEND")
                .unwrap_or_else(|_| "universal".to_string()),
            port: songbird_process_env::var("SONGBIRD_DISCOVERY_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(crate::constants::BROADCAST_DISCOVERY_PORT),
            broadcast_addresses: songbird_process_env::var("SONGBIRD_BROADCAST_ADDRESSES")
                .ok()
                .and_then(|s| {
                    let addrs: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
                    if addrs.is_empty() {
                        None
                    } else {
                        Some(addrs)
                    }
                })
                .unwrap_or_else(|| {
                    vec![format!(
                        "{}:{}",
                        crate::constants::MDNS_MULTICAST_GROUP,
                        crate::constants::BROADCAST_DISCOVERY_PORT
                    )]
                }),
            known_peers: songbird_process_env::var("SONGBIRD_KNOWN_PEERS")
                .ok()
                .and_then(|s| {
                    let addrs: Vec<String> = s.split(',').map(|s| s.trim().to_string()).collect();
                    if addrs.is_empty() {
                        None
                    } else {
                        Some(addrs)
                    }
                })
                .unwrap_or_default(),
            protocol_version: "2.0".to_string(),
            session_rotation_interval: 3600, // 1 hour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), json);
    }

    #[test]
    fn default_canonical_discovery_config() {
        let c = CanonicalDiscoveryConfig::default();
        assert_eq!(c.protocol_version, "2.0");
        assert_eq!(c.session_rotation_interval, 3600);
        assert!(c.port > 0);
    }

    #[test]
    fn discovery_mode_helpers() {
        assert!(!DiscoveryMode::Disabled.is_enabled());
        assert!(DiscoveryMode::Anonymous.is_enabled());
        assert!(DiscoveryMode::Anonymous.is_anonymous());
        assert!(!DiscoveryMode::CapabilityAware.is_anonymous());
        assert!(DiscoveryMode::CapabilityAware.shares_capabilities());
        assert!(!DiscoveryMode::CapabilityAware.shares_identity());
        assert!(DiscoveryMode::FullDisclosure.shares_identity());
    }

    #[test]
    fn roundtrip_discovery_mode_variants() {
        assert_json_roundtrip(&DiscoveryMode::Disabled);
        assert_json_roundtrip(&DiscoveryMode::Anonymous);
        assert_json_roundtrip(&DiscoveryMode::CapabilityAware);
        assert_json_roundtrip(&DiscoveryMode::FullDisclosure);
    }

    #[test]
    fn roundtrip_canonical_discovery_config() {
        assert_json_roundtrip(&CanonicalDiscoveryConfig::default());
    }

    #[test]
    fn roundtrip_discovery_config_edge_port() {
        let mut c = CanonicalDiscoveryConfig::default();
        c.port = u16::MAX;
        c.session_rotation_interval = 0;
        assert_json_roundtrip(&c);
    }
}
