// SPDX-License-Identifier: AGPL-3.0-or-later
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
            .or_else(|_| match songbird_process_env::var("BEARDOG_SOCKET") {
                Ok(p) => {
                    warn!(
                        "Using legacy env var BEARDOG_SOCKET — migrate to SECURITY_PROVIDER_SOCKET"
                    );
                    Ok(p)
                }
                Err(e) => Err(e),
            })
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

    /// Perform the BTSP handshake as a client connecting to a target primal socket.
    ///
    /// Implements the `ClientHello → ServerHello → ChallengeResponse → HandshakeComplete`
    /// flow per `BTSP_PROTOCOL_STANDARD.md` v1.0. All cryptographic operations are
    /// delegated to the security provider via JSON-RPC (`btsp.session.*` methods).
    ///
    /// # Arguments
    /// * `target_socket` - Path to the target primal's Unix domain socket.
    /// * `preferred_cipher` - Cipher suite preference (e.g., `"chacha20_poly1305"`).
    ///
    /// # Returns
    /// A [`BtspSession`] representing the authenticated, optionally encrypted session.
    ///
    /// # Errors
    /// Returns an error if the handshake fails at any stage (connection, crypto, verification).
    pub async fn handshake(
        &self,
        target_socket: &std::path::Path,
        preferred_cipher: &str,
    ) -> Result<BtspSession> {
        debug!("Starting BTSP handshake with {}", target_socket.display());

        // Step 1: Ask security provider to create a session (generates our ephemeral X25519 keypair)
        let create_resp = self
            .send_request(json!({
                "jsonrpc": "2.0",
                "method": "btsp.session.create",
                "params": {
                    "family_seed_ref": "env:FAMILY_SEED",
                    "role": "client"
                },
                "id": 10
            }))
            .await?;

        let client_ephemeral_pub = create_resp["result"]["client_ephemeral_pub"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing client_ephemeral_pub from btsp.session.create"))?
            .to_string();
        let handshake_key_ref = create_resp["result"]["session_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing session_id from btsp.session.create"))?
            .to_string();

        // Step 2: Connect to target socket, send ClientHello
        let mut target_stream =
            connect_platform(&target_socket.to_path_buf()).await.map_err(|e| {
                anyhow!("Failed to connect to target socket {}: {}", target_socket.display(), e)
            })?;

        let client_hello = json!({
            "type": "ClientHello",
            "version": 1,
            "client_ephemeral_pub": client_ephemeral_pub
        });
        let hello_bytes = serde_json::to_vec(&client_hello)?;
        target_stream.write_all(&hello_bytes).await?;
        target_stream.write_all(b"\n").await?;

        // Step 3: Read ServerHello (server_ephemeral_pub + challenge)
        let mut buf = Vec::new();
        let mut reader = BufReader::new(&mut target_stream);
        reader.read_until(b'\n', &mut buf).await?;
        let server_hello: serde_json::Value = serde_json::from_slice(&buf)?;

        if server_hello.get("error").is_some() {
            return Err(anyhow!(
                "Server rejected handshake: {}",
                server_hello["error"]["reason"].as_str().unwrap_or("unknown")
            ));
        }

        let server_ephemeral_pub = server_hello["server_ephemeral_pub"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing server_ephemeral_pub in ServerHello"))?;
        let challenge = server_hello["challenge"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing challenge in ServerHello"))?;

        // Step 4: Ask security provider to verify/compute challenge response
        let verify_resp = self
            .send_request(json!({
                "jsonrpc": "2.0",
                "method": "btsp.session.verify",
                "params": {
                    "session_id": handshake_key_ref,
                    "client_ephemeral_pub": client_ephemeral_pub,
                    "server_ephemeral_pub": server_ephemeral_pub,
                    "challenge": challenge,
                    "role": "client"
                },
                "id": 11
            }))
            .await?;

        let challenge_response = verify_resp["result"]["client_response"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing client_response from btsp.session.verify"))?;

        // Step 5: Send ChallengeResponse to target
        let cr_msg = json!({
            "type": "ChallengeResponse",
            "response": challenge_response,
            "preferred_cipher": preferred_cipher
        });
        let cr_bytes = serde_json::to_vec(&cr_msg)?;
        buf.clear();
        target_stream.write_all(&cr_bytes).await?;
        target_stream.write_all(b"\n").await?;

        // Step 6: Read HandshakeComplete
        buf.clear();
        reader = BufReader::new(&mut target_stream);
        reader.read_until(b'\n', &mut buf).await?;
        let hs_complete: serde_json::Value = serde_json::from_slice(&buf)?;

        if hs_complete.get("error").is_some() {
            return Err(anyhow!(
                "Handshake verification failed: {}",
                hs_complete["error"]["reason"].as_str().unwrap_or("family_verification")
            ));
        }

        let negotiated_cipher =
            hs_complete["cipher"].as_str().unwrap_or("chacha20_poly1305").to_string();
        let session_id = hs_complete["session_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing session_id in HandshakeComplete"))?
            .to_string();

        // Step 7: Negotiate cipher via security provider
        let _negotiate_resp = self
            .send_request(json!({
                "jsonrpc": "2.0",
                "method": "btsp.negotiate",
                "params": {
                    "session_id": session_id,
                    "preferred_cipher": negotiated_cipher,
                    "bond_type": "Covalent"
                },
                "id": 12
            }))
            .await?;

        info!(
            session_id = %session_id,
            cipher = %negotiated_cipher,
            target = %target_socket.display(),
            "BTSP handshake complete"
        );

        Ok(BtspSession {
            session_id,
            cipher: negotiated_cipher,
            target_socket: target_socket.to_path_buf(),
            client_ephemeral_pub,
            server_ephemeral_pub: server_ephemeral_pub.to_string(),
        })
    }
}

impl Default for BtspClient {
    fn default() -> Self {
        Self::new()
    }
}

/// An authenticated BTSP session after a successful handshake.
///
/// Holds the session key reference and negotiated cipher suite. Use this to send
/// encrypted JSON-RPC frames through the BTSP tunnel.
#[derive(Debug, Clone)]
pub struct BtspSession {
    /// Unique session identifier (maps to session key in security provider).
    pub session_id: String,
    /// Negotiated cipher suite (e.g., `chacha20_poly1305`, `hmac_plain`, `null`).
    pub cipher: String,
    /// Target socket path this session connects to.
    pub target_socket: PathBuf,
    /// Our ephemeral public key (base64).
    pub client_ephemeral_pub: String,
    /// Server's ephemeral public key (base64).
    pub server_ephemeral_pub: String,
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
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_discovery() {
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
                    matches!(primal, Some("security provider" | "security" | "beardog")),
                    "unexpected primal name in BTSP ping: {primal:?}"
                );
            }
        }
    }

    #[test]
    fn btsp_session_fields_are_accessible() {
        let session = BtspSession {
            session_id: "sess-42".to_string(),
            cipher: "chacha20_poly1305".to_string(),
            target_socket: PathBuf::from("/run/user/1000/biomeos/security.sock"),
            client_ephemeral_pub: "Y2xpZW50X2tleQ==".to_string(),
            server_ephemeral_pub: "c2VydmVyX2tleQ==".to_string(),
        };
        assert_eq!(session.session_id, "sess-42");
        assert_eq!(session.cipher, "chacha20_poly1305");
        assert!(
            session
                .target_socket
                .to_str()
                .expect("target_socket should be valid UTF-8")
                .contains("security")
        );
    }

    #[test]
    fn btsp_session_clone_is_independent() {
        let session = BtspSession {
            session_id: "sess-1".to_string(),
            cipher: "hmac_plain".to_string(),
            target_socket: PathBuf::from("/tmp/test.sock"),
            client_ephemeral_pub: "a2V5MQ==".to_string(),
            server_ephemeral_pub: "a2V5Mg==".to_string(),
        };
        let cloned = session.clone();
        assert_eq!(cloned.session_id, session.session_id);
        assert_eq!(cloned.cipher, session.cipher);
    }
}
