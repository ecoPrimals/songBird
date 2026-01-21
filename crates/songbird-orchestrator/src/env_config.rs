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
    std::env::var("PRIMAL_NAME")
        .unwrap_or_else(|_| "songbird".to_string())
}

/// Get family/biome ID (self-knowledge)
pub fn family_id() -> String {
    std::env::var("SONGBIRD_FAMILY_ID")
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
/// Resolution order:
/// 1. `SONGBIRD_SOCKET` (explicit override)
/// 2. `/tmp/songbird-{family_id}.sock` (BiomeOS standard)
pub fn socket_path() -> PathBuf {
    if let Ok(path) = std::env::var("SONGBIRD_SOCKET") {
        return PathBuf::from(path);
    }

    // BiomeOS standard: /tmp/{primal}-{family}.sock
    let family = family_id();
    PathBuf::from(format!("/tmp/songbird-{}.sock", family))
}

/// Get data directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_DATA_DIR` (explicit override)
/// 2. `/tmp/songbird-data` (default)
pub fn data_dir() -> PathBuf {
    std::env::var("SONGBIRD_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/songbird-data"))
}

/// Get deployment directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_DEPLOY_DIR` (explicit override)
/// 2. `/tmp/songbird-deployments` (default)
pub fn deployment_dir() -> PathBuf {
    std::env::var("SONGBIRD_DEPLOY_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/songbird-deployments"))
}

/// Get cache directory (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_CACHE_DIR` (explicit override)
/// 2. `/tmp/songbird-cache` (default)
pub fn cache_dir() -> PathBuf {
    std::env::var("SONGBIRD_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/songbird-cache"))
}

/// Get HTTP server bind address (self-knowledge)
///
/// Resolution order:
/// 1. `SONGBIRD_HTTP_ADDR` (explicit override)
/// 2. `0.0.0.0:8080` (default - bind all interfaces)
pub fn http_bind_address() -> String {
    std::env::var("SONGBIRD_HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
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
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        std::env::remove_var("FAMILY_ID");
        assert_eq!(socket_path(), PathBuf::from("/tmp/songbird-nat0.sock"));
    }

    #[test]
    fn test_socket_path_custom_family() {
        std::env::set_var("SONGBIRD_FAMILY_ID", "prod");
        let path = socket_path();
        std::env::remove_var("SONGBIRD_FAMILY_ID");
        assert_eq!(path, PathBuf::from("/tmp/songbird-prod.sock"));
    }

    #[test]
    fn test_data_dir_default() {
        std::env::remove_var("SONGBIRD_DATA_DIR");
        assert_eq!(data_dir(), PathBuf::from("/tmp/songbird-data"));
    }

    #[test]
    fn test_http_port_default() {
        std::env::remove_var("SONGBIRD_HTTP_PORT");
        std::env::remove_var("SONGBIRD_HTTP_ADDR");
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

