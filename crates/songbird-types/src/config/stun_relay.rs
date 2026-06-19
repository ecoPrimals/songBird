// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN/Relay Multi-Tier Configuration
//!
//! **Pure Rust, Runtime-Configurable NAT Traversal**
//!
//! Multi-tier strategy:
//! - Tier 1: Genetic lineage relay (HIGHEST TRUST, zero external dependency)
//! - Tier 2: User-provided STUN servers (HIGH TRUST, custom infrastructure)
//! - Tier 3: Public STUN list (MEDIUM TRUST, global friend gaming)
//! - Tier 4: Rendezvous STUN (LOW TRUST, gaming platforms)
//!
//! ## Sovereignty First
//!
//! Default configuration prioritizes sovereignty:
//! - Genetic lineage relay enabled by default
//! - Public STUN disabled by default (opt-in for convenience)
//! - Zero external trust required for core functionality
//!
//! ## Modern Rust (v3.20.0)
//!
//! - Zero hardcoding (all runtime configuration)
//! - Capability-based discovery
//! - Type-safe configuration
//! - No unsafe code

use serde::{Deserialize, Serialize};

/// Multi-tier STUN/relay configuration
///
/// Enables flexible NAT traversal strategies from maximum sovereignty
/// (genetic lineage only) to maximum convenience (public STUN + rendezvous).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunRelayConfig {
    /// Enable STUN/relay system
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Fallback strategy
    #[serde(default)]
    pub strategy: StunStrategy,

    /// Timeout per tier (seconds)
    #[serde(default = "default_timeout")]
    pub tier_timeout_secs: u64,

    /// Tier 1: Genetic lineage relay
    #[serde(default)]
    pub lineage: LineageRelayConfig,

    /// Tier 2: User-provided STUN servers
    #[serde(default)]
    pub user_provided: Vec<StunServerConfig>,

    /// Tier 3: Public STUN list
    #[serde(default)]
    pub public_stun: PublicStunConfig,

    /// Tier 4: Rendezvous STUN (future: Steam, Discord)
    #[serde(default)]
    pub rendezvous: RendezvousConfig,

    /// Advanced settings
    #[serde(default)]
    pub advanced: AdvancedStunConfig,
}

impl Default for StunRelayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: StunStrategy::SovereigntyFirst,
            tier_timeout_secs: 5,
            lineage: LineageRelayConfig::default(),
            user_provided: Vec::new(),
            public_stun: PublicStunConfig::default(),
            rendezvous: RendezvousConfig::default(),
            advanced: AdvancedStunConfig::default(),
        }
    }
}

/// STUN/relay fallback strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StunStrategy {
    /// Try genetic lineage first, then user-provided, then public (default)
    ///
    /// Prioritizes sovereignty and zero external trust.
    #[default]
    SovereigntyFirst,

    /// Try all methods in parallel, use first success
    ///
    /// Optimizes for speed and convenience (e.g., friend gaming).
    FastestFirst,

    /// Only use genetic lineage (maximum sovereignty)
    ///
    /// Never uses external STUN servers. Fails if lineage unavailable.
    LineageOnly,
}

/// Tier 1: Genetic lineage relay configuration
///
/// Uses genetic family as relay infrastructure (zero external trust).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRelayConfig {
    /// Enable genetic lineage relay
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Prefer lineage over public STUN
    #[serde(default = "default_true")]
    pub prefer_lineage: bool,

    /// Maximum hops through lineage for relay discovery
    ///
    /// - 1 = parent only
    /// - 2 = parent + grandparent
    /// - 3 = great-grandparent (default)
    #[serde(default = "default_max_hops")]
    pub max_lineage_hops: u8,

    /// Relay offer mode
    #[serde(default)]
    pub relay_offer_mode: RelayOfferMode,

    /// Bandwidth limit for relay (MB/s, 0 = unlimited)
    #[serde(default = "default_bandwidth_limit")]
    pub relay_bandwidth_limit_mbps: u32,

    /// Maximum concurrent relay connections
    #[serde(default = "default_max_relays")]
    pub max_concurrent_relays: u32,
}

impl Default for LineageRelayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefer_lineage: true,
            max_lineage_hops: 3,
            relay_offer_mode: RelayOfferMode::Automatic,
            relay_bandwidth_limit_mbps: 100,
            max_concurrent_relays: 10,
        }
    }
}

