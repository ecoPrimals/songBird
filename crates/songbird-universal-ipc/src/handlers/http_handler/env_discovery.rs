// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use async_trait::async_trait;
use tracing::{debug, info};

use super::traits::CryptoCapabilityDiscovery;

/// Discovers crypto capability via environment variables
///
/// Priority:
/// 1. `CRYPTO_ENDPOINT` env var
/// 2. `BEARDOG_SOCKET` env var
/// 3. Default: /primal/beardog
pub struct EnvCryptoDiscovery;

#[async_trait]
impl CryptoCapabilityDiscovery for EnvCryptoDiscovery {
    async fn discover(&self, capability: &str) -> IpcResult<String> {
        Self::discover_with(capability, |key| songbird_process_env::var(key).ok())
    }
}

impl EnvCryptoDiscovery {
    /// Discover crypto capability with injectable environment reader (concurrent-safe)
    pub fn discover_with<F>(capability: &str, env_reader: F) -> IpcResult<String>
    where
        F: Fn(&str) -> Option<String>,
    {
        debug!("Discovering capability via environment: {}", capability);

        // Try capability-based env var first
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase().replace('.', "_"));
        if let Some(endpoint) = env_reader(&env_key) {
            info!("Found {} at {} (via {})", capability, endpoint, env_key);
            return Ok(endpoint);
        }

        // Fall back to BEARDOG_SOCKET
        if let Some(socket) = env_reader("BEARDOG_SOCKET") {
            info!("Found crypto provider at {} (via BEARDOG_SOCKET)", socket);
            return Ok(socket);
        }

        // Default to standard primal namespace
        let default = "/primal/beardog".to_string();
        info!("Using default crypto provider: {}", default);
        Ok(default)
    }
}
