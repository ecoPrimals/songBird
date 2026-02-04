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
//! 4. **Capability Discovery**: Other primals discovered via primal_discovery module
//!
//! ## Environment Variables
//!
//! ### Identity (Self-Knowledge)
//! - `PRIMAL_NAME`: This primal's name (default: "songbird")
//! - `FAMILY_ID` / `SONGBIRD_FAMILY_ID`: Family/biome ID (default: "nat0")
//! - `NODE_ID` / `SONGBIRD_NODE_ID`: Node ID (default: "default")
//!
//! ### Paths (Self-Knowledge)
//! - `SONGBIRD_SOCKET`: This primal's IPC socket path
//! - `SONGBIRD_DATA_DIR`: Data directory (default: "/tmp/songbird-data")
//! - `SONGBIRD_DEPLOY_DIR`: Deployment directory (default: "/tmp/songbird-deployments")
//! - `SONGBIRD_CACHE_DIR`: Cache directory (default: "/tmp/songbird-cache")
//!
//! ### Discovery (Other Primals)
//! - See `primal_discovery` module for discovering other primals

use std::path::PathBuf;

/// Get this primal's name (self-knowledge)
pub fn primal_name() -> String {
    std::env::var("PRIMAL_NAME").unwrap_or_else(|_| "songbird".to_string())
}

/// Get family/biome ID (self-knowledge)
///
/// Priority order (BiomeOS Neural API compatible):
/// 1. `SONGBIRD_ORCHESTRATOR_FAMILY_ID` (highest - Neural API standard)
/// 2. `SONGBIRD_ORCHESTRATOR_FAMILY` (alternative)
/// 3. `BIOMEOS_FAMILY_ID` (generic orchestrator)
/// 4. `SONGBIRD_FAMILY_ID` (legacy)
/// 5. `FAMILY_ID` (generic)
/// 6. Default: `"nat0"` (NAT-friendly network family 0)
pub fn family_id() -> String {
    std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
        .or_else(|_| std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY"))
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "nat0".to_string())
}

