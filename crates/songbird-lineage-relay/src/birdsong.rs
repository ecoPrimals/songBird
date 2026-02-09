//! `BirdSong` - Lineage-gated broadcast system
//!
//! ## Concept
//!
//! "A broadcast that is obvious to family and noise otherwise"
//!
//! - Family (lineage) can decrypt messages
//! - Non-family sees encrypted noise
//! - Privacy through selective intelligibility

use crate::error::Result;
use crate::types::NodeId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Hint for which lineage members should receive message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineageHint {
    /// Only my direct parent
    DirectParent,
    /// All ancestors (parent, grandparent, etc.)
    DirectAncestors,
    /// My direct children
    DirectChildren,
    /// All descendants (children, grandchildren, etc.)
    AllDescendants,
    /// Specific ancestor by ID
    SpecificAncestor(NodeId),
}

/// `BirdSong` message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BirdSongType {
    /// Presence announcement
    Presence,
    /// Capability announcement
    CapabilityAnnouncement,
    /// Transport endpoint announcement
    TransportAnnouncement,
    /// Relay request (need help connecting)
    RelayRequest,
    /// Relay offer (can help you connect)
    RelayOffer,
    /// Federation event
    FederationEvent,
    /// Custom application message
    Custom(String),
}

/// `BirdSong` message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongMessage {
    /// Protocol version
    pub version: u8,
    /// Message type
    pub message_type: BirdSongType,
    /// Sender (encrypted for family)
    pub sender: NodeId,
    /// Lineage hint (who should receive)
    pub lineage_hint: LineageHint,
    /// Payload (encrypted by `BearDog`)
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

/// `BirdSong` crypto provider (implemented by `BearDog`)
#[async_trait]
pub trait BirdSongCrypto: Send + Sync {
    /// Encrypt message for lineage
    async fn encrypt_for_lineage(&self, message: &[u8], hint: LineageHint) -> Result<Vec<u8>>;

    /// Decrypt `BirdSong` message (returns None if not in lineage)
    async fn decrypt_birdsong(&self, encrypted: &[u8], sender: &NodeId) -> Result<Option<Vec<u8>>>;
}

/// `BirdSong` broadcaster
pub struct BirdSongBroadcaster {
    socket: Arc<UdpSocket>,
    crypto: Arc<dyn BirdSongCrypto>,
    my_id: NodeId,
    broadcast_addr: SocketAddr,
    received_messages: Arc<RwLock<Vec<BirdSongMessage>>>,
    /// Notify waiters when a new message arrives (replaces polling anti-pattern)
    message_notify: Arc<tokio::sync::Notify>,
}

impl BirdSongBroadcaster {
    /// Create new `BirdSong` broadcaster
    ///
    /// # Errors
    ///
    /// Returns error if UDP socket cannot be bound
    pub async fn new(
        crypto: Arc<dyn BirdSongCrypto>,
        my_id: NodeId,
        bind_addr: SocketAddr,
        broadcast_addr: SocketAddr,
    ) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.set_broadcast(true)?;

        info!("BirdSong broadcaster bound to {}", bind_addr);

