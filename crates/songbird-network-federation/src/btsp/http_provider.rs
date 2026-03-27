// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! RPC-based BTSP Provider Client (Pure Rust)
//!
//! This module implements a BTSP provider that communicates with a remote
//! security provider (like `BearDog`) over Unix socket RPC (Pure Rust).
//!
//! **Modern Idiomatic Rust**:
//! - Async/await throughout
//! - No `unwrap()` in production paths
//! - Proper error handling with `SongbirdError`
//! - Type-safe JSON-RPC via `UnixRpcClient`
//! - Timeout and retry logic

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::provider::{BtspProvider, PeerInfo};
use super::tunnel::{SecurityContext, TunnelHandle, TunnelStatus};
use songbird_types::{SongbirdError, SongbirdResult};

/// RPC client for communicating with a remote BTSP provider
/// **Pure Rust**: Uses Unix socket RPC instead of HTTP
pub struct HttpBtspProvider {
    /// Unix socket path for the security provider
    socket_path: PathBuf,
    /// RPC client for JSON-RPC communication
    rpc_client: UnixRpcClient,
    /// Provider name (e.g., "beardog", "secureprimal")
    provider_name: String,
}

impl HttpBtspProvider {
    /// Create a new RPC BTSP provider (Pure Rust Unix socket)
    ///
    /// # Arguments
    /// * `base_url` - Legacy parameter (converted to socket path)
    /// * `provider_name` - Name of the provider for logging (e.g., "beardog")
    pub fn new(_base_url: String, provider_name: String) -> SongbirdResult<Self> {
        // Convert base_url to socket path or use env var
        let socket_path =
            songbird_process_env::var(format!("{}_BTSP_SOCKET_PATH", provider_name.to_uppercase()))
                .or_else(|_| songbird_process_env::var("BTSP_SOCKET_PATH"))
                .map_or_else(
                    |_| PathBuf::from(format!("/tmp/{provider_name}_btsp.sock")),
                    PathBuf::from,
                );

        let rpc_client = UnixRpcClient::new(&socket_path).map_err(|e| {
            SongbirdError::network(format!("Failed to create RPC client for {provider_name}: {e}"))
        })?;

        info!("🔒 Created RPC BTSP provider for {} at {:?}", provider_name, socket_path);

        Ok(Self {
            socket_path,
            rpc_client,
            provider_name,
        })
    }

    /// Verify the provider is reachable via RPC
    pub async fn verify_connection(&self) -> SongbirdResult<()> {
        debug!("🔍 Verifying RPC connection to {} at {:?}", self.provider_name, self.socket_path);

        // Call health check via JSON-RPC
        let _response: serde_json::Value =
            self.rpc_client.call_no_params("health").await.map_err(|e| {
                SongbirdError::network(format!(
                    "Failed to connect to security provider {} at {}: {}",
                    self.provider_name,
                    self.socket_path.display(),
                    e
                ))
            })?;

        info!("✅ RPC connection to {} verified", self.provider_name);
        Ok(())
    }
}

#[async_trait]
impl BtspProvider for HttpBtspProvider {
    async fn establish_tunnel(&self, peer: &PeerInfo) -> SongbirdResult<TunnelHandle> {
        debug!("🔐 Establishing tunnel to {} via RPC", peer.id);

        #[derive(Serialize)]
        struct EstablishRequest<'a> {
            peer_id: &'a str,
            endpoint: &'a str,
            public_key: Option<&'a [u8]>,
            protocols: &'a [String],
        }

        #[derive(Deserialize)]
        struct EstablishResponse {
            tunnel_id: String,
            status: String,
        }

        let request = EstablishRequest {
            peer_id: &peer.id,
            endpoint: &peer.endpoint,
            public_key: peer.public_key.as_deref(),
            protocols: &peer.protocols,
        };

        let establish_response: EstablishResponse = self
            .rpc_client
            .call("btsp.tunnel.establish", &request)
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to establish tunnel: {e}")))?;

        info!(
            "✅ Tunnel established: {} (status: {})",
            establish_response.tunnel_id, establish_response.status
        );

        Ok(TunnelHandle {
            id: establish_response.tunnel_id,
        })
    }

    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        debug!("🔐 Encrypting {} bytes via RPC", data.len());

        #[derive(Serialize)]
        struct EncryptRequest<'a> {
            data: &'a [u8],
            tunnel_id: &'a str,
            peer_id: &'a str,
        }

        #[derive(Deserialize)]
        struct EncryptResponse {
            encrypted_data: Vec<u8>,
        }

        let request = EncryptRequest {
            data,
            tunnel_id: &context.tunnel_id,
            peer_id: &context.peer_id,
        };

        let encrypt_response: EncryptResponse = self
            .rpc_client
            .call("btsp.encrypt", &request)
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to encrypt data: {e}")))?;

        debug!(
            "✅ Encrypted {} bytes → {} bytes",
            data.len(),
            encrypt_response.encrypted_data.len()
        );

        Ok(encrypt_response.encrypted_data)
    }

    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        debug!("🔓 Decrypting {} bytes via RPC", data.len());

        #[derive(Serialize)]
        struct DecryptRequest<'a> {
            encrypted_data: &'a [u8],
            tunnel_id: &'a str,
            peer_id: &'a str,
        }

        #[derive(Deserialize)]
        struct DecryptResponse {
            data: Vec<u8>,
        }

        let request = DecryptRequest {
            encrypted_data: data,
            tunnel_id: &context.tunnel_id,
            peer_id: &context.peer_id,
        };

        let decrypt_response: DecryptResponse = self
            .rpc_client
            .call("btsp.decrypt", &request)
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to decrypt data: {e}")))?;

        debug!("✅ Decrypted {} bytes → {} bytes", data.len(), decrypt_response.data.len());

        Ok(decrypt_response.data)
    }

    async fn tunnel_status(&self, handle: &TunnelHandle) -> SongbirdResult<TunnelStatus> {
        debug!("🔍 Checking tunnel status: {} via RPC", handle.id);

        let request = serde_json::json!({
            "tunnel_id": handle.id,
        });

        let status: TunnelStatus = self
            .rpc_client
            .call("btsp.tunnel.status", &request)
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to get tunnel status: {e}")))?;

        debug!("✅ Tunnel {} status: {:?}", handle.id, status);

        Ok(status)
    }

    async fn close_tunnel(&self, handle: &TunnelHandle) -> SongbirdResult<()> {
        debug!("🔒 Closing tunnel: {} via RPC", handle.id);

        let request = serde_json::json!({
            "tunnel_id": handle.id,
        });

        // Call RPC method - ignore errors as tunnel might already be closed
        match self.rpc_client.call::<_, serde_json::Value>("btsp.tunnel.close", &request).await {
            Ok(_) => info!("✅ Tunnel {} closed", handle.id),
            Err(e) => {
                warn!("⚠️ Failed to close tunnel {}: {} (may already be closed)", handle.id, e);
            }
        }

        Ok(())
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }

    fn supports_genetic_auth(&self) -> bool {
        // Remote provider capabilities - assume true if connected
        true
    }

    fn supports_key_lineage(&self) -> bool {
        // Remote provider capabilities - assume true if connected
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_provider_creation() {
        let provider = HttpBtspProvider::new(
            "https://localhost:8091".to_string(),
            "test-provider".to_string(),
        )
        .unwrap();

        assert_eq!(provider.provider_name(), "test-provider");
        assert!(provider.supports_genetic_auth());
        assert!(provider.supports_key_lineage());
    }

    // Note: Integration tests require a running security provider
    // See showcase/13-beardog-integration/03-btsp-live-integration-test.sh
}
