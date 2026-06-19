// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Local BTSP Implementation
//!
//! ⚠️ **TRUE PRIMAL NOTE**: This module is for TESTING ONLY!
//! Requires `local-btsp` feature (which includes crypto deps).
//! Production builds should use `security provider` delegation (no crypto in Songbird).
//!
//! This module provides a local implementation of BTSP for testing and
//! development without requiring `security provider` to be running. It uses standard
//! Rust cryptography libraries to simulate the BTSP protocol.
//!
//! ## Security Notice
//!
//! This implementation is for TESTING ONLY. It does not provide the same
//! security guarantees as real `security provider` genetic cryptography:
//!
//! - Uses AES-256-GCM instead of genetic crypto
//! - No key lineage tracking
//! - No multi-party consent
//! - No threshold key schemes
//!
//! When `security provider` is available, it will be discovered and used automatically.

#![cfg(feature = "local-btsp")]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;

use super::provider::PeerInfo;
use super::tunnel::{SecurityContext, Tunnel, TunnelHandle, TunnelStatus};
use songbird_types::{SongbirdError, SongbirdResult};

/// Local BTSP provider for testing
///
/// This implementation uses standard AES-256-GCM encryption and is suitable
/// for testing federation without `security provider`. It maintains tunnels in memory
/// and uses symmetric encryption.
pub struct LocalBtspProvider {
    tunnels: Arc<RwLock<HashMap<String, Tunnel>>>,
    key_manager: Arc<LocalKeyManager>,
}

impl LocalBtspProvider {
    /// Create a new local BTSP provider
    pub fn new() -> Self {
        debug!("🔧 Initializing Local BTSP provider (testing mode)");
        warn!("⚠️  Local BTSP is for TESTING ONLY - not production-secure");

        Self {
            tunnels: Arc::new(RwLock::new(HashMap::new())),
            key_manager: Arc::new(LocalKeyManager::new()),
        }
    }

    /// Establish a secure tunnel with peer
    pub async fn establish_tunnel(&self, peer: &PeerInfo) -> SongbirdResult<TunnelHandle> {
        debug!("🔗 Establishing local BTSP tunnel with peer: {}", peer.id);

        // Generate shared key for this tunnel
        let shared_key = self.key_manager.generate_shared_key(peer).await?;

        // Create tunnel
        let tunnel = Tunnel::new(peer.id.clone(), peer.endpoint.clone(), shared_key);

        let handle = tunnel.handle.clone();

        // Store tunnel
        self.tunnels.write().await.insert(handle.id.clone(), tunnel);

        debug!("✅ Local BTSP tunnel established: {}", handle.id);
        Ok(handle)
    }

    /// Encrypt data for transmission through tunnel
    pub async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        // Get tunnel
        let tunnels = self.tunnels.read().await;
        let mut tunnel = tunnels
            .get(&context.tunnel_id)
            .ok_or_else(|| {
                SongbirdError::service("btsp", format!("Tunnel not found: {}", context.tunnel_id))
            })?
            .clone();
        drop(tunnels);

        // Encrypt using AES-256-GCM
        let encrypted = self.key_manager.encrypt(data, &tunnel.shared_key, context)?;

        // Update tunnel statistics
        tunnel.record_sent(encrypted.len());
        self.tunnels.write().await.insert(context.tunnel_id.clone(), tunnel);

        Ok(encrypted)
    }

    /// Decrypt data received through tunnel
    pub async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>> {
        // Get tunnel
        let tunnels = self.tunnels.read().await;
        let mut tunnel = tunnels
            .get(&context.tunnel_id)
            .ok_or_else(|| {
                SongbirdError::service("btsp", format!("Tunnel not found: {}", context.tunnel_id))
            })?
            .clone();
        drop(tunnels);

        // Decrypt using AES-256-GCM
        let decrypted = self.key_manager.decrypt(data, &tunnel.shared_key, context)?;

        // Update tunnel statistics
        tunnel.record_received(data.len());
        self.tunnels.write().await.insert(context.tunnel_id.clone(), tunnel);

        Ok(decrypted)
    }

    /// Get tunnel status
    pub async fn tunnel_status(&self, handle: &TunnelHandle) -> SongbirdResult<TunnelStatus> {
        let status = self
            .tunnels
            .read()
            .await
            .get(&handle.id)
            .ok_or_else(|| {
                SongbirdError::service("btsp", format!("Tunnel not found: {}", handle.id))
            })
            .map(super::tunnel::Tunnel::status)?;
        Ok(status)
    }

    /// Close tunnel
    pub async fn close_tunnel(&self, handle: &TunnelHandle) -> SongbirdResult<()> {
        debug!("🔒 Closing local BTSP tunnel: {}", handle.id);

        if let Some(tunnel) = self.tunnels.write().await.get_mut(&handle.id) {
            tunnel.close();
        }
        self.tunnels.write().await.remove(&handle.id);

        Ok(())
    }

    /// Get provider name (for logging/debugging)
    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        "Local"
    }

    /// Check if provider supports genetic auth
    #[must_use]
    pub fn supports_genetic_auth(&self) -> bool {
        false // Local implementation doesn't support genetic auth
    }

    /// Check if provider supports key lineage
    #[must_use]
    pub fn supports_key_lineage(&self) -> bool {
        false // Local implementation doesn't track key lineage
    }
}

