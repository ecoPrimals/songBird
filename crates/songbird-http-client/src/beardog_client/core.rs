//! BearDog client core
//!
//! Client struct, mode enum, and constructors.

use std::sync::atomic::AtomicU64;
use tracing::info;

/// BearDog communication mode
///
/// Songbird supports two modes of communication with BearDog:
/// - **Direct mode**: Talk directly to BearDog (testing, simple deployments)
/// - **Neural API mode**: Route through Neural API (production, orchestration)
#[derive(Debug, Clone)]
pub enum BearDogMode {
    /// Direct RPC to BearDog (testing, simple deployments)
    ///
    /// - Fast (no routing overhead)
    /// - Simple (no discovery needed)
    /// - Fixed topology (you know what you need)
    /// - Uses actual BearDog method names (e.g., "x25519_generate_ephemeral")
    Direct {
        socket_path: String,
    },

    /// Via Neural API (production, orchestration, evolution)
    ///
    /// - Capability discovery
    /// - Semantic translation
    /// - Evolution support
    /// - Load balancing & failover
    /// - Uses semantic capability names (e.g., "crypto.generate_keypair")
    NeuralApi {
        socket_path: String,
    },
}

/// BearDog RPC client with dual-mode support
///
/// Routes through Neural API for capability translation in NeuralApi mode,
/// or talks directly to BearDog in Direct mode.
#[derive(Debug)]
pub struct BearDogClient {
    pub(super) mode: BearDogMode,
    pub(super) request_id: AtomicU64,
}

impl BearDogClient {
    /// Create client in Direct mode (testing, simple deployments)
    ///
    /// Talks directly to BearDog via Unix socket.
    /// Uses actual BearDog method names (e.g., "x25519_generate_ephemeral").
    ///
    /// # Example
    /// ```rust,ignore
    /// use songbird_http_client::BearDogClient;
    /// let beardog = BearDogClient::new_direct("/tmp/beardog.sock");
    /// ```
    pub fn new_direct(beardog_socket: impl Into<String>) -> Self {
        info!("🔧 BearDog client: DIRECT mode (testing/simple deployments)");
        Self {
            mode: BearDogMode::Direct {
                socket_path: beardog_socket.into(),
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Create client in Neural API mode (production, orchestration)
    ///
    /// Routes through Neural API for capability discovery and translation.
    /// Uses semantic capability names (e.g., "crypto.generate_keypair").
    ///
    /// # Example
    /// ```rust,ignore
    /// use songbird_http_client::BearDogClient;
    /// let beardog = BearDogClient::new_neural_api("/tmp/neural-api.sock");
    /// ```
    pub fn new_neural_api(neural_api_socket: impl Into<String>) -> Self {
        info!("🌐 BearDog client: NEURAL API mode (production/orchestration)");
        Self {
            mode: BearDogMode::NeuralApi {
                socket_path: neural_api_socket.into(),
            },
            request_id: AtomicU64::new(1),
        }
    }

    /// Existing constructor (backward compatible)
    /// Defaults to Neural API mode for compatibility
    pub fn new(neural_api_socket: impl Into<String>) -> Self {
        Self::new_neural_api(neural_api_socket)
    }

    /// Create from environment variable
    ///
    /// Checks BEARDOG_MODE env var to determine mode:
    /// - "direct" → Direct mode (BEARDOG_SOCKET) - DEPRECATED for production
    /// - "neural" or default → Neural API mode (NEURAL_API_SOCKET) - TRUE PRIMAL pattern
    pub fn from_env() -> Self {
        let mode = std::env::var("BEARDOG_MODE").unwrap_or_else(|_| "neural".to_string());

        match mode.to_lowercase().as_str() {
            "direct" => {
                let socket = std::env::var("BEARDOG_SOCKET")
                    .unwrap_or_else(|_| "/tmp/beardog.sock".to_string());
                info!("🔧 BearDog mode from env: DIRECT → {}", socket);
                Self::new_direct(socket)
            }
            _ => {
                // Default to Neural API (TRUE PRIMAL pattern)
                let socket = std::env::var("NEURAL_API_SOCKET")
                    .or_else(|_| std::env::var("NEURALS_SOCKET"))
                    .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
                info!("🌐 BearDog mode from env: NEURAL API → {}", socket);
                Self::new_neural_api(socket)
            }
        }
    }

    /// Get socket path based on mode
    pub(super) fn socket_path(&self) -> &str {
        match &self.mode {
            BearDogMode::Direct { socket_path } => socket_path,
            BearDogMode::NeuralApi { socket_path } => socket_path,
        }
    }

    /// Check if in Neural API mode
    pub(super) fn is_neural_api(&self) -> bool {
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
    fn test_socket_path_direct() {
        let client = BearDogClient::new_direct("/tmp/test.sock");
        assert_eq!(client.socket_path(), "/tmp/test.sock");
    }

    #[test]
    fn test_socket_path_neural() {
        let client = BearDogClient::new_neural_api("/tmp/neural.sock");
        assert_eq!(client.socket_path(), "/tmp/neural.sock");
    }

    #[test]
    fn test_is_neural_api() {
        let direct = BearDogClient::new_direct("/tmp/beardog.sock");
        let neural = BearDogClient::new_neural_api("/tmp/neural.sock");

        assert!(!direct.is_neural_api());
        assert!(neural.is_neural_api());
    }
}

