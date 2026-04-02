// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Simultaneous-open hole punch and signaling ack handling.

use crate::error::{OnionRelayError, Result};
use crate::signaling::SignalingMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

use super::core::HolePunchCoordinator;
use super::types::PunchResult;
use super::util::{rand_nonce, unix_epoch_millis_u64};

impl HolePunchCoordinator {
    /// Runs the simultaneous-open punch flow toward a previously registered peer.
    ///
    /// # Errors
    ///
    /// Returns when local/peer info is missing, signaling times out, or UDP operations fail.
    pub async fn punch_to_peer(&self, peer_node_id: &str) -> Result<PunchResult> {
        info!("🥊 Initiating hole punch to {}", peer_node_id);

        // Get our info
        let my_info = self.my_info.read().await.clone().ok_or_else(|| {
            OnionRelayError::Other("Must discover public address first".to_string())
        })?;

        // Get peer info
        let peer_info = self
            .peers
            .read()
            .await
            .get(peer_node_id)
            .cloned()
            .ok_or_else(|| OnionRelayError::PeerNotFound(peer_node_id.to_string()))?;

        // Generate nonce for this attempt
        let nonce: [u8; 16] = rand_nonce();

        // Send punch request via signaling
        let request = SignalingMessage::PunchRequest {
            from: my_info.clone(),
            to_node_id: peer_node_id.to_string(),
            nonce,
        };

        self.signal_tx
            .send(request)
            .await
            .map_err(|e| OnionRelayError::Transport(e.to_string()))?;

        // Wait for ack with start time
        let start_time = self.wait_for_punch_ack(&nonce).await?;

        // Create socket for punching
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

        // Wait until coordinated start time
        let now_ms = unix_epoch_millis_u64()?;

        if start_time > now_ms {
            sleep(Duration::from_millis(start_time - now_ms)).await;
        }

        // Execute hole punch
        let result = self.execute_punch(socket.clone(), peer_info.public_addr).await;

        // Report result via signaling
        let result_msg = SignalingMessage::PunchResult {
            nonce,
            success: result.is_ok(),
            connected_addr: result.as_ref().ok().map(|_| peer_info.public_addr),
        };
        let _ = self.signal_tx.send(result_msg).await;

        if let Ok(latency) = result {
            info!("✅ Hole punch successful! Latency: {:?}", latency);
            Ok(PunchResult::Direct {
                peer_addr: peer_info.public_addr,
                local_socket: socket,
                latency,
            })
        } else {
            warn!("⚠️ Hole punch failed, falling back to relay");
            Ok(PunchResult::Relay {
                attempts: self.config.max_attempts,
            })
        }
    }

    /// Wait for `PunchAck` from peer via signaling channel
    ///
    /// This is a real implementation that:
    /// 1. Takes the `signal_rx` receiver
    /// 2. Waits for matching `PunchAck` with timeout
    /// 3. Returns coordinated start time from peer
    async fn wait_for_punch_ack(&self, nonce: &[u8; 16]) -> Result<u64> {
        // Take the receiver (one-shot per punch attempt)
        let rx = {
            let mut rx_guard = self.signal_rx.write().await;
            rx_guard.take()
        };

        let Some(mut rx) = rx else {
            warn!("⚠️ No signal receiver available - using fallback timing");
            // Fallback: coordinate 100ms in future
            return Ok(unix_epoch_millis_u64()? + 100);
        };

        // Wait for matching PunchAck with timeout
        let result = timeout(self.config.ack_timeout, async {
            while let Some(msg) = rx.recv().await {
                match msg {
                    SignalingMessage::PunchAck {
                        from: _,
                        nonce: ack_nonce,
                        start_at_ms,
                    } => {
                        // Check nonce matches
                        if &ack_nonce == nonce {
                            debug!("✅ Received PunchAck, start at {}ms", start_at_ms);
                            return Ok(start_at_ms);
                        }
                        debug!("⚠️ PunchAck nonce mismatch, continuing...");
                    }
                    other => {
                        // Handle other messages through the coordinator
                        if let Some(response) = self.handle_message(other).await {
                            let _ = self.signal_tx.send(response).await;
                        }
                    }
                }
            }
            Err(OnionRelayError::SignalingTimeout)
        })
        .await;

        // Return receiver for future use
        *self.signal_rx.write().await = Some(rx);

        match result {
            Ok(Ok(start_time)) => Ok(start_time),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!("⚠️ PunchAck timeout after {:?}", self.config.ack_timeout);
                Err(OnionRelayError::SignalingTimeout)
            }
        }
    }

    async fn execute_punch(
        &self,
        socket: Arc<UdpSocket>,
        peer_addr: SocketAddr,
    ) -> Result<Duration> {
        let punch_msg = b"SONGBIRD_PUNCH_V2";
        let mut recv_buf = vec![0u8; 1024];
        let start = Instant::now();

        for attempt in 0..self.config.max_attempts {
            debug!("  Punch attempt {}/{}", attempt + 1, self.config.max_attempts);

            // Send punch packet
            if let Err(e) = socket.send_to(punch_msg, peer_addr).await {
                warn!("  Send failed: {}", e);
                continue;
            }

            // Try to receive with short timeout
            match timeout(self.config.attempt_timeout, socket.recv_from(&mut recv_buf)).await {
                Ok(Ok((len, from_addr))) => {
                    if from_addr.ip() == peer_addr.ip() {
                        debug!("  Received {} bytes from {}", len, from_addr);
                        return Ok(start.elapsed());
                    }
                }
                Ok(Err(e)) => {
                    debug!("  Recv error: {}", e);
                }
                Err(_) => {
                    // Timeout - expected, continue trying
                }
            }

            sleep(self.config.packet_interval).await;
        }

        Err(OnionRelayError::HolePunchFailed {
            attempts: self.config.max_attempts,
        })
    }
}
