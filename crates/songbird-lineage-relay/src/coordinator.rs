//! Lineage relay coordinator - main entry point
//!
//! Evolution beyond NAT/STUN/TURN

use crate::birdsong::BirdSongBroadcaster;
use crate::error::{LineageRelayError, Result};
use crate::multi_tier_coordinator::MultiTierCoordinator;
use crate::relay::{RelayAuthority, RelayDiscovery};
use crate::session::{ConnectionSession, DirectConnection, RelayedConnection};
use crate::types::NodeId;
use crate::udp_hole_punch::{create_hole_punch_socket, udp_hole_punch, HolePunchConfig};
use songbird_types::config::stun_relay::StunRelayConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Configuration for lineage relay coordinator
#[derive(Debug, Clone)]
pub struct LineageRelayConfig {
    /// My node ID
    pub my_id: NodeId,
    /// Bind address for `BirdSong`
    pub birdsong_bind: SocketAddr,
    /// Broadcast address for `BirdSong`
    pub birdsong_broadcast: SocketAddr,
    /// My relay address (if offering relay service)
    pub my_relay_address: Option<SocketAddr>,
    /// Timeout for direct connection attempts
    pub direct_timeout: Duration,
    /// Multi-tier STUN/relay configuration (optional)
    pub stun_relay: Option<StunRelayConfig>,
}

impl Default for LineageRelayConfig {
    fn default() -> Self {
        // These are well-known IPv4 addresses that will always parse successfully
        // 0.0.0.0:42424 = bind to all interfaces
        // 255.255.255.255:42424 = broadcast address
        Self {
            my_id: NodeId::from("default-node"),
            birdsong_bind: "0.0.0.0:42424"
                .parse()
                .expect("hardcoded IPv4 bind address should always parse"),
            birdsong_broadcast: "255.255.255.255:42424"
                .parse()
                .expect("hardcoded IPv4 broadcast address should always parse"),
            my_relay_address: None,
            direct_timeout: Duration::from_secs(5),
            stun_relay: None, // Disabled by default (sovereignty-first: lineage only)
        }
    }
}

/// Lineage relay coordinator
///
/// Main entry point for lineage-based connectivity with optional multi-tier STUN/relay
pub struct LineageRelayCoordinator {
    config: LineageRelayConfig,
    broadcaster: Arc<BirdSongBroadcaster>,
    relay_discovery: Arc<RelayDiscovery>,
    multi_tier: Option<Arc<MultiTierCoordinator>>,
}

impl LineageRelayCoordinator {
    /// Create new lineage relay coordinator
    ///
    /// # Errors
    ///
    /// Returns error if `BirdSong` broadcaster cannot be created
    pub async fn new(
        config: LineageRelayConfig,
        broadcaster: Arc<BirdSongBroadcaster>,
        relay_authority: Arc<dyn RelayAuthority>,
    ) -> Result<Self> {
        let relay_discovery = Arc::new(RelayDiscovery::new(
            broadcaster.clone(),
            relay_authority,
            config.my_id.clone(),
        ));

        // Create multi-tier coordinator if STUN/relay config provided
        let multi_tier = config
            .stun_relay
            .as_ref()
            .map(|stun_config| Arc::new(MultiTierCoordinator::new(stun_config.clone())));

        if multi_tier.is_some() {
            info!("🌐 Multi-tier STUN/relay enabled");
        } else {
            info!("🛡️  Lineage-only mode (maximum sovereignty)");
        }

        Ok(Self {
            config,
            broadcaster,
            relay_discovery,
            multi_tier,
        })
    }

