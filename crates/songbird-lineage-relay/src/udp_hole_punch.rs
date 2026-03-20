// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! UDP Hole Punching Implementation
//!
//! **Pure Rust | Zero Unsafe Code | Modern Async**
//!
//! Implements UDP hole punching for direct P2P connections through NAT.
//! Uses simultaneous open technique coordinated via genetic lineage.
//!
//! ## How It Works
//!
//! 1. Both peers discover their public addresses (via STUN)
//! 2. Exchange addresses via `BirdSong` or existing secure channel
//! 3. Both peers send UDP packets simultaneously to each other
//! 4. NATs create temporary "holes" in the mapping
//! 5. Direct P2P connection established!
//!
//! ## NAT Traversal Success Rates
//!
//! - Full Cone NAT: ~95% success
//! - Restricted Cone NAT: ~90% success
//! - Port-Restricted Cone NAT: ~80% success
//! - Symmetric NAT: ~30% success (requires relay fallback)

use crate::error::{LineageRelayError, Result};
use crate::session::DirectConnection;
use crate::types::NodeId;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

/// UDP hole punch configuration
#[derive(Debug, Clone)]
pub struct HolePunchConfig {
    /// Maximum attempts for hole punching
    pub max_attempts: u32,

    /// Timeout per attempt
    pub attempt_timeout: Duration,

    /// Delay between attempts
    pub attempt_delay: Duration,

    /// Overall timeout for entire hole punch process
    pub total_timeout: Duration,
}

impl Default for HolePunchConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            attempt_timeout: Duration::from_millis(200),
            attempt_delay: Duration::from_millis(50),
            total_timeout: Duration::from_secs(5),
        }
    }
}

/// Attempt UDP hole punching to establish direct connection
///
/// # Arguments
///
/// * `local_socket` - Pre-bound local UDP socket
/// * `peer_id` - Peer's node ID
/// * `peer_addr` - Peer's public address (discovered via STUN or lineage)
/// * `config` - Hole punch configuration
///
/// # Returns
///
/// Direct connection on success, error if hole punch fails.
///
/// # Strategy
///
/// Uses simultaneous open technique:
/// 1. Both peers send "PUNCH" packets to each other's public address
/// 2. NATs observe outbound traffic and create temporary mappings
/// 3. When packets cross, NATs allow inbound traffic through the "hole"
/// 4. Direct P2P connection established
///
/// # Note
///
/// Success rate depends on NAT type. Symmetric NAT has lower success rate
/// and may require genetic lineage relay fallback.
pub async fn udp_hole_punch(
    local_socket: UdpSocket,
    peer_id: NodeId,
    peer_addr: SocketAddr,
    config: HolePunchConfig,
) -> Result<DirectConnection> {
    info!("🔗 Attempting UDP hole punch to {} ({})", peer_id, peer_addr);
    debug!(
        "   Local address: {}",
        local_socket.local_addr().map_or_else(|_| "unavailable".to_string(), |a| a.to_string())
    );
    debug!("   Max attempts: {}", config.max_attempts);

    let punch_message = b"SONGBIRD_PUNCH_V1";
    let mut recv_buf = vec![0u8; 1024];

    let mut attempts = 0;

    // Wrap in overall timeout
    let result = timeout(config.total_timeout, async {
        while attempts < config.max_attempts {
            attempts += 1;
            debug!("   Hole punch attempt {}/{}", attempts, config.max_attempts);

            // Send punch packet
            match local_socket.send_to(punch_message, peer_addr).await {
                Ok(sent) => {
                    debug!("     Sent {} bytes to {}", sent, peer_addr);
                }
                Err(e) => {
                    warn!("     Failed to send punch packet: {}", e);
                    sleep(config.attempt_delay).await;
                    continue;
                }
            }

            // Try to receive response with short timeout
            match timeout(config.attempt_timeout, local_socket.recv_from(&mut recv_buf)).await {
                Ok(Ok((len, addr))) => {
                    debug!("     Received {} bytes from {}", len, addr);

                    // Verify it's from the expected peer
                    if addr.ip() == peer_addr.ip() {
                        // Port might differ due to NAT, but IP should match
                        info!("✅ UDP hole punch successful! Connected to {}", addr);

                        // Create direct connection
                        // Note: DirectConnection expects NodeId and SocketAddr
                        // The socket is dropped here; in production, we'd pass ownership
                        return Ok(DirectConnection::new(peer_id, addr));
                    }
                    debug!(
                        "     Received from unexpected address: {} (expected IP: {})",
                        addr,
                        peer_addr.ip()
                    );
                }
                Ok(Err(e)) => {
                    debug!("     Receive error: {}", e);
                }
                Err(_) => {
                    // Timeout - expected on early attempts
                    debug!("     Receive timeout (attempt {}/{})", attempts, config.max_attempts);
                }
            }

            // Delay before next attempt (prevents flooding)
            sleep(config.attempt_delay).await;
        }

        Err(LineageRelayError::DirectConnectionFailed(format!(
            "UDP hole punch failed after {} attempts to {}",
            config.max_attempts, peer_addr
        )))
    })
    .await;

    if let Ok(conn) = result {
        conn
    } else {
        warn!("⏱️  UDP hole punch timeout after {:?}", config.total_timeout);
        Err(LineageRelayError::DirectConnectionFailed(format!(
            "UDP hole punch timeout after {:?}",
            config.total_timeout
        )))
    }
}

