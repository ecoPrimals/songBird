// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` client core
//!
//! Client struct, mode enum, and constructors.

use crate::crypto::socket_discovery::IpcEndpoint;
use songbird_types::primal_names::NEURAL_API;
use std::sync::atomic::AtomicU64;
use tracing::info;

/// `BearDog` communication mode
///
/// Songbird supports two modes of communication with `BearDog`:
/// - **Direct mode**: Talk directly to `BearDog` (testing, simple deployments)
/// - **Neural API mode**: Route through Neural API (production, orchestration)
#[derive(Debug, Clone)]
pub enum BearDogMode {
    /// Direct RPC to `BearDog` (testing, simple deployments)
    ///
    /// - Fast (no routing overhead)
    /// - Simple (no discovery needed)
    /// - Fixed topology (you know what you need)
    /// - Uses actual `BearDog` method names (e.g., `x25519_generate_ephemeral`)
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

/// `BearDog` RPC client with dual-mode support
///
/// Routes through Neural API for capability translation in `NeuralApi` mode,
/// or talks directly to `BearDog` in Direct mode.
#[derive(Debug)]
pub struct BearDogClient {
    pub(super) mode: BearDogMode,
    pub(super) request_id: AtomicU64,
}

impl BearDogClient {
    /// Create client in Direct mode (testing, simple deployments)
    ///
    /// Talks directly to `BearDog` via Unix socket or TCP (automatic fallback).
    /// Uses actual `BearDog` method names (e.g., `x25519_generate_ephemeral`).
    ///
    /// # Example
    /// ```rust,ignore
    /// use songbird_http_client::BearDogClient;
    /// let beardog = BearDogClient::new_direct("/tmp/beardog.sock");
    /// ```
    pub fn new_direct(beardog_socket: impl Into<String>) -> Self {
        info!("🔧 BearDog client: DIRECT mode (testing/simple deployments)");
        let socket_path = beardog_socket.into();
        let endpoint = IpcEndpoint::UnixSocket(socket_path);
        Self {
            mode: BearDogMode::Direct {
                endpoint,
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Create client in Direct mode with explicit endpoint (isomorphic)
    ///
    /// Allows specifying TCP or Unix socket explicitly for testing.
    pub fn new_direct_with_endpoint(endpoint: IpcEndpoint) -> Self {
        info!("🔧 BearDog client: DIRECT mode (explicit endpoint)");
        Self {
            mode: BearDogMode::Direct {
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
    /// use songbird_http_client::BearDogClient;
    /// let beardog = BearDogClient::new_neural_api("/tmp/neural-api.sock");
    /// ```
    pub fn new_neural_api(neural_api_socket: impl Into<String>) -> Self {
        info!("🌐 BearDog client: NEURAL API mode (production/orchestration)");
        let socket_path = neural_api_socket.into();
        let endpoint = IpcEndpoint::UnixSocket(socket_path);
        Self {
            mode: BearDogMode::NeuralApi {
                endpoint,
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Create client in Neural API mode with explicit endpoint (isomorphic)
    ///
    /// Allows specifying TCP or Unix socket explicitly for production.
    pub fn new_neural_api_with_endpoint(endpoint: IpcEndpoint) -> Self {
        info!("🌐 BearDog client: NEURAL API mode (explicit endpoint)");
        Self {
            mode: BearDogMode::NeuralApi {
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
    /// Checks `SECURITY_PROVIDER_MODE` (preferred) or `BEARDOG_MODE` to determine mode:
    /// - "direct" → Direct mode (discovers `BearDog` endpoint) - DEPRECATED for production
    /// - "neural" or default → Neural API mode (discovers Neural API endpoint) - TRUE PRIMAL pattern
    ///
    /// Uses isomorphic discovery to automatically find Unix socket or TCP endpoint.
    pub fn from_env() -> Self {
        use crate::crypto::socket_discovery;

        let mode = songbird_process_env::var("SECURITY_PROVIDER_MODE")
            .or_else(|_| songbird_process_env::var("BEARDOG_MODE"))
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
            info!("🔧 BearDog mode from env: DIRECT → {:?}", endpoint);
            Self::new_direct_with_endpoint(endpoint)
        } else {
            // Default to Neural API (TRUE PRIMAL pattern)
            let endpoint = socket_discovery::discover_ipc_endpoint(
                "NEURAL_API_SOCKET",
                NEURAL_API,
                &format!("{}/neural-api-nat0.sock", std::env::temp_dir().display()),
            );
            info!("🌐 BearDog mode from env: NEURAL API → {:?}", endpoint);
            Self::new_neural_api_with_endpoint(endpoint)
        }
    }

    /// Get endpoint based on mode (for diagnostics/debugging)
    #[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    pub(super) const fn endpoint(&self) -> &IpcEndpoint {
        match &self.mode {
            BearDogMode::Direct {
                endpoint,
                ..
            }
            | BearDogMode::NeuralApi {
                endpoint,
                ..
            } => endpoint,
        }
    }

    /// Check if in Neural API mode (for diagnostics/debugging)
    #[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    pub(super) const fn is_neural_api(&self) -> bool {
        matches!(self.mode, BearDogMode::NeuralApi { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation_direct() {
        let client = BearDogClient::new_direct("/tmp/beardog.sock");
        assert!(matches!(client.mode, BearDogMode::Direct { .. }));
    }

    #[test]
    fn test_beardog_client_creation_neural_api() {
        let client = BearDogClient::new_neural_api("/tmp/neural-api-nat0.sock");
        assert!(matches!(client.mode, BearDogMode::NeuralApi { .. }));
    }

    #[test]
    fn test_beardog_client_creation_backward_compat() {
        let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
        assert!(matches!(client.mode, BearDogMode::NeuralApi { .. }));
    }

    #[test]
    fn test_endpoint_direct() {
        let client = BearDogClient::new_direct("/tmp/test.sock");
        assert!(matches!(client.endpoint(), IpcEndpoint::UnixSocket(_)));
    }

    #[test]
    fn test_endpoint_neural() {
        let client = BearDogClient::new_neural_api("/tmp/neural.sock");
        assert!(matches!(client.endpoint(), IpcEndpoint::UnixSocket(_)));
    }

    #[test]
    fn test_is_neural_api() {
        let direct = BearDogClient::new_direct("/tmp/beardog.sock");
        let neural = BearDogClient::new_neural_api("/tmp/neural.sock");

        assert!(!direct.is_neural_api());
        assert!(neural.is_neural_api());
    }

    #[test]
    fn test_endpoint_tcp_explicit() {
        use std::net::SocketAddr;
        let tcp_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let endpoint = IpcEndpoint::TcpLocal(tcp_addr);
        let client = BearDogClient::new_direct_with_endpoint(endpoint);
        assert!(matches!(client.endpoint(), IpcEndpoint::TcpLocal(_)));
    }
}
