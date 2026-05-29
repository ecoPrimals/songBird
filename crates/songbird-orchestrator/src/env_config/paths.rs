// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use std::path::PathBuf;

use songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR;

/// Default Unix socket path for a peer when `*_SOCKET_PATH` / `PEER_SOCKET_PATH` are unset.
#[must_use]
pub fn peer_fallback_socket_path(peer_id: &str) -> PathBuf {
    let base = super::runtime_or_tmp_base();
    PathBuf::from(base).join(BIOMEOS_RUNTIME_SUBDIR).join(format!("{peer_id}.sock"))
}

/// Get data directory (self-knowledge)
///
/// Resolution order (DH-1 compliant — zero `/tmp` writes):
/// 1. `SONGBIRD_DATA_DIR` (explicit override)
/// 2. `$XDG_DATA_HOME/songbird` (XDG spec — typically `~/.local/share/songbird`)
/// 3. `$HOME/.local/share/songbird` (XDG default when `XDG_DATA_HOME` unset)
/// 4. `/var/lib/songbird` (VPS fallback — works under `ProtectSystem=strict`)
pub fn data_dir() -> PathBuf {
    data_dir_with(|k| songbird_process_env::var(k))
}

/// [`data_dir`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn data_dir_with<F>(env: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    if let Ok(explicit) = env("SONGBIRD_DATA_DIR") {
        return PathBuf::from(explicit);
    }
    if let Ok(xdg_data) = env("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data).join("songbird");
    }
    if let Ok(home) = env("HOME") {
        return PathBuf::from(home).join(".local/share/songbird");
    }
    PathBuf::from("/var/lib/songbird")
}

/// Get deployment directory (self-knowledge)
///
/// Resolution order (DH-1 compliant — zero `/tmp` writes):
/// 1. `SONGBIRD_DEPLOY_DIR` (explicit override)
/// 2. `$XDG_DATA_HOME/songbird/deployments`
/// 3. `$HOME/.local/share/songbird/deployments`
/// 4. `/var/lib/songbird/deployments` (VPS fallback)
pub fn deployment_dir() -> PathBuf {
    deployment_dir_with(|k| songbird_process_env::var(k))
}

/// [`deployment_dir`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn deployment_dir_with<F>(env: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    if let Ok(explicit) = env("SONGBIRD_DEPLOY_DIR") {
        return PathBuf::from(explicit);
    }
    data_dir_with(env).join("deployments")
}

/// Get cache directory (self-knowledge)
///
/// Resolution order (DH-1 compliant — zero `/tmp` writes):
/// 1. `SONGBIRD_CACHE_DIR` (explicit override)
/// 2. `$XDG_CACHE_HOME/songbird` (XDG spec — typically `~/.cache/songbird`)
/// 3. `$HOME/.cache/songbird` (XDG default when `XDG_CACHE_HOME` unset)
/// 4. `/var/cache/songbird` (VPS fallback — works under `ProtectSystem=strict`)
pub fn cache_dir() -> PathBuf {
    cache_dir_with(|k| songbird_process_env::var(k))
}

/// [`cache_dir`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn cache_dir_with<F>(env: F) -> PathBuf
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    if let Ok(explicit) = env("SONGBIRD_CACHE_DIR") {
        return PathBuf::from(explicit);
    }
    if let Ok(xdg_cache) = env("XDG_CACHE_HOME") {
        return PathBuf::from(xdg_cache).join("songbird");
    }
    if let Ok(home) = env("HOME") {
        return PathBuf::from(home).join(".cache/songbird");
    }
    PathBuf::from("/var/cache/songbird")
}
