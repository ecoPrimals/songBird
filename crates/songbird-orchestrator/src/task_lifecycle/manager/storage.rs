// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task storage backend selection (IPC storage provider or in-memory).

use super::super::TaskStorage;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

#[cfg(unix)]
fn log_storage_provider_unreachable(error: &impl std::fmt::Display, path: &std::path::Path) {
    tracing::warn!(
        error = %error,
        path = %path.display(),
        "storage provider unreachable; using in-memory task storage"
    );
}

fn open_memory(database_url: &str) -> Arc<TaskStorage> {
    let _ = database_url;
    Arc::new(TaskStorage::Memory(crate::storage_memory::InMemoryStorage::new()))
}

/// Resolve and connect the appropriate [`TaskStorage`] for this process.
pub async fn connect_task_storage(database_url: &str) -> Result<Arc<TaskStorage>> {
    #[cfg(unix)]
    {
        if let Ok(ep) = songbird_config::primal_discovery::get_storage_endpoint().await
            && let Some(path) = crate::storage_ipc::storage_socket_path_from_endpoint(&ep)
        {
            match songbird_universal_ipc::tower_atomic::TowerAtomicClient::connect_unix_path(&path)
                .await
            {
                Ok(_) => {
                    info!(
                        path = %path.display(),
                        "Task storage: IPC JSON-RPC (storage.* capability)"
                    );
                    return Ok(Arc::new(TaskStorage::Ipc(
                        crate::storage_ipc::IpcStorageBackend::new(path),
                    )));
                }
                Err(e) => {
                    log_storage_provider_unreachable(&e, &path);
                }
            }
        }
    }

    Ok(open_memory(database_url))
}
