// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default filesystem paths
//!
//! Uses XDG base directory spec where applicable.
//! Actual paths should be discovered via environment or runtime config.
//! Legacy fallbacks under the OS temp directory use [`std::env::temp_dir()`] instead of a hardcoded `/tmp`.

use crate::constants::BIOMEOS_SYSTEM_RUNTIME_DIR;
use std::path::PathBuf;

/// Subdirectory under `XDG_RUNTIME_DIR` or `/run/user/<uid>/` for biomeOS sockets
pub const BIOMEOS_RUNTIME_SUBDIR: &str = "biomeos";

/// Legacy security provider socket filename (retained for migration scan).
///
/// Code scanning for this name should use this constant rather than the
/// raw string `"beardog.sock"` so we can track and eventually remove it.
/// Capability-based discovery uses `security.sock` and `crypto.sock`.
#[deprecated(since = "0.2.1", note = "use capability-based 'security.sock' or 'crypto.sock'")]
pub const LEGACY_SECURITY_SOCKET_FILENAME: &str = "beardog.sock";

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

/// Legacy AI/coordination provider socket filename (Squirrel).
#[deprecated(since = "0.2.1", note = "use capability-based 'ai.sock' or 'neural-api.sock'")]
pub const LEGACY_AI_SOCKET_FILENAME: &str = "squirrel.sock";

/// Legacy compute provider socket filename (toadStool).
#[deprecated(since = "0.2.1", note = "use capability-based 'compute.sock' or 'bridge.sock'")]
pub const LEGACY_COMPUTE_SOCKET_FILENAME: &str = "toadstool.sock";

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
#[allow(deprecated, reason = "intentional backward-compat fallback path")]
pub fn security_socket_legacy_tmp_path() -> PathBuf {
    std::env::temp_dir().join(LEGACY_SECURITY_SOCKET_FILENAME)
}

/// Last-resort legacy flat path `{temp}/security-provider.sock`.
///
/// Returned by `discover_security_socket` when all XDG and env-based lookups fail.
#[must_use]
pub fn security_provider_legacy_flat_path() -> PathBuf {
    std::env::temp_dir().join("security-provider.sock")
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
        PathBuf::from(BIOMEOS_SYSTEM_RUNTIME_DIR).join("security.sock"),
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

/// Default data directory (FHS fallback when env is unset).
pub const DEFAULT_DATA_DIR: &str = "/var/lib/songbird";

/// Resolve data directory from environment, then XDG, then FHS fallback.
///
/// Priority: `SONGBIRD_DATA_DIR` > `SONGBIRD_STATE_DIR/data` > `$XDG_DATA_HOME/songbird` > `$HOME/.local/share/songbird` > `/var/lib/songbird`
#[must_use]
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = songbird_process_env::var("SONGBIRD_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(state_dir) = songbird_process_env::var("SONGBIRD_STATE_DIR") {
        return PathBuf::from(state_dir).join("data");
    }
    if let Ok(xdg) = songbird_process_env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("songbird");
    }
    if let Ok(home) = songbird_process_env::var("HOME") {
        return PathBuf::from(home).join(".local/share/songbird");
    }
    PathBuf::from(DEFAULT_DATA_DIR)
}

/// Resolve config directory from environment, then XDG, then home fallback.
///
/// Priority: `SONGBIRD_CONFIG_DIR` > `$XDG_CONFIG_HOME/songbird` > `$HOME/.config/songbird`
#[must_use]
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = songbird_process_env::var("SONGBIRD_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = songbird_process_env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("songbird");
    }
    if let Ok(home) = songbird_process_env::var("HOME") {
        return PathBuf::from(home).join(".config/songbird");
    }
    PathBuf::from("/etc/songbird")
}

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
    #[allow(deprecated, reason = "intentional backward-compat fallback path")]
    let legacy = LEGACY_AI_SOCKET_FILENAME;
    [b.join("ai.sock"), b.join("neural-api.sock"), b.join(legacy)]
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
    [b.join("compute.sock"), b.join("bridge.sock"), std::env::temp_dir().join("compute.sock"), {
        #[allow(deprecated, reason = "intentional backward-compat fallback path")]
        let legacy = LEGACY_COMPUTE_SOCKET_FILENAME;
        b.join(legacy)
    }]
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

// --- Network / orchestration capability provider (Songbird self-discovery) --------------------

/// Network capability socket filenames (domain-named, for filesystem discovery).
///
/// Consumers prefer `ipc.resolve({ "capability": "network" })` when a broker
/// connection exists. Filesystem scanning is the bootstrap fallback.
pub const NETWORK_CAPABILITY_SOCKET_FILENAMES: &[&str] = &["network.sock"];

