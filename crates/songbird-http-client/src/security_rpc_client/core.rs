// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider RPC client core
//!
//! Client struct, mode enum, and constructors.

use crate::crypto::socket_discovery::IpcEndpoint;
use songbird_types::primal_names::NEURAL_API;
use std::sync::atomic::AtomicU64;
use tracing::{info, warn};

/// Security-provider communication mode
///
/// Songbird supports two modes of communication with the `security provider`:
/// - **Direct mode**: Talk directly to the `security provider` (testing, simple deployments)
/// - **Neural API mode**: Route through Neural API (production, orchestration)
#[derive(Debug, Clone)]
pub enum SecurityRpcMode {
    /// Direct RPC to the `security provider` (testing, simple deployments)
    ///
    /// - Fast (no routing overhead)
    /// - Simple (no discovery needed)
    /// - Fixed topology (you know what you need)
    /// - Uses actual provider method names (e.g., `x25519_generate_ephemeral`)
    Direct {
        endpoint: IpcEndpoint,
    },

    /// Via Neural API (production, orchestration, evolution)
    ///
    /// - Capability discovery
    /// - Semantic translation
    /// - Evolution support
    /// - Load balancing & failover
    /// - Uses semantic capability names (e.g., `crypto.generate_keypair`)
    NeuralApi {
        endpoint: IpcEndpoint,
    },
}

/// Security-provider RPC client with dual-mode support
///
/// Routes through Neural API for capability translation in `NeuralApi` mode,
/// or talks directly to the `security provider` in Direct mode.
#[derive(Debug)]
pub struct SecurityRpcClient {
    pub(super) mode: SecurityRpcMode,
    pub(super) request_id: AtomicU64,
}

