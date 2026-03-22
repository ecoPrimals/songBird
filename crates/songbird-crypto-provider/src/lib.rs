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
//! - **`Direct`**: Calls BearDog directly (bootstrap / fallback).

mod rpc;
pub mod socket_discovery;

use std::sync::atomic::AtomicU64;

pub use rpc::RoutingMode;

/// Crypto provider error.
#[derive(Debug, thiserror::Error)]
pub enum CryptoProviderError {
    #[error("RPC error: {0}")]
    Rpc(String),
}

pub type Result<T> = std::result::Result<T, CryptoProviderError>;

/// Crypto provider that routes operations via Neural API or direct BearDog.
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
    pub fn from_env() -> Self {
        use tracing::info;

        let mode = std::env::var("BEARDOG_MODE").unwrap_or_else(|_| "neural".to_string());

        if mode.as_str() == "direct" {
            let socket = socket_discovery::discover_beardog_socket();
            info!("🔧 Crypto provider: DIRECT mode → {}", socket);
            Self {
                socket_path: socket,
                request_id: AtomicU64::new(1),
                mode: RoutingMode::Direct,
            }
        } else {
            let socket = socket_discovery::discover_neural_api_socket();
            info!("🌐 Crypto provider: NEURAL API mode (capability.call) → {}", socket);
            Self {
                socket_path: socket,
                request_id: AtomicU64::new(1),
                mode: RoutingMode::NeuralApi,
            }
        }
    }

    #[allow(dead_code)]
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
