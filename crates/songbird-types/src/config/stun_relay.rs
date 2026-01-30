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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StunStrategy {
    /// Try genetic lineage first, then user-provided, then public (default)
    ///
    /// Prioritizes sovereignty and zero external trust.
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

impl Default for StunStrategy {
    fn default() -> Self {
        Self::SovereigntyFirst
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelayOfferMode {
    /// Offer relay automatically to all descendants
    Automatic,

    /// Require explicit approval per relay request
    Manual,
}

impl Default for RelayOfferMode {
    fn default() -> Self {
        Self::Automatic
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StunProtocol {
    /// UDP STUN (RFC 5389, most common)
    Udp,

    /// TCP STUN (for networks that block UDP)
    Tcp,

    /// TLS-wrapped STUN (encrypted, but slower)
    Tls,
}

impl Default for StunProtocol {
    fn default() -> Self {
        Self::Udp
    }
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
    vec![
        // Tier 3A: Open-source friendly (VETTED)
        StunServerConfig {
            address: "stun.nextcloud.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 10,
            enabled: true,
            verified: false,
            vetted: true, // Community-vetted
            comment: "Nextcloud community STUN server (open-source friendly)".to_string(),
        },
        // Tier 3B: VoIP providers (UNVETTED - USE WITH CAUTION)
        StunServerConfig {
            address: "stun.voipawesome.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 20,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        StunServerConfig {
            address: "stun.counterpath.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 21,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        StunServerConfig {
            address: "stun.3cx.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 22,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - 3CX PBX provider".to_string(),
        },
        StunServerConfig {
            address: "stun.antisip.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 23,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        StunServerConfig {
            address: "stun.callwithus.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 24,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        StunServerConfig {
            address: "stun.voipbuster.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 25,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        StunServerConfig {
            address: "stun.voipstunt.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 26,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        StunServerConfig {
            address: "stun.voxgratia.org:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 27,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - VoIP provider".to_string(),
        },
        // Tier 3C: European providers (UNVETTED)
        StunServerConfig {
            address: "stun.1und1.de:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 30,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - German ISP (1&1)".to_string(),
        },
        StunServerConfig {
            address: "stun.acrobits.cz:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 31,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - Czech provider (Acrobits)".to_string(),
        },
        // Tier 3D: Generic services (LOWEST PRIORITY, UNVETTED)
        StunServerConfig {
            address: "stun.services:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 40,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - Generic service".to_string(),
        },
        StunServerConfig {
            address: "stun.12connect.com:3478".to_string(),
            protocol: StunProtocol::Udp,
            priority: 41,
            enabled: true,
            verified: false,
            vetted: false,
            comment: "UNVETTED - Generic service".to_string(),
        },
    ]
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
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub use_steam_relay: bool,

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
    "Future: Piggyback Steam's infrastructure for friend connections".to_string()
}

/// Discord rendezvous configuration (FUTURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordRendezvousConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub use_discord_relay: bool,

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
    "Future: Use Discord's voice infrastructure for friend gaming".to_string()
}

/// Custom rendezvous configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRendezvousConfig {
    pub name: String,

    #[serde(default)]
    pub enabled: bool,

    pub address: String,

    #[serde(default = "default_https")]
    pub protocol: String,

    #[serde(default)]
    pub comment: String,
}

/// Advanced STUN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Configuration struct - bools are appropriate for feature flags
pub struct AdvancedStunConfig {
    /// Try multiple tiers in parallel (for FastestFirst strategy)
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
fn default_true() -> bool {
    true
}
fn default_timeout() -> u64 {
    5
}
fn default_max_hops() -> u8 {
    3
}
fn default_bandwidth_limit() -> u32 {
    100
}
fn default_max_relays() -> u32 {
    10
}
fn default_https() -> String {
    "https".to_string()
}
fn default_priority() -> u32 {
    100
}
fn default_rotation_interval() -> u64 {
    3600
}
fn default_latency_threshold() -> u32 {
    50
}
fn default_packet_loss_threshold() -> f32 {
    5.0
}

#[cfg(test)]
mod tests {
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
