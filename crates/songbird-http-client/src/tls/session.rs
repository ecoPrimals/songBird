// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS session management

use crate::security_rpc_client::SecurityRpcClient;
use std::sync::Arc;
use tokio::sync::RwLock;

/// TLS session state
#[derive(Debug)]
pub struct TlsSession {
    /// Security provider RPC client for crypto operations
    security_rpc_client: Arc<SecurityRpcClient>,
    /// Session keys
    keys: Arc<RwLock<Option<SessionKeys>>>,
    /// Server name (SNI)
    server_name: String,
}

/// Session keys
#[derive(Debug, Clone)]
pub struct SessionKeys {
    pub client_write_key: Vec<u8>,
    pub server_write_key: Vec<u8>,
    pub client_write_iv: Vec<u8>,
    pub server_write_iv: Vec<u8>,
    /// TLS 1.3 cipher suite (0x1301=AES-128-GCM, 0x1302=AES-256-GCM, 0x1303=ChaCha20-Poly1305)
    pub cipher_suite: u16,
    /// Initial read sequence number (accounts for post-handshake messages consumed during handshake)
    pub initial_read_sequence: u64,
}

impl TlsSession {
    /// Create a new TLS session
    pub fn new(security_rpc_client: Arc<SecurityRpcClient>, server_name: String) -> Self {
        Self {
            security_rpc_client,
            keys: Arc::new(RwLock::new(None)),
            server_name,
        }
    }

    /// Get server name
    #[must_use]
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Set session keys
    pub async fn set_keys(&self, keys: SessionKeys) {
        let mut guard = self.keys.write().await;
        *guard = Some(keys);
    }

    /// Get session keys
    pub async fn keys(&self) -> Option<SessionKeys> {
        let guard = self.keys.read().await;
        guard.clone()
    }

    /// Get the security provider RPC client
    #[must_use]
    pub fn security_rpc_client(&self) -> &SecurityRpcClient {
        &self.security_rpc_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let rpc = Arc::new(SecurityRpcClient::new("/tmp/beardog.sock"));
        let session = TlsSession::new(rpc, "example.com".to_string());
        assert_eq!(session.server_name(), "example.com");
        assert!(session.keys().await.is_none());
    }

    #[tokio::test]
    async fn test_session_keys() {
        let rpc = Arc::new(SecurityRpcClient::new("/tmp/beardog.sock"));
        let session = TlsSession::new(rpc, "example.com".to_string());

        let keys = SessionKeys {
            client_write_key: vec![1, 2, 3],
            server_write_key: vec![4, 5, 6],
            client_write_iv: vec![7, 8, 9],
            server_write_iv: vec![10, 11, 12],
            cipher_suite: 0x1303, // ChaCha20-Poly1305 for test
            initial_read_sequence: 0,
        };

        session.set_keys(keys.clone()).await;
        let retrieved = session.keys().await;
        assert!(retrieved.is_some());
    }
}