/// Relay offer mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RelayOfferMode {
    /// Offer relay automatically to all descendants
    #[default]
    Automatic,

    /// Require explicit approval per relay request
    Manual,
}

/// STUN server configuration
///
/// Used for both user-provided (Tier 2) and public (Tier 3) STUN servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServerConfig {
    /// Server address (host:port format, e.g., "stun.example.com:3478")
    pub address: String,

    /// Protocol (udp, tcp, tls)
    #[serde(default)]
    pub protocol: StunProtocol,

    /// Priority (lower number = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Enable this server
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// User has verified/vouches for this server (Tier 2)
    #[serde(default)]
    pub verified: bool,

    /// Vetted by ecoPrimals community (Tier 3)
    #[serde(default)]
    pub vetted: bool,

    /// Human-readable comment/description
    #[serde(default)]
    pub comment: String,
}

/// STUN protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StunProtocol {
    /// UDP STUN (RFC 5389, most common)
    #[default]
    Udp,

    /// TCP STUN (for networks that block UDP)
    Tcp,

    /// TLS-wrapped STUN (encrypted, but slower)
    Tls,
}

/// Tier 3: Public STUN configuration
///
/// Fallback to public STUN servers for global friend gaming.
/// Disabled by default (sovereignty-first).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicStunConfig {
    /// Enable public STUN fallback
    ///
    /// **Privacy Warning**: Public STUN servers can observe your public IP/port.
    #[serde(default)]
    pub enabled: bool,

    /// Only use as fallback (after lineage and user-provided)
    #[serde(default = "default_true")]
    pub use_as_fallback_only: bool,

    /// Rotate servers for privacy (prevent tracking)
    #[serde(default = "default_true")]
    pub rotate_servers: bool,

    /// Rotation interval (seconds)
    #[serde(default = "default_rotation_interval")]
    pub rotation_interval_secs: u64,

    /// List of public STUN servers
    #[serde(default = "default_public_stun_servers")]
    pub servers: Vec<StunServerConfig>,
}

impl Default for PublicStunConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default (sovereignty first)
            use_as_fallback_only: true,
            rotate_servers: true,
            rotation_interval_secs: 3600, // 1 hour
            servers: default_public_stun_servers(),
        }
    }
}

/// Default public STUN server list
///
/// **IMPORTANT**: Most servers are UNVETTED and provided as-is.
/// Only use if convenience > absolute sovereignty.
fn default_public_stun_servers() -> Vec<StunServerConfig> {
    // Data-driven server definitions: (address, priority, vetted, comment)
    // Tier 3A: Open-source friendly (VETTED)
    let vetted_servers = [("stun.nextcloud.com:3478", 10, "Nextcloud community (open-source)")];

    // Tier 3B-D: Unvetted servers by priority tier
    let unvetted_servers = [
        // Tier 3B: VoIP providers (priority 20-29)
        ("stun.voipawesome.com:3478", 20, "VoIP provider"),
        ("stun.counterpath.com:3478", 21, "VoIP provider"),
        ("stun.3cx.com:3478", 22, "3CX PBX provider"),
        ("stun.antisip.com:3478", 23, "VoIP provider"),
        ("stun.callwithus.com:3478", 24, "VoIP provider"),
        ("stun.voipbuster.com:3478", 25, "VoIP provider"),
        ("stun.voipstunt.com:3478", 26, "VoIP provider"),
        ("stun.voxgratia.org:3478", 27, "VoIP provider"),
        // Tier 3C: European providers (priority 30-39)
        ("stun.1und1.de:3478", 30, "German ISP (1&1)"),
        ("stun.acrobits.cz:3478", 31, "Czech provider (Acrobits)"),
        // Tier 3D: Generic services (priority 40+)
        ("stun.services:3478", 40, "Generic service"),
        ("stun.12connect.com:3478", 41, "Generic service"),
    ];

    let mut servers = Vec::with_capacity(vetted_servers.len() + unvetted_servers.len());

    // Add vetted servers
    for (addr, priority, comment) in vetted_servers {
        servers.push(StunServerConfig::new_vetted(addr, priority, comment));
    }

    // Add unvetted servers
    for (addr, priority, comment) in unvetted_servers {
        servers.push(StunServerConfig::new_unvetted(addr, priority, comment));
    }

    servers
}

impl StunServerConfig {
    /// Create a new vetted STUN server config
    fn new_vetted(address: &str, priority: u32, comment: &str) -> Self {
        Self {
            address: address.to_string(),
            protocol: StunProtocol::Udp,
            priority,
            enabled: true,
            verified: false,
            vetted: true,
            comment: comment.to_string(),
        }
    }