/// Network/orchestration socket candidates (capability domain + primal name).
///
/// Tries: capability domain socket first (`network.sock`), then primal-named
/// socket (`songbird.sock`). Both resolve to the same listener (the domain
/// name is a symlink to the primal name).
#[must_use]
pub fn network_socket_candidates() -> [PathBuf; 2] {
    let b = biomeos_socket_dir_tmp();
    [b.join("network.sock"), b.join(format!("{}.sock", crate::primal_names::SELF_NAME))]
}

/// Primary IPC socket path for the songBird orchestrator.
///
/// Returns the first candidate from [`network_socket_candidates`] that exists,
/// or the domain-named socket path as fallback (for connection attempts that
/// will produce a clear "not found" error).
#[must_use]
pub fn primary_ipc_socket_path() -> PathBuf {
    let candidates = network_socket_candidates();
    candidates.iter().find(|p| p.exists()).cloned().unwrap_or_else(|| candidates[0].clone())
}

/// tarpc UDS socket path (G64 cephalization dual-socket pattern).
///
/// Lives alongside the JSON-RPC socket: `{socket_dir}/songbird.tarpc.sock`.
/// JSON-RPC on `.sock` handles discovery/diagnostics; tarpc on `.tarpc.sock`
/// carries high-frequency binary RPC for intra-gate primal-to-primal calls.
#[must_use]
pub fn tarpc_uds_socket_path() -> PathBuf {
    let dir = biomeos_socket_dir_tmp();
    dir.join(format!("{}.tarpc.sock", crate::primal_names::SELF_NAME))
}

// --- Mesh gossip capability provider (consumed at runtime; not a primal name) ---------------

/// Mesh gossip capability socket filenames (domain-named, for filesystem discovery).
///
/// Consumers prefer `ipc.resolve({ "capability": "mesh_gossip" })` when a broker
/// connection exists. Filesystem scanning is the bootstrap fallback.
pub const MESH_GOSSIP_CAPABILITY_SOCKET_FILENAMES: &[&str] =
    &["mesh-gossip.sock", "mesh_gossip.sock"];

/// Mesh gossip provider socket candidates (capability domain names only).
#[must_use]
pub fn mesh_gossip_socket_candidates() -> [PathBuf; 2] {
    let b = biomeos_socket_dir_tmp();
    [
        b.join(MESH_GOSSIP_CAPABILITY_SOCKET_FILENAMES[0]),
        b.join(MESH_GOSSIP_CAPABILITY_SOCKET_FILENAMES[1]),
    ]
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[allow(deprecated, reason = "tests intentionally verify deprecated constants for backward-compat")]
mod tests {
    use super::*;

    #[test]
    fn legacy_security_socket_filename_is_beardog() {
        assert_eq!(LEGACY_SECURITY_SOCKET_FILENAME, "beardog.sock");
    }

    #[test]
    fn security_socket_legacy_tmp_path_uses_constant() {
        let path = security_socket_legacy_tmp_path();
        assert!(path.to_string_lossy().ends_with(LEGACY_SECURITY_SOCKET_FILENAME));
    }

    #[test]
    fn data_dir_explicit_env_wins() {
        songbird_process_env::set_var("SONGBIRD_DATA_DIR", "/custom/data");
        let dir = data_dir();
        songbird_process_env::remove_var("SONGBIRD_DATA_DIR");
        assert_eq!(dir, PathBuf::from("/custom/data"));
    }

    #[test]
    fn data_dir_xdg_fallback() {
        songbird_process_env::remove_var("SONGBIRD_DATA_DIR");
        songbird_process_env::set_var("XDG_DATA_HOME", "/xdg/share");
        let dir = data_dir();
        songbird_process_env::remove_var("XDG_DATA_HOME");
        assert_eq!(dir, PathBuf::from("/xdg/share/songbird"));
    }

    #[test]
    fn ipc_port_file_path_under_temp_dir() {
        let path = ipc_port_file_path();
        assert!(path.to_string_lossy().contains("songbird-ipc-port"));
    }

    #[test]
    fn security_socket_candidates_capability_first() {
        let candidates = security_socket_candidates();
        let first = candidates[0].to_string_lossy();
        assert!(
            first.contains("security.sock"),
            "first candidate should be capability-named, got: {first}"
        );
        let last = candidates[candidates.len() - 1].to_string_lossy();
        assert!(
            last.contains(LEGACY_SECURITY_SOCKET_FILENAME),
            "last candidate should be legacy, got: {last}"
        );
    }

    #[test]
    fn network_socket_candidates_domain_then_primal() {
        let candidates = network_socket_candidates();
        let first = candidates[0].to_string_lossy();
        assert!(
            first.ends_with("network.sock"),
            "first candidate should be domain-named, got: {first}"
        );
        let second = candidates[1].to_string_lossy();
        assert!(
            second.ends_with("songbird.sock"),
            "second candidate should be primal-named, got: {second}"
        );
    }

    #[test]
    fn capability_domain_constant_is_network() {
        assert_eq!(crate::primal_names::CAPABILITY_DOMAIN, "network");
    }
}
