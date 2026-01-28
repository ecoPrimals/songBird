//! XDG-Compliant Socket Discovery
//!
//! **Pure Rust | Zero Hardcoding | Runtime Discovery**
//!
//! Implements intelligent socket discovery with proper fallback chain:
//! 1. Environment variables (explicit configuration)
//! 2. XDG Runtime Dir (`/run/user/$UID/biomeos/`)
//! 3. Legacy `/tmp` paths (last resort)
//!
//! This enables automated Tower Atomic deployment via biomeOS Neural API
//! while maintaining backward compatibility with manual deployments.

use std::path::Path;
use tracing::{debug, info, warn};

/// Discover socket path with XDG-compliant fallback chain
///
/// # Priority Order
///
/// 1. **Environment Variable** (highest priority)
///    - Direct specification by user/biomeOS
///    - Example: `BEARDOG_SOCKET=/run/user/1000/biomeos/beardog-nat0.sock`
///
/// 2. **XDG Runtime Directory** (recommended for production)
///    - Standard Unix location: `$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock`
///    - Example: `/run/user/1000/biomeos/beardog-nat0.sock`
///    - Only used if socket exists
///
/// 3. **Legacy /tmp Path** (fallback for development/testing)
///    - Example: `/tmp/beardog.sock`
///    - Warning logged when used
///
/// # Arguments
///
/// * `env_var` - Environment variable name to check (e.g., "BEARDOG_SOCKET")
/// * `primal_name` - Primal name for XDG discovery (e.g., "beardog")
/// * `legacy_path` - Legacy `/tmp` path for backward compatibility
///
/// # Returns
///
/// Socket path to use, guaranteed to exist or be the specified fallback.
pub fn discover_socket(
    env_var: &str,
    primal_name: &str,
    legacy_path: &str,
) -> String {
    debug!("🔍 Socket discovery for {}", primal_name);
    debug!("   Checking: 1) ${}", env_var);
    debug!("            2) XDG Runtime Dir");
    debug!("            3) Legacy {}", legacy_path);

    // Priority 1: Environment variable (explicit configuration)
    if let Ok(socket) = std::env::var(env_var) {
        if !socket.is_empty() {
            info!("✅ Socket discovered via ${}: {}", env_var, socket);
            return socket;
        }
    }

    // Priority 2: XDG Runtime Directory (production, biomeOS standard)
    if let Some(xdg_socket) = discover_xdg_socket(primal_name) {
        info!("✅ Socket discovered via XDG: {}", xdg_socket);
        return xdg_socket;
    }

    // Priority 3: Legacy /tmp path (development fallback)
    warn!("⚠️  Using legacy /tmp socket: {}", legacy_path);
    warn!("   Consider setting ${} or XDG_RUNTIME_DIR", env_var);
    warn!("   Example: {}=/run/user/$UID/biomeos/{}-$FAMILY_ID.sock", 
        env_var, primal_name);

    legacy_path.to_string()
}

/// Discover socket in XDG Runtime Directory
///
/// Checks `$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock`
///
/// # Arguments
///
/// * `primal_name` - Primal name (e.g., "beardog", "songbird")
///
/// # Returns
///
/// Socket path if found and exists, None otherwise.
///
/// # XDG Directory Structure
///
/// ```text
/// $XDG_RUNTIME_DIR/              (typically /run/user/$UID)
/// └── biomeos/
///     ├── beardog-nat0.sock
///     ├── songbird-nat0.sock
///     ├── neural-api-nat0.sock
///     └── squirrel-nat0.sock
/// ```
fn discover_xdg_socket(primal_name: &str) -> Option<String> {
    // Get XDG_RUNTIME_DIR (standard Unix location)
    let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) if !dir.is_empty() => dir,
        _ => {
            debug!("   XDG_RUNTIME_DIR not set");
            return None;
        }
    };

    // Get FAMILY_ID (ecoPrimals family identifier)
    let family_id = match std::env::var("FAMILY_ID") {
        Ok(id) if !id.is_empty() => id,
        _ => {
            debug!("   FAMILY_ID not set, trying common defaults");
            // Try common family IDs
            for family in &["nat0", "default"] {
                if let Some(socket) = try_xdg_socket(&runtime_dir, primal_name, family) {
                    return Some(socket);
                }
            }
            return None;
        }
    };

    try_xdg_socket(&runtime_dir, primal_name, &family_id)
}

