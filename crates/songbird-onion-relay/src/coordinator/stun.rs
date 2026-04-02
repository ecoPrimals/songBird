// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! STUN binding and NAT type probing.

use crate::error::{OnionRelayError, Result};
use crate::signaling::{NatType, PeerInfo};
use std::net::SocketAddr;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

use super::core::HolePunchCoordinator;

impl HolePunchCoordinator {
    /// Performs STUN binding against configured servers and caches [`PeerInfo`].
    ///
    /// # Errors
    ///
    /// Returns [`OnionRelayError::StunFailed`] when every STUN server fails.
    pub async fn discover_public_address(&self) -> Result<PeerInfo> {
        info!("🔍 Discovering public address via STUN...");

        // Bind local socket
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let local_addr = socket.local_addr()?;

        // Try each STUN server
        for stun_server in &self.config.stun_servers {
            match self.stun_bind(&socket, stun_server).await {
                Ok(public_addr) => {
                    info!("✅ Discovered public address: {}", public_addr);

                    // Detect NAT type by checking if port varies
                    let nat_type = self.detect_nat_type(&socket).await;

                    let info = PeerInfo {
                        node_id: self.my_node_id.clone(),
                        public_addr,
                        local_addr: Some(local_addr),
                        nat_type,
                        timestamp: SystemTime::now(),
                        capabilities: vec!["relay".to_string()],
                    };

                    *self.my_info.write().await = Some(info.clone());
                    return Ok(info);
                }
                Err(e) => {
                    warn!("⚠️ STUN {} failed: {}", stun_server, e);
                }
            }
        }

        Err(OnionRelayError::StunFailed("All STUN servers failed".to_string()))
    }

    pub(super) async fn stun_bind(&self, _socket: &UdpSocket, server: &str) -> Result<SocketAddr> {
        use songbird_stun::StunClient;

        let client = StunClient::new();
        client
            .discover_public_address(server)
            .await
            .map_err(|e| OnionRelayError::StunFailed(e.to_string()))
    }

    pub(super) async fn detect_nat_type(&self, socket: &UdpSocket) -> NatType {
        // Quick NAT type detection by checking port allocation
        // Full implementation would test against multiple STUN servers

        if self.config.stun_servers.len() < 2 {
            return NatType::Unknown;
        }

        let addr1 = self.stun_bind(socket, &self.config.stun_servers[0]).await;
        let addr2 = self.stun_bind(socket, &self.config.stun_servers[1]).await;

        match (addr1, addr2) {
            (Ok(a1), Ok(a2)) if a1.port() == a2.port() => {
                debug!("NAT type: Same port for different destinations → Cone NAT");
                NatType::PortRestricted // Conservative estimate
            }
            (Ok(a1), Ok(a2)) => {
                debug!(
                    "NAT type: Different ports ({} vs {}) → Symmetric NAT",
                    a1.port(),
                    a2.port()
                );
                NatType::Symmetric
            }
            _ => NatType::Unknown,
        }
    }
}
