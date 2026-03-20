// SPDX-License-Identifier: AGPL-3.0-only
//! Mock BearDog server for testing BTSP client
//!
//! Provides a lightweight mock implementation of BearDog's Unix socket server
//! for testing BTSP tunnel establishment, encryption/decryption, and error handling.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, info, warn};

/// Mock BearDog server for testing
pub struct BearDogMock {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    fault_mode: FaultMode,
}

/// Fault injection modes for testing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaultMode {
    /// Normal operation (no faults)
    None,
    /// Simulate network partition (connection drops)
    NetworkPartition,
    /// Simulate slow responses
    SlowResponse {
        delay_ms: u64,
    },
    /// Simulate connection drop after N bytes
    ConnectionDrop {
        after_bytes: usize,
    },
    /// Simulate JSON-RPC error responses
    JsonRpcError {
        code: i32,
        message: String,
    },
}

impl BearDogMock {
    /// Create a new mock BearDog server
    ///
    /// # Arguments
    /// * `socket_path` - Path to Unix socket (e.g., "/tmp/beardog-test.sock")
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            listener: None,
            fault_mode: FaultMode::None,
        }
    }

    /// Start the mock server
    pub async fn start(&mut self) -> Result<()> {
        // Clean up old socket if exists
        let _ = std::fs::remove_file(&self.socket_path);

        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("🐻 BearDog mock started on {:?}", self.socket_path);

        self.listener = Some(listener);
        Ok(())
    }

    /// Accept a single connection and handle it
    pub async fn handle_connection(&self) -> Result<()> {
        let listener = self.listener.as_ref().ok_or_else(|| anyhow!("Mock server not started"))?;

        let (stream, _) = listener.accept().await?;
        debug!("🐻 BearDog mock accepted connection");

        self.handle_stream(stream).await
    }

    /// Handle a Unix stream connection
    async fn handle_stream(&self, mut stream: UnixStream) -> Result<()> {
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(&mut stream);

        // Read JSON-RPC request
        reader.read_until(b'\n', &mut buffer).await?;

        let request: Value = serde_json::from_slice(&buffer)?;
        debug!("🐻 BearDog mock received request: {:?}", request);

        // Inject faults if configured
        match self.fault_mode {
            FaultMode::NetworkPartition => {
                // Simulate network partition (drop connection)
                return Ok(());
            }
            FaultMode::SlowResponse {
                delay_ms,
            } => {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }
            FaultMode::ConnectionDrop {
                after_bytes,
            } => {
                if buffer.len() >= after_bytes {
                    return Ok(()); // Drop connection
                }
            }
            FaultMode::JsonRpcError {
                code,
                ref message,
            } => {
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "error": {
                        "code": code,
                        "message": message
                    }
                });

                let response_bytes = serde_json::to_vec(&error_response)?;
                stream.write_all(&response_bytes).await?;
                stream.write_all(b"\n").await?;
                return Ok(());
            }
            FaultMode::None => {}
        }

        // Extract method from request
        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .ok_or_else(|| anyhow!("Missing method in request"))?;

        // Handle different BTSP methods
        let response = match method {
            "ping" => self.handle_ping(&request),
            "btsp.tunnel_establish" => self.handle_tunnel_establish(&request),
            "btsp.tunnel_encrypt" => self.handle_tunnel_encrypt(&request),
            "btsp.tunnel_decrypt" => self.handle_tunnel_decrypt(&request),
            "btsp.tunnel_status" => self.handle_tunnel_status(&request),
            "btsp.tunnel_close" => self.handle_tunnel_close(&request),
            "btsp.contact_exchange" => self.handle_contact_exchange(&request),
            _ => json!({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }),
        };

        // Send response
        let response_bytes = serde_json::to_vec(&response)?;
        stream.write_all(&response_bytes).await?;
        stream.write_all(b"\n").await?;

        Ok(())
    }

    /// Handle ping request
    fn handle_ping(&self, request: &Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "primal": "beardog",
                "version": "mock-1.0.0",
                "status": "healthy"
            }
        })
    }

    /// Handle tunnel establishment
    fn handle_tunnel_establish(&self, request: &Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "id": "test-tunnel-123",
                "peer_id": "test-peer",
                "state": "Established"
            }
        })
    }

    /// Handle tunnel encryption
    fn handle_tunnel_encrypt(&self, request: &Value) -> Value {
        // For testing, just return the data as-is (mock encryption)
        let data = request
            .get("params")
            .and_then(|p| p.get("data"))
            .and_then(|d| d.as_str())
            .unwrap_or("");

        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "ciphertext": data // Mock: return same data
            }
        })
    }

    /// Handle tunnel decryption
    fn handle_tunnel_decrypt(&self, request: &Value) -> Value {
        // For testing, just return the data as-is (mock decryption)
        let data = request
            .get("params")
            .and_then(|p| p.get("data"))
            .and_then(|d| d.as_str())
            .unwrap_or("");

        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "plaintext": data // Mock: return same data
            }
        })
    }

    /// Handle tunnel status
    fn handle_tunnel_status(&self, request: &Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "tunnel_id": "test-tunnel-123",
                "state": "Established",
                "bytes_sent": 1024,
                "bytes_received": 2048
            }
        })
    }

    /// Handle tunnel close
    fn handle_tunnel_close(&self, request: &Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "success": true
            }
        })
    }

    /// Handle contact exchange
    fn handle_contact_exchange(&self, request: &Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "result": {
                "peer_id": "discovered-peer",
                "endpoint": "unix:///tmp/discovered-peer.sock",
                "capabilities": ["btsp_enabled"]
            }
        })
    }

    /// Set fault injection mode
    pub fn set_fault_mode(&mut self, mode: FaultMode) {
        self.fault_mode = mode;
    }

    /// Clear fault injection
    pub fn clear_faults(&mut self) {
        self.fault_mode = FaultMode::None;
    }

    /// Stop the mock server and clean up
    pub fn stop(self) -> Result<()> {
        drop(self.listener);
        let _ = std::fs::remove_file(&self.socket_path);
        info!("🐻 BearDog mock stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_beardog_mock_creation() {
        let mock = BearDogMock::new("/tmp/test-beardog-mock.sock");
        assert_eq!(mock.socket_path, PathBuf::from("/tmp/test-beardog-mock.sock"));
    }

    #[tokio::test]
    async fn test_beardog_mock_start_stop() {
        let mut mock = BearDogMock::new("/tmp/test-beardog-mock-2.sock");
        mock.start().await.unwrap();
        assert!(mock.listener.is_some());
        mock.stop().unwrap();
    }
}