impl SecurityRpcClient {
    /// Create client in Direct mode (testing, simple deployments)
    ///
    /// Talks directly to the `security provider` via Unix socket or TCP (automatic fallback).
    /// Uses actual provider method names (e.g., `x25519_generate_ephemeral`).
    ///
    /// # Example
    /// ```rust,ignore
    /// use songbird_http_client::SecurityRpcClient;
    /// let sock = std::env::temp_dir().join("songbird-test-security.sock");
    /// let client = SecurityRpcClient::new_direct(sock.to_string_lossy());
    /// ```
    pub fn new_direct(socket_path: impl Into<String>) -> Self {
        info!("🔧 Security provider client: DIRECT mode (testing/simple deployments)");
        let socket_path = socket_path.into();
        let endpoint = IpcEndpoint::UnixSocket(socket_path);
        Self {
            mode: SecurityRpcMode::Direct {
                endpoint,
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Create client in Direct mode with explicit endpoint (isomorphic)
    ///
    /// Allows specifying TCP or Unix socket explicitly for testing.
    pub fn new_direct_with_endpoint(endpoint: IpcEndpoint) -> Self {
        info!("🔧 Security provider client: DIRECT mode (explicit endpoint)");
        Self {
            mode: SecurityRpcMode::Direct {
                endpoint,
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Create client in Neural API mode (production, orchestration)
    ///
    /// Routes through Neural API for capability discovery and translation.
    /// Uses semantic capability names (e.g., `crypto.generate_keypair`).
    ///
    /// # Example
    /// ```rust,ignore
    /// use songbird_http_client::SecurityRpcClient;
    /// let client = SecurityRpcClient::new_neural_api("/tmp/neural-api.sock");
    /// ```
    pub fn new_neural_api(neural_api_socket: impl Into<String>) -> Self {
        info!("🌐 Security provider client: NEURAL API mode (production/orchestration)");
        let socket_path = neural_api_socket.into();
        let endpoint = IpcEndpoint::UnixSocket(socket_path);
        Self {
            mode: SecurityRpcMode::NeuralApi {
                endpoint,
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Create client in Neural API mode with explicit endpoint (isomorphic)
    ///
    /// Allows specifying TCP or Unix socket explicitly for production.
    pub fn new_neural_api_with_endpoint(endpoint: IpcEndpoint) -> Self {
        info!("🌐 Security provider client: NEURAL API mode (explicit endpoint)");
        Self {
            mode: SecurityRpcMode::NeuralApi {
                endpoint,
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Existing constructor (backward compatible)
    /// Defaults to Neural API mode for compatibility
    pub fn new(neural_api_socket: impl Into<String>) -> Self {
        Self::new_neural_api(neural_api_socket)
    }

    /// Create from environment variable with isomorphic discovery
    ///
    /// Checks `SECURITY_PROVIDER_MODE` (preferred) or legacy `BEARDOG_MODE` (prefer `SECURITY_PROVIDER_MODE`) to determine mode:
    /// - "direct" → Direct mode (discovers `security provider` endpoint) - DEPRECATED for production
    /// - "neural" or default → Neural API mode (discovers Neural API endpoint) - TRUE PRIMAL pattern
    ///
    /// Uses isomorphic discovery to automatically find Unix socket or TCP endpoint.
    pub fn from_env() -> Self {
        use crate::crypto::socket_discovery;

        let mode = songbird_process_env::var("SECURITY_PROVIDER_MODE")
            .or_else(|_| {
                songbird_process_env::var("BEARDOG_MODE").inspect(|_| {
                    warn!(
                        "BEARDOG_MODE is deprecated — migrate to SECURITY_PROVIDER_MODE; prefer CAPABILITY_* or SECURITY_PROVIDER_* env vars (capability-first)"
                    );
                })
            })
            .unwrap_or_else(|_| "neural".to_string());

        if mode.to_lowercase() == "direct" {
            // Direct mode: Discover crypto provider endpoint via capability
            let legacy_crypto = std::env::temp_dir().join("crypto.sock");
            let legacy_crypto = legacy_crypto.to_string_lossy();
            let endpoint = socket_discovery::discover_ipc_endpoint(
                "CRYPTO_PROVIDER_SOCKET",
                "crypto",
                legacy_crypto.as_ref(),
            );
            info!("🔧 Security provider mode from env: DIRECT → {:?}", endpoint);
            Self::new_direct_with_endpoint(endpoint)
        } else {
            // Default to Neural API (TRUE PRIMAL pattern)
            let endpoint = socket_discovery::discover_ipc_endpoint(
                "NEURAL_API_SOCKET",
                NEURAL_API,
                &format!("{}/neural-api-nat0.sock", std::env::temp_dir().display()),
            );
            info!("🌐 Security provider mode from env: NEURAL API → {:?}", endpoint);
            Self::new_neural_api_with_endpoint(endpoint)
        }
    }

    /// Get endpoint based on mode (for diagnostics/debugging)
    #[allow(dead_code, reason = "diagnostic accessor; used from unit tests and future logging")]
    pub(super) const fn endpoint(&self) -> &IpcEndpoint {
        match &self.mode {
            SecurityRpcMode::Direct {
                endpoint,
                ..
            }
            | SecurityRpcMode::NeuralApi {
                endpoint,
                ..
            } => endpoint,
        }
    }

    /// Check if in Neural API mode (for diagnostics/debugging)
    #[allow(dead_code, reason = "diagnostic accessor; used from unit tests and future logging")]
    pub(super) const fn is_neural_api(&self) -> bool {
        matches!(self.mode, SecurityRpcMode::NeuralApi { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_direct_socket_path() -> String {
        tempfile::env::temp_dir().join("songbird-test-security.sock").to_string_lossy().into_owned()
    }

    #[test]
    fn test_security_rpc_client_creation_direct() {
        let client = SecurityRpcClient::new_direct(test_direct_socket_path());
        assert!(matches!(client.mode, SecurityRpcMode::Direct { .. }));
    }

    #[test]
    fn test_security_rpc_client_creation_neural_api() {
        let client = SecurityRpcClient::new_neural_api("/tmp/neural-api-nat0.sock");
        assert!(matches!(client.mode, SecurityRpcMode::NeuralApi { .. }));
    }

    #[test]
    fn test_security_rpc_client_creation_backward_compat() {
        let client = SecurityRpcClient::new("/tmp/neural-api-nat0.sock");
        assert!(matches!(client.mode, SecurityRpcMode::NeuralApi { .. }));
    }

    #[test]
    fn test_endpoint_direct() {
        let client = SecurityRpcClient::new_direct("/tmp/test.sock");
        assert!(matches!(client.endpoint(), IpcEndpoint::UnixSocket(_)));
    }

    #[test]
    fn test_endpoint_neural() {
        let client = SecurityRpcClient::new_neural_api("/tmp/neural.sock");
        assert!(matches!(client.endpoint(), IpcEndpoint::UnixSocket(_)));
    }

    #[test]
    fn test_is_neural_api() {
        let direct = SecurityRpcClient::new_direct(test_direct_socket_path());
        let neural = SecurityRpcClient::new_neural_api("/tmp/neural.sock");

        assert!(!direct.is_neural_api());
        assert!(neural.is_neural_api());
    }

    #[test]
    fn test_endpoint_tcp_explicit() {
        use std::net::SocketAddr;
        let tcp_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let endpoint = IpcEndpoint::TcpLocal(tcp_addr);
        let client = SecurityRpcClient::new_direct_with_endpoint(endpoint);
        assert!(matches!(client.endpoint(), IpcEndpoint::TcpLocal(_)));
    }
}
