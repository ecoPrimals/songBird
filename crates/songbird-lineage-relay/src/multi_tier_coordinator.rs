// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Multi-Tier NAT Traversal Coordinator
//!
//! **Sovereignty-First Strategy** with fallback tiers
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │          Multi-Tier NAT Traversal Coordinator                   │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  Tier 1: Genetic Lineage Relay                                 │
//! │  ├─ Zero external trust                                        │
//! │  ├─ security provider lineage verification                               │
//! │  └─ Ancestors relay for descendants                            │
//! │                                                                 │
//! │  Tier 2: User-Provided STUN                                    │
//! │  ├─ High trust (user vouched)                                  │
//! │  ├─ Personal/family STUN servers                               │
//! │  └─ Self-hosted relay infrastructure                           │
//! │                                                                 │
//! │  Tier 3: Public STUN List                                      │
//! │  ├─ Medium trust (community vetted)                            │
//! │  ├─ Opt-in only (disabled by default)                          │
//! │  └─ Privacy warnings (IP observation)                          │
//! │                                                                 │
//! │  Tier 4: Rendezvous (Future)                                   │
//! │  ├─ Low trust (piggyback on existing services)                 │
//! │  ├─ Steam, Discord, etc.                                       │
//! │  └─ Last resort for global friend gaming                       │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Strategies
//!
//! ### Sovereignty-First (Default)
//! 1. Try genetic lineage relay
//! 2. Fall back to user-provided STUN
//! 3. Stop (public STUN disabled)
//!
//! ### Fastest-First (Opt-In)
//! 1. Attempt all tiers in parallel
//! 2. Use first successful connection
//! 3. Cancel remaining attempts
//!
//! ## Privacy
//!
//! - Public STUN servers can observe your public IP
//! - Genetic lineage relay preserves privacy
//! - User-provided STUN assumes user trust
//! - Rendezvous piggybacking may leak metadata

use crate::error::{LineageRelayError, Result};
use crate::relay::{RelayDiscovery, RelaySession};
use crate::types::NodeId;
use songbird_stun::{StunClient, TurnClient};
use songbird_types::config::stun_relay::{StunRelayConfig, StunStrategy};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// H2-16: Connection Fallback Chain Tier Identifiers
// ---------------------------------------------------------------------------

/// Tier attempted during the connection fallback chain (H2-16).
///
/// Order follows the sovereignty-first path:
///   direct → STUN punch → lineage relay → TURN relay → emergency tunnel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionTier {
    /// Direct UDP hole-punch (no relay infrastructure).
    Direct,
    /// STUN-assisted hole-punch using reflexive address discovery.
    StunPunch,
    /// Genetic lineage relay (sovereign, zero external trust).
    LineageRelay,
    /// TURN relay (RFC 5766 — self-hosted VPS, `BearDog` key-authenticated; H2-14).
    TurnRelay,
    /// Emergency tunnel fallback (e.g. `cloudflared` — last resort).
    EmergencyTunnel,
}

impl std::fmt::Display for ConnectionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::StunPunch => write!(f, "stun-punch"),
            Self::LineageRelay => write!(f, "lineage-relay"),
            Self::TurnRelay => write!(f, "turn-relay"),
            Self::EmergencyTunnel => write!(f, "emergency-tunnel"),
        }
    }
}

/// Multi-tier coordinator for NAT traversal
pub struct MultiTierCoordinator {
    config: StunRelayConfig,
    stun_client: Arc<StunClient>,
    relay_discovery: Option<Arc<RelayDiscovery>>,
    turn_client: Option<Arc<TurnClient>>,
}

impl MultiTierCoordinator {
    /// Create new multi-tier coordinator
    #[must_use]
    pub fn new(config: StunRelayConfig) -> Self {
        let stun_client = Arc::new(StunClient::with_timeout(Duration::from_secs(3)));
        Self {
            config,
            stun_client,
            relay_discovery: None,
            turn_client: None,
        }
    }

    /// Attach a lineage relay discovery instance for Tier 3 (lineage relay).
    #[must_use]
    pub fn with_relay_discovery(mut self, rd: Arc<RelayDiscovery>) -> Self {
        self.relay_discovery = Some(rd);
        self
    }

    /// Attach a TURN client for Tier 4 (RFC 5766 relay).
    #[must_use]
    pub fn with_turn_client(mut self, tc: Arc<TurnClient>) -> Self {
        self.turn_client = Some(tc);
        self
    }

