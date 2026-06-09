// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Shared crypto provider for all Songbird crates.
//!
//! Extracted from `songbird-http-client` so that `songbird-tor-protocol`,
//! `songbird-orchestrator`, `songbird-nfc`, `songbird-sovereign-onion`,
//! and `songbird-quic` can all route crypto through the Neural API
//! without pulling in HTTP dependencies.
//!
//! ## Routing Modes
//!
//! - **`NeuralApi`** (default): Routes calls as `capability.call` to the
//!   Neural API, which translates and forwards to the security provider.
//! - **`Direct`**: Calls `security provider` directly (bootstrap / fallback).

mod rpc;
pub mod socket_discovery;

use std::sync::atomic::AtomicU64;

pub use rpc::RoutingMode;

/// Interprets `SECURITY_PROVIDER_MODE` env var.
///
/// - Missing or any value other than `"direct"` selects [`RoutingMode::NeuralApi`].
/// - `"direct"` selects [`RoutingMode::Direct`] (bootstrap / fallback).
#[must_use]
pub fn routing_mode_from_env(mode_value: Option<&str>) -> RoutingMode {
    match mode_value.unwrap_or("api") {
        "direct" => RoutingMode::Direct,
        _ => RoutingMode::NeuralApi,
    }
}

/// JSON-RPC transport and wire-format errors from [`CryptoProvider::call`](CryptoProvider::call).
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Failed to serialize the outgoing JSON-RPC request body.
    #[error("failed to serialize JSON-RPC request")]
    RequestSerialize(#[source] serde_json::Error),

    /// Could not connect to the Unix socket at `path`.
    #[error("failed to connect to {target} at {path}: {source}")]
    Connect {
        /// Human-readable label for the target service (e.g. "neural-api", "security-provider").
        target: &'static str,
        /// Filesystem path to the Unix socket.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Write to the socket failed after connection was established.
    #[error("failed to send request: {0}")]
    SendRequest(#[source] std::io::Error),

    /// Half-close of the socket's write side failed.
    #[error("failed to shutdown write side of socket: {0}")]
    ShutdownWrite(#[source] std::io::Error),

    /// Reading the response bytes from the socket failed.
    #[error("failed to read response: {0}")]
    ReadResponse(#[source] std::io::Error),

    /// The response bytes were not valid JSON-RPC.
    #[error("failed to parse JSON-RPC response: {source} (raw: {raw_preview})")]
    ResponseParse {
        /// First bytes of the raw response (for diagnostics).
        raw_preview: String,
        /// Serde parse error.
        #[source]
        source: serde_json::Error,
    },

    /// The remote returned a JSON-RPC error object.
    #[error("JSON-RPC error: {message} (code: {code})")]
    Remote {
        /// JSON-RPC error code.
        code: i32,
        /// JSON-RPC error message.
        message: String,
    },

    /// The response's `result` field was JSON `null`.
    #[error("JSON-RPC response contained null result")]
    NullResult,
}

/// Crypto provider error.
#[derive(Debug, thiserror::Error)]
pub enum CryptoProviderError {
    /// An RPC-layer error occurred (transport, serialization, or remote error).
    #[error(transparent)]
    Rpc(#[from] RpcError),
}

/// Convenience alias for crypto provider operations.
pub type Result<T> = std::result::Result<T, CryptoProviderError>;

/// Crypto provider that routes operations via Neural API or direct `security provider`.
#[derive(Debug)]
pub struct CryptoProvider {
    pub(crate) socket_path: String,
    pub(crate) request_id: AtomicU64,
    pub(crate) mode: RoutingMode,
}

impl CryptoProvider {
    /// Create a provider in [`RoutingMode::Direct`] targeting the given socket path.
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: AtomicU64::new(1),
            mode: RoutingMode::Direct,
        }
    }

    /// Create a provider with an explicit [`RoutingMode`] and socket path.
    #[must_use]
    pub fn with_mode(socket_path: impl Into<String>, mode: RoutingMode) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: AtomicU64::new(1),
            mode,
        }
    }

    /// Create a provider from environment variables.
    ///
    /// Defaults to `NeuralApi` mode. Set `SECURITY_PROVIDER_MODE=direct`
    /// (or legacy `BEARDOG_MODE=direct`) to bypass the Neural API (bootstrap only).
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with(|key| songbird_process_env::var(key).ok())
    }

    /// Create a provider using a custom environment lookup (for testing and embedding).
    pub fn from_env_with<F>(get_var: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        use tracing::info;

        let mode_val = get_var("SECURITY_PROVIDER_MODE").or_else(|| {
            get_var("BEARDOG_MODE").inspect(|_| {
                tracing::warn!(
                    "BEARDOG_MODE is deprecated — migrate to SECURITY_PROVIDER_MODE; prefer CAPABILITY_* or SECURITY_PROVIDER_* env vars (capability-first)"
                );
            })
        });
        let mode = routing_mode_from_env(mode_val.as_deref());
        match mode {
            RoutingMode::Direct => {
                let socket = socket_discovery::discover_security_socket_with(
                    |k| get_var(k),
                    std::path::Path::exists,
                );
                info!("🔧 Crypto provider: DIRECT mode → {}", socket);
                Self {
                    socket_path: socket,
                    request_id: AtomicU64::new(1),
                    mode: RoutingMode::Direct,
                }
            }
            RoutingMode::NeuralApi => {
                let socket = socket_discovery::discover_neural_api_socket_with(
                    |k| get_var(k),
                    std::path::Path::exists,
                );
                info!("🌐 Crypto provider: NEURAL API mode (capability.call) → {}", socket);
                Self {
                    socket_path: socket,
                    request_id: AtomicU64::new(1),
                    mode: RoutingMode::NeuralApi,
                }
            }
        }
    }

    /// Returns the Unix socket path this provider is configured to use.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

impl Clone for CryptoProvider {
    fn clone(&self) -> Self {
        Self {
            socket_path: self.socket_path.clone(),
            request_id: AtomicU64::new(1),
            mode: self.mode,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn socket_path_returns_configured_path() {
        let p = CryptoProvider::with_mode("/var/run/crypto.sock", RoutingMode::NeuralApi);
        assert_eq!(
            p.socket_path(),
            "/var/run/crypto.sock",
            "socket_path should expose the configured Unix socket path"
        );
    }

    #[test]
    fn new_uses_direct_mode_and_stores_socket_path() {
        let p = CryptoProvider::new("/tmp/security-provider.sock");
        assert_eq!(p.mode, RoutingMode::Direct);
        assert_eq!(p.socket_path, "/tmp/security-provider.sock");
        assert_eq!(p.request_id.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn with_mode_sets_routing_mode() {
        let direct = CryptoProvider::with_mode("/x", RoutingMode::Direct);
        assert_eq!(direct.mode, RoutingMode::Direct);
        let neural = CryptoProvider::with_mode("/y", RoutingMode::NeuralApi);
        assert_eq!(neural.mode, RoutingMode::NeuralApi);
    }

    #[test]
    fn clone_resets_request_id_and_copies_path_and_mode() {
        let a = CryptoProvider::with_mode("/sock", RoutingMode::NeuralApi);
        a.request_id.fetch_add(5, std::sync::atomic::Ordering::SeqCst);
        let b = a.clone();
        assert_eq!(
            a.request_id.load(std::sync::atomic::Ordering::SeqCst),
            6,
            "original keeps its counter"
        );
        assert_eq!(b.socket_path, "/sock");
        assert_eq!(b.mode, RoutingMode::NeuralApi);
        assert_eq!(b.request_id.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn routing_mode_from_env_direct() {
        assert_eq!(routing_mode_from_env(Some("direct")), RoutingMode::Direct);
    }

    #[test]
    fn routing_mode_from_env_defaults_to_neural_when_unset() {
        assert_eq!(routing_mode_from_env(None), RoutingMode::NeuralApi);
    }

    #[test]
    fn routing_mode_from_env_non_direct_is_neural() {
        for v in ["", "neural", "NEURAL", "bogus"] {
            assert_eq!(routing_mode_from_env(Some(v)), RoutingMode::NeuralApi, "value={v:?}");
        }
    }

    #[test]
    fn from_env_with_neural_uses_neural_socket_from_env() {
        let p = CryptoProvider::from_env_with(|key| match key {
            "SECURITY_PROVIDER_MODE" => Some("neural".to_string()),
            "NEURAL_API_SOCKET" => Some("/run/neural.sock".to_string()),
            _ => None,
        });
        assert_eq!(p.mode, RoutingMode::NeuralApi);
        assert_eq!(p.socket_path, "/run/neural.sock");
    }

    #[test]
    fn from_env_with_defaults_to_neural_when_security_provider_mode_absent() {
        let p = CryptoProvider::from_env_with(|key| match key {
            "NEURAL_API_SOCKET" => Some("/only/neural.sock".to_string()),
            _ => None,
        });
        assert_eq!(p.mode, RoutingMode::NeuralApi, "unset mode env should default to NeuralApi");
        assert_eq!(p.socket_path, "/only/neural.sock");
    }

    /// Backward-compat: `BEARDOG_MODE` / `BEARDOG_SOCKET` still select direct routing and socket
    /// path when canonical `SECURITY_*` keys are absent (deprecated env shim).
    #[test]
    fn from_env_with_direct_prefers_legacy_beardog_socket_env() {
        let p = CryptoProvider::from_env_with(|key| match key {
            "BEARDOG_MODE" => Some("direct".to_string()),
            "BEARDOG_SOCKET" => Some("/custom/security-provider.sock".to_string()),
            _ => None,
        });
        assert_eq!(p.mode, RoutingMode::Direct);
        assert_eq!(p.socket_path, "/custom/security-provider.sock");
    }

    /// Backward-compat: direct mode via deprecated `BEARDOG_MODE` still discovers XDG crypto.sock
    /// when `BEARDOG_SOCKET` is unset (same behavior as production `from_env`).
    #[test]
    fn from_env_with_direct_discovers_xdg_biomeos_crypto_when_file_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let xdg = dir.path().to_string_lossy().to_string();
        let socket_path = crate::socket_discovery::crypto_socket_path_in_biomeos_runtime(&xdg, "");
        std::fs::create_dir_all(socket_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&socket_path, b"x").expect("touch socket path");
        let p = CryptoProvider::from_env_with(move |key| match key {
            "BEARDOG_MODE" => Some("direct".to_string()),
            "XDG_RUNTIME_DIR" => Some(xdg.clone()),
            _ => None,
        });
        assert_eq!(p.mode, RoutingMode::Direct);
        assert_eq!(
            p.socket_path,
            socket_path.to_string_lossy(),
            "direct mode without legacy BEARDOG_SOCKET should use XDG biomeos crypto.sock when present"
        );
    }
}
