// SPDX-License-Identifier: AGPL-3.0-only
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
//! - `SONGBIRD_DATA_DIR`: Data directory (default: "/tmp/songbird-data")
//! - `SONGBIRD_DEPLOY_DIR`: Deployment directory (default: "/tmp/songbird-deployments")
//! - `SONGBIRD_CACHE_DIR`: Cache directory (default: "/tmp/songbird-cache")
//!
//! ### Discovery (Other Primals)
//! - See `primal_discovery` module for discovering other primals

use std::path::PathBuf;

use songbird_types::defaults::{hosts::DEFAULT_BIND_ALL, ports::DEFAULT_HTTP_PORT};

/// Get this primal's name (self-knowledge)
#[must_use]
pub fn primal_name() -> String {
    std::env::var("PRIMAL_NAME").unwrap_or_else(|_| "songbird".to_string())
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
    std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
        .or_else(|_| std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY"))
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Get node ID (self-knowledge)
#[must_use]
pub fn node_id() -> String {
    std::env::var("SONGBIRD_NODE_ID")
        .or_else(|_| std::env::var("NODE_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Get this primal's IPC socket path (self-knowledge)
///
/// Resolution order (`BiomeOS` XDG Standard):
/// 1. `SONGBIRD_SOCKET` (explicit override - full path)
/// 2. `BIOMEOS_SOCKET_DIR` + socket name (shared socket directory)
/// 3. `/run/user/$UID/biomeos/` + socket name (XDG-compliant default)
/// 4. `/tmp/` + socket name (legacy fallback if XDG unavailable)
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
    if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
        return PathBuf::from(path);
    }

    let sock_name = socket_name();

    // Priority 2: BIOMEOS_SOCKET_DIR + socket name
    if let Ok(socket_dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        let path = PathBuf::from(socket_dir).join(&sock_name);
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return path;
    }

    // Priority 3: XDG-compliant default (/run/user/$UID/biomeos/)
    // Extract UID from XDG_RUNTIME_DIR (Pure Rust, no unsafe!)
    let xdg_socket = std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| {
            std::env::var("UID").map_or_else(
                |_| PathBuf::from(format!("/tmp/{sock_name}")),
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

    // Priority 4: Legacy /tmp fallback (if XDG unavailable or directory creation failed)
    PathBuf::from(format!("/tmp/{sock_name}"))
}

/// Get the socket filename based on multi-family configuration
///
/// Returns:
/// - `songbird.sock` in single-family mode (default)
/// - `songbird-{family_id}.sock` in multi-family mode
///
/// Multi-family mode is activated by:
/// - `SONGBIRD_MULTI_FAMILY=true` or `SONGBIRD_FAMILY_SOCKET=true`
///
/// This enables the "shared machine" architecture where multiple
/// ecosystem families coexist, each with their own Songbird instance.
#[must_use]
pub fn socket_name() -> String {
    let multi_family = std::env::var("SONGBIRD_MULTI_FAMILY")
        .or_else(|_| std::env::var("SONGBIRD_FAMILY_SOCKET"))
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if multi_family {
        let fam_id = family_id();
        format!("songbird-{fam_id}.sock")
    } else {
        "songbird.sock".to_string()
    }
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
/// 2. `DEFAULT_BIND_ALL`:`DEFAULT_HTTP_PORT` from `songbird_types::defaults` (bind all interfaces)
#[must_use]
pub fn http_bind_address() -> String {
    std::env::var("SONGBIRD_HTTP_ADDR")
        .unwrap_or_else(|_| format!("{DEFAULT_BIND_ALL}:{DEFAULT_HTTP_PORT}"))
}

/// Get HTTP server port (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_PORT` (explicit override)
/// 2. Extract from `SONGBIRD_HTTP_ADDR` if set
/// 3. `DEFAULT_HTTP_PORT` from `songbird_types::defaults` (default)
#[must_use]
pub fn http_port() -> u16 {
    if let Ok(port_str) = std::env::var("SONGBIRD_HTTP_PORT") {
        return port_str.parse().unwrap_or(DEFAULT_HTTP_PORT);
    }

    // Try to extract from bind address
    if let Ok(addr) = std::env::var("SONGBIRD_HTTP_ADDR")
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
#[must_use]
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
/// When `false`, Songbird broadcasts legacy `BirdSongPacket` (version 1.0, plaintext `family_id`).
///
/// **Default**: `false` (opt-in for privacy, requires `BearDog` beacon.* RPC)
#[must_use]
pub fn dark_forest_enabled() -> bool {
    std::env::var("SONGBIRD_DARK_FOREST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
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
    std::env::var("SONGBIRD_ACCEPT_LEGACY_BIRDSONG")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(true)
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
    std::env::var("SONGBIRD_DUAL_BROADCAST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_socket_name_single_family() {
        // Default: single-family mode returns "songbird.sock"
        // (unless SONGBIRD_MULTI_FAMILY is set in the environment)
        let name = socket_name();
        assert!(name.ends_with(".sock"));
        // Either "songbird.sock" or "songbird-{family_id}.sock"
        assert!(name.starts_with("songbird"));
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
}
