// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Background lifecycle tasks: tarpc, connectivity checks, TTL and registry cleanup.

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use super::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Start the tarpc binary RPC server for high-performance primal-to-primal communication.
    pub(crate) async fn start_tarpc_server(&self) -> Result<()> {
        let enabled = songbird_process_env::var("SONGBIRD_TARPC_ENABLED")
            .map(|v| songbird_types::error_helpers::parse_bool_relaxed(&v).unwrap_or(true))
            .unwrap_or(true);

        if !enabled {
            info!("tarpc server disabled via SONGBIRD_TARPC_ENABLED=false");
            return Ok(());
        }

        let bind_host = &self._config.network.bind_host;
        let port = songbird_config::defaults::ports::tarpc_port();

        crate::app::http_server::start_tarpc_server(
            Arc::clone(&self.federation_state),
            Arc::clone(&self.federated_service_registry),
            bind_host,
            port,
        )
        .await?;

        info!("tarpc binary RPC listening on {bind_host}:{port}");

        #[cfg(unix)]
        {
            let socket_path = songbird_types::defaults::paths::tarpc_uds_socket_path();
            let registry = Arc::clone(&self.federated_service_registry);
            tokio::spawn(async move {
                if let Err(e) =
                    crate::rpc::tarpc_server::start_tarpc_uds_server(registry, socket_path.clone())
                        .await
                {
                    tracing::warn!("tarpc UDS server exited: {e}");
                }
            });
            info!(
                "tarpc UDS listening on {}",
                songbird_types::defaults::paths::tarpc_uds_socket_path().display()
            );
        }

        Ok(())
    }

    /// Verify external connectivity after startup.
    pub(crate) async fn verify_external_connectivity(&self) -> Result<()> {
        super::super::connectivity::verify_external_connectivity().await
    }

    /// Start session TTL cleanup task for federation stale node removal.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub(crate) async fn start_session_ttl_cleanup(&self) -> Result<()> {
        let federation_state = Arc::clone(&self.federation_state);

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(songbird_types::defaults::timeouts::DEFAULT_CACHE_TTL);
            let ttl_secs = 600;

            info!("🧹 Session TTL cleanup task started (interval: 5min, TTL: 10min)");

            loop {
                interval.tick().await;

                let removed = federation_state.cleanup_stale_nodes(ttl_secs).await;

                if removed > 0 {
                    info!("🧹 TTL cleanup: Removed {} stale sessions", removed);
                }
            }
        });

        info!("✅ Session TTL cleanup task spawned");
        Ok(())
    }

    /// Start service registry cleanup task (Universal Port Authority)
    pub(crate) fn start_service_registry_cleanup(&self) {
        let registry = Arc::clone(&self.service_registry);

        drop(crate::service_registry::spawn_cleanup_task((*registry).clone(), 60));

        info!("✅ Service registry cleanup task started");
    }
}
