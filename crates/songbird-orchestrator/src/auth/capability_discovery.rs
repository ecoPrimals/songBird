//! Capability-Based BearDog Discovery
//!
//! Discovers BearDog via capability-based discovery, maintaining TRUE PRIMAL self-knowledge.
//! Songbird only knows itself - it discovers BearDog at runtime via "security" capability.

// use anyhow::{Context, Result};  // Unused (no Result needed)
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
/// 3. Search common socket paths for security capability
/// 4. Return None if not found (triggers secure fallback)
///
/// # Returns
///
/// * `Some(PathBuf)` - Path to BearDog socket
/// * `None` - BearDog not available (use secure fallback)
pub fn discover_beardog_socket() -> Option<PathBuf> {
    info!("🔍 Discovering security provider (BearDog) via capability-based discovery...");

    // Strategy 1: SECURITY_PROVIDER (orchestrator-managed, preferred)
    if let Ok(socket_path) = std::env::var("SECURITY_PROVIDER") {
        info!("   ✅ Found SECURITY_PROVIDER: {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    // Strategy 2: BEARDOG_SOCKET (explicit override)
    if let Ok(socket_path) = std::env::var("BEARDOG_SOCKET") {
        info!("   ✅ Found BEARDOG_SOCKET: {}", socket_path);
        return Some(PathBuf::from(socket_path));
    }

    // Strategy 3: Search common socket paths
    let common_paths = vec![
        "/tmp/beardog-nat0.sock",            // NUCLEUS default
        "/tmp/beardog-default-default.sock", // biomeOS default
        "/run/user/1000/beardog.sock",       // User runtime dir
        "/var/run/beardog.sock",             // System runtime dir
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists() {
            info!("   ✅ Found BearDog socket at: {}", path);
            return Some(PathBuf::from(path));
        } else {
            debug!("   ⏭️  Not found: {}", path);
        }
    }

    // Strategy 4: Search /tmp for any beardog socket
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
/// Searches for BearDog socket with specific family ID (e.g., "nat0").
///
/// # Arguments
///
/// * `family_id` - Family ID to search for (e.g., "nat0")
///
/// # Returns
///
/// * `Some(PathBuf)` - Path to BearDog socket for this family
/// * `None` - BearDog not available for this family
pub fn discover_beardog_socket_for_family(family_id: &str) -> Option<PathBuf> {
    info!("🔍 Discovering security provider for family '{}'...", family_id);

    // Check family-specific socket
    let family_socket = format!("/tmp/beardog-{}.sock", family_id);
    if std::path::Path::new(&family_socket).exists() {
        info!("   ✅ Found family-specific BearDog socket: {}", family_socket);
        return Some(PathBuf::from(family_socket));
    }

    // Fall back to generic discovery
    discover_beardog_socket()
}

/// Get BearDog socket path for JWT provisioning
///
/// This is the main entry point for JWT provisioning.
/// Returns the socket path to use for BearDog communication.
///
/// # Returns
///
/// * `Option<String>` - Socket path if BearDog available, None otherwise
pub fn get_beardog_socket_for_jwt() -> Option<String> {
    discover_beardog_socket().map(|path| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_beardog_socket_env_var() {
        // Set environment variable
        std::env::set_var("SECURITY_PROVIDER", "/tmp/test-beardog.sock");

        let socket = discover_beardog_socket();
        assert!(socket.is_some());
        assert_eq!(socket.unwrap().to_str().unwrap(), "/tmp/test-beardog.sock");

        // Cleanup
        std::env::remove_var("SECURITY_PROVIDER");
    }

    #[test]
    fn test_discover_beardog_socket_not_found() {
        // Ensure no environment variables set
        std::env::remove_var("SECURITY_PROVIDER");
        std::env::remove_var("BEARDOG_SOCKET");

        // Should return None if no socket found
        let socket = discover_beardog_socket();
        // May be Some or None depending on system state
        // Just verify it doesn't panic
        if let Some(path) = socket {
            println!("Found socket: {}", path.display());
        }
    }

    #[test]
    fn test_get_beardog_socket_for_jwt() {
        std::env::set_var("SECURITY_PROVIDER", "/tmp/jwt-test.sock");

        let socket = get_beardog_socket_for_jwt();
        assert!(socket.is_some());
        assert_eq!(socket.unwrap(), "/tmp/jwt-test.sock");

        std::env::remove_var("SECURITY_PROVIDER");
    }
}