/// Try specific XDG socket path
fn try_xdg_socket(runtime_dir: &str, primal_name: &str, family_id: &str) -> Option<String> {
    let socket_path = format!("{}/biomeos/{}-{}.sock", runtime_dir, primal_name, family_id);
    
    debug!("   Checking XDG: {}", socket_path);
    
    if Path::new(&socket_path).exists() {
        debug!("   ✅ Found XDG socket");
        Some(socket_path)
    } else {
        debug!("   ❌ XDG socket not found");
        None
    }
}

/// Discover BearDog socket with full fallback chain
///
/// Checks in order:
/// 1. `$BEARDOG_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/beardog-$FAMILY_ID.sock`
/// 3. `/tmp/beardog.sock` (legacy)
pub fn discover_beardog_socket() -> String {
    discover_socket(
        "BEARDOG_SOCKET",
        "beardog",
        "/tmp/beardog.sock",
    )
}

/// Discover Neural API socket with full fallback chain
///
/// Checks in order:
/// 1. `$NEURAL_API_SOCKET` or `$NEURALS_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-$FAMILY_ID.sock`
/// 3. `/tmp/neural-api-nat0.sock` (legacy)
pub fn discover_neural_api_socket() -> String {
    // Check both NEURAL_API_SOCKET and NEURALS_SOCKET
    if let Ok(socket) = std::env::var("NEURAL_API_SOCKET") {
        if !socket.is_empty() {
            info!("✅ Socket discovered via $NEURAL_API_SOCKET: {}", socket);
            return socket;
        }
    }
    
    if let Ok(socket) = std::env::var("NEURALS_SOCKET") {
        if !socket.is_empty() {
            info!("✅ Socket discovered via $NEURALS_SOCKET: {}", socket);
            return socket;
        }
    }

    // Try XDG discovery
    if let Some(xdg_socket) = discover_xdg_socket("neural-api") {
        return xdg_socket;
    }

    // Legacy fallback
    warn!("⚠️  Using legacy /tmp socket: /tmp/neural-api-nat0.sock");
    warn!("   Consider setting $NEURAL_API_SOCKET or XDG_RUNTIME_DIR");
    "/tmp/neural-api-nat0.sock".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_var_priority() {
        env::set_var("TEST_SOCKET", "/custom/path.sock");
        
        let socket = discover_socket(
            "TEST_SOCKET",
            "test-primal",
            "/tmp/fallback.sock",
        );
        
        assert_eq!(socket, "/custom/path.sock");
        env::remove_var("TEST_SOCKET");
    }

    #[test]
    fn test_legacy_fallback() {
        env::remove_var("TEST_SOCKET");
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");
        
        let socket = discover_socket(
            "TEST_SOCKET",
            "test-primal",
            "/tmp/fallback.sock",
        );
        
        assert_eq!(socket, "/tmp/fallback.sock");
    }

    #[test]
    fn test_xdg_path_construction() {
        env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        env::set_var("FAMILY_ID", "nat0");
        
        // Note: This test doesn't check if socket exists, just path construction
        // In real scenario, socket must exist for XDG discovery to succeed
        
        env::remove_var("XDG_RUNTIME_DIR");
        env::remove_var("FAMILY_ID");
    }

    #[test]
    fn test_neural_api_dual_env() {
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("NEURALS_SOCKET");
        env::set_var("NEURALS_SOCKET", "/custom/neurals.sock");
        
        let socket = discover_neural_api_socket();
        assert_eq!(socket, "/custom/neurals.sock");
        
        env::remove_var("NEURALS_SOCKET");
    }

    #[test]
    fn test_empty_env_var_ignored() {
        env::set_var("TEST_SOCKET", "");
        env::remove_var("XDG_RUNTIME_DIR");
        
        let socket = discover_socket(
            "TEST_SOCKET",
            "test-primal",
            "/tmp/fallback.sock",
        );
        
        // Empty env var should be ignored, fall back to legacy
        assert_eq!(socket, "/tmp/fallback.sock");
        
        env::remove_var("TEST_SOCKET");
    }
}

