// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use std::path::PathBuf;

use super::runtime_or_tmp_base_with;

/// Default Unix socket path for a peer when `*_SOCKET_PATH` / `PEER_SOCKET_PATH` are unset.
#[must_use]
pub fn peer_fallback_socket_path(peer_id: &str) -> PathBuf {
    let base = super::runtime_or_tmp_base();
    PathBuf::from(format!("{base}/biomeos/{peer_id}.sock"))
}

/// Get data directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_DATA_DIR` (explicit override)
/// 2. `{XDG_RUNTIME_DIR|TMPDIR|/tmp}/songbird-data` (default)
pub fn data_dir() -> PathBuf {
    data_dir_with(|k| songbird_process_env::var(k))
}

/// [`data_dir`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn data_dir_with<F>(env: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env("SONGBIRD_DATA_DIR").map_or_else(
        |_| PathBuf::from(format!("{}/songbird-data", runtime_or_tmp_base_with(&env))),
        PathBuf::from,
    )
}

/// Get deployment directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_DEPLOY_DIR` (explicit override)
/// 2. `{XDG_RUNTIME_DIR|TMPDIR|/tmp}/songbird-deployments` (default)
pub fn deployment_dir() -> PathBuf {
    deployment_dir_with(|k| songbird_process_env::var(k))
}

/// [`deployment_dir`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn deployment_dir_with<F>(env: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env("SONGBIRD_DEPLOY_DIR").map_or_else(
        |_| PathBuf::from(format!("{}/songbird-deployments", runtime_or_tmp_base_with(&env))),
        PathBuf::from,
    )
}

/// Get cache directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_CACHE_DIR` (explicit override)
/// 2. `{XDG_RUNTIME_DIR|TMPDIR|/tmp}/songbird-cache` (default)
pub fn cache_dir() -> PathBuf {
    cache_dir_with(|k| songbird_process_env::var(k))
}

/// [`cache_dir`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn cache_dir_with<F>(env: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env("SONGBIRD_CACHE_DIR").map_or_else(
        |_| PathBuf::from(format!("{}/songbird-cache", runtime_or_tmp_base_with(&env))),
        PathBuf::from,
    )
}
