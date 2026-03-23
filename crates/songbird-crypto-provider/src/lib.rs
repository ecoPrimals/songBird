// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
//! - **`Direct`**: Calls `BearDog` directly (bootstrap / fallback).

mod rpc;
pub mod socket_discovery;

use std::sync::atomic::AtomicU64;

pub use rpc::RoutingMode;

/// Interprets `BEARDOG_MODE` the same way as [`CryptoProvider::from_env`].
///
/// - Missing or any value other than `"direct"` selects [`RoutingMode::NeuralApi`].
/// - `"direct"` selects [`RoutingMode::Direct`].
#[must_use]
pub fn routing_mode_from_beardog_env_value(beardog_mode: Option<&str>) -> RoutingMode {
    match beardog_mode.unwrap_or("neural") {
        "direct" => RoutingMode::Direct,
        _ => RoutingMode::NeuralApi,
    }
}

/// Crypto provider error.
#[derive(Debug, thiserror::Error)]
pub enum CryptoProviderError {
    #[error("RPC error: {0}")]
    Rpc(String),
}

pub type Result<T> = std::result::Result<T, CryptoProviderError>;

/// Crypto provider that routes operations via Neural API or direct `BearDog`.
#[derive(Debug)]
pub struct CryptoProvider {
    pub(crate) socket_path: String,
    pub(crate) request_id: AtomicU64,
    pub(crate) mode: RoutingMode,
}

impl CryptoProvider {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: AtomicU64::new(1),
            mode: RoutingMode::Direct,
        }
    }

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
    /// Defaults to `NeuralApi` mode. Set `BEARDOG_MODE=direct` to bypass
    /// the Neural API (bootstrap only).
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with(|key| std::env::var(key).ok())
    }

    /// Create a provider using a custom environment lookup (for testing and embedding).
    pub fn from_env_with<F>(get_var: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        use tracing::info;

        let mode = routing_mode_from_beardog_env_value(get_var("BEARDOG_MODE").as_deref());
        match mode {
            RoutingMode::Direct => {
                let socket = socket_discovery::discover_beardog_socket_with(
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

    #[allow(dead_code, reason = "public API reserved for BearDog integration consumers")]
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
mod tests {
    use super::*;

    #[test]
    fn new_uses_direct_mode_and_stores_socket_path() {
        let p = CryptoProvider::new("/tmp/beardog.sock");
        assert_eq!(p.mode, RoutingMode::Direct);
        assert_eq!(p.socket_path, "/tmp/beardog.sock");
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
    fn routing_mode_from_beardog_env_value_direct() {
        assert_eq!(routing_mode_from_beardog_env_value(Some("direct")), RoutingMode::Direct);
    }

    #[test]
    fn routing_mode_from_beardog_env_value_defaults_to_neural_when_unset() {
        assert_eq!(routing_mode_from_beardog_env_value(None), RoutingMode::NeuralApi);
    }

    #[test]
    fn routing_mode_from_beardog_env_value_non_direct_is_neural() {
        for v in ["", "neural", "NEURAL", "bogus"] {
            assert_eq!(
                routing_mode_from_beardog_env_value(Some(v)),
                RoutingMode::NeuralApi,
                "value={v:?}"
            );
        }
    }

    #[test]
    fn from_env_with_neural_uses_neural_socket_from_env() {
        let p = CryptoProvider::from_env_with(|key| match key {
            "BEARDOG_MODE" => Some("neural".to_string()),
            "NEURAL_API_SOCKET" => Some("/run/neural.sock".to_string()),
            _ => None,
        });
        assert_eq!(p.mode, RoutingMode::NeuralApi);
        assert_eq!(p.socket_path, "/run/neural.sock");
    }

    #[test]
    fn from_env_with_direct_prefers_beardog_socket_env() {
        let p = CryptoProvider::from_env_with(|key| match key {
            "BEARDOG_MODE" => Some("direct".to_string()),
            "BEARDOG_SOCKET" => Some("/custom/beardog.sock".to_string()),
            _ => None,
        });
        assert_eq!(p.mode, RoutingMode::Direct);
        assert_eq!(p.socket_path, "/custom/beardog.sock");
    }
}
