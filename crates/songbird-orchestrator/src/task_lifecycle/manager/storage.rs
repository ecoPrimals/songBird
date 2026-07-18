// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task storage backend selection (IPC storage provider or in-memory).
//!
//! Sled was removed in Wave 135 (SB-03). Storage is now exclusively IPC-based
//! (`NestGate` `storage.*` capability) with in-memory fallback. On startup we clean
//! any orphaned sled database artifacts left by pre-Wave-135 installations to
//! prevent corruption issues on unclean shutdown.

use super::super::TaskStorage;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

#[cfg(unix)]
fn log_storage_provider_unreachable(error: &impl std::fmt::Display, path: &Path) {
    tracing::warn!(
        error = %error,
        path = %path.display(),
        "storage provider unreachable; using in-memory task storage"
    );
}

/// Remove orphaned sled database artifacts from `data_dir`.
///
/// Pre-Wave-135 Songbird used sled with a `task_lifecycle.db` directory.
/// These files serve no purpose now and corrupt on unclean shutdown,
/// requiring manual cleanup. This function removes them automatically.
pub fn clean_legacy_sled_artifacts(data_dir: &Path) {
    let legacy_db_dir = data_dir.join("task_lifecycle.db");
    if legacy_db_dir.exists() {
        match std::fs::remove_dir_all(&legacy_db_dir) {
            Ok(()) => {
                info!(
                    path = %legacy_db_dir.display(),
                    "removed orphaned sled database directory (legacy pre-Wave-135)"
                );
            }
            Err(e) => {
                tracing::warn!(
                    path = %legacy_db_dir.display(),
                    error = %e,
                    "failed to remove orphaned sled database directory"
                );
            }
        }
    }
}

/// Resolve and connect the appropriate [`TaskStorage`] for this process.
///
/// Also cleans up any orphaned sled database artifacts from pre-Wave-135
/// installations that cause corruption on unclean shutdown.
pub async fn connect_task_storage(database_url: &str) -> Result<Arc<TaskStorage>> {
    if let Some(parent) = Path::new(database_url).parent() {
        clean_legacy_sled_artifacts(parent);
    }

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

    let _ = database_url;
    Ok(Arc::new(TaskStorage::Memory(crate::storage_memory::InMemoryStorage::new())))
}
