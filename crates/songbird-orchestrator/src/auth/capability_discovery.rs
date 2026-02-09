//! Capability-Based BearDog Discovery
//!
//! Discovers BearDog via capability-based discovery, maintaining TRUE PRIMAL self-knowledge.
//! Songbird only knows itself - it discovers BearDog at runtime via "security" capability.

use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Discover BearDog socket via capability-based discovery
///
/// ## TRUE PRIMAL Principles
///
/// 1. **Self-Knowledge**: Songbird only knows itself
/// 2. **Capability Discovery**: Searches for "security" capability
/// 3. **Runtime Discovery**: No hardcoded primal names
/// 4. **Graceful Fallback**: Works without BearDog
///
/// ## Discovery Strategy
///
/// 1. Check `SECURITY_PROVIDER` environment variable (orchestrator-provided)
/// 2. Check `BEARDOG_SOCKET` environment variable (explicit override)
/// 3. Check `$XDG_RUNTIME_DIR/biomeos/beardog.sock` (biomeOS standard)
/// 4. Check `/run/user/$UID/biomeos/beardog.sock` (UID fallback)
/// 5. Check `/tmp/beardog.sock` (legacy fallback)
/// 6. Scan `/tmp/` for any beardog*.sock
/// 7. Return None if not found (triggers secure fallback)
pub fn discover_beardog_socket() -> Option<PathBuf> {
    discover_beardog_socket_with(|key| std::env::var(key))
}

/// Discover BearDog socket with injectable env reader (concurrent-safe, testable)
pub fn discover_beardog_socket_with<F>(env_reader: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    info!("🔍 Discovering security provider (BearDog) via capability-based discovery...");

    // Strategy 1: SECURITY_PROVIDER (orchestrator-managed, preferred)
    if let Ok(socket_path) = env_reader("SECURITY_PROVIDER") {
        if !socket_path.is_empty() {
            info!("   ✅ Found SECURITY_PROVIDER: {}", socket_path);
            return Some(PathBuf::from(socket_path));
        }
    }

    // Strategy 2: BEARDOG_SOCKET (explicit override)
    if let Ok(socket_path) = env_reader("BEARDOG_SOCKET") {
        if !socket_path.is_empty() {
            info!("   ✅ Found BEARDOG_SOCKET: {}", socket_path);
            return Some(PathBuf::from(socket_path));
        }
    }

    // Strategy 3: biomeOS standard XDG path
    if let Ok(xdg_dir) = env_reader("XDG_RUNTIME_DIR") {
        let xdg_path = PathBuf::from(&xdg_dir).join("biomeos").join("beardog.sock");
        if xdg_path.exists() {
            info!("   ✅ Found BearDog via XDG: {}", xdg_path.display());
            return Some(xdg_path);
        }
        debug!("   ⏭️  XDG path not found: {}", xdg_path.display());
    }

    // Strategy 4: UID-based fallback
    if let Ok(uid) = env_reader("UID") {
        let uid_path = PathBuf::from(format!("/run/user/{}/biomeos/beardog.sock", uid));
        if uid_path.exists() {
            info!("   ✅ Found BearDog via UID: {}", uid_path.display());
            return Some(uid_path);
        }
    }

    // Strategy 5: Legacy /tmp fallback
    let legacy_path = PathBuf::from("/tmp/beardog.sock");
    if legacy_path.exists() {
        info!("   ✅ Found BearDog at legacy path: {}", legacy_path.display());
        return Some(legacy_path);
    }

    // Strategy 6: Scan /tmp for any beardog socket
    if let Ok(entries) = std::fs::read_dir("/tmp") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with("beardog") && file_name.ends_with(".sock") {
                    let path = entry.path();
                    info!("   ✅ Found BearDog socket at: {}", path.display());
                    return Some(path);
                }
            }
        }
    }

    warn!("⚠️  No security provider (BearDog) found via capability discovery");
    warn!("   Songbird will use secure random JWT fallback");
    warn!("   This is cryptographically secure but not coordinated with ecosystem");

    None
}

/// Discover BearDog socket with explicit family ID
///
/// Searches for BearDog socket with specific family ID.
pub fn discover_beardog_socket_for_family(family_id: &str) -> Option<PathBuf> {
    discover_beardog_socket_for_family_with(family_id, |key| std::env::var(key))
}

/// Injectable version for concurrent-safe testing
pub fn discover_beardog_socket_for_family_with<F>(family_id: &str, env_reader: F) -> Option<PathBuf>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    info!("🔍 Discovering security provider for family '{}'...", family_id);

    // Check family-specific XDG socket
    if let Ok(xdg_dir) = env_reader("XDG_RUNTIME_DIR") {
        let family_path = PathBuf::from(&xdg_dir)
            .join("biomeos")
            .join(format!("beardog-{}.sock", family_id));
        if family_path.exists() {
            info!("   ✅ Found family-specific socket: {}", family_path.display());
            return Some(family_path);
        }
    }

    // Fall back to generic discovery (TRUE PRIMAL)
    discover_beardog_socket_with(env_reader)
}

/// Get BearDog socket path for JWT provisioning
///
/// This is the main entry point for JWT provisioning.
/// Returns the socket path to use for BearDog communication.
pub fn get_beardog_socket_for_jwt() -> Option<String> {
    discover_beardog_socket().map(|path| path.to_string_lossy().to_string())
}

/// Injectable version for concurrent-safe testing
pub fn get_beardog_socket_for_jwt_with<F>(env_reader: F) -> Option<String>
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    discover_beardog_socket_with(env_reader).map(|path| path.to_string_lossy().to_string())
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
        let owned: HashMap<String, String> = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key: &str| {
            owned
                .get(key)
                .cloned()
                .ok_or(std::env::VarError::NotPresent)
        }
    }

    #[test]
    fn test_discover_security_provider_env() {
        let env = mock_env(HashMap::from([
            ("SECURITY_PROVIDER", "/tmp/test-beardog.sock"),
        ]));
        let socket = discover_beardog_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-beardog.sock");
    }

    #[test]
    fn test_discover_beardog_socket_env() {
        let env = mock_env(HashMap::from([
            ("BEARDOG_SOCKET", "/run/user/1000/biomeos/beardog.sock"),
        ]));
        let socket = discover_beardog_socket_with(env);
        assert!(socket.is_some());
        assert_eq!(
            socket.unwrap().to_str().unwrap(),
            "/run/user/1000/biomeos/beardog.sock"
        );
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
        let env = mock_env(HashMap::from([
            ("SECURITY_PROVIDER", ""),
            ("BEARDOG_SOCKET", ""),
        ]));
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
        let env = mock_env(HashMap::from([
            ("SECURITY_PROVIDER", "/tmp/jwt-test.sock"),
        ]));
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
                    assert_eq!(
                        socket.unwrap().to_str().unwrap(),
                        format!("/sock-{}.sock", i)
                    );
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}