    /// Create a new unvetted STUN server config
    fn new_unvetted(address: &str, priority: u32, description: &str) -> Self {
        Self {
            address: address.to_string(),
            protocol: StunProtocol::Udp,
            priority,
            enabled: true,
            verified: false,
            vetted: false,
            comment: format!("UNVETTED - {description}"),
        }
    }
}

/// Tier 4: Rendezvous configuration
///
/// Integration with gaming platforms for friend connections.
/// FUTURE: Requires Steam/Discord API integration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RendezvousConfig {
    /// Enable rendezvous STUN
    #[serde(default)]
    pub enabled: bool,

    /// Steam integration (future)
    #[serde(default)]
    pub steam: SteamRendezvousConfig,

    /// Discord integration (future)
    #[serde(default)]
    pub discord: DiscordRendezvousConfig,

    /// Custom rendezvous servers
    #[serde(default)]
    pub custom: Vec<CustomRendezvousConfig>,
}

// Default derived automatically - all fields have Default implementations

/// Steam rendezvous configuration (FUTURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamRendezvousConfig {
    /// Allow Steam-backed rendezvous when integrated.
    #[serde(default)]
    pub enabled: bool,

    /// Route friend traffic through Steam relay when available.
    #[serde(default)]
    pub use_steam_relay: bool,

    /// Human-readable note for operators (integration status and intent).
    #[serde(default = "default_steam_comment")]
    pub comment: String,
}

impl Default for SteamRendezvousConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            use_steam_relay: false,
            comment: default_steam_comment(),
        }
    }
}

fn default_steam_comment() -> String {
    String::from("Future: Piggyback Steam's infrastructure for friend connections")
}

/// Discord rendezvous configuration (FUTURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordRendezvousConfig {
    /// Allow Discord-backed rendezvous when integrated.
    #[serde(default)]
    pub enabled: bool,

    /// Route friend traffic through Discord relay when available.
    #[serde(default)]
    pub use_discord_relay: bool,

    /// Human-readable note for operators (integration status and intent).
    #[serde(default = "default_discord_comment")]
    pub comment: String,
}

impl Default for DiscordRendezvousConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            use_discord_relay: false,
            comment: default_discord_comment(),
        }
    }
}

fn default_discord_comment() -> String {
    String::from("Future: Use Discord's voice infrastructure for friend gaming")
}

/// Custom rendezvous configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRendezvousConfig {
    /// Label shown in logs and UI for this rendezvous target.
    pub name: String,

    /// Enable or disable this custom rendezvous endpoint.
    #[serde(default)]
    pub enabled: bool,

    /// Host or URL used to reach the rendezvous service.
    pub address: String,

    /// Scheme or transport label (for example `https`).
    #[serde(default = "default_https")]
    pub protocol: String,

    /// Optional operator notes for this endpoint.
    #[serde(default)]
    pub comment: String,
}

/// Advanced STUN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "intentional pattern; clippy false positive for this API"
)] // Configuration struct - bools are appropriate for feature flags
pub struct AdvancedStunConfig {
    /// Try multiple tiers in parallel (for `FastestFirst` strategy)
    #[serde(default)]
    pub parallel_attempts: bool,

    /// Monitor connection quality
    #[serde(default = "default_true")]
    pub monitor_quality: bool,

    /// Auto upgrade to direct connection if possible
    #[serde(default = "default_true")]
    pub auto_upgrade_to_direct: bool,

    /// Upgrade latency threshold (milliseconds)
    #[serde(default = "default_latency_threshold")]
    pub upgrade_latency_threshold_ms: u32,

    /// Upgrade packet loss threshold (percent)
    #[serde(default = "default_packet_loss_threshold")]
    pub upgrade_packet_loss_threshold_percent: f32,

    /// Log STUN attempts
    #[serde(default = "default_true")]
    pub log_stun_attempts: bool,

    /// Log relay usage
    #[serde(default = "default_true")]
    pub log_relay_usage: bool,

    /// Privacy protection settings
    #[serde(default)]
    pub privacy: PrivacyConfig,
}

