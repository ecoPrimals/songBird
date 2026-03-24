// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration for Neural API capability registration.

use anyhow::{Context, Result};
use songbird_types::primal_names;
use std::env;

/// Configuration for capability registration (supports dependency injection)
#[derive(Debug, Clone)]
pub struct CapabilityRegistrationConfig {
    /// Neural API socket path
    pub neural_socket: String,
    /// Songbird's own socket path
    pub songbird_socket: String,
    /// Primal ID
    pub primal_id: String,
}

impl CapabilityRegistrationConfig {
    /// Build config from environment variables (production use)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_env() -> Result<Self> {
        let neural_socket = env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| {
            if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
                format!("{runtime_dir}/biomeos/neural-api.sock")
            } else {
                "/tmp/biomeos/neural-api.sock".to_string()
            }
        });

        let songbird_socket = env::var("SONGBIRD_SOCKET_PATH")
            .or_else(|_| env::var("SONGBIRD_SOCKET"))
            .or_else(|_| env::var("SONGBIRD_IPC_SOCKET"))
            .context(
                "SONGBIRD_SOCKET_PATH not set. Songbird must know its own socket path for registration.",
            )?;

        let primal_id = env::var("PRIMAL_ID")
            .or_else(|_| env::var("SONGBIRD_PRIMAL_ID"))
            .unwrap_or_else(|_| primal_names::SELF_NAME.to_string());

        Ok(Self {
            neural_socket,
            songbird_socket,
            primal_id,
        })
    }

    /// Build config with explicit values (test use)
    #[cfg(test)]
    #[must_use]
    pub fn for_testing(neural_socket: &str, songbird_socket: &str) -> Self {
        Self {
            neural_socket: neural_socket.to_string(),
            songbird_socket: songbird_socket.to_string(),
            primal_id: "songbird".to_string(),
        }
    }
}
