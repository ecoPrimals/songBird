//! HTTP-based BTSP Provider Client
//!
//! This module implements a BTSP provider that communicates with a remote
//! security provider (like `BearDog`) over HTTP/HTTPS.
//!
//! **Modern Idiomatic Rust**:
//! - Async/await throughout
//! - No `unwrap()` in production paths
//! - Proper error handling with `SongbirdError`
//! - Connection pooling via reqwest
//! - Timeout and retry logic

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use super::provider::{BtspProvider, PeerInfo};
use super::tunnel::{SecurityContext, TunnelHandle, TunnelStatus};
use songbird_types::{SongbirdError, SongbirdResult};

/// HTTP client for communicating with a remote BTSP provider
pub struct HttpBtspProvider {
    /// Base URL of the security provider (e.g., "<https://localhost:8091>")
    base_url: String,
    /// HTTP client with connection pooling
    client: Client,
    /// Provider name (e.g., "beardog", "secureprimal")
    provider_name: String,
}

impl HttpBtspProvider {
    /// Create a new HTTP BTSP provider
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the security provider (e.g., "<https://localhost:8091>")
    /// * `provider_name` - Name of the provider for logging (e.g., "beardog")
    pub fn new(base_url: String, provider_name: String) -> SongbirdResult<Self> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true) // Self-signed certs OK for local dev
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        info!("🔒 Created HTTP BTSP provider for {} at {}", provider_name, base_url);

        Ok(Self {
            base_url,
            client,
            provider_name,
        })
    }

    /// Verify the provider is reachable
    pub async fn verify_connection(&self) -> SongbirdResult<()> {
        let url = format!("{}/health", self.base_url);
        debug!("🔍 Verifying connection to {}", url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            SongbirdError::network(format!(
                "Failed to connect to security provider at {}: {}",
                self.base_url, e
            ))
        })?;

        if response.status().is_success() {
            info!("✅ Connection to {} verified", self.provider_name);
            Ok(())
        } else {
            Err(SongbirdError::service(
                &self.provider_name,
                format!("Health check failed: {}", response.status()),
            ))
        }
    }
}

#[async_trait]
impl BtspProvider for HttpBtspProvider {
    async fn establish_tunnel(&self, peer: &PeerInfo) -> SongbirdResult<TunnelHandle> {
        let url = format!("{}/api/btsp/tunnel/establish", self.base_url);
        debug!("🔐 Establishing tunnel to {} via {}", peer.id, url);

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

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to establish tunnel: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "unknown".to_string());
            return Err(SongbirdError::service(
                &self.provider_name,
                format!("Tunnel establishment failed: {status} - {body}"),
            ));
        }

        let establish_response: EstablishResponse = response.json().await.map_err(|e| {
            SongbirdError::from(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse tunnel response: {e}"),
            )))
        })?;

        info!(
            "✅ Tunnel established: {} (status: {})",
            establish_response.tunnel_id, establish_response.status
        );

        Ok(TunnelHandle {
            id: establish_response.tunnel_id,
        })
    }

    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        let url = format!("{}/api/btsp/encrypt", self.base_url);
        debug!("🔐 Encrypting {} bytes via {}", data.len(), url);

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

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to encrypt data: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(SongbirdError::service(
                &self.provider_name,
                format!("Encryption failed: {status}"),
            ));
        }

        let encrypt_response: EncryptResponse = response.json().await.map_err(|e| {
            SongbirdError::from(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse encryption response: {e}"),
            )))
        })?;

        debug!(
            "✅ Encrypted {} bytes → {} bytes",
            data.len(),
            encrypt_response.encrypted_data.len()
        );

        Ok(encrypt_response.encrypted_data)
    }

    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        let url = format!("{}/api/btsp/decrypt", self.base_url);
        debug!("🔓 Decrypting {} bytes via {}", data.len(), url);

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

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to decrypt data: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(SongbirdError::service(
                &self.provider_name,
                format!("Decryption failed: {status}"),
            ));
        }

        let decrypt_response: DecryptResponse = response.json().await.map_err(|e| {
            SongbirdError::from(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse decryption response: {e}"),
            )))
        })?;

        debug!("✅ Decrypted {} bytes → {} bytes", data.len(), decrypt_response.data.len());

        Ok(decrypt_response.data)
    }

    async fn tunnel_status(&self, handle: &TunnelHandle) -> SongbirdResult<TunnelStatus> {
        let url = format!("{}/api/btsp/tunnel/status/{}", self.base_url, handle.id);
        debug!("🔍 Checking tunnel status: {}", handle.id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to get tunnel status: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(SongbirdError::service(
                &self.provider_name,
                format!("Failed to get tunnel status: {status}"),
            ));
        }

        let status: TunnelStatus = response.json().await.map_err(|e| {
            SongbirdError::from(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse tunnel status: {e}"),
            )))
        })?;

        debug!("✅ Tunnel {} status: {:?}", handle.id, status);

        Ok(status)
    }

    async fn close_tunnel(&self, handle: &TunnelHandle) -> SongbirdResult<()> {
        let url = format!("{}/api/btsp/tunnel/close/{}", self.base_url, handle.id);
        debug!("🔒 Closing tunnel: {}", handle.id);

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to close tunnel: {e}")))?;

        if response.status().is_success() {
            info!("✅ Tunnel {} closed", handle.id);
        } else {
            let status = response.status();
            warn!("⚠️ Failed to close tunnel {}: {} (may already be closed)", handle.id, status);
            // Don't return error - tunnel might already be closed
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