    /// Discover public address via configured STUN servers
    ///
    /// # Strategy
    ///
    /// Follows sovereignty-first or fastest-first based on config.
    ///
    /// # Errors
    ///
    /// Returns error if all tiers fail.
    pub async fn discover_public_address(&self) -> Result<SocketAddr> {
        match self.config.strategy {
            StunStrategy::SovereigntyFirst => self.sovereignty_first_discovery().await,
            StunStrategy::FastestFirst => self.fastest_first_discovery().await,
            StunStrategy::LineageOnly => {
                // LineageOnly doesn't use STUN, relies solely on genetic lineage relay
                Err(LineageRelayError::NoRelayAvailable(
                    "LineageOnly strategy does not use STUN discovery".to_string(),
                ))
            }
        }
    }

    /// Sovereignty-first strategy: Try tiers sequentially
    ///
    /// 1. User-provided STUN (if configured)
    /// 2. Public STUN (if enabled)
    /// 3. Error if all fail
    ///
    /// Note: Genetic lineage relay doesn't need STUN discovery
    async fn sovereignty_first_discovery(&self) -> Result<SocketAddr> {
        info!("🛡️  Sovereignty-first STUN discovery");

        // Tier 2: User-provided STUN
        if !self.config.user_provided.is_empty() {
            info!("   Tier 2: Trying user-provided STUN servers...");

            // Try each user-provided server
            for user_stun in &self.config.user_provided {
                if !user_stun.enabled {
                    continue;
                }

                match self.stun_client.discover_public_address(&user_stun.address).await {
                    Ok(addr) => {
                        info!("✅ User-provided STUN successful ({}): {}", user_stun.address, addr);
                        return Ok(addr);
                    }
                    Err(e) => {
                        warn!("⚠️  User-provided STUN failed ({}): {}", user_stun.address, e);
                    }
                }
            }
        }

        // Tier 3: Public STUN (opt-in only)
        if self.config.public_stun.enabled {
            info!("   Tier 3: Trying public STUN servers...");
            warn!("⚠️  Privacy Warning: Public STUN servers can observe your IP");

            // Try each public STUN server
            for public_server in &self.config.public_stun.servers {
                match self.stun_client.discover_public_address(&public_server.address).await {
                    Ok(addr) => {
                        info!("✅ Public STUN successful ({}): {}", public_server.address, addr);
                        return Ok(addr);
                    }
                    Err(e) => {
                        warn!("⚠️  Public STUN failed ({}): {}", public_server.address, e);
                    }
                }
            }
        } else {
            debug!("   Tier 3: Public STUN disabled (sovereignty-first)");
        }

        // All tiers failed
        Err(LineageRelayError::NoRelayAvailable("All STUN discovery tiers failed".to_string()))
    }

    /// Fastest-first strategy: Try all tiers in parallel
    ///
    /// Attempts all enabled tiers simultaneously and returns first success.
    async fn fastest_first_discovery(&self) -> Result<SocketAddr> {
        info!("⚡ Fastest-first STUN discovery (parallel)");

        let mut tasks = Vec::new();

        // Tier 2: User-provided STUN
        if !self.config.user_provided.is_empty() {
            for user_stun in self.config.user_provided.clone() {
                if !user_stun.enabled {
                    continue;
                }

                let client = Arc::clone(&self.stun_client);
                tasks.push(tokio::spawn(async move {
                    info!("   Tier 2: Trying user-provided STUN ({})...", user_stun.address);
                    client.discover_public_address(&user_stun.address).await
                }));
            }
        }

        // Tier 3: Public STUN (if enabled)
        if self.config.public_stun.enabled {
            for public_server in self.config.public_stun.servers.clone() {
                let client = Arc::clone(&self.stun_client);
                tasks.push(tokio::spawn(async move {
                    info!("   Tier 3: Trying public STUN ({})...", public_server.address);
                    warn!("⚠️  Privacy Warning: Public STUN servers can observe your IP");
                    client.discover_public_address(&public_server.address).await
                }));
            }
        }

        // Wait for first success
        for task in tasks {
            if let Ok(Ok(addr)) = task.await {
                info!("✅ Fastest-first discovery successful: {}", addr);
                return Ok(addr);
            }
        }

        // All failed
        Err(LineageRelayError::NoRelayAvailable(
            "All parallel STUN discovery attempts failed".to_string(),
        ))
    }

