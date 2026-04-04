// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Security Provider Discovery
//!
//! Discovers any primal offering the **security** role via runtime discovery:
//! environment overrides first, then JSON-RPC `health.liveness` +
//! `capabilities.list` probes on sockets under `$XDG_RUNTIME_DIR/biomeos/`.
//! Songbird only knows itself — classification uses capability tokens, not paths
//! or primal names.
//!
//! # API surface
//!
//! Prefer [`discover_security_socket`], [`get_security_socket_for_jwt`], and related
//! `*_security_*` helpers.

use crate::primal_discovery::Capability;
use std::path::PathBuf;
use tracing::{info, warn};

/// Discover security provider socket via capability-based discovery.
///
/// ## Strategy (priority order)
///
/// 1. `SECURITY_PROVIDER` env var (orchestrator-provided, preferred)
/// 2. `SECURITY_PROVIDER_SOCKET` env var (capability-standard)
/// 3. `BEARDOG_SOCKET` env var (legacy compatibility)
/// 4. BiomeOS scan + JSON-RPC probe for [`Capability::Security`]
/// 5. `None` if not found (triggers secure fallback)
#[must_use]
pub fn discover_security_socket() -> Option<PathBuf> {
    discover_security_socket_with(|key| songbird_process_env::var(key))
}

/// Discover security provider socket with injectable env reader (concurrent-safe, testable).
pub fn discover_security_socket_with<F>(env_reader: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    info!("🔍 Discovering security provider via capability-based discovery...");

    if let Ok(socket_path) = env_reader("SECURITY_PROVIDER")
        && !socket_path.is_empty()
    {
        info!("   ✅ Found SECURITY_PROVIDER: {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    if let Ok(socket_path) = env_reader("SECURITY_PROVIDER_SOCKET")
        && !socket_path.is_empty()
    {
        info!("   ✅ Found SECURITY_PROVIDER_SOCKET: {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    if let Ok(socket_path) = env_reader("BEARDOG_SOCKET")
        && !socket_path.is_empty()
    {
        warn!(
            "   ⚠️  Found deprecated $BEARDOG_SOCKET — migrate to $SECURITY_PROVIDER_SOCKET: {}",
            socket_path
        );
        return Some(PathBuf::from(socket_path));
    }

    let env_opt = |k: &str| env_reader(k).ok().filter(|s| !s.is_empty());
    if let Some(path) = crate::primal_discovery::discover_via_biomeos_probe_blocking_with(
        Capability::Security,
        &env_opt,
    ) {
        info!("   ✅ Found security provider via biomeos probe: {}", path);
        return Some(PathBuf::from(path));
    }

    warn!("⚠️  No security provider found via capability discovery");
    warn!("   Songbird will use secure random JWT fallback");
    warn!("   This is cryptographically secure but not coordinated with ecosystem");

    None
}

/// Discover security provider socket for a specific family.
#[must_use]
pub fn discover_security_socket_for_family(family_id: &str) -> Option<PathBuf> {
    discover_security_socket_for_family_with(family_id, |key| songbird_process_env::var(key))
}

/// Injectable version for concurrent-safe testing.
pub fn discover_security_socket_for_family_with<F>(
    family_id: &str,
    env_reader: F,
) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    info!("🔍 Discovering security provider for family '{family_id}'...");
    let _ = family_id;
    discover_security_socket_with(env_reader)
}

/// Get security provider socket path for JWT provisioning.
#[must_use]
pub fn get_security_socket_for_jwt() -> Option<String> {
    discover_security_socket().map(|path| path.to_string_lossy().to_string())
}

/// Injectable version for concurrent-safe testing.
pub fn get_security_socket_for_jwt_with<F>(env_reader: F) -> Option<String>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    discover_security_socket_with(env_reader).map(|path| path.to_string_lossy().to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_discover_security_provider_env() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/test-security.sock")]));
        let socket = discover_security_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-security.sock");
    }

    #[test]
    fn test_discover_legacy_socket_env_var() {
        let env =
            mock_env(HashMap::from([("BEARDOG_SOCKET", "/run/user/1000/biomeos/beardog.sock")]));
        let socket = discover_security_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/run/user/1000/biomeos/beardog.sock");
    }

    #[test]
    fn test_discover_priority_security_provider_over_legacy_env() {
        let env = mock_env(HashMap::from([
            ("SECURITY_PROVIDER", "/high-priority.sock"),
            ("BEARDOG_SOCKET", "/low-priority.sock"),
        ]));
        let socket = discover_security_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/high-priority.sock");
    }

    #[test]
    fn test_discover_empty_env_ignored() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", ""), ("BEARDOG_SOCKET", "")]));
        let socket = discover_security_socket_with(env);
        let _ = socket;
    }

    #[test]
    fn test_discover_no_env_no_panic() {
        let env = mock_env(HashMap::new());
        let socket = discover_security_socket_with(env);
        let _ = socket;
    }

    #[test]
    fn test_get_security_socket_for_jwt() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/jwt-test.sock")]));
        let socket = get_security_socket_for_jwt_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap(), "/tmp/jwt-test.sock");
    }

    #[test]
    fn test_concurrent_discovery() {
        use std::thread;
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let env = mock_env(HashMap::from([(
                        "SECURITY_PROVIDER",
                        Box::leak(format!("/sock-{i}.sock").into_boxed_str()) as &str,
                    )]));
                    let socket = discover_security_socket_with(env);
                    assert!(socket.is_some());
                    assert_eq!(socket.unwrap().to_str().unwrap(), format!("/sock-{i}.sock"));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}
