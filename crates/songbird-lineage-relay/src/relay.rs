//! Relay discovery and session management
//!
//! Evolution beyond TURN: Ancestors relay for descendants

use crate::birdsong::{BirdSongBroadcaster, BirdSongType, LineageHint};
use crate::error::{LineageRelayError, Result};
use crate::types::{ConnectionEndpoint, MaskingLevel, NodeId, RelayAuthorization};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Relay authority provider (implemented by BearDog)
#[async_trait]
pub trait RelayAuthority: Send + Sync {
    /// Authorize relay service for requester
    async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization>;

    /// Get masking level based on lineage relationship
    async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel>;
}

/// Relay request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayRequest {
    pub requester: NodeId,
    pub target: NodeId,
    pub target_address: Option<SocketAddr>,
    pub timestamp: SystemTime,
}

/// Relay offer message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayOffer {
    pub relay_node: NodeId,
    pub relay_address: SocketAddr,
    pub authorization: RelayAuthorization,
    pub timestamp: SystemTime,
}

/// Active relay session
#[derive(Debug, Clone)]
pub struct RelaySession {
    pub session_id: uuid::Uuid,
    pub relay_node: NodeId,
    pub relay_address: SocketAddr,
    pub requester: NodeId,
    pub target: NodeId,
    pub masking_level: MaskingLevel,
    pub established_at: SystemTime,
    pub bytes_relayed: Arc<Mutex<u64>>,
}

impl RelaySession {
    /// Create new relay session
    #[must_use]
    pub fn new(
        relay_node: NodeId,
        relay_address: SocketAddr,
        requester: NodeId,
        target: NodeId,
        masking_level: MaskingLevel,
    ) -> Self {
        Self {
            session_id: uuid::Uuid::new_v4(),
            relay_node,
            relay_address,
            requester,
            target,
            masking_level,
            established_at: SystemTime::now(),
            bytes_relayed: Arc::new(Mutex::new(0)),
        }
    }

    /// Send data through relay
    ///
    /// # Errors
    ///
    /// Returns error if sending fails
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        // In real implementation, this would send through UDP socket to relay
        debug!(
            "Sending {} bytes through relay {} (masked: {:?})",
            data.len(),
            self.relay_node,
            self.masking_level
        );

        let mut bytes = self.bytes_relayed.lock().await;
        *bytes += data.len() as u64;

        Ok(())
    }

    /// Get relay statistics
    pub async fn stats(&self) -> u64 {
        *self.bytes_relayed.lock().await
    }
}

/// Relay discovery system
pub struct RelayDiscovery {
    broadcaster: Arc<BirdSongBroadcaster>,
    relay_authority: Arc<dyn RelayAuthority>,
    my_id: NodeId,
    active_sessions: Arc<RwLock<Vec<RelaySession>>>,
}