    /// Establish connection using the H2-16 fallback chain:
    ///
    /// ```text
    /// direct → STUN punch → lineage relay → TURN relay → emergency tunnel
    /// ```
    ///
    /// Each tier is attempted in order. The first success wins; each failure
    /// is logged and the chain advances to the next tier. The method returns
    /// a [`ConnectionResult`] that records which tier ultimately succeeded.
    ///
    /// # Errors
    ///
    /// Returns error if **all** tiers fail.
    #[allow(
        clippy::too_many_lines,
        reason = "sequential fallback chain — splitting adds indirection without clarity"
    )]
    pub async fn establish_connection(
        &self,
        peer: NodeId,
        peer_addr: Option<SocketAddr>,
    ) -> Result<ConnectionResult> {
        info!("H2-16 fallback chain: establishing connection to {peer}");

        let mut tier_attempts: Vec<(ConnectionTier, String)> = Vec::new();

        // --- Tier 1: Direct UDP hole-punch ---
        if let Some(addr) = peer_addr {
            info!("  tier=direct: attempting direct punch to {addr}");
            match self.try_direct_punch(addr).await {
                Ok(latency) => {
                    info!("  tier=direct: success ({latency:?})");
                    return Ok(ConnectionResult::Direct {
                        peer_addr: addr,
                        latency,
                        tier: ConnectionTier::Direct,
                    });
                }
                Err(e) => {
                    let msg = format!("{e}");
                    warn!("  tier=direct: failed — {msg}");
                    tier_attempts.push((ConnectionTier::Direct, msg));
                }
            }
        } else {
            tier_attempts.push((ConnectionTier::Direct, "no peer address available".to_string()));
        }

        // --- Tier 2: STUN-assisted punch ---
        match self.try_stun_punch(&peer).await {
            Ok(addr) => {
                info!("  tier=stun-punch: discovered reflexive addr {addr}");
                return Ok(ConnectionResult::Direct {
                    peer_addr: addr,
                    latency: Duration::ZERO,
                    tier: ConnectionTier::StunPunch,
                });
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!("  tier=stun-punch: failed — {msg}");
                tier_attempts.push((ConnectionTier::StunPunch, msg));
            }
        }

        // --- Tier 3: Lineage relay ---
        if let Some(ref rd) = self.relay_discovery {
            info!("  tier=lineage-relay: requesting relay for {peer}");
            match rd.request_relay(peer.clone(), peer_addr).await {
                Ok(session) => {
                    info!("  tier=lineage-relay: session established via {}", session.relay_node);
                    return Ok(ConnectionResult::Relayed {
                        session,
                        tier: ConnectionTier::LineageRelay,
                    });
                }
                Err(e) => {
                    let msg = format!("{e}");
                    warn!("  tier=lineage-relay: failed — {msg}");
                    tier_attempts.push((ConnectionTier::LineageRelay, msg));
                }
            }
        } else {
            tier_attempts
                .push((ConnectionTier::LineageRelay, "relay discovery not configured".to_string()));
        }

        // --- Tier 4: TURN relay (H2-14) ---
        if let Some(ref tc) = self.turn_client {
            info!("  tier=turn-relay: attempting TURN allocation");
            match self.try_turn_allocation(tc).await {
                Ok(alloc) => {
                    info!("  tier=turn-relay: allocated {}", alloc.relay_addr);
                    return Ok(ConnectionResult::TurnRelayed {
                        relay_addr: alloc.relay_addr,
                        tier: ConnectionTier::TurnRelay,
                    });
                }
                Err(e) => {
                    let msg = format!("{e}");
                    warn!("  tier=turn-relay: failed — {msg}");
                    tier_attempts.push((ConnectionTier::TurnRelay, msg));
                }
            }
        } else {
            tier_attempts
                .push((ConnectionTier::TurnRelay, "TURN client not configured".to_string()));
        }

        // --- Tier 5: Emergency tunnel (cloudflared) ---
        match self.try_emergency_tunnel().await {
            Ok(endpoint) => {
                info!("  tier=emergency-tunnel: tunnel established at {endpoint}");
                return Ok(ConnectionResult::Tunneled {
                    endpoint,
                    tier: ConnectionTier::EmergencyTunnel,
                });
            }
            Err(e) => {
                let msg = format!("{e}");
                warn!("  tier=emergency-tunnel: failed — {msg}");
                tier_attempts.push((ConnectionTier::EmergencyTunnel, msg));
            }
        }

        let summary = tier_attempts
            .iter()
            .map(|(tier, err)| format!("{tier}: {err}"))
            .collect::<Vec<_>>()
            .join("; ");

        Err(LineageRelayError::NoRelayAvailable(format!(
            "H2-16 fallback chain exhausted — {summary}"
        )))
    }

    /// Attempt a direct UDP punch to the peer.
    async fn try_direct_punch(&self, peer_addr: SocketAddr) -> Result<Duration> {
        let start = std::time::Instant::now();

        let bind_addr = if peer_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };

        let socket = tokio::net::UdpSocket::bind(bind_addr)
            .await
            .map_err(|e| LineageRelayError::NetworkError(format!("bind failed: {e}")))?;

        let probe = b"SONGBIRD_PUNCH_PROBE";
        socket
            .send_to(probe, peer_addr)
            .await
            .map_err(|e| LineageRelayError::DirectConnectionFailed(format!("send failed: {e}")))?;

        let mut buf = [0u8; 64];
        match tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await {
            Ok(Ok(_)) => Ok(start.elapsed()),
            Ok(Err(e)) => {
                Err(LineageRelayError::DirectConnectionFailed(format!("recv failed: {e}")))
            }
            Err(_) => Err(LineageRelayError::DirectConnectionFailed(
                "punch probe timed out (2 s)".to_string(),
            )),
        }
    }

    /// Attempt STUN-assisted NAT discovery.
    async fn try_stun_punch(&self, _peer: &NodeId) -> Result<SocketAddr> {
        self.discover_public_address().await
    }

    /// Attempt a TURN allocation via the configured [`TurnClient`].
    async fn try_turn_allocation(
        &self,
        turn_client: &TurnClient,
    ) -> Result<songbird_stun::TurnAllocation> {
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| LineageRelayError::NetworkError(format!("TURN bind: {e}")))?;

        turn_client
            .allocate(&socket)
            .await
            .map_err(|e| LineageRelayError::NetworkError(format!("TURN allocate: {e}")))
    }

    /// Attempt an emergency `cloudflared` tunnel.
    ///
    /// Looks for a `cloudflared` binary on `$PATH`, then starts a quick-tunnel
    /// or named-tunnel session. Returns the tunnel URL on success.
    async fn try_emergency_tunnel(&self) -> Result<String> {
        // Check if cloudflared is available
        let status = tokio::process::Command::new("cloudflared")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await;

        match status {
            Ok(s) if s.success() => {
                debug!("cloudflared binary found, quick-tunnel support available");
                // Full integration requires spawning a tunnel process and
                // parsing the assigned URL. For now, report that the binary
                // exists but tunnel orchestration is pending.
                Err(LineageRelayError::NoRelayAvailable(
                    "cloudflared found but tunnel orchestration not yet wired".to_string(),
                ))
            }
            _ => Err(LineageRelayError::NoRelayAvailable(
                "cloudflared binary not found on $PATH".to_string(),
            )),
        }
    }

    /// Check connection quality across tiers
    ///
    /// # Returns
    ///
    /// Quality metrics for each tier (latency, packet loss, etc.)
    pub async fn check_tier_quality(&self) -> TierQualityReport {
        info!("📊 Checking multi-tier connection quality");

        let mut report = TierQualityReport::default();

        // Check user-provided STUN latency (use first enabled server)
        if let Some(user_stun) = self.config.user_provided.iter().find(|s| s.enabled) {
            let start = std::time::Instant::now();

            if self.stun_client.discover_public_address(&user_stun.address).await.is_ok() {
                let latency = start.elapsed();
                report.user_provided_latency = Some(latency);
                info!("   User-provided STUN latency: {:?}", latency);
            }
        }

        // Check public STUN latency (if enabled, use first server)
        if self.config.public_stun.enabled
            && let Some(public_server) = self.config.public_stun.servers.first()
        {
            let start = std::time::Instant::now();

            if self.stun_client.discover_public_address(&public_server.address).await.is_ok() {
                let latency = start.elapsed();
                report.public_stun_latency = Some(latency);
                info!("   Public STUN latency: {:?}", latency);
            }
        }

        report
    }
}