impl Default for AdvancedStunConfig {
    fn default() -> Self {
        Self {
            parallel_attempts: false, // SovereigntyFirst by default
            monitor_quality: true,
            auto_upgrade_to_direct: true,
            upgrade_latency_threshold_ms: 50,
            upgrade_packet_loss_threshold_percent: 5.0,
            log_stun_attempts: true,
            log_relay_usage: true,
            privacy: PrivacyConfig::default(),
        }
    }
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Randomize STUN request timing (prevent traffic analysis)
    #[serde(default = "default_true")]
    pub randomize_timing: bool,

    /// Use Tor for STUN requests (EXPERIMENTAL - requires Tor integration)
    #[serde(default)]
    pub use_tor_for_stun: bool,

    /// Minimal metadata in requests
    #[serde(default = "default_true")]
    pub minimal_metadata: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            randomize_timing: true,
            use_tor_for_stun: false,
            minimal_metadata: true,
        }
    }
}

// Default value helpers
const fn default_true() -> bool {
    true
}
const fn default_timeout() -> u64 {
    5
}
const fn default_max_hops() -> u8 {
    3
}
const fn default_bandwidth_limit() -> u32 {
    100
}
const fn default_max_relays() -> u32 {
    10
}
fn default_https() -> String {
    String::from("https")
}
const fn default_priority() -> u32 {
    100
}
const fn default_rotation_interval() -> u64 {
    3600
}
const fn default_latency_threshold() -> u32 {
    50
}
const fn default_packet_loss_threshold() -> f32 {
    5.0
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions and harness ergonomics")]

    use super::*;

    #[test]
    fn test_default_config_sovereignty_first() {
        let config = StunRelayConfig::default();

        // Verify sovereignty-first defaults
        assert!(config.enabled);
        assert_eq!(config.strategy, StunStrategy::SovereigntyFirst);
        assert!(config.lineage.enabled);
        assert!(!config.public_stun.enabled); // Disabled by default
        assert!(!config.rendezvous.enabled);
    }

    #[test]
    fn test_lineage_relay_defaults() {
        let config = LineageRelayConfig::default();

        assert!(config.enabled);
        assert!(config.prefer_lineage);
        assert_eq!(config.max_lineage_hops, 3);
        assert_eq!(config.relay_offer_mode, RelayOfferMode::Automatic);
        assert_eq!(config.relay_bandwidth_limit_mbps, 100);
        assert_eq!(config.max_concurrent_relays, 10);
    }

    #[test]
    fn test_public_stun_servers_include_unvetted() {
        let servers = default_public_stun_servers();

        // Should have multiple servers
        assert!(!servers.is_empty());

        // Nextcloud should be vetted
        let nextcloud = servers.iter().find(|s| s.address.contains("nextcloud"));
        assert!(nextcloud.is_some());
        assert!(nextcloud.unwrap().vetted);

        // Count unvetted servers
        let unvetted_count = servers.iter().filter(|s| !s.vetted).count();
        assert!(unvetted_count > 0, "Should have unvetted servers");
    }

    #[test]
    fn test_stun_strategy_variants() {
        assert_eq!(StunStrategy::default(), StunStrategy::SovereigntyFirst);

        // Test serialization
        let json = serde_json::to_string(&StunStrategy::SovereigntyFirst).unwrap();
        assert_eq!(json, r#""sovereignty-first""#);

        let json = serde_json::to_string(&StunStrategy::FastestFirst).unwrap();
        assert_eq!(json, r#""fastest-first""#);

        let json = serde_json::to_string(&StunStrategy::LineageOnly).unwrap();
        assert_eq!(json, r#""lineage-only""#);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = StunRelayConfig::default();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(!json.is_empty());

        // Deserialize back
        let deserialized: StunRelayConfig = serde_json::from_str(&json).unwrap();

        // Verify key fields
        assert_eq!(deserialized.strategy, config.strategy);
        assert_eq!(deserialized.lineage.enabled, config.lineage.enabled);
        assert_eq!(deserialized.public_stun.enabled, config.public_stun.enabled);
    }

    #[test]
    fn test_unvetted_servers_all_marked() {
        let servers = default_public_stun_servers();

        // All servers except Nextcloud should be marked unvetted
        for server in &servers {
            if server.address.contains("nextcloud") {
                assert!(server.vetted, "Nextcloud should be vetted");
            } else {
                assert!(!server.vetted, "Server {} should be unvetted", server.address);
                assert!(
                    server.comment.contains("UNVETTED"),
                    "Server {} should have UNVETTED in comment",
                    server.address
                );
            }
        }
    }
}
