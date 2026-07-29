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

#![forbid(unsafe_code)]

mod strategy;
mod tiers;

#[path = "tests.rs"]
#[cfg(test)]
mod tests;

use crate::error::{LineageRelayError, Result};
use crate::relay::{RelayDiscovery, RelaySession};
use crate::types::NodeId;
use songbird_stun::{StunClient, TurnClient};
use songbird_types::config::stun_relay::StunRelayConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

pub use tiers::CloudflaredTunnel;

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
    pub(crate) config: StunRelayConfig,
    pub(crate) stun_client: Arc<StunClient>,
    pub(crate) relay_discovery: Option<Arc<RelayDiscovery>>,
    pub(crate) turn_client: Option<Arc<TurnClient>>,
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
        self.discover_by_strategy(self.config.strategy.into()).await
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
            tier_attempts.push((ConnectionTier::Direct, String::from("no peer address available")));
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
            tier_attempts.push((
                ConnectionTier::LineageRelay,
                String::from("relay discovery not configured"),
            ));
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
                .push((ConnectionTier::TurnRelay, String::from("TURN client not configured")));
        }

        // --- Tier 5: Emergency tunnel (cloudflared) — opt-in only ---
        if self.config.emergency_tunnel_enabled {
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
        } else {
            tier_attempts.push((
                ConnectionTier::EmergencyTunnel,
                String::from("emergency tunnel disabled (sovereignty-first default)"),
            ));
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