/// Connection result from the H2-16 fallback chain.
#[derive(Debug, Clone)]
pub enum ConnectionResult {
    /// Direct or STUN-punched connection established.
    Direct {
        /// Peer address (may be reflexive for STUN-punch tier).
        peer_addr: SocketAddr,
        /// Connection latency (zero when latency is not measurable).
        latency: Duration,
        /// Which tier succeeded.
        tier: ConnectionTier,
    },
    /// Relayed connection through genetic lineage.
    Relayed {
        /// Relay session details.
        session: Arc<RelaySession>,
        /// Which relay tier.
        tier: ConnectionTier,
    },
    /// TURN-allocated relay connection (H2-14).
    TurnRelayed {
        /// Relay address allocated by the TURN server.
        relay_addr: SocketAddr,
        /// Which tier.
        tier: ConnectionTier,
    },
    /// Emergency tunnel connection.
    Tunneled {
        /// Tunnel endpoint URL or address.
        endpoint: String,
        /// Which tier.
        tier: ConnectionTier,
    },
}

/// Quality report for each tier
#[derive(Debug, Default)]
pub struct TierQualityReport {
    /// User-provided STUN latency
    pub user_provided_latency: Option<Duration>,
    /// Public STUN latency
    pub public_stun_latency: Option<Duration>,
    /// Genetic lineage relay latency (future)
    pub lineage_relay_latency: Option<Duration>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use songbird_types::config::stun_relay::{StunRelayConfig, StunServerConfig, StunStrategy};

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = StunRelayConfig::default();
        let coordinator = MultiTierCoordinator::new(config.clone());
        assert_eq!(coordinator.config.strategy, config.strategy);
    }

    #[tokio::test]
    async fn test_sovereignty_first_with_empty_config() {
        let config = StunRelayConfig::default();
        let coordinator = MultiTierCoordinator::new(config);

        // Should fail with no servers configured
        let result = coordinator.discover_public_address().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires network access"] // Requires network access
    async fn test_stun_discovery_with_public_servers() {
        let mut config = StunRelayConfig::default();
        config.public_stun.enabled = true;
        config.public_stun.servers.push(StunServerConfig {
            address: "stun.nextcloud.com:3478".to_string(),
            protocol: Default::default(),
            priority: 10,
            enabled: true,
            verified: false,
            vetted: true,
            comment: "Community STUN server".to_string(),
        });

        let coordinator = MultiTierCoordinator::new(config);
        let result = coordinator.discover_public_address().await;

        // Should succeed with real network
        if let Ok(addr) = result {
            println!("Discovered public address: {addr}");
            assert!(addr.port() > 0);
        }
    }

    #[tokio::test]
    async fn test_tier_quality_report_empty() {
        let config = StunRelayConfig::default();
        let coordinator = MultiTierCoordinator::new(config);

        let report = coordinator.check_tier_quality().await;
        assert!(report.user_provided_latency.is_none());
        assert!(report.public_stun_latency.is_none());
    }

    #[tokio::test]
    async fn discover_public_address_lineage_only_errors_without_network() {
        let mut config = StunRelayConfig::default();
        config.strategy = StunStrategy::LineageOnly;
        let coordinator = MultiTierCoordinator::new(config);
        let err = coordinator.discover_public_address().await.expect_err("lineage-only skips STUN");
        assert!(
            err.to_string().contains("LineageOnly") || err.to_string().contains("STUN"),
            "{}",
            err
        );
    }

    #[tokio::test]
    async fn establish_connection_exhausts_fallback_chain() {
        let config = StunRelayConfig::default();
        let coordinator = MultiTierCoordinator::new(config);
        let err = coordinator
            .establish_connection(NodeId::from("p"), None)
            .await
            .expect_err("all tiers should fail in test without network");
        let msg = err.to_string();
        assert!(msg.contains("fallback chain exhausted"), "unexpected: {msg}");
        assert!(msg.contains("direct"), "should mention direct tier: {msg}");
        assert!(msg.contains("stun-punch"), "should mention stun tier: {msg}");
    }

    #[test]
    fn connection_tier_display_all_variants() {
        use super::ConnectionTier;
        assert_eq!(ConnectionTier::Direct.to_string(), "direct");
        assert_eq!(ConnectionTier::StunPunch.to_string(), "stun-punch");
        assert_eq!(ConnectionTier::LineageRelay.to_string(), "lineage-relay");
        assert_eq!(ConnectionTier::TurnRelay.to_string(), "turn-relay");
        assert_eq!(ConnectionTier::EmergencyTunnel.to_string(), "emergency-tunnel");
    }

    #[test]
    fn tier_quality_report_default_is_empty() {
        let r = TierQualityReport::default();
        assert!(r.user_provided_latency.is_none());
        assert!(r.lineage_relay_latency.is_none());
    }
}
