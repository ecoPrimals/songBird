// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Environment Configuration - TRUE PRIMAL Self-Knowledge
//!
//! Centralizes ALL environment-based configuration for Songbird.
//! This module is Songbird's self-knowledge - how it knows itself at runtime.
//!
//! ## Principles
//!
//! 1. **Self-Knowledge**: Songbird knows ONLY itself (name, family, paths)
//! 2. **No Hardcoding**: All paths/IDs from environment or sensible defaults
//! 3. **Runtime Configuration**: No compile-time assumptions
//! 4. **Capability Discovery**: Other primals discovered via `primal_discovery` module
//!
//! ## Environment Variables
//!
//! ### Identity (Self-Knowledge)
//! - `PRIMAL_NAME`: This primal's name (default: "songbird")
//! - `FAMILY_ID` / `SONGBIRD_FAMILY_ID`: Family/biome ID (default: "default")
//! - `NODE_ID` / `SONGBIRD_NODE_ID`: Node ID (default: "default")
//!
//! ### Paths (Self-Knowledge)
//! - `SONGBIRD_SOCKET`: This primal's IPC socket path
//! - `SONGBIRD_DATA_DIR`: Data directory (default under `XDG_RUNTIME_DIR`/`TMPDIR`, else `/tmp`: `…/songbird-data`)
//! - `SONGBIRD_DEPLOY_DIR`: Deployment directory (default: `…/songbird-deployments`)
//! - `SONGBIRD_CACHE_DIR`: Cache directory (default: `…/songbird-cache`)
//!
//! ### Discovery (Other Primals)
//! - See `primal_discovery` module for discovering other primals

mod btsp;
mod dark_forest;
mod http;
mod identity;
mod paths;
mod security_ipc;
mod socket;

#[cfg(test)]
mod tests;

pub use btsp::{validate_btsp_insecure_guard, validate_btsp_insecure_guard_with};
pub use dark_forest::{accept_legacy_birdsong, dark_forest_enabled, dual_broadcast};
pub use http::{http_bind_address, http_port, is_production, log_level};
pub use identity::{family_id, family_id_with, node_id, primal_name};
pub(crate) use paths::peer_fallback_socket_path;
pub use paths::{
    cache_dir, cache_dir_with, data_dir, data_dir_with, deployment_dir, deployment_dir_with,
};
pub(crate) use security_ipc::security_crypto_ipc_socket_from_env;
pub use socket::{
    create_domain_socket_symlink, legacy_socket_name, legacy_socket_name_with,
    remove_domain_socket_symlink_if_matches, socket_name, socket_name_with, socket_path,
};

/// Convenience alias — reads from overlay first, then OS.
pub(super) fn env(key: &str) -> Result<String, std::env::VarError> {
    songbird_process_env::var(key)
}

/// Prefer `XDG_RUNTIME_DIR`, then `TMPDIR`, then `/tmp` (same resolution as peer socket fallbacks).
pub(super) fn runtime_or_tmp_base_with<F>(env_fn: &F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env_fn("XDG_RUNTIME_DIR").or_else(|_| env_fn("TMPDIR")).unwrap_or_else(|_| "/tmp".to_string())
}

/// Prefer `XDG_RUNTIME_DIR`, then `TMPDIR`, then `/tmp` (same resolution as peer socket fallbacks).
pub(super) fn runtime_or_tmp_base() -> String {
    runtime_or_tmp_base_with(&|k| songbird_process_env::var(k))
}