impl Default for LocalBtspProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Local key manager for testing
///
/// Uses standard Rust crypto libraries (AES-256-GCM) for encryption.
struct LocalKeyManager;

impl LocalKeyManager {
    const fn new() -> Self {
        Self
    }

    /// Generate a shared key for a tunnel (simulated)
    async fn generate_shared_key(&self, _peer: &PeerInfo) -> SongbirdResult<Vec<u8>> {
        // Generate a random 256-bit key
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        Ok(key)
    }

    /// Encrypt data using AES-256-GCM
    #[allow(
        clippy::unused_self,
        reason = "instance method for symmetry with decrypt and future context"
    )]
    fn encrypt(
        &self,
        data: &[u8],
        key: &[u8],
        context: &SecurityContext,
    ) -> SongbirdResult<Vec<u8>> {
        // Create cipher from key
        let key_array: [u8; 32] =
            key.try_into().map_err(|_| SongbirdError::security("Invalid key length"))?;
        let cipher = Aes256Gcm::new(&key_array.into());

        // Generate or use provided nonce
        let nonce_bytes = if let Some(ref nonce) = context.nonce {
            if nonce.len() != 12 {
                return Err(SongbirdError::security("Invalid nonce length"));
            }
            let mut arr = [0u8; 12];
            arr.copy_from_slice(nonce);
            arr
        } else {
            let mut arr = [0u8; 12];
            OsRng.fill_bytes(&mut arr);
            arr
        };
        // Use GenericArray constructor instead of deprecated from_slice
        let nonce = Nonce::from(nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| SongbirdError::security(format!("Encryption failed: {e}")))?;

        // Prepend nonce to ciphertext for decryption
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    #[allow(
        clippy::unused_self,
        reason = "instance method for symmetry with encrypt and future context"
    )]
    fn decrypt(
        &self,
        data: &[u8],
        key: &[u8],
        _context: &SecurityContext,
    ) -> SongbirdResult<Vec<u8>> {
        if data.len() < 12 {
            return Err(SongbirdError::security("Data too short"));
        }

        // Extract nonce (first 12 bytes) - use array copy instead of deprecated from_slice
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&data[..12]);
        let nonce = Nonce::from(nonce_arr);
        let ciphertext = &data[12..];

        // Create cipher from key
        let key_array: [u8; 32] =
            key.try_into().map_err(|_| SongbirdError::security("Invalid key length"))?;
        let cipher = Aes256Gcm::new(&key_array.into());

        // Decrypt
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| SongbirdError::security(format!("Decryption failed: {e}")))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_btsp_provider_creation() {
        let provider = LocalBtspProvider::new();
        assert_eq!(provider.provider_name(), "Local");
        assert!(!provider.supports_genetic_auth());
        assert!(!provider.supports_key_lineage());
    }

    #[tokio::test]
    async fn test_tunnel_establishment() {
        let provider = LocalBtspProvider::new();
        let peer = PeerInfo {
            id: String::from("test-peer"),
            endpoint: String::from("http://localhost:8080"),
            public_key: None,
            protocols: vec![String::from("https")],
        };

        let handle = provider.establish_tunnel(&peer).await.unwrap();
        assert!(!handle.id.is_empty());

        // Check tunnel status
        let status = provider.tunnel_status(&handle).await.unwrap();
        assert_eq!(status.peer_id, "test-peer");
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let provider = LocalBtspProvider::new();
        let peer = PeerInfo {
            id: String::from("test-peer"),
            endpoint: String::from("http://localhost:8080"),
            public_key: None,
            protocols: vec![String::from("https")],
        };

        let handle = provider.establish_tunnel(&peer).await.unwrap();

        let context = SecurityContext {
            tunnel_id: handle.id.clone(),
            peer_id: peer.id.clone(),
            nonce: None,
            aad: None,
        };

        let plaintext = b"Hello, security provider!";
        let encrypted = provider.encrypt(plaintext, &context).await.unwrap();
        let decrypted = provider.decrypt(&encrypted, &context).await.unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[tokio::test]
    async fn test_tunnel_close() {
        let provider = LocalBtspProvider::new();
        let peer = PeerInfo {
            id: String::from("test-peer"),
            endpoint: String::from("http://localhost:8080"),
            public_key: None,
            protocols: vec![String::from("https")],
        };

        let handle = provider.establish_tunnel(&peer).await.unwrap();
        provider.close_tunnel(&handle).await.unwrap();

        // Tunnel should no longer be found
        let result = provider.tunnel_status(&handle).await;
        assert!(result.is_err());
    }
}
