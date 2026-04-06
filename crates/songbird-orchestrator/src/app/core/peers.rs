// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::Result;

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Get discovered peers from the discovery listener (v3.19.1)
    ///
    /// Used by Unix socket IPC handlers to implement `discover_by_family` API
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_discovered_peers(
        &self,
    ) -> Result<Vec<songbird_discovery::anonymous::DiscoveredPeer>> {
        if let Some(ref listener) = self.discovery_listener {
            Ok(listener.get_peers().await)
        } else {
            // No discovery listener = no peers
            Ok(vec![])
        }
    }

    /// Establish a connection to a peer (v3.19.1)
    ///
    /// Used by Unix socket IPC handlers to implement `create_genetic_tunnel` API
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn establish_connection(
        &mut self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        trust_level: songbird_types::TrustLevel,
        discovery_method: String,
    ) -> Result<()> {
        self.connection_manager
            .establish_connection(
                peer_id,
                endpoint,
                capabilities,
                peer_tags,
                trust_level,
                discovery_method,
            )
            .await
    }
}
