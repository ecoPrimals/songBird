//! Lineage relay coordinator - main entry point
//!
//! Evolution beyond NAT/STUN/TURN

use crate::birdsong::BirdSongBroadcaster;
use crate::error::{LineageRelayError, Result};
use crate::relay::{RelayAuthority, RelayDiscovery};
use crate::session::{ConnectionSession, DirectConnection, RelayedConnection};
use crate::types::{ConnectionEndpoint, NodeId};
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
    /// Bind address for BirdSong
    pub birdsong_bind: SocketAddr,
    /// Broadcast address for BirdSong
    pub birdsong_broadcast: SocketAddr,
    /// My relay address (if offering relay service)
    pub my_relay_address: Option<SocketAddr>,
    /// Timeout for direct connection attempts
    pub direct_timeout: Duration,
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
        }
    }
}

/// Lineage relay coordinator
///
/// Main entry point for lineage-based connectivity
pub struct LineageRelayCoordinator {
    config: LineageRelayConfig,
    broadcaster: Arc<BirdSongBroadcaster>,
    relay_discovery: Arc<RelayDiscovery>,
}

impl LineageRelayCoordinator {
    /// Create new lineage relay coordinator
    ///
    /// # Errors
    ///
    /// Returns error if BirdSong broadcaster cannot be created
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

        Ok(Self {
            config,
            broadcaster,
            relay_discovery,
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

    /// Try direct connection (legacy "STUN" concept)
    ///
    /// # Errors
    ///
    /// Returns error if direct connection fails (not unexpected)
    async fn try_direct_connection(
        &self,
        peer: &NodeId,
        address: SocketAddr,
    ) -> Result<DirectConnection> {
        // In real implementation, would attempt UDP hole punching or direct TCP
        // For now, simulate with timeout
        timeout(self.config.direct_timeout, async {
            // Simulate connection attempt
            tokio::time::sleep(Duration::from_millis(100)).await;

            // For mock: always fail (to demonstrate relay)
            // In real implementation: try actual connection
            Err(LineageRelayError::DirectConnectionFailed(format!(
                "Could not establish direct connection to {}",
                address
            )))
        })
        .await
        .map_err(|_| {
            LineageRelayError::Timeout("Direct connection attempt timed out".to_string())
        })?
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
        let relay_discovery = self.relay_discovery.clone();
        let my_relay_address = relay_address;

        tokio::spawn(async move {
            loop {
                // Get relay request messages
                // In real implementation, would continuously process requests
                tokio::time::sleep(Duration::from_secs(1)).await;
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