impl RelayDiscovery {
    /// Create new relay discovery
    #[must_use]
    pub fn new(
        broadcaster: Arc<BirdSongBroadcaster>,
        relay_authority: Arc<dyn RelayAuthority>,
        my_id: NodeId,
    ) -> Self {
        Self {
            broadcaster,
            relay_authority,
            my_id,
            active_sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Request relay for target peer
    ///
    /// # Errors
    ///
    /// Returns error if no relay is available
    pub async fn request_relay(
        &self,
        target: NodeId,
        target_address: Option<SocketAddr>,
    ) -> Result<RelaySession> {
        info!("Requesting relay for target: {}", target);

        // Create relay request
        let request = RelayRequest {
            requester: self.my_id.clone(),
            target: target.clone(),
            target_address,
            timestamp: SystemTime::now(),
        };

        // Serialize request
        let payload = serde_json::to_vec(&request)?;

        // Broadcast to ancestors (only they can decrypt)
        self.broadcaster
            .broadcast(
                BirdSongType::RelayRequest,
                &payload,
                LineageHint::DirectAncestors,
            )
            .await?;

        debug!("Relay request broadcast, waiting for offers...");

        // Wait for relay offers
        let offer = self.wait_for_relay_offer(Duration::from_secs(5)).await?;

        // Create relay session
        let session = RelaySession::new(
            offer.relay_node,
            offer.relay_address,
            self.my_id.clone(),
            target,
            offer.authorization.masking_level,
        );

        // Store active session
        self.active_sessions.write().await.push(session.clone());

        info!("Relay session established with {}", session.relay_node);

        Ok(session)
    }

    /// Wait for relay offer (from ancestors)
    async fn wait_for_relay_offer(&self, duration: Duration) -> Result<RelayOffer> {
        timeout(duration, async {
            loop {
                // Check for relay offer messages
                let messages = self
                    .broadcaster
                    .get_messages_by_type(BirdSongType::RelayOffer)
                    .await;

                for msg in messages {
                    // Deserialize offer
                    if let Ok(offer) = serde_json::from_slice::<RelayOffer>(&msg.payload) {
                        if offer.authorization.authorized {
                            return Ok(offer);
                        }
                    }
                }

                // Wait a bit before checking again
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| LineageRelayError::NoRelayAvailable("No relay offers received".to_string()))?
    }

    /// Offer relay service (as ancestor)
    ///
    /// # Errors
    ///
    /// Returns error if authorization fails
    pub async fn offer_relay(
        &self,
        request: RelayRequest,
        my_relay_address: SocketAddr,
    ) -> Result<()> {
        info!("Considering relay offer for {}", request.requester);

        // Verify authorization through BearDog
        let authorization = self
            .relay_authority
            .authorize_relay(&self.my_id, &request.requester)
            .await?;

        if !authorization.authorized {
            warn!("Relay denied for {}", request.requester);
            return Err(LineageRelayError::RelayDenied(
                "Not authorized to relay for this node".to_string(),
            ));
        }

        // Create relay offer
        let offer = RelayOffer {
            relay_node: self.my_id.clone(),
            relay_address: my_relay_address,
            authorization,
            timestamp: SystemTime::now(),
        };

        // Serialize offer
        let payload = serde_json::to_vec(&offer)?;

        // Broadcast offer (encrypted for requester)
        let requester_id = request.requester.clone();
        self.broadcaster
            .broadcast(
                BirdSongType::RelayOffer,
                &payload,
                LineageHint::SpecificAncestor(request.requester),
            )
            .await?;

        info!("Relay offer sent to {}", requester_id);

        Ok(())
    }

    /// Get active relay sessions
    pub async fn active_sessions(&self) -> Vec<RelaySession> {
        self.active_sessions.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRelayAuthority;

    #[async_trait]
    impl RelayAuthority for MockRelayAuthority {
        async fn authorize_relay(
            &self,
            relay_node: &NodeId,
            requester: &NodeId,
        ) -> Result<RelayAuthorization> {
            Ok(RelayAuthorization {
                relay_node: relay_node.clone(),
                requester: requester.clone(),
                authorized: true,
                masking_level: MaskingLevel::Masked,
                ttl_seconds: 300,
                issued_at: SystemTime::now(),
                audit_token: "mock_token".to_string(),
            })
        }

        async fn determine_masking(
            &self,
            _relay_node: &NodeId,
            _requester: &NodeId,
        ) -> Result<MaskingLevel> {
            Ok(MaskingLevel::Masked)
        }
    }

    #[tokio::test]
    async fn test_relay_session_creation() {
        let session = RelaySession::new(
            NodeId::from("relay-1"),
            "127.0.0.1:8080".parse().unwrap(),
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        );

        assert_eq!(session.relay_node.0, "relay-1");
        assert_eq!(session.masking_level, MaskingLevel::Masked);
    }

    #[tokio::test]
    async fn test_relay_session_send() {
        let session = RelaySession::new(
            NodeId::from("relay-1"),
            "127.0.0.1:8080".parse().unwrap(),
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        );

        session.send(b"test data").await.unwrap();
        assert_eq!(session.stats().await, 9);
    }

    #[tokio::test]
    async fn test_mock_relay_authorization() {
        let authority = MockRelayAuthority;
        let auth = authority
            .authorize_relay(&NodeId::from("relay"), &NodeId::from("requester"))
            .await
            .unwrap();

        assert!(auth.authorized);
        assert_eq!(auth.masking_level, MaskingLevel::Masked);
    }
}

