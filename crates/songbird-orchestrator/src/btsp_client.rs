// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// BTSP Client - Unix Socket Integration with security provider
// Migrated from HTTP to Unix sockets (Jan 16, 2026)
// Aligned with BiomeOS "Concentrated Gap" strategy

use anyhow::{Result, anyhow};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// Platform-agnostic IPC transport
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
use tracing::{debug, info, warn};

/// Platform-agnostic connection helper
#[cfg(unix)]
async fn connect_platform(path: &PathBuf) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(path).await
}

#[cfg(windows)]
async fn connect_platform(path: &PathBuf) -> std::io::Result<PlatformStream> {
    let addr = path.to_string_lossy();
    PlatformStream::connect(addr.as_ref()).await
}

/// BTSP Client for communicating with the security provider via Unix socket
///
/// This client connects to the security provider's BTSP server for secure inter-primal tunnel
/// communication.
///
/// # Migration Note
/// Migrated from HTTP to Unix sockets on Jan 16, 2026 to align with
/// `BiomeOS` "Concentrated Gap" strategy (HTTP deprecated for inter-primal).
#[derive(Debug, Clone)]
pub struct BtspClient {
    socket_path: PathBuf,
}

impl BtspClient {
    /// Create new BTSP client with environment-based socket discovery
    ///
    /// Socket path priority:
    /// 1. `SECURITY_PROVIDER_SOCKET` (capability-standard)
    /// 2. `BEARDOG_SOCKET` (legacy)
    /// 3. `BIOMEOS_SOCKET_PATH` (`BiomeOS` orchestrator)
    /// 4. XDG_RUNTIME_DIR/security-{family_id}.sock
    /// 5. /tmp/security-default.sock (fallback)
    pub fn new() -> Self {
        let socket_path = Self::discover_socket_path();
        info!("BTSP client initialized with socket: {:?}", socket_path);
        Self {
            socket_path,
        }
    }

    /// Create BTSP client with explicit socket path (concurrent-safe, testable)
    #[must_use]
    pub const fn with_socket(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
        }
    }

    fn discover_socket_path() -> PathBuf {
        let path = songbird_process_env::var("SECURITY_PROVIDER_SOCKET")
            .or_else(|_| songbird_process_env::var("BEARDOG_SOCKET"))
            .or_else(|_| songbird_process_env::var("BIOMEOS_SOCKET_PATH"))
            .or_else(|_| {
                // Try XDG runtime directory (capability-based, primal-agnostic)
                songbird_process_env::var("XDG_RUNTIME_DIR").map(|dir| {
                    let family_id = crate::env_config::family_id();
                    format!("{dir}/security-{family_id}.sock")
                })
            })
            .unwrap_or_else(|_| {
                warn!("No crypto provider socket configured, using fallback discovery");
                songbird_types::defaults::paths::tmp_flat_security_sock_path()
                    .to_string_lossy()
                    .into_owned()
            });

        PathBuf::from(path)
    }

    /// Establish a secure BTSP tunnel with a peer
    ///
    /// # Arguments
    /// * `peer` - The peer endpoint to establish tunnel with
    ///
    /// # Returns
    /// * `TunnelHandle` - Handle to the established tunnel for encrypt/decrypt
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn establish_tunnel(&self, peer: PeerEndpoint) -> Result<TunnelHandle> {
        debug!("Establishing BTSP tunnel with peer: {:?}", peer.id);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.tunnel_establish",
            "params": { "peer": peer },
            "id": 1
        });

        let response = self.send_request(request).await?;
        let tunnel: TunnelHandle = serde_json::from_value(response["result"].clone())
            .map_err(|e| anyhow!("Failed to parse tunnel handle: {e}"))?;

        info!("BTSP tunnel established: {}", tunnel.id);
        Ok(tunnel)
    }

    /// Encrypt data through a BTSP tunnel
    ///
    /// # Arguments
    /// * `tunnel` - The tunnel handle to use
    /// * `data` - The plaintext data to encrypt
    /// * `direction` - The tunnel direction (Outbound/Inbound)
    ///
    /// # Returns
    /// * `Vec<u8>` - The encrypted ciphertext
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn tunnel_encrypt(
        &self,
        tunnel: &TunnelHandle,
        data: &[u8],
        direction: Direction,
    ) -> Result<Vec<u8>> {
        debug!("Encrypting {} bytes via tunnel {}", data.len(), tunnel.id);

        let data_b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.tunnel_encrypt",
            "params": {
                "tunnel": tunnel,
                "direction": direction,
                "data": data_b64
            },
            "id": 2
        });

        let response = self.send_request(request).await?;
        let ciphertext_b64 = response["result"]["ciphertext"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing ciphertext in response"))?;

        base64::engine::general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|e| anyhow!("Failed to decode ciphertext: {e}"))
    }

    /// Decrypt data from a BTSP tunnel
    ///
    /// # Arguments
    /// * `tunnel` - The tunnel handle to use
    /// * `data` - The ciphertext to decrypt
    ///
    /// # Returns
    /// * `Vec<u8>` - The decrypted plaintext
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn tunnel_decrypt(&self, tunnel: &TunnelHandle, data: &[u8]) -> Result<Vec<u8>> {
        debug!("Decrypting {} bytes via tunnel {}", data.len(), tunnel.id);

        let data_b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.tunnel_decrypt",
            "params": {
                "tunnel": tunnel,
                "data": data_b64
            },
            "id": 3
        });

        let response = self.send_request(request).await?;
        let plaintext_b64 = response["result"]["plaintext"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing plaintext in response"))?;

        base64::engine::general_purpose::STANDARD
            .decode(plaintext_b64)
            .map_err(|e| anyhow!("Failed to decode plaintext: {e}"))
    }

    /// Get tunnel status
    ///
    /// # Arguments
    /// * `tunnel` - The tunnel handle to query
    ///
    /// # Returns
    /// * `TunnelStatus` - Current status of the tunnel
    ///
    /// # Panics
    /// May panic if JSON-RPC communication fails unexpectedly.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn tunnel_status(&self, tunnel: &TunnelHandle) -> Result<TunnelStatus> {
        debug!("Querying status for tunnel {}", tunnel.id);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.tunnel_status",
            "params": { "tunnel": tunnel },
            "id": 4
        });

        let response = self.send_request(request).await?;
        serde_json::from_value(response["result"].clone())
            .map_err(|e| anyhow!("Failed to parse tunnel status: {e}"))
    }

    /// Close a BTSP tunnel
    ///
    /// # Arguments
    /// * `tunnel` - The tunnel handle to close
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn tunnel_close(&self, tunnel: &TunnelHandle) -> Result<()> {
        debug!("Closing tunnel {}", tunnel.id);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.tunnel_close",
            "params": { "tunnel": tunnel },
            "id": 5
        });

        self.send_request(request).await?;
        info!("Tunnel {} closed successfully", tunnel.id);
        Ok(())
    }

    /// Close a tunnel by ID (compatibility alias for `tunnel_close`)
    ///
    /// # Arguments
    /// * `tunnel_id` - The tunnel ID to close
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn close_tunnel(&self, tunnel_id: &str) -> Result<()> {
        let tunnel = TunnelHandle {
            id: tunnel_id.to_string(),
            peer_id: String::new(), // Not needed for close
            created_at: None,
        };
        self.tunnel_close(&tunnel).await
    }

    /// Contact exchange for peer discovery
    ///
    /// # Arguments
    /// * `target_peer_id` - The peer ID to discover
    /// * `lineage` - The lineage chain for routing
    /// * `max_hops` - Maximum hops for discovery
    ///
    /// # Returns
    /// * Contact information for the discovered peer
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn contact_exchange(
        &self,
        target_peer_id: &str,
        lineage: Vec<String>,
        max_hops: u32,
    ) -> Result<serde_json::Value> {
        debug!("Contact exchange for peer: {}", target_peer_id);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "btsp.contact_exchange",
            "params": {
                "target_peer_id": target_peer_id,
                "lineage": lineage,
                "max_hops": max_hops
            },
            "id": 6
        });

        let response = self.send_request(request).await?;
        Ok(response["result"].clone())
    }

    /// Ping the security provider to check health
    ///
    /// # Panics
    /// May panic if JSON-RPC communication fails unexpectedly.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub async fn ping(&self) -> Result<serde_json::Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "ping",
            "id": 0
        });

        self.send_request(request).await
    }

    /// Send JSON-RPC request to the security provider and receive response
    ///
    /// # Arguments
    /// * `request` - The JSON-RPC 2.0 request object
    ///
    /// # Returns
    /// * The JSON-RPC 2.0 response object
    ///
    /// # Errors
    /// * Socket connection failures
    /// * JSON-RPC protocol errors
    /// * Timeout errors
    async fn send_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        // Connect to security provider (platform-agnostic)
        let mut stream = connect_platform(&self.socket_path).await.map_err(|e| {
            anyhow!(
                "Failed to connect to security provider socket {}: {}",
                self.socket_path.display(),
                e
            )
        })?;

        // Send request (newline-delimited JSON)
        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await?;
        stream.write_all(b"\n").await?; // JSON-RPC delimiter

        // Read response (newline-delimited)
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_until(b'\n', &mut buffer).await?;

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_slice(&buffer)?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(anyhow!(
                "BTSP JSON-RPC error: {}",
                error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error")
            ));
        }

        Ok(response)
    }
}

