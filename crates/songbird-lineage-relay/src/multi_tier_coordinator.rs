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
use crate::relay::RelaySession;
use crate::types::NodeId;
use songbird_stun::StunClient;
use songbird_types::config::stun_relay::{StunRelayConfig, StunStrategy};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Multi-tier coordinator for NAT traversal
pub struct MultiTierCoordinator {
    config: StunRelayConfig,
    stun_client: Arc<StunClient>,
}

impl MultiTierCoordinator {
    /// Create new multi-tier coordinator
    #[must_use]
    pub fn new(config: StunRelayConfig) -> Self {
        let stun_client = Arc::new(StunClient::with_timeout(Duration::from_secs(3)));
        Self {
            config,
            stun_client,
        }
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

    /// Establish connection using multi-tier strategy
    ///
    /// # Strategy
    ///
    /// 1. Try direct connection (UDP hole punch)
    /// 2. Fall back to genetic lineage relay
    /// 3. Use STUN for public address discovery if needed
    ///
    /// # Errors
    ///
    /// Returns error if all connection methods fail.
    pub async fn establish_connection(
        &self,
        _peer: NodeId,
        _peer_addr: Option<SocketAddr>,
    ) -> Result<ConnectionResult> {
        info!("Multi-tier connection establishment for peer");

        Err(LineageRelayError::Other(
            "Multi-tier coordinator requires LineageRelayCoordinator injection — \
             direct UDP punch, relay fallback, and STUN discovery are handled by \
             the orchestrator's connection pipeline"
                .to_string(),
        ))
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

/// Connection result from multi-tier coordinator
#[derive(Debug, Clone)]
pub enum ConnectionResult {
    /// Direct connection established
    Direct {
        /// Peer address
        peer_addr: SocketAddr,
        /// Connection latency
        latency: Duration,
    },
    /// Relayed connection through genetic lineage
    Relayed {
        /// Relay session details
        session: Arc<RelaySession>,
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
    async fn establish_connection_returns_configuration_error_string() {
        let config = StunRelayConfig::default();
        let coordinator = MultiTierCoordinator::new(config);
        let err =
            coordinator.establish_connection(NodeId::from("p"), None).await.expect_err("not wired");
        assert!(err.to_string().contains("orchestrator") || err.to_string().contains("pipeline"));
    }

    #[test]
    fn tier_quality_report_default_is_empty() {
        let r = TierQualityReport::default();
        assert!(r.user_provided_latency.is_none());
        assert!(r.lineage_relay_latency.is_none());
    }
}
