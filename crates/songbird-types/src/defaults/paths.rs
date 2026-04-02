// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Default filesystem paths
//!
//! Uses XDG base directory spec where applicable.
//! Actual paths should be discovered via environment or runtime config.
//! Legacy fallbacks under the OS temp directory use [`std::env::temp_dir()`] instead of a hardcoded `/tmp`.

use std::path::PathBuf;

/// Subdirectory under `XDG_RUNTIME_DIR` or `/run/user/<uid>/` for biomeOS sockets
pub const BIOMEOS_RUNTIME_SUBDIR: &str = "biomeos";

/// Crypto/security provider socket basenames (XDG scan order: crypto first)
///
/// Capability-only names — no primal identities. Primals advertise by capability.
pub const CRYPTO_PROVIDER_SOCKET_FILENAMES_XDG: &[&str] = &["crypto.sock", "security.sock"];

/// Crypto/security provider socket basenames (UID scan order: security first)
pub const CRYPTO_PROVIDER_SOCKET_FILENAMES_UID: &[&str] = &["security.sock", "crypto.sock"];

/// Neural / AI capability socket basenames (XDG and UID scan use the same order)
///
/// Capability-only names — no primal identities.
pub const NEURAL_API_CAPABILITY_SOCKET_FILENAMES: &[&str] = &["ai.sock", "neural-api.sock"];

/// `/run/user/{uid}/biomeos/{socket_filename}` for UID-based socket discovery
#[must_use]
pub fn run_user_biomeos_socket(uid: &str, socket_filename: &str) -> PathBuf {
    PathBuf::from(format!("/run/user/{uid}/{BIOMEOS_RUNTIME_SUBDIR}/{socket_filename}"))
}

/// Preferred default security provider socket path under the OS temp directory (capability-based naming).
#[must_use]
pub fn security_socket_default_path() -> PathBuf {
    biomeos_socket_dir_tmp().join("security.sock")
}

/// Last-resort `{temp}/beardog.sock` path for legacy discovery chains (same role as deprecated [`beardog_socket_legacy_path`]).
#[must_use]
pub fn security_socket_tmp_fallback_path() -> PathBuf {
    std::env::temp_dir().join("beardog.sock")
}

/// Default security provider socket paths (tried in order during discovery; preferred naming).
#[must_use]
pub fn security_socket_candidates() -> [PathBuf; 4] {
    [
        security_socket_default_path(),
        security_socket_tmp_fallback_path(),
        PathBuf::from("/run/user/1000/beardog-default.sock"),
        PathBuf::from("/var/run/beardog.sock"),
    ]
}

/// Default `BearDog` socket paths (legacy naming; use [`security_socket_candidates`])
#[deprecated(note = "Use security_socket_candidates() or capability-based discovery")]
#[must_use]
pub fn beardog_socket_candidates() -> [PathBuf; 4] {
    security_socket_candidates()
}

/// Default biomeOS socket directory under the OS temp directory.
#[must_use]
pub fn biomeos_socket_dir_tmp() -> PathBuf {
    std::env::temp_dir().join(BIOMEOS_RUNTIME_SUBDIR)
}

/// biomeOS socket fallback paths (capability names, tried in order).
#[must_use]
pub fn biomeos_socket_fallback_paths() -> [PathBuf; 4] {
    let b = biomeos_socket_dir_tmp();
    [
        b.join("security.sock"),
        b.join("crypto.sock"),
        b.join("beardog.sock"),
        security_socket_tmp_fallback_path(),
    ]
}

/// Default security socket path (final fallback under temp).
#[must_use]
pub fn biomeos_security_socket_default_path() -> PathBuf {
    security_socket_default_path()
}

/// Legacy `{temp}/beardog.sock` path (compatibility; prefer [`security_socket_default_path`])
#[deprecated(note = "Use security_socket_tmp_fallback_path() or security_socket_default_path()")]
#[must_use]
pub fn beardog_socket_legacy_path() -> PathBuf {
    security_socket_tmp_fallback_path()
}

/// Legacy Neural API socket path: `{temp}/neural-api-{family_id}.sock`.
#[must_use]
pub fn neural_api_socket_legacy_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("neural-api-{family_id}.sock"))
}

/// Default data directory
pub const DEFAULT_DATA_DIR: &str = "/var/lib/songbird";

/// IPC port file path under the OS temp directory (`songbird-ipc-port`).
#[must_use]
pub fn ipc_port_file_path() -> PathBuf {
    std::env::temp_dir().join("songbird-ipc-port")
}

/// `{temp}/{primal}-ipc-port` for TCP discovery file fallback.
#[must_use]
pub fn ipc_discovery_primal_port_path(primal_name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{primal_name}-ipc-port"))
}

/// Neural API legacy fallback paths (tried in order).
#[must_use]
pub fn neural_api_socket_fallback_paths() -> [PathBuf; 3] {
    let b = biomeos_socket_dir_tmp();
    [b.join("ai.sock"), b.join("neural-api.sock"), b.join("squirrel.sock")]
}

/// Default Neural API socket when discovery finds nothing (legacy, under temp `biomeos/`).
#[must_use]
pub fn neural_api_socket_default_path() -> PathBuf {
    biomeos_socket_dir_tmp().join("ai.sock")
}

/// `{temp}/crypto-{family_id}.sock` (family-scoped crypto socket fallback).
#[must_use]
pub fn family_scoped_crypto_socket_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("crypto-{family_id}.sock"))
}

/// `{temp}/security-{family_id}.sock` (family-scoped security socket fallback).
#[must_use]
pub fn family_scoped_security_socket_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("security-{family_id}.sock"))
}

/// `{temp}/beardog-{family_id}.sock` (family-scoped BearDog socket fallback).
#[must_use]
pub fn family_scoped_beardog_socket_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("beardog-{family_id}.sock"))
}

/// `{temp}/security.sock` (last-resort flat fallback used when XDG is unset).
#[must_use]
pub fn tmp_flat_security_sock_path() -> PathBuf {
    std::env::temp_dir().join("security.sock")
}