    /// Establish connection to peer (direct or relayed)
    ///
    /// This is the main entry point for connectivity. It:
    /// 1. Attempts direct connection first (legacy STUN concepts)
    /// 2. Falls back to lineage-based relay if direct fails
    ///
    /// # Errors
    ///
    /// Returns error if neither direct nor relayed connection succeeds
    pub async fn establish_connection(
        &self,
        peer: NodeId,
        peer_address: SocketAddr,
    ) -> Result<ConnectionSession> {
        info!("Establishing connection to {}", peer);

        // Phase 1: Attempt direct connection
        debug!("Attempting direct connection to {}...", peer_address);
        match self.try_direct_connection(&peer, peer_address).await {
            Ok(conn) => {
                info!("✅ Direct connection established to {}", peer);
                return Ok(ConnectionSession::Direct(conn));
            }
            Err(e) => {
                debug!("Direct connection failed: {}", e);
                info!("📡 Falling back to lineage relay...");
            }
        }

        // Phase 2: Request lineage relay
        let relay_session =
            self.relay_discovery.request_relay(peer.clone(), Some(peer_address)).await?;

        info!("✅ Relayed connection established through {}", relay_session.relay_node);

        Ok(ConnectionSession::Relayed(RelayedConnection::new(relay_session)))
    }