/// Coordinate UDP hole punch with peer via address exchange
///
/// # Arguments
///
/// * `local_socket` - Pre-bound local UDP socket
/// * `peer_id` - Peer's node ID
/// * `my_public_addr` - My public address (discovered via STUN)
/// * `peer_public_addr` - Peer's public address (received via secure channel)
/// * `config` - Hole punch configuration
///
/// # Returns
///
/// Direct connection on success.
///
/// # Coordination
///
/// Both peers must call this function simultaneously (or near-simultaneously)
/// for hole punching to work. Address exchange happens via:
/// 1. `BirdSong` encrypted broadcast (for same lineage)
/// 2. Existing secure channel (HTTPS/BTSP)
/// 3. Out-of-band exchange (manual, for testing)
pub async fn coordinated_hole_punch(
    local_socket: UdpSocket,
    peer_id: NodeId,
    my_public_addr: SocketAddr,
    peer_public_addr: SocketAddr,
    config: HolePunchConfig,
) -> Result<DirectConnection> {
    info!("🤝 Coordinated UDP hole punch");
    info!("   My public address: {}", my_public_addr);
    info!("   Peer public address: {}", peer_public_addr);

    // Both peers attempt hole punch simultaneously
    udp_hole_punch(local_socket, peer_id, peer_public_addr, config).await
}

/// Create and bind local UDP socket for hole punching
///
/// # Arguments
///
/// * `bind_addr` - Optional bind address (None = "0.0.0.0:0" for OS-assigned port)
///
/// # Returns
///
/// Bound UDP socket ready for hole punching.
pub async fn create_hole_punch_socket(bind_addr: Option<SocketAddr>) -> Result<UdpSocket> {
    let addr = bind_addr.unwrap_or_else(|| "0.0.0.0:0".parse().expect("valid static address"));

    UdpSocket::bind(addr)
        .await
        .map_err(|e| LineageRelayError::NetworkError(format!("Failed to bind UDP socket: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_hole_punch_config_defaults() {
        let config = HolePunchConfig::default();
        assert_eq!(config.max_attempts, 10);
        assert_eq!(config.attempt_timeout, Duration::from_millis(200));
        assert_eq!(config.total_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_create_hole_punch_socket() {
        let socket = create_hole_punch_socket(None).await.unwrap();
        let local_addr = socket.local_addr().unwrap();

        // Should bind to any address with OS-assigned port
        assert_eq!(local_addr.ip().to_string(), "0.0.0.0");
        assert!(local_addr.port() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires two processes for real hole punch test
    async fn test_udp_hole_punch_loopback() {
        // This test demonstrates the hole punch logic but won't actually
        // punch through NAT since it's loopback. Use for logic verification only.

        let socket1 = create_hole_punch_socket(Some("127.0.0.1:0".parse().unwrap())).await.unwrap();
        let addr1 = socket1.local_addr().unwrap();

        let socket2 = create_hole_punch_socket(Some("127.0.0.1:0".parse().unwrap())).await.unwrap();
        let addr2 = socket2.local_addr().unwrap();

        let peer1 = NodeId::from("test-peer-1");
        let peer2 = NodeId::from("test-peer-2");

        // Spawn concurrent hole punch attempts
        let handle1 = tokio::spawn(async move {
            let config = HolePunchConfig::default();
            udp_hole_punch(socket1, peer2, addr2, config).await
        });

        // Slight delay to simulate simultaneous attempts
        sleep(Duration::from_millis(10)).await;

        let handle2 = tokio::spawn(async move {
            let config = HolePunchConfig::default();
            udp_hole_punch(socket2, peer1, addr1, config).await
        });

        // Wait for results
        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        // At least one should succeed (loopback is permissive)
        assert!(result1.is_ok() || result2.is_ok());
    }
}
