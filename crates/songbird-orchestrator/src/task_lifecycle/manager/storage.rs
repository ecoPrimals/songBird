// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task storage backend selection (NestGate or in-memory).

use super::super::TaskStorageBackend;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

#[cfg(unix)]
fn log_nestgate_unreachable(error: &impl std::fmt::Display, path: &std::path::Path) {
    tracing::warn!(
        error = %error,
        path = %path.display(),
        "NestGate task storage unreachable; using in-memory task storage"
    );
}

fn open_memory(database_url: &str) -> Arc<dyn TaskStorageBackend> {
    let _ = database_url;
    Arc::new(crate::storage_memory::InMemoryStorage::new())
}

/// Resolve and connect the appropriate [`TaskStorageBackend`] for this process.
pub async fn connect_task_storage(database_url: &str) -> Result<Arc<dyn TaskStorageBackend>> {
    #[cfg(unix)]
    {
        if let Ok(ep) = songbird_config::primal_discovery::get_storage_endpoint().await
            && let Some(path) = crate::storage_nestgate::storage_socket_path_from_endpoint(&ep)
        {
            match songbird_universal_ipc::tower_atomic::TowerAtomicClient::connect_unix_path(&path)
                .await
            {
                Ok(_) => {
                    info!(
                        path = %path.display(),
                        "Task storage: NestGate JSON-RPC (storage.* capability)"
                    );
                    return Ok(Arc::new(crate::storage_nestgate::NestGateStorage::new(path)));
                }
                Err(e) => {
                    log_nestgate_unreachable(&e, &path);
                }
            }
        }
    }

    Ok(open_memory(database_url))
}