    /// Try direct connection via UDP hole punching with optional STUN discovery
    ///
    /// **EVOLVED FROM MOCK** (Jan 28, 2026): Now implements real UDP hole punching + multi-tier STUN!
    ///
    /// # Strategy
    ///
    /// 1. Optionally discover public address via STUN (if multi-tier enabled)
    /// 2. Create local UDP socket
    /// 3. Attempt UDP hole punch to peer's public address
    /// 4. Use simultaneous open technique for NAT traversal
    ///
    /// # Errors
    ///
    /// Returns error if direct connection fails (expected for symmetric NAT).
    /// Caller should fall back to lineage relay on failure.
    ///
    /// # Success Rate by NAT Type
    ///
    /// - Full Cone NAT: ~95%
    /// - Restricted Cone NAT: ~90%
    /// - Port-Restricted Cone NAT: ~80%
    /// - Symmetric NAT: ~30% (relay recommended)
    async fn try_direct_connection(
        &self,
        peer: &NodeId,
        address: SocketAddr,
    ) -> Result<DirectConnection> {
        debug!("🔗 Attempting UDP hole punch to peer: {}", peer);
        debug!("   Peer address: {}", address);

        // Optional: Discover our public address via STUN if multi-tier enabled
        if let Some(multi_tier) = &self.multi_tier {
            match multi_tier.discover_public_address().await {
                Ok(my_public_addr) => {
                    info!("   Discovered my public address via STUN: {}", my_public_addr);
                    // In production, we'd exchange this with peer via BirdSong
                    // For now, proceed with hole punch using known peer address
                }
                Err(e) => {
                    debug!("   STUN discovery failed (non-fatal): {}", e);
                    // Continue with hole punch anyway
                }
            }
        }

        // Create local UDP socket for hole punching
        let socket = create_hole_punch_socket(None).await?;
        let local_addr = socket.local_addr().map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to get local address: {e}"))
        })?;

        debug!("   Local socket bound: {}", local_addr);

        // Configure hole punch (use coordinator timeout)
        let config = HolePunchConfig {
            max_attempts: 10,
            attempt_timeout: Duration::from_millis(200),
            attempt_delay: Duration::from_millis(50),
            total_timeout: self.config.direct_timeout,
        };

        // Attempt UDP hole punch with overall timeout
        match timeout(self.config.direct_timeout, async {
            udp_hole_punch(socket, peer.clone(), address, config).await
        })
        .await
        {
            Ok(Ok(conn)) => {
                info!("✅ Direct UDP connection established via hole punch");
                Ok(conn)
            }
            Ok(Err(e)) => {
                warn!("⚠️  UDP hole punch failed: {}", e);
                Err(e)
            }
            Err(_) => {
                warn!("⏱️  Direct connection timeout after {:?}", self.config.direct_timeout);
                Err(LineageRelayError::Timeout("Direct connection attempt timed out".to_string()))
            }
        }
    }

    /// Get multi-tier connection quality report (if enabled)
    ///
    /// # Returns
    ///
    /// Quality metrics for each STUN/relay tier, or None if multi-tier disabled.
    pub async fn get_tier_quality(
        &self,
    ) -> Option<crate::multi_tier_coordinator::TierQualityReport> {
        if let Some(multi_tier) = &self.multi_tier {
            Some(multi_tier.check_tier_quality().await)
        } else {
            None
        }
    }

    /// Start offering relay service (for ancestors)
    ///
    /// # Errors
    ///
    /// Returns error if relay service cannot be started
    pub async fn start_relay_service(&self) -> Result<()> {
        let relay_address = self.config.my_relay_address.ok_or_else(|| {
            LineageRelayError::ConfigError("No relay address configured".to_string())
        })?;

        info!("Starting relay service on {}", relay_address);

        // Listen for relay requests in background
        let _relay_discovery = self.relay_discovery.clone();
        let _my_relay_address = relay_address;

        // 🚨 DEEP DEBT (v3.10.4 - Jan 6, 2026): Polling loop with sleep
        //
        // CURRENT: Polls every 1 second (blocking, wasteful, high latency)
        // SHOULD BE: Event-driven with mpsc channel
        //
        // Modern Rust Solution:
        // ```rust
        // let (request_tx, mut request_rx) = tokio::sync::mpsc::channel(100);
        // tokio::spawn(async move {
        //     while let Some(request) = request_rx.recv().await {
        //         process_relay_request(request).await;
        //     }
        // });
        // ```
        //
        // Benefits:
        // - Zero latency (instant processing)
        // - No CPU waste (event-driven)
        // - Proper backpressure (bounded queue)
        // - Clean shutdown (channel close)
        //
        // Status: INCOMPLETE - This module needs architectural evolution
        // Priority: MEDIUM (relay functionality is experimental)
        tokio::spawn(async move {
            loop {
                // TODO: Replace with mpsc channel-based request queue
                tokio::time::sleep(Duration::from_secs(1)).await; // ❌ POLLING ANTI-PATTERN
                                                                  // Process any pending relay requests here
            }
        });

        Ok(())
    }

    /// Get relay statistics
    pub async fn relay_stats(&self) -> Vec<(NodeId, u64)> {
        let sessions = self.relay_discovery.active_sessions().await;
        let mut stats = Vec::new();

        for session in sessions {
            stats.push((session.requester.clone(), session.stats().await));
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beardog::{MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority};
    use crate::birdsong::BirdSongBroadcaster;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let lineage_provider = Arc::new(MockLineageProvider::new());
        let crypto =
            Arc::new(MockBirdSongCrypto::new(lineage_provider.clone(), "node-1".to_string()));
        let relay_authority = Arc::new(MockRelayAuthority::new(lineage_provider));

        let broadcaster = Arc::new(
            BirdSongBroadcaster::new(
                crypto,
                NodeId::from("node-1"),
                "127.0.0.1:42424".parse().unwrap(),
                "255.255.255.255:42424".parse().unwrap(),
            )
            .await
            .unwrap(),
        );

        let config = LineageRelayConfig {
            my_id: NodeId::from("node-1"),
            ..Default::default()
        };

        let coordinator =
            LineageRelayCoordinator::new(config, broadcaster, relay_authority).await.unwrap();

        assert_eq!(coordinator.config.my_id.0, "node-1");
    }

    #[tokio::test]
    async fn test_direct_connection_attempt() {
        let lineage_provider = Arc::new(MockLineageProvider::new());
        let crypto =
            Arc::new(MockBirdSongCrypto::new(lineage_provider.clone(), "node-1".to_string()));
        let relay_authority = Arc::new(MockRelayAuthority::new(lineage_provider));

        let broadcaster = Arc::new(
            BirdSongBroadcaster::new(
                crypto,
                NodeId::from("node-1"),
                "127.0.0.1:42425".parse().unwrap(),
                "255.255.255.255:42425".parse().unwrap(),
            )
            .await
            .unwrap(),
        );

        let config = LineageRelayConfig {
            my_id: NodeId::from("node-1"),
            direct_timeout: Duration::from_millis(500),
            ..Default::default()
        };

        let coordinator =
            LineageRelayCoordinator::new(config, broadcaster, relay_authority).await.unwrap();

        // Try direct connection (will fail in mock)
        let result = coordinator
            .try_direct_connection(&NodeId::from("peer"), "127.0.0.1:8080".parse().unwrap())
            .await;

        assert!(result.is_err());
    }
}
