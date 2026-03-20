// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` Provider for `CryptoCapability`
//!
//! Implements `CryptoCapability` trait using `BearDog` via JSON-RPC 2.0
//! over Unix sockets.

mod rpc;

mod crypto_impl;

#[cfg(test)]
mod tests;

use std::sync::atomic::AtomicU64;

pub use rpc::RoutingMode;

/// `BearDog` implementation of `CryptoCapability`
#[derive(Debug)]
pub struct BearDogProvider {
    pub(super) socket_path: String,
    pub(super) request_id: AtomicU64,
    pub(super) mode: RoutingMode,
}

impl BearDogProvider {
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

    pub fn from_env() -> Self {
        use super::socket_discovery;
        use tracing::info;

        let mode = std::env::var("BEARDOG_MODE").unwrap_or_else(|_| "neural".to_string());

        if mode.as_str() == "direct" {
            let socket = socket_discovery::discover_beardog_socket();
            info!("🔧 BearDog provider: DIRECT mode → {}", socket);
            Self {
                socket_path: socket,
                request_id: AtomicU64::new(1),
                mode: RoutingMode::Direct,
            }
        } else {
            let socket = socket_discovery::discover_neural_api_socket();
            info!("🌐 BearDog provider: NEURAL API mode (capability.call) → {}", socket);
            Self {
                socket_path: socket,
                request_id: AtomicU64::new(1),
                mode: RoutingMode::NeuralApi,
            }
        }
    }

    #[allow(dead_code, reason = "public accessor retained for IPC diagnostics and future callers")]
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}
