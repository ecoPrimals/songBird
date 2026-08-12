// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security identity query and JWT secret provisioning.

use anyhow::Result;
use tracing::{debug, info, warn};

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Query security provider for node identity and encryption tags
    ///
    /// **EVOLVED (v3.15.0)**: Uses capability discovery (zero vendor hardcoding!)
    ///
    /// This is called on startup to get our encryption tag for USB seed integration.
    /// Discovers security provider via generic capability discovery.
    pub(crate) async fn query_security_identity(&self) -> Result<()> {
        use crate::security_capability_client::SecurityCapabilityClient;

        // EVOLVED: Use capability discovery (not hardcoded vendor name!)
        let security_url = crate::app::security_setup::discover_security_endpoint(None).await;

        if let Ok(url) = security_url {
            info!("🔐 Security provider configured: {}", url);

            // Query for identity
            let security_client = SecurityCapabilityClient::from_endpoint(url).await;

            match security_client?.get_identity().await {
                Ok(identity) => {
                    info!("✅ Got encryption tag: {}", identity.encryption_tag);
                    if let Some(family_id) = &identity.family_id {
                        info!("👨‍👩‍👧‍👦 Family ID: {}", family_id);
                    }
                    info!("🔑 Capabilities: {:?}", identity.capabilities);

                    // ✅ v3.14.0: Tags now broadcast in discovery via discover_identity_tags()
                    // For now, it's logged and can be accessed via SecurityCapabilityClient
                }
                Err(e) => {
                    warn!("⚠️  Could not query security identity: {}", e);
                    warn!("   Continuing without encryption tags");
                }
            }
        } else {
            debug!(
                "No security provider configured (capability-based discovery did not find security provider)"
            );
            debug!("Continuing without encryption tags");
        }

        Ok(())
    }

    /// Provision JWT secret from `security provider` via capability-based discovery
    ///
    /// ## TRUE PRIMAL Architecture
    ///
    /// - **Self-Knowledge**: Songbird only knows itself
    /// - **Capability Discovery**: Discovers `security provider` via "security" capability
    /// - **Graceful Fallback**: Uses secure random if `security provider` unavailable
    /// - **Pure Rust**: JSON-RPC over Unix socket (no C dependencies!)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - JWT secret (512 bits, base64-encoded)
    /// * `Err` - Only on critical failure (fallback always succeeds)
    pub(crate) async fn provision_jwt_secret(&self) -> Result<String> {
        use crate::auth::{get_security_socket_for_jwt, provision_jwt_secret};

        let security_socket = get_security_socket_for_jwt();

        let jwt_secret =
            provision_jwt_secret(security_socket.as_deref(), "songbird_authentication").await?;

        Ok(jwt_secret)
    }
}
