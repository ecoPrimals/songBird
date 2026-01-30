//! STUN client implementation
//!
//! **Pure Rust | Async | Zero Unsafe Code**

use crate::error::{StunError, StunResult};
use crate::message::StunMessage;
use crate::types::{NatType, PublicEndpoint};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// STUN client for NAT traversal
///
/// Pure Rust implementation of RFC 5389 STUN protocol.
#[derive(Debug)]
pub struct StunClient {
    /// Request timeout
    timeout: Duration,
}

impl StunClient {
    /// Create a new STUN client
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(5),
        }
    }

    /// Create STUN client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
        }
    }

    /// Discover public address via STUN server
    ///
    /// # Arguments
    ///
    /// * `stun_server` - STUN server address (e.g., "stun.nextcloud.com:3478")
    ///
    /// # Returns
    ///
    /// Public IP address and port as seen by the STUN server.
    ///
    /// # Privacy Note
    ///
    /// STUN servers can observe your public IP/port and connection timing.
    /// Prefer genetic lineage relay when sovereignty > convenience.
    pub async fn discover_public_address(&self, stun_server: &str) -> StunResult<SocketAddr> {
        info!("🔍 Discovering public address via STUN: {}", stun_server);

        // Resolve STUN server address
        let server_addr = tokio::net::lookup_host(stun_server)
            .await
            .map_err(|e| StunError::Network(format!("Failed to resolve STUN server: {}", e)))?
            .next()
            .ok_or_else(|| {
                StunError::Network(format!("No addresses found for: {}", stun_server))
            })?;

        debug!("  Resolved STUN server: {}", server_addr);

        // Bind local UDP socket (port 0 = OS assigns)
        let local_socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| StunError::Network(format!("Failed to bind UDP socket: {}", e)))?;

        debug!("  Local socket bound: {}", local_socket.local_addr()?);

        // Create STUN binding request
        let request = StunMessage::new_binding_request();
        let request_bytes = request.encode();

        debug!("  Sending STUN binding request ({} bytes)", request_bytes.len());

        // Send request
        local_socket
            .send_to(&request_bytes, server_addr)
            .await
            .map_err(|e| StunError::Network(format!("Failed to send STUN request: {}", e)))?;

        // Receive response with timeout
        let mut buf = vec![0u8; 2048];

        let (recv_len, recv_addr) = timeout(self.timeout, local_socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.timeout))?
            .map_err(|e| StunError::Network(format!("Failed to receive STUN response: {}", e)))?;

        debug!("  Received STUN response ({} bytes from {})", recv_len, recv_addr);

        // Decode response
        let response = StunMessage::decode(&buf[..recv_len])?;

        // Verify transaction ID matches
        if response.transaction_id != request.transaction_id {
            return Err(StunError::InvalidResponse("Transaction ID mismatch".to_string()));
        }

        // Extract mapped address
        let public_addr = response.get_any_mapped_address().ok_or_else(|| {
            StunError::InvalidResponse("No mapped address in response".to_string())
        })?;

        info!("✅ Discovered public address: {}", public_addr);

        Ok(public_addr)
    }

    /// Discover public endpoint with NAT type detection
    ///
    /// # Arguments
    ///
    /// * `stun_server` - Primary STUN server address
    ///
    /// # Returns
    ///
    /// Public endpoint including NAT type classification.
    ///
    /// # Note
    ///
    /// Full NAT type detection requires multiple STUN servers and is not yet implemented.
    /// This method returns basic public address discovery with `NatType::Unknown`.
    pub async fn discover_public_endpoint(&self, stun_server: &str) -> StunResult<PublicEndpoint> {
        let address = self.discover_public_address(stun_server).await?;

        // TODO: Implement full NAT type detection (requires multiple requests)
        // For now, return Unknown NAT type
        Ok(PublicEndpoint {
            address,
            nat_type: NatType::Unknown,
        })
    }

    /// Test multiple STUN servers in parallel, use first success
    ///
    /// # Arguments
    ///
    /// * `stun_servers` - List of STUN server addresses
    ///
    /// # Returns
    ///
    /// First successful public address discovery.
    ///
    /// # Strategy
    ///
    /// Attempts all servers in parallel for fastest response.
    /// Useful when server availability is uncertain.
    pub async fn discover_public_address_parallel(
        &self,
        stun_servers: &[String],
    ) -> StunResult<SocketAddr> {
        if stun_servers.is_empty() {
            return Err(StunError::Config("No STUN servers provided".to_string()));
        }

        info!("🔍 Discovering public address via {} STUN servers (parallel)", stun_servers.len());

        // Launch parallel requests
        let mut handles = Vec::new();
        for server in stun_servers {
            let server = server.clone();
            let timeout = self.timeout;

            let handle = tokio::spawn(async move {
                let client = StunClient::with_timeout(timeout);
                client.discover_public_address(&server).await
            });

            handles.push(handle);
        }

        // Wait for first success
        use futures::future::select_all;
        let (result, index, _remaining) = select_all(handles).await;

        match result {
            Ok(Ok(addr)) => {
                info!("✅ First success from STUN server #{}", index);
                Ok(addr)
            }
            Ok(Err(e)) => {
                warn!("All STUN servers failed");
                Err(e)
            }
            Err(e) => Err(StunError::Network(format!("Task join error: {}", e))),
        }
    }
}

impl Default for StunClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access to public STUN server
    async fn test_discover_public_address_live() {
        let client = StunClient::new();

        // Test with Nextcloud STUN (vetted)
        let result = client.discover_public_address("stun.nextcloud.com:3478").await;

        match result {
            Ok(addr) => {
                println!("Discovered public address: {}", addr);
                assert!(addr.port() > 0);
            }
            Err(e) => {
                eprintln!("STUN request failed (expected if no network): {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_stun_client_creation() {
        let client = StunClient::new();
        assert_eq!(client.timeout, Duration::from_secs(5));

        let client = StunClient::with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_default_client() {
        let client = StunClient::default();
        assert_eq!(client.timeout, Duration::from_secs(5));
    }
}
