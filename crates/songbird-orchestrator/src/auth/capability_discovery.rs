// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Security Provider Discovery
//!
//! Discovers ANY primal offering "security" capability via runtime discovery.
//! Songbird only knows itself — discovers security providers at runtime
//! by capability, not by name.
//!
//! **Philosophy**: Capability-first discovery. Search for `security.sock`
//! and `SECURITY_PROVIDER` before falling back to known provider names.

use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Well-known search terms for security capability socket scanning.
/// Capability terms come first; known provider names are secondary hints.
const SECURITY_SEARCH_TERMS: &[&str] = &["security", "auth", "encryption"];

/// Discover security provider socket via capability-based discovery.
///
/// ## TRUE PRIMAL Principles
///
/// 1. **Self-Knowledge**: Songbird only knows itself
/// 2. **Capability Discovery**: Searches for "security" capability first
/// 3. **Runtime Discovery**: No compile-time dependencies on providers
/// 4. **Graceful Fallback**: Works without any security provider
///
/// ## Discovery Strategy (priority order)
///
/// 1. `SECURITY_PROVIDER` env var (orchestrator-provided, preferred)
/// 2. `SECURITY_PROVIDER_SOCKET` env var (capability-standard)
/// 3. `BEARDOG_SOCKET` env var (legacy compatibility)
/// 4. Capability-named sockets: `security.sock` (XDG → UID → `/tmp/biomeos`)
/// 5. Known-provider sockets: `beardog.sock` (XDG → UID → `/tmp`)
/// 6. Filesystem scan for any socket matching security search terms
/// 7. `None` if not found (triggers secure fallback)
#[must_use]
pub fn discover_security_socket() -> Option<PathBuf> {
    discover_security_socket_with(|key| std::env::var(key))
}

/// Backward-compatible alias for [`discover_security_socket`].
#[must_use]
pub fn discover_beardog_socket() -> Option<PathBuf> {
    discover_security_socket()
}

/// Discover security provider socket with injectable env reader (concurrent-safe, testable).
pub fn discover_security_socket_with<F>(env_reader: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    info!("🔍 Discovering security provider via capability-based discovery...");

    // Strategy 1: SECURITY_PROVIDER (orchestrator-managed, preferred)
    if let Ok(socket_path) = env_reader("SECURITY_PROVIDER")
        && !socket_path.is_empty()
    {
        info!("   ✅ Found SECURITY_PROVIDER: {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    // Strategy 2: SECURITY_PROVIDER_SOCKET (capability-standard)
    if let Ok(socket_path) = env_reader("SECURITY_PROVIDER_SOCKET")
        && !socket_path.is_empty()
    {
        info!("   ✅ Found SECURITY_PROVIDER_SOCKET: {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    // Strategy 3: BEARDOG_SOCKET (legacy compatibility)
    if let Ok(socket_path) = env_reader("BEARDOG_SOCKET")
        && !socket_path.is_empty()
    {
        info!("   ✅ Found BEARDOG_SOCKET (security capability): {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    // Strategy 4+5: Capability-named sockets first, then known providers
    if let Ok(xdg_dir) = env_reader("XDG_RUNTIME_DIR") {
        // Capability name first
        let cap_path = PathBuf::from(&xdg_dir).join("biomeos").join("security.sock");
        if cap_path.exists() {
            info!("   ✅ Found security provider via XDG: {}", cap_path.display());
            return Some(cap_path);
        }
        debug!("   ⏭️  XDG capability path not found: {}", cap_path.display());
    }

    if let Ok(uid) = env_reader("UID") {
        let uid_path = PathBuf::from(format!("/run/user/{uid}/biomeos/security.sock"));
        if uid_path.exists() {
            info!("   ✅ Found security provider via UID: {}", uid_path.display());
            return Some(uid_path);
        }
    }

    let path = PathBuf::from("/tmp/biomeos/security.sock");
    if path.exists() {
        info!("   ✅ Found security provider at: {}", path.display());
        return Some(path);
    }
    let legacy = PathBuf::from("/tmp/security.sock");
    if legacy.exists() {
        info!("   ✅ Found security provider at legacy path: {}", legacy.display());
        return Some(legacy);
    }

    // Strategy 6: Scan socket directories for any security-capable socket
    if let Some(found) = scan_security_sockets() {
        info!("   ✅ Found security provider via scanning: {}", found.display());
        return Some(found);
    }

    warn!("⚠️  No security provider found via capability discovery");
    warn!("   Songbird will use secure random JWT fallback");
    warn!("   This is cryptographically secure but not coordinated with ecosystem");

    None
}

/// Backward-compatible alias for [`discover_security_socket_with`].
pub fn discover_beardog_socket_with<F>(env_reader: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    discover_security_socket_with(env_reader)
}

/// Scan socket directories for sockets matching security search terms.
fn scan_security_sockets() -> Option<PathBuf> {
    let mut dirs = Vec::with_capacity(3);
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(format!("{xdg}/biomeos"));
    }
    dirs.push("/tmp/biomeos".to_string());
    dirs.push("/tmp".to_string());

    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    let lower = file_name.to_ascii_lowercase();
                    if std::path::Path::new(&lower)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                        && SECURITY_SEARCH_TERMS.iter().any(|term| lower.contains(term))
                    {
                        return Some(entry.path());
                    }
                }
            }
        }
    }
    None
}

/// Discover security provider socket for a specific family.
#[must_use]
pub fn discover_security_socket_for_family(family_id: &str) -> Option<PathBuf> {
    discover_security_socket_for_family_with(family_id, |key| std::env::var(key))
}

/// Backward-compatible alias for [`discover_security_socket_for_family`].
#[must_use]
pub fn discover_beardog_socket_for_family(family_id: &str) -> Option<PathBuf> {
    discover_security_socket_for_family(family_id)
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

    // Check family-specific sockets — capability name first
    if let Ok(xdg_dir) = env_reader("XDG_RUNTIME_DIR") {
        for base in &["security", "auth"] {
            let family_path =
                PathBuf::from(&xdg_dir).join("biomeos").join(format!("{base}-{family_id}.sock"));
            if family_path.exists() {
                info!("   ✅ Found family-specific socket: {}", family_path.display());
                return Some(family_path);
            }
        }
    }

    // Fall back to generic capability discovery
    discover_security_socket_with(env_reader)
}

/// Backward-compatible alias for [`discover_security_socket_for_family_with`].
pub fn discover_beardog_socket_for_family_with<F>(family_id: &str, env_reader: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    discover_security_socket_for_family_with(family_id, env_reader)
}

/// Get security provider socket path for JWT provisioning.
///
/// This is the main entry point for JWT provisioning.
/// Returns the socket path to use for security provider communication.
#[must_use]
pub fn get_security_socket_for_jwt() -> Option<String> {
    discover_security_socket().map(|path| path.to_string_lossy().to_string())
}

/// Backward-compatible alias for [`get_security_socket_for_jwt`].
#[must_use]
pub fn get_beardog_socket_for_jwt() -> Option<String> {
    get_security_socket_for_jwt()
}

/// Injectable version for concurrent-safe testing.
pub fn get_security_socket_for_jwt_with<F>(env_reader: F) -> Option<String>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    discover_security_socket_with(env_reader).map(|path| path.to_string_lossy().to_string())
}

