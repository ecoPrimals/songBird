// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Last-resort `{temp}/security.sock` flat fallback (capability-named).
#[must_use]
pub fn security_socket_tmp_fallback_path() -> PathBuf {
    std::env::temp_dir().join("security.sock")
}

/// Legacy on-disk filename `{temp}/beardog.sock` (older security deployments; filename retained on disk).
#[must_use]
pub fn security_socket_legacy_tmp_path() -> PathBuf {
    std::env::temp_dir().join("beardog.sock")
}

/// Default security provider socket paths (tried in order during discovery; capability-named first).
///
/// Order: capability paths (`security.sock`, `crypto.sock`), then legacy on-disk security socket file,
/// system path last before legacy temp.
#[must_use]
pub fn security_socket_candidates() -> [PathBuf; 5] {
    let b = biomeos_socket_dir_tmp();
    [
        security_socket_default_path(),
        security_socket_tmp_fallback_path(),
        PathBuf::from("/var/run/biomeos/security.sock"),
        b.join("crypto.sock"),
        security_socket_legacy_tmp_path(),
    ]
}

/// Alias for [`security_socket_candidates`] using security-provider naming.
#[must_use]
pub fn security_provider_socket_candidates() -> [PathBuf; 5] {
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
        security_socket_tmp_fallback_path(),
        security_socket_legacy_tmp_path(),
    ]
}

/// Default security socket path (final fallback under temp).
#[must_use]
pub fn biomeos_security_socket_default_path() -> PathBuf {
    security_socket_default_path()
}

/// Deprecated alias (legacy filename `beardog.sock`; prefer [`security_socket_default_path`])
#[deprecated(note = "Use security_socket_tmp_fallback_path() or security_socket_default_path()")]
#[must_use]
pub fn beardog_socket_legacy_path() -> PathBuf {
    security_socket_tmp_fallback_path()
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

// --- AI / neural provider (legacy on-disk names may still appear last) -------------------------

/// AI / neural provider socket fallbacks (capability names first; legacy AI socket filename last).
#[must_use]
pub fn coordination_socket_candidates() -> [PathBuf; 3] {
    let b = biomeos_socket_dir_tmp();
    [b.join("ai.sock"), b.join("neural-api.sock"), b.join("squirrel.sock")]
}

/// Deprecated alias for [`coordination_socket_candidates`].
#[deprecated(note = "use coordination_socket_candidates (capability-based naming)")]
#[must_use]
pub fn neural_api_socket_fallback_paths() -> [PathBuf; 3] {
    coordination_socket_candidates()
}

/// AI capability provider socket fallbacks (alias for [`coordination_socket_candidates`]).
#[must_use]
pub fn ai_socket_candidates() -> [PathBuf; 3] {
    coordination_socket_candidates()
}

/// Default Neural API socket when discovery finds nothing (under temp `biomeos/`).
#[must_use]
pub fn ai_provider_socket_default_path() -> PathBuf {
    biomeos_socket_dir_tmp().join("ai.sock")
}

/// Legacy Neural API socket path: `{temp}/neural-api-{family_id}.sock`.
#[must_use]
pub fn ai_provider_socket_legacy_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("neural-api-{family_id}.sock"))
}

// --- Compute capability provider (legacy on-disk compute filename may appear last) ------------

/// Compute provider socket fallbacks (capability names first; legacy compute socket filename last).
#[must_use]
pub fn compute_socket_candidates() -> [PathBuf; 4] {
    let b = biomeos_socket_dir_tmp();
    [
        b.join("compute.sock"),
        b.join("bridge.sock"),
        std::env::temp_dir().join("compute.sock"),
        b.join("toadstool.sock"),
    ]
}

/// `{temp}/crypto-{family_id}.sock` (family-scoped crypto socket fallback).
#[must_use]
pub fn family_scoped_crypto_socket_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("crypto-{family_id}.sock"))
}

/// `{temp}/security-{family_id}.sock` (family-scoped security socket fallback — capability-named).
#[must_use]
pub fn family_scoped_security_socket_path(family_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("security-{family_id}.sock"))
}

/// Family-scoped security provider socket path (capability-based; same as [`family_scoped_security_socket_path`]).
#[must_use]
pub fn family_scoped_security_provider_socket_path(family_id: &str) -> PathBuf {
    family_scoped_security_socket_path(family_id)
}

/// `{temp}/security.sock` (last-resort flat fallback used when XDG is unset).
#[must_use]
pub fn tmp_flat_security_sock_path() -> PathBuf {
    std::env::temp_dir().join("security.sock")
}