        Ok(Self {
            socket: Arc::new(socket),
            crypto,
            my_id,
            broadcast_addr,
            received_messages: Arc::new(RwLock::new(Vec::new())),
            message_notify: Arc::new(tokio::sync::Notify::new()),
        })
    }

    /// Broadcast a `BirdSong` message
    ///
    /// # Errors
    ///
    /// Returns error if encryption or broadcasting fails
    pub async fn broadcast(
        &self,
        message_type: BirdSongType,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> Result<()> {
        debug!("Broadcasting BirdSong message: {:?}", message_type);

        // Encrypt payload for lineage
        let encrypted_payload =
            self.crypto.encrypt_for_lineage(payload, lineage_hint.clone()).await?;

        // Create BirdSong message
        let message = BirdSongMessage {
            version: 1,
            message_type,
            sender: self.my_id.clone(),
            lineage_hint,
            payload: encrypted_payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Serialize message
        let serialized = serde_json::to_vec(&message)?;

        // Broadcast UDP packet
        let sent = self.socket.send_to(&serialized, self.broadcast_addr).await?;
        debug!("Broadcast {} bytes to {}", sent, self.broadcast_addr);

        Ok(())
    }

    /// Listen for incoming `BirdSong` messages
    ///
    /// # Errors
    ///
    /// Returns error if receiving or decryption fails
    pub async fn listen(&self) -> Result<()> {
        let mut buf = vec![0u8; 65536];

        loop {
            let (len, _addr) = self.socket.recv_from(&mut buf).await?;

            // Deserialize message
            let message: BirdSongMessage = match serde_json::from_slice(&buf[..len]) {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("Failed to deserialize BirdSong message: {}", e);
                    continue;
                }
            };

            // Skip our own messages
            if message.sender == self.my_id {
                continue;
            }

            // Try to decrypt (will succeed only if we're in lineage)
            match self.crypto.decrypt_birdsong(&message.payload, &message.sender).await? {
                Some(decrypted_payload) => {
                    info!("Received BirdSong from family: {:?}", message.message_type);

                    // Store decrypted message
                    let mut messages = self.received_messages.write().await;
                    let mut decrypted_message = message;
                    decrypted_message.payload = decrypted_payload;
                    messages.push(decrypted_message);
                    drop(messages); // Release lock before notifying

                    // Wake any waiters (replaces polling anti-pattern)
                    self.message_notify.notify_waiters();
                }
                None => {
                    // Not in lineage - just noise
                    debug!("Received BirdSong noise (not in lineage)");
                }
            }
        }
    }

    /// Get received messages (clears the buffer)
    pub async fn get_messages(&self) -> Vec<BirdSongMessage> {
        let mut messages = self.received_messages.write().await;
        std::mem::take(&mut *messages)
    }

    /// Get messages of specific type
    pub async fn get_messages_by_type(&self, msg_type: BirdSongType) -> Vec<BirdSongMessage> {
        let mut messages = self.received_messages.write().await;
        let (matching, remaining): (Vec<_>, Vec<_>) = messages.drain(..).partition(|msg| {
            std::mem::discriminant(&msg.message_type) == std::mem::discriminant(&msg_type)
        });
        *messages = remaining;
        matching
    }

    /// Wait for a message of specific type (event-driven, zero polling)
    ///
    /// This replaces the previous polling anti-pattern where callers would
    /// `sleep(100ms)` in a loop checking `get_messages_by_type()`.
    ///
    /// Now callers await this method which is woken instantly when a new
    /// message arrives, giving zero latency and zero CPU waste.
    pub async fn wait_for_message_by_type(
        &self,
        msg_type: BirdSongType,
        timeout_duration: std::time::Duration,
    ) -> Result<Vec<BirdSongMessage>> {
        tokio::time::timeout(timeout_duration, async {
            loop {
                // Check for matching messages
                let matching = self.get_messages_by_type(msg_type.clone()).await;
                if !matching.is_empty() {
                    return Ok(matching);
                }
                // Wait for notification (instant wake, no polling)
                self.message_notify.notified().await;
            }
        })
        .await
        .map_err(|_| crate::error::LineageRelayError::NoRelayAvailable(
            "Timed out waiting for message".to_string()
        ))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCrypto;

    #[async_trait]
    impl BirdSongCrypto for MockCrypto {
        async fn encrypt_for_lineage(&self, message: &[u8], _hint: LineageHint) -> Result<Vec<u8>> {
            // Mock: just prepend "ENCRYPTED:" for testing
            let mut encrypted = b"ENCRYPTED:".to_vec();
            encrypted.extend_from_slice(message);
            Ok(encrypted)
        }

        async fn decrypt_birdsong(
            &self,
            encrypted: &[u8],
            _sender: &NodeId,
        ) -> Result<Option<Vec<u8>>> {
            // Mock: remove "ENCRYPTED:" prefix
            if encrypted.starts_with(b"ENCRYPTED:") {
                Ok(Some(encrypted[10..].to_vec()))
            } else {
                Ok(None)
            }
        }
    }

    #[tokio::test]
    async fn test_birdsong_message_creation() {
        let message = BirdSongMessage {
            version: 1,
            message_type: BirdSongType::Presence,
            sender: NodeId::from("test-node"),
            lineage_hint: LineageHint::DirectAncestors,
            payload: b"hello".to_vec(),
            timestamp: 12345,
        };

        assert_eq!(message.version, 1);
        assert_eq!(message.sender.0, "test-node");
    }

    #[tokio::test]
    async fn test_mock_encryption() {
        let crypto = MockCrypto;
        let message = b"test message";

        let encrypted =
            crypto.encrypt_for_lineage(message, LineageHint::DirectAncestors).await.unwrap();
        assert!(encrypted.starts_with(b"ENCRYPTED:"));

        let decrypted = crypto.decrypt_birdsong(&encrypted, &NodeId::from("sender")).await.unwrap();
        assert_eq!(decrypted, Some(message.to_vec()));
    }
}