/// Get node ID (self-knowledge)
pub fn node_id() -> String {
    std::env::var("SONGBIRD_NODE_ID")
        .or_else(|_| std::env::var("NODE_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Get this primal's IPC socket path (self-knowledge)
///
/// Resolution order (BiomeOS XDG Standard):
/// 1. `SONGBIRD_SOCKET` (explicit override - full path)
/// 2. `BIOMEOS_SOCKET_DIR` + `songbird.sock` (shared socket directory)
/// 3. `/run/user/$UID/biomeos/songbird.sock` (XDG-compliant default)
/// 4. `/tmp/songbird.sock` (legacy fallback if XDG unavailable)
///
/// **Socket Naming Standard**: Uses primal name only (`songbird.sock`),
/// NOT binary name (`songbird-orchestrator.sock`). Family ID is NOT
/// included in the socket name for biomeOS compliance.
pub fn socket_path() -> PathBuf {
    // Priority 1: Explicit SONGBIRD_SOCKET override
    if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
        return PathBuf::from(path);
    }

    // Priority 2: BIOMEOS_SOCKET_DIR + primal name
    if let Ok(socket_dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        let path = PathBuf::from(socket_dir).join("songbird.sock");
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return path;
    }

    // Priority 3: XDG-compliant default (/run/user/$UID/biomeos/)
    // Extract UID from XDG_RUNTIME_DIR (Pure Rust, no unsafe!)
    let xdg_socket = if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        // XDG_RUNTIME_DIR is typically /run/user/{uid}
        PathBuf::from(xdg_runtime_dir).join("biomeos/songbird.sock")
    } else if let Ok(uid_str) = std::env::var("UID") {
        // Fallback to UID env var
        PathBuf::from(format!("/run/user/{}/biomeos/songbird.sock", uid_str))
    } else {
        // Final fallback: legacy /tmp
        PathBuf::from("/tmp/songbird.sock")
    };

    // Ensure directory exists (Pure Rust!)
    if let Some(parent) = xdg_socket.parent() {
        if std::fs::create_dir_all(parent).is_ok() {
            return xdg_socket;
        }
    }

    // Priority 4: Legacy /tmp fallback (if XDG unavailable or directory creation failed)
    PathBuf::from("/tmp/songbird.sock")
}

/// Get data directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_DATA_DIR` (explicit override)
/// 2. `/tmp/songbird-data` (default)
pub fn data_dir() -> PathBuf {
    std::env::var("SONGBIRD_DATA_DIR")
        .map_or_else(|_| PathBuf::from("/tmp/songbird-data"), PathBuf::from)
}

/// Get deployment directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_DEPLOY_DIR` (explicit override)
/// 2. `/tmp/songbird-deployments` (default)
pub fn deployment_dir() -> PathBuf {
    std::env::var("SONGBIRD_DEPLOY_DIR")
        .map_or_else(|_| PathBuf::from("/tmp/songbird-deployments"), PathBuf::from)
}

/// Get cache directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_CACHE_DIR` (explicit override)
/// 2. `/tmp/songbird-cache` (default)
pub fn cache_dir() -> PathBuf {
    std::env::var("SONGBIRD_CACHE_DIR")
        .map_or_else(|_| PathBuf::from("/tmp/songbird-cache"), PathBuf::from)
}

/// Get HTTP server bind address (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_ADDR` (explicit override)
/// 2. `0.0.0.0:8080` (default - bind all interfaces)
pub fn http_bind_address() -> String {
    std::env::var("SONGBIRD_HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string())
}

/// Get HTTP server port (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_PORT` (explicit override)
/// 2. Extract from `SONGBIRD_HTTP_ADDR` if set
/// 3. `8080` (default)
pub fn http_port() -> u16 {
    if let Ok(port_str) = std::env::var("SONGBIRD_HTTP_PORT") {
        return port_str.parse().unwrap_or(8080);
    }

    // Try to extract from bind address
    if let Ok(addr) = std::env::var("SONGBIRD_HTTP_ADDR") {
        if let Some(port_part) = addr.split(':').nth(1) {
            if let Ok(port) = port_part.parse() {
                return port;
            }
        }
    }

    8080
}

/// Check if running in production mode
///
/// Resolution order:
/// 1. `SONGBIRD_ENV == "production"`
/// 2. `RUST_ENV == "production"`
/// 3. `false` (default to development)
pub fn is_production() -> bool {
    std::env::var("SONGBIRD_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .map(|v| v == "production")
        .unwrap_or(false)
}

/// Get log level (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_LOG` (explicit override)
/// 2. `RUST_LOG` (Rust standard)
/// 3. `"info"` (default)
pub fn log_level() -> String {
    std::env::var("SONGBIRD_LOG")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".to_string())
}

// ═══════════════════════════════════════════════════════════════════════════
// Dark Forest Beacon Genetics Configuration (NEW - Feb 3, 2026)
// ═══════════════════════════════════════════════════════════════════════════

/// Check if Dark Forest beacons are enabled
///
/// Reads `SONGBIRD_DARK_FOREST` environment variable.
///
/// When `true`, Songbird broadcasts Dark Forest beacons (version 2, fully encrypted).
/// When `false`, Songbird broadcasts legacy BirdSongPacket (version 1.0, plaintext family_id).
///
/// **Default**: `false` (opt-in for privacy, requires BearDog beacon.* RPC)
pub fn dark_forest_enabled() -> bool {
    std::env::var("SONGBIRD_DARK_FOREST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}

/// Check if legacy BirdSongPacket format should be accepted
///
/// Reads `SONGBIRD_ACCEPT_LEGACY_BIRDSONG` environment variable.
///
/// When `true`, accepts both Dark Forest beacons AND legacy BirdSongPacket.
/// When `false`, only accepts Dark Forest beacons (rejects legacy).
///
/// **Default**: `true` (backward compatible during migration)
pub fn accept_legacy_birdsong() -> bool {
    std::env::var("SONGBIRD_ACCEPT_LEGACY_BIRDSONG")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(true)
}

/// Check if dual broadcast is enabled (both formats)
///
/// Reads `SONGBIRD_DUAL_BROADCAST` environment variable.
///
/// When `true`, broadcasts BOTH Dark Forest beacons AND legacy BirdSongPacket.
/// When `false`, only broadcasts Dark Forest beacons (if enabled).
///
/// **Default**: `false` (minimize network overhead)
pub fn dual_broadcast() -> bool {
    std::env::var("SONGBIRD_DUAL_BROADCAST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_name_default() {
        std::env::remove_var("PRIMAL_NAME");
        assert_eq!(primal_name(), "songbird");
    }

    #[test]
    fn test_family_id_default() {
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        std::env::remove_var("FAMILY_ID");
        assert_eq!(family_id(), "nat0");
    }

    #[test]
    fn test_socket_path_default() {
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");

        let path = socket_path();

        // Should be either XDG (/run/user/{uid}/biomeos/songbird.sock) or /tmp fallback
        let path_str = path.to_string_lossy();
        assert!(
            path_str.ends_with("/biomeos/songbird.sock") || path_str == "/tmp/songbird.sock",
            "Expected XDG or /tmp fallback, got: {}",
            path_str
        );
    }

    #[test]
    fn test_socket_path_explicit_override() {
        std::env::set_var("SONGBIRD_SOCKET", "/custom/path/test.sock");
        let path = socket_path();
        std::env::remove_var("SONGBIRD_SOCKET");
        assert_eq!(path, PathBuf::from("/custom/path/test.sock"));
    }

    #[test]
    fn test_socket_path_biomeos_dir() {
        // Clear explicit override
        std::env::remove_var("SONGBIRD_SOCKET");

        std::env::set_var("BIOMEOS_SOCKET_DIR", "/tmp/test-biomeos");
        let path = socket_path();
        std::env::remove_var("BIOMEOS_SOCKET_DIR");

        assert_eq!(path, PathBuf::from("/tmp/test-biomeos/songbird.sock"));
    }

    #[test]
    fn test_data_dir_default() {
        std::env::remove_var("SONGBIRD_DATA_DIR");
        assert_eq!(data_dir(), PathBuf::from("/tmp/songbird-data"));
    }

    #[test]
    fn test_http_port_default() {
        // Clear ALL related env vars to prevent test pollution
        std::env::remove_var("SONGBIRD_HTTP_PORT");
        std::env::remove_var("SONGBIRD_HTTP_ADDR");
        std::env::remove_var("HTTP_PORT");
        std::env::remove_var("PORT");
        assert_eq!(http_port(), 8080);
    }

    #[test]
    fn test_http_port_from_addr() {
        // Clean environment first
        std::env::remove_var("SONGBIRD_HTTP_PORT");
        std::env::remove_var("SONGBIRD_HTTP_ADDR");

        std::env::set_var("SONGBIRD_HTTP_ADDR", "0.0.0.0:9090");
        let port = http_port();
        std::env::remove_var("SONGBIRD_HTTP_ADDR");
        assert_eq!(port, 9090);
    }

    #[test]
    fn test_is_production_default() {
        std::env::remove_var("SONGBIRD_ENV");
        std::env::remove_var("RUST_ENV");
        assert!(!is_production());
    }

    #[test]
    fn test_log_level_default() {
        std::env::remove_var("SONGBIRD_LOG");
        std::env::remove_var("RUST_LOG");
        assert_eq!(log_level(), "info");
    }
}