impl Default for BtspClient {
    fn default() -> Self {
        Self::new()
    }
}

// Type definitions (aligned with the security provider's wire types)

/// Peer endpoint for BTSP tunnel establishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEndpoint {
    /// Peer identifier
    pub id: String,
    /// Network endpoint (IP:port)
    pub endpoint: String,
    /// Optional public key for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Peer capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Handle to an established BTSP tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelHandle {
    /// Unique tunnel identifier
    pub id: String,
    /// Peer this tunnel connects to
    pub peer_id: String,
    /// Tunnel creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Tunnel data flow direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Outbound traffic (Songbird → Peer)
    Outbound,
    /// Inbound traffic (Peer → Songbird)
    Inbound,
}

/// Current status of a BTSP tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    /// Tunnel state
    pub state: String,
    /// Bytes sent through tunnel
    pub bytes_sent: u64,
    /// Bytes received through tunnel
    pub bytes_received: u64,
    /// Last activity timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_discovery() {
        // ✅ Concurrent-safe: Uses with_socket (no env vars)
        let unique_path = format!("/tmp/test_socket_{}.sock", std::process::id());
        let client = BtspClient::with_socket(PathBuf::from(&unique_path));
        assert_eq!(
            client.socket_path,
            PathBuf::from(&unique_path),
            "Socket path should match the explicitly provided path"
        );
    }

    #[tokio::test]
    async fn test_btsp_ping() {
        if std::env::var("SECURITY_PROVIDER_SOCKET")
            .or_else(|_| std::env::var("BEARDOG_SOCKET"))
            .is_ok()
        {
            let client = BtspClient::new();
            let response = client.ping().await;
            if let Ok(resp) = response {
                let primal = resp["result"]["primal"].as_str();
                assert!(
                    matches!(primal, Some("security provider" | "beardog")),
                    "unexpected primal name in BTSP ping: {primal:?}"
                );
            }
        }
    }
}