/// Backward-compatible alias for [`get_security_socket_for_jwt_with`].
pub fn get_beardog_socket_for_jwt_with<F>(env_reader: F) -> Option<String>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    get_security_socket_for_jwt_with(env_reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ============================================================================
    // ✅ ALL TESTS FULLY CONCURRENT — Zero env var mutation!
    // ============================================================================

    /// Create a mock env reader from a HashMap
    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_discover_security_provider_env() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/test-beardog.sock")]));
        let socket = discover_beardog_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-beardog.sock");
    }

    #[test]
    fn test_discover_beardog_socket_env() {
        let env =
            mock_env(HashMap::from([("BEARDOG_SOCKET", "/run/user/1000/biomeos/beardog.sock")]));
        let socket = discover_beardog_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/run/user/1000/biomeos/beardog.sock");
    }

    #[test]
    fn test_discover_priority_security_over_beardog() {
        // SECURITY_PROVIDER takes priority over BEARDOG_SOCKET
        let env = mock_env(HashMap::from([
            ("SECURITY_PROVIDER", "/high-priority.sock"),
            ("BEARDOG_SOCKET", "/low-priority.sock"),
        ]));
        let socket = discover_beardog_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/high-priority.sock");
    }

    #[test]
    fn test_discover_empty_env_ignored() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", ""), ("BEARDOG_SOCKET", "")]));
        let socket = discover_beardog_socket_with(env);
        // Empty env vars ignored — may find socket on filesystem or return None
        // Just verify no panic
        let _ = socket;
    }

    #[test]
    fn test_discover_no_env_no_panic() {
        let env = mock_env(HashMap::new());
        let socket = discover_beardog_socket_with(env);
        // May find a socket on this system, or None — just verify no panic
        let _ = socket;
    }

    #[test]
    fn test_get_beardog_socket_for_jwt() {
        let env = mock_env(HashMap::from([("SECURITY_PROVIDER", "/tmp/jwt-test.sock")]));
        let socket = get_beardog_socket_for_jwt_with(env);
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
                        Box::leak(format!("/sock-{}.sock", i).into_boxed_str()) as &str,
                    )]));
                    let socket = discover_beardog_socket_with(env);
                    assert!(socket.is_some());
                    assert_eq!(socket.unwrap().to_str().unwrap(), format!("/sock-{}.sock", i));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}
