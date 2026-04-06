// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use async_trait::async_trait;
use tracing::{debug, info, warn};

use super::traits::CryptoCapabilityDiscovery;

/// Discovers crypto capability via environment variables
///
/// Priority:
/// 1. `{CAPABILITY}_ENDPOINT` derived from the capability string
/// 2. `SECURITY_PROVIDER_SOCKET` / `SECURITY_SOCKET` (capability-standard)
/// 3. `BEARDOG_SOCKET` (legacy; logs deprecation)
/// 4. Default: /primal/security
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

        if let Some(socket) = env_reader("SECURITY_PROVIDER_SOCKET") {
            info!("Found crypto provider at {} (via SECURITY_PROVIDER_SOCKET)", socket);
            return Ok(socket);
        }

        if let Some(socket) = env_reader("SECURITY_SOCKET") {
            info!("Found crypto provider at {} (via SECURITY_SOCKET)", socket);
            return Ok(socket);
        }

        if let Some(socket) = env_reader("BEARDOG_SOCKET") {
            warn!(
                "BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT (capability-first)"
            );
            return Ok(socket);
        }

        // Default to standard primal namespace
        let default = "/primal/security".to_string();
        info!("Using default crypto provider: {}", default);
        Ok(default)
    }
}
