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

use std::path::PathBuf;

use songbird_types::defaults::{hosts::DEFAULT_BIND_ALL, ports::DEFAULT_HTTP_PORT};
use songbird_types::primal_names;

/// BTSP guard: refuse to start when both `FAMILY_ID` (non-default) and
/// `BIOMEOS_INSECURE=1` are set.
///
/// Per `BTSP_PROTOCOL_STANDARD.md` v1.0 and `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1:
/// you cannot claim a family AND skip authentication. This is a hard error.
///
/// # Errors
///
/// Returns an error message if the conflicting configuration is detected.
pub fn validate_btsp_insecure_guard() -> Result<(), String> {
    validate_btsp_insecure_guard_with(|k| songbird_process_env::var(k))
}

/// Injectable variant for concurrent-safe testing.
pub fn validate_btsp_insecure_guard_with<F>(env_reader: F) -> Result<(), String>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let fid = env_reader("FAMILY_ID")
        .or_else(|_| env_reader("SONGBIRD_FAMILY_ID"))
        .or_else(|_| env_reader("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let insecure = env_reader("BIOMEOS_INSECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if fid != "default" && !fid.is_empty() && insecure {
        return Err(format!(
            "FATAL: FAMILY_ID={fid:?} and BIOMEOS_INSECURE=1 are both set. \
             Per BTSP_PROTOCOL_STANDARD.md v1.0: you cannot claim a family AND skip authentication. \
             Either remove BIOMEOS_INSECURE or unset FAMILY_ID."
        ));
    }
    Ok(())
}

/// Convenience alias — reads from overlay first, then OS.
fn env(key: &str) -> Result<String, std::env::VarError> {
    songbird_process_env::var(key)
}

/// Prefer `XDG_RUNTIME_DIR`, then `TMPDIR`, then `/tmp` (same resolution as peer socket fallbacks).
fn runtime_or_tmp_base_with<F>(env: &F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env("XDG_RUNTIME_DIR").or_else(|_| env("TMPDIR")).unwrap_or_else(|_| "/tmp".to_string())
}

/// Prefer `XDG_RUNTIME_DIR`, then `TMPDIR`, then `/tmp` (same resolution as peer socket fallbacks).
fn runtime_or_tmp_base() -> String {
    runtime_or_tmp_base_with(&|k| songbird_process_env::var(k))
}

/// Default Unix socket path for a peer when `*_SOCKET_PATH` / `PEER_SOCKET_PATH` are unset.
#[must_use]
pub(crate) fn peer_fallback_socket_path(peer_id: &str) -> PathBuf {
    let base = runtime_or_tmp_base();
    PathBuf::from(format!("{base}/biomeos/{peer_id}.sock"))
}

/// Resolve security/crypto provider Unix socket from environment (capability-first).
///
/// Order: `SECURITY_PROVIDER_SOCKET`, `CRYPTO_PROVIDER_SOCKET`, `SECURITY_SOCKET`, then
/// deprecated `BEARDOG_SOCKET` (emits [`tracing::warn!`]).
///
/// Prefer `CAPABILITY_SECURITY_ENDPOINT` (capability discovery) or `SECURITY_PROVIDER_*` /
/// `SECURITY_*` variables over legacy primal-named env keys.
#[must_use]
pub(crate) fn security_crypto_ipc_socket_from_env(default_fn: impl FnOnce() -> String) -> String {
    if let Ok(p) = env("SECURITY_PROVIDER_SOCKET") {
        return p;
    }
    if let Ok(p) = env("CRYPTO_PROVIDER_SOCKET") {
        return p;
    }
    if let Ok(p) = env("SECURITY_SOCKET") {
        return p;
    }
    if let Ok(p) = env("BEARDOG_SOCKET") {
        tracing::warn!(
            "DEPRECATED: BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT or SECURITY_PROVIDER_* for capability-first configuration"
        );
        return p;
    }
    default_fn()
}

/// Get this primal's name (self-knowledge)
#[must_use]
pub fn primal_name() -> String {
    env("PRIMAL_NAME").unwrap_or_else(|_| primal_names::SELF_NAME.to_string())
}

/// Get family/biome ID (self-knowledge)
///
/// Priority order (`BiomeOS` Neural API compatible):
/// 1. `SONGBIRD_ORCHESTRATOR_FAMILY_ID` (highest - Neural API standard)
/// 2. `SONGBIRD_ORCHESTRATOR_FAMILY` (alternative)
/// 3. `BIOMEOS_FAMILY_ID` (generic orchestrator)
/// 4. `SONGBIRD_FAMILY_ID` (legacy)
/// 5. `FAMILY_ID` (generic)
/// 6. Default: `"default"` (seed-derived family ID should be set via env)
#[must_use]
pub fn family_id() -> String {
    family_id_with(|k| songbird_process_env::var(k))
}

/// [`family_id`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn family_id_with<F>(env: F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    env("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
        .or_else(|_| env("SONGBIRD_ORCHESTRATOR_FAMILY"))
        .or_else(|_| env("BIOMEOS_FAMILY_ID"))
        .or_else(|_| env("SONGBIRD_FAMILY_ID"))
        .or_else(|_| env("FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Get node ID (self-knowledge)
#[must_use]
pub fn node_id() -> String {
    env("SONGBIRD_NODE_ID").or_else(|_| env("NODE_ID")).unwrap_or_else(|_| "default".to_string())
}

/// Get this primal's IPC socket path (self-knowledge)
///
/// Resolution order (`BiomeOS` XDG Standard):
/// 1. `SONGBIRD_SOCKET` (explicit override - full path)
/// 2. `BIOMEOS_SOCKET_DIR` + socket name (shared socket directory)
/// 3. `/run/user/$UID/biomeos/` + socket name (XDG-compliant default)
/// 4. `{TMPDIR|/tmp}` + socket name (legacy fallback if XDG unavailable)
///
/// **Socket Naming Standard**:
/// - Default: `songbird.sock` (single-family mode, biomeOS compliant)
/// - Multi-family: `songbird-{family_id}.sock` when `SONGBIRD_MULTI_FAMILY=true`
///   or `SONGBIRD_FAMILY_SOCKET=true`
///
/// This enables multiple Songbird instances serving different families
/// on the same machine, each with its own isolated socket.
#[must_use]
pub fn socket_path() -> PathBuf {
    // Priority 1: Explicit SONGBIRD_SOCKET override
    if let Ok(path) = env("SONGBIRD_SOCKET") {
        return PathBuf::from(path);
    }

    let sock_name = socket_name();

    // Priority 2: BIOMEOS_SOCKET_DIR + socket name
    if let Ok(socket_dir) = env("BIOMEOS_SOCKET_DIR") {
        let path = PathBuf::from(socket_dir).join(&sock_name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return path;
    }

    // Priority 3: XDG-compliant default (/run/user/$UID/biomeos/)
    let xdg_socket = env("XDG_RUNTIME_DIR").map_or_else(
        |_| {
            env("UID").map_or_else(
                |_| PathBuf::from(format!("{}/{}", runtime_or_tmp_base(), sock_name)),
                |uid_str| PathBuf::from(format!("/run/user/{uid_str}/biomeos/{sock_name}")),
            )
        },
        |xdg_runtime_dir| PathBuf::from(xdg_runtime_dir).join("biomeos").join(&sock_name),
    );

    // Ensure directory exists (Pure Rust!)
    if let Some(parent) = xdg_socket.parent()
        && std::fs::create_dir_all(parent).is_ok()
    {
        return xdg_socket;
    }

    // Priority 4: Legacy fallback (if XDG unavailable or directory creation failed)
    PathBuf::from(format!("{}/{}", runtime_or_tmp_base(), sock_name))
}

/// Capability domain stem for socket naming per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1.
const DOMAIN_SOCKET_STEM: &str = "network";

/// Get the socket filename based on family configuration.
///
/// Returns domain-based names per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1:
/// - `network.sock` in development mode (no `FAMILY_ID`)
/// - `network-{family_id}.sock` in production mode (`FAMILY_ID` set, non-default)
///
/// Use [`legacy_socket_name`] for backward-compatible `songbird*.sock` names
/// (for symlink creation during Phase 1 migration).
#[must_use]
pub fn socket_name() -> String {
    socket_name_with(|k| songbird_process_env::var(k))
}

/// [`socket_name`] with an injectable env reader (for unit tests and alternate backends).
#[must_use]
pub fn socket_name_with<F>(env: F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let fid = family_id_with(&env);
    if fid != "default" && !fid.is_empty() {
        format!("{DOMAIN_SOCKET_STEM}-{fid}.sock")
    } else {
        format!("{DOMAIN_SOCKET_STEM}.sock")
    }
}

/// Legacy primal-named socket filename for backward-compatible symlinks.
///
/// Returns `songbird.sock` or `songbird-{family_id}.sock` — the old naming
/// convention. At startup, a symlink from this name to the domain-based
/// [`socket_name`] should be created to avoid breaking existing consumers.
#[must_use]
pub fn legacy_socket_name() -> String {
    legacy_socket_name_with(|k| songbird_process_env::var(k))
}

/// [`legacy_socket_name`] with an injectable env reader.
#[must_use]
pub fn legacy_socket_name_with<F>(env: F) -> String
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    let fid = family_id_with(&env);
    if fid != "default" && !fid.is_empty() {
        format!("songbird-{fid}.sock")
    } else {
        "songbird.sock".to_string()
    }
}

/// Create a legacy backward-compatibility symlink from `songbird*.sock` to `network*.sock`.
///
/// Per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` v1.1 §3 (Legacy compatibility):
/// > a primal MAY also bind or symlink the legacy primal-named socket
///
/// This is best-effort — symlink failure is logged but does not prevent startup.
pub fn create_legacy_socket_symlink(domain_socket: &std::path::Path) {
    let Some(parent) = domain_socket.parent() else {
        return;
    };
    let legacy_name = legacy_socket_name();
    let legacy_path = parent.join(&legacy_name);
    let _ = std::fs::remove_file(&legacy_path);
    if let Err(e) = std::os::unix::fs::symlink(domain_socket, &legacy_path) {
        tracing::warn!(
            legacy = %legacy_path.display(),
            domain = %domain_socket.display(),
            "Could not create legacy socket symlink: {e}"
        );
    } else {
        tracing::info!(
            legacy = %legacy_path.display(),
            domain = %domain_socket.display(),
            "Created legacy socket symlink for backward compatibility"
        );
    }
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

/// Get HTTP server bind address (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_ADDR` (explicit override)
/// 2. `DEFAULT_BIND_ALL`:`DEFAULT_HTTP_PORT` from `songbird_types::defaults` (bind all interfaces)
#[must_use]
pub fn http_bind_address() -> String {
    env("SONGBIRD_HTTP_ADDR").unwrap_or_else(|_| format!("{DEFAULT_BIND_ALL}:{DEFAULT_HTTP_PORT}"))
}

/// Get HTTP server port (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_PORT` (explicit override)
/// 2. Extract from `SONGBIRD_HTTP_ADDR` if set
/// 3. `DEFAULT_HTTP_PORT` from `songbird_types::defaults` (default)
#[must_use]
pub fn http_port() -> u16 {
    if let Ok(port_str) = env("SONGBIRD_HTTP_PORT") {
        return port_str.parse().unwrap_or(DEFAULT_HTTP_PORT);
    }

    if let Ok(addr) = env("SONGBIRD_HTTP_ADDR")
        && let Some(port_part) = addr.split(':').nth(1)
        && let Ok(port) = port_part.parse()
    {
        return port;
    }

    DEFAULT_HTTP_PORT
}

/// Check if running in production mode
///
/// Resolution order:
/// 1. `SONGBIRD_ENV == "production"`
/// 2. `RUST_ENV == "production"`
/// 3. `false` (default to development)
#[must_use]
pub fn is_production() -> bool {
    env("SONGBIRD_ENV").or_else(|_| env("RUST_ENV")).map(|v| v == "production").unwrap_or(false)
}

/// Get log level (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_LOG` (explicit override)
/// 2. `RUST_LOG` (Rust standard)
/// 3. `"info"` (default)
#[must_use]
pub fn log_level() -> String {
    env("SONGBIRD_LOG").or_else(|_| env("RUST_LOG")).unwrap_or_else(|_| "info".to_string())
}

// ═══════════════════════════════════════════════════════════════════════════
// Dark Forest Beacon Genetics Configuration (NEW - Feb 3, 2026)
// ═══════════════════════════════════════════════════════════════════════════

/// Check if Dark Forest beacons are enabled
///
/// Reads `SONGBIRD_DARK_FOREST` environment variable.
///
/// When `true`, Songbird broadcasts Dark Forest beacons (version 2, fully encrypted).
/// When `false`, Songbird broadcasts legacy `BirdSongPacket` (version 1.0, plaintext `family_id`).
///
/// **Default**: `false` (opt-in for privacy, requires `security provider` beacon.* RPC)
#[must_use]
pub fn dark_forest_enabled() -> bool {
    env("SONGBIRD_DARK_FOREST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}

/// Check if legacy `BirdSongPacket` format should be accepted
///
/// Reads `SONGBIRD_ACCEPT_LEGACY_BIRDSONG` environment variable.
///
/// When `true`, accepts both Dark Forest beacons AND legacy `BirdSongPacket`.
/// When `false`, only accepts Dark Forest beacons (rejects legacy).
///
/// **Default**: `true` (backward compatible during migration)
#[must_use]
pub fn accept_legacy_birdsong() -> bool {
    env("SONGBIRD_ACCEPT_LEGACY_BIRDSONG").ok().and_then(|v| v.parse().ok()).unwrap_or(true)
}

/// Check if dual broadcast is enabled (both formats)
///
/// Reads `SONGBIRD_DUAL_BROADCAST` environment variable.
///
/// When `true`, broadcasts BOTH Dark Forest beacons AND legacy `BirdSongPacket`.
/// When `false`, only broadcasts Dark Forest beacons (if enabled).
///
/// **Default**: `false` (minimize network overhead)
#[must_use]
pub fn dual_broadcast() -> bool {
    env("SONGBIRD_DUAL_BROADCAST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::collections::HashMap;
    use std::env::VarError;
    use std::sync::Mutex;

    use songbird_process_env;
    use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

    use super::*;

    /// Injectable env map for [`super::*_with`] tests (no shared process env).
    fn env_map(
        pairs: Vec<(&'static str, &'static str)>,
    ) -> impl Fn(&str) -> Result<String, VarError> {
        let map: HashMap<String, String> =
            pairs.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| map.get(key).cloned().ok_or(VarError::NotPresent)
    }

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    // Note: These tests validate default behavior when env vars are NOT set.
    // We avoid set_var/remove_var where possible to prevent concurrent test pollution.
    // Functions like primal_name() and family_id() have stable defaults that are
    // testable without env manipulation.

    #[test]
    fn test_primal_name_returns_string() {
        // primal_name() always returns a value (either env or default)
        let name = primal_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_family_id_returns_string() {
        // family_id() always returns a value (either env or default "default")
        let fid = family_id();
        assert!(!fid.is_empty());
    }

    #[test]
    fn test_socket_path_returns_valid_path() {
        let path = socket_path();
        let path_str = path.to_string_lossy();
        // Should end with .sock
        assert!(path_str.ends_with(".sock"), "Expected .sock extension, got: {path_str}");
    }

    #[test]
    fn test_socket_name_domain_based() {
        let name = socket_name();
        assert!(name.ends_with(".sock"));
        assert!(name.starts_with("network"), "Expected domain-based name, got: {name}");
    }

    #[test]
    fn test_data_dir_returns_valid_path() {
        let dir = data_dir();
        assert!(!dir.to_string_lossy().is_empty());
    }

    #[test]
    fn test_http_port_returns_valid_port() {
        let port = http_port();
        assert!(port > 0);
    }

    #[test]
    fn test_log_level_returns_string() {
        let level = log_level();
        assert!(!level.is_empty());
    }

    #[test]
    fn test_dark_forest_config() {
        // These functions always return a bool
        let _dark = dark_forest_enabled();
        let _legacy = accept_legacy_birdsong();
        let _dual = dual_broadcast();
    }

    #[test]
    fn http_port_reads_songbird_http_port() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::set_var("SONGBIRD_HTTP_PORT", "9443");
        songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
        assert_eq!(http_port(), 9443);
        songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    }

    #[test]
    fn http_port_invalid_falls_back_to_default() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::set_var("SONGBIRD_HTTP_PORT", "not-a-number");
        songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
        assert_eq!(http_port(), DEFAULT_HTTP_PORT);
        songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
    }

    #[test]
    fn http_port_parsed_from_bind_addr_when_port_env_unset() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::remove_var("SONGBIRD_HTTP_PORT");
        songbird_process_env::set_var("SONGBIRD_HTTP_ADDR", "0.0.0.0:18080");
        assert_eq!(http_port(), 18080);
        songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    }

    #[test]
    fn http_bind_address_respects_override() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::set_var("SONGBIRD_HTTP_ADDR", "10.0.0.2:9000");
        assert_eq!(http_bind_address(), "10.0.0.2:9000");
        songbird_process_env::remove_var("SONGBIRD_HTTP_ADDR");
    }

    #[test]
    fn is_production_true_when_songbird_env_set() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::set_var("SONGBIRD_ENV", "production");
        songbird_process_env::remove_var("RUST_ENV");
        assert!(is_production());
        songbird_process_env::remove_var("SONGBIRD_ENV");
    }

    #[test]
    fn is_production_checks_rust_env_when_songbird_unset() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::remove_var("SONGBIRD_ENV");
        songbird_process_env::set_var("RUST_ENV", "production");
        assert!(is_production());
        songbird_process_env::remove_var("RUST_ENV");
    }

    #[test]
    fn primal_name_env_override() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::set_var("PRIMAL_NAME", "custom-primal");
        assert_eq!(primal_name(), "custom-primal");
        songbird_process_env::remove_var("PRIMAL_NAME");
    }

    #[test]
    fn family_id_prefers_songbird_orchestrator_family_id() {
        let _g = ENV_TEST_LOCK.lock().unwrap();
        songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY");
        songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");
        songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
        songbird_process_env::remove_var("FAMILY_ID");
        songbird_process_env::set_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "orch-family");
        assert_eq!(family_id(), "orch-family");
        songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    }

    #[test]
    fn socket_name_with_no_family_returns_domain_sock() {
        assert_eq!(socket_name_with(env_map(vec![])), "network.sock");
    }

    #[test]
    fn socket_name_with_family_id_returns_domain_scoped() {
        let n = socket_name_with(env_map(vec![("FAMILY_ID", "fam-a")]));
        assert_eq!(n, "network-fam-a.sock");
    }

    #[test]
    fn socket_name_with_family_respects_priority_chain() {
        let n = socket_name_with(env_map(vec![
            ("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "orch-fam"),
            ("FAMILY_ID", "ignored"),
        ]));
        assert_eq!(n, "network-orch-fam.sock");
    }

    #[test]
    fn legacy_socket_name_with_no_family() {
        assert_eq!(legacy_socket_name_with(env_map(vec![])), "songbird.sock");
    }

    #[test]
    fn legacy_socket_name_with_family_id() {
        let n = legacy_socket_name_with(env_map(vec![("FAMILY_ID", "edge")]));
        assert_eq!(n, "songbird-edge.sock");
    }

    #[test]
    fn btsp_insecure_guard_ok_when_no_conflict() {
        assert!(validate_btsp_insecure_guard_with(env_map(vec![])).is_ok());
        assert!(validate_btsp_insecure_guard_with(env_map(vec![("FAMILY_ID", "fam")])).is_ok());
        assert!(
            validate_btsp_insecure_guard_with(env_map(vec![("BIOMEOS_INSECURE", "1")])).is_ok()
        );
    }

    #[test]
    fn btsp_insecure_guard_rejects_family_plus_insecure() {
        let result = validate_btsp_insecure_guard_with(env_map(vec![
            ("FAMILY_ID", "production-fam"),
            ("BIOMEOS_INSECURE", "1"),
        ]));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("BTSP_PROTOCOL_STANDARD"), "{msg}");
    }

    #[test]
    fn btsp_insecure_guard_allows_default_family_with_insecure() {
        assert!(
            validate_btsp_insecure_guard_with(env_map(vec![
                ("FAMILY_ID", "default"),
                ("BIOMEOS_INSECURE", "1"),
            ]))
            .is_ok()
        );
    }

    #[test]
    fn family_id_with_priority_chain() {
        assert_eq!(family_id_with(env_map(vec![("SONGBIRD_ORCHESTRATOR_FAMILY_ID", "p1")])), "p1");
        assert_eq!(
            family_id_with(env_map(vec![
                ("SONGBIRD_ORCHESTRATOR_FAMILY", "p2"),
                ("FAMILY_ID", "x"),
            ])),
            "p2"
        );
        assert_eq!(
            family_id_with(env_map(vec![("BIOMEOS_FAMILY_ID", "p3"), ("FAMILY_ID", "x")])),
            "p3"
        );
        assert_eq!(
            family_id_with(env_map(vec![("SONGBIRD_FAMILY_ID", "p4"), ("FAMILY_ID", "x")])),
            "p4"
        );
        assert_eq!(family_id_with(env_map(vec![("FAMILY_ID", "p5")])), "p5");
        assert_eq!(family_id_with(env_map(vec![])), "default");
    }

    #[test]
    fn data_dir_with_explicit_override() {
        let p = data_dir_with(env_map(vec![("SONGBIRD_DATA_DIR", "/var/sb/data")]));
        assert_eq!(p, PathBuf::from("/var/sb/data"));
    }

    #[test]
    fn data_dir_with_defaults_under_xdg_runtime() {
        let p = data_dir_with(env_map(vec![("XDG_RUNTIME_DIR", "/run/user/1000")]));
        assert_eq!(p, PathBuf::from("/run/user/1000/songbird-data"));
    }

    #[test]
    fn data_dir_with_defaults_under_tmpdir_when_xdg_unset() {
        let p = data_dir_with(env_map(vec![("TMPDIR", "/var/tmp/sb")]));
        assert_eq!(p, PathBuf::from("/var/tmp/sb/songbird-data"));
    }

    #[test]
    fn data_dir_with_fallback_tmp_base() {
        let p = data_dir_with(env_map(vec![]));
        assert_eq!(p, PathBuf::from("/tmp/songbird-data"));
    }

    #[test]
    fn deployment_dir_with_explicit_and_defaults() {
        assert_eq!(
            deployment_dir_with(env_map(vec![("SONGBIRD_DEPLOY_DIR", "/deploy")])),
            PathBuf::from("/deploy")
        );
        assert_eq!(
            deployment_dir_with(env_map(vec![("XDG_RUNTIME_DIR", "/xdg")])),
            PathBuf::from("/xdg/songbird-deployments")
        );
        assert_eq!(
            deployment_dir_with(env_map(vec![("TMPDIR", "/t")])),
            PathBuf::from("/t/songbird-deployments")
        );
        assert_eq!(
            deployment_dir_with(env_map(vec![])),
            PathBuf::from("/tmp/songbird-deployments")
        );
    }

    #[test]
    fn cache_dir_with_explicit_and_defaults() {
        assert_eq!(
            cache_dir_with(env_map(vec![("SONGBIRD_CACHE_DIR", "/cache")])),
            PathBuf::from("/cache")
        );
        assert_eq!(
            cache_dir_with(env_map(vec![("XDG_RUNTIME_DIR", "/xdg")])),
            PathBuf::from("/xdg/songbird-cache")
        );
        assert_eq!(
            cache_dir_with(env_map(vec![("TMPDIR", "/t")])),
            PathBuf::from("/t/songbird-cache")
        );
        assert_eq!(cache_dir_with(env_map(vec![])), PathBuf::from("/tmp/songbird-cache"));
    }
}
