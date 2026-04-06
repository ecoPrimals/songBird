// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security / crypto provider for `CryptoCapability`
//!
//! Implements `CryptoCapability` trait using the security provider via JSON-RPC 2.0
//! over Unix sockets.

mod rpc;

mod crypto_impl;

#[cfg(test)]
mod tests;

use std::sync::atomic::AtomicU64;

pub use rpc::RoutingMode;

/// Security / crypto provider implementation of `CryptoCapability`
#[derive(Debug)]
pub struct SecurityCryptoProvider {
    pub(super) socket_path: String,
    pub(super) request_id: AtomicU64,
    pub(super) mode: RoutingMode,
}

impl SecurityCryptoProvider {
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

        let mode = songbird_process_env::var("SECURITY_PROVIDER_MODE")
            .or_else(|_| {
                songbird_process_env::var("BEARDOG_MODE").inspect(|_| {
                    tracing::warn!(
                        "BEARDOG_MODE is deprecated — migrate to SECURITY_PROVIDER_MODE; prefer CAPABILITY_* or SECURITY_PROVIDER_* env vars (capability-first)"
                    );
                })
            })
            .unwrap_or_else(|_| "neural".to_string());

        if mode.as_str() == "direct" {
            let socket = socket_discovery::discover_security_socket();
            info!("🔧 Security provider: DIRECT mode → {}", socket);
            Self {
                socket_path: socket,
                request_id: AtomicU64::new(1),
                mode: RoutingMode::Direct,
            }
        } else {
            let socket = socket_discovery::discover_neural_api_socket();
            info!("🌐 Security provider: NEURAL API mode (capability.call) → {}", socket);
            Self {
                socket_path: socket,
                request_id: AtomicU64::new(1),
                mode: RoutingMode::NeuralApi,
            }
        }
    }

    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}
