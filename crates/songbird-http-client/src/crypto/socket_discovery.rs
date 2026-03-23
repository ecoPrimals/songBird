// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! XDG-Compliant Socket Discovery (Isomorphic - TRUE ecoBin v2.0)
//!
//! **Pure Rust | Zero Hardcoding | Runtime Discovery | Isomorphic Adaptation**
//!
//! Implements intelligent socket discovery with proper fallback chain:
//! 1. Environment variables (explicit configuration)
//! 2. Unix sockets via XDG Runtime Dir (`/run/user/$UID/biomeos/`)
//! 3. TCP endpoints via discovery files (isomorphic fallback)
//! 4. Legacy `/tmp` paths (last resort)
//!
//! ## Isomorphic IPC Support (v8.23.0+)
//!
//! When Unix sockets are unavailable (Android/SELinux, Windows), the server
//! automatically falls back to TCP localhost and writes a discovery file.
//! Clients automatically detect this and connect via TCP transparently.
//!
//! **Try → Detect → Adapt → Succeed**
//!
//! This enables automated Tower Atomic deployment via biomeOS Neural API
//! while maintaining backward compatibility with manual deployments.

use songbird_types::defaults::paths::{
    BEARDOG_SOCKET_LEGACY, BIOMEOS_RUNTIME_SUBDIR, IPC_DISCOVERY_TMP_DIR,
    NEURAL_API_SOCKET_LEGACY_PATTERN,
};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// IPC Endpoint type (isomorphic support)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcEndpoint {
    /// Unix domain socket (optimal)
    UnixSocket(String),
    /// TCP localhost (fallback for Android/Windows)
    TcpLocal(std::net::SocketAddr),
}

impl IpcEndpoint {
    /// Get display string for logging
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::UnixSocket(path) => format!("unix://{path}"),
            Self::TcpLocal(addr) => format!("tcp://{addr}"),
        }
    }
}

/// Discover IPC endpoint with full isomorphic fallback chain (v8.23.0+)
///
/// # Priority Order (Isomorphic Discovery)
///
/// 1. **Environment Variable** (highest priority)
///    - Direct specification by user/biomeOS
///    - Example: `SONGBIRD_SOCKET=/run/user/1000/biomeos/songbird-default.sock`
///
/// 2. **Unix Socket via XDG** (optimal, when available)
///    - Standard Unix location: `$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock`
///    - Example: `/run/user/1000/biomeos/songbird-default.sock`
///    - Only used if socket exists
///
/// 3. **TCP Endpoint via Discovery File** (isomorphic fallback)
///    - Server writes discovery file when Unix sockets unavailable
///    - Locations: `$XDG_RUNTIME_DIR/{primal}-ipc-port`, `$HOME/.local/share/`, `/tmp/`
///    - Format: `tcp:127.0.0.1:12345`
///    - Transparent fallback for Android/SELinux/Windows
///
/// 4. **Legacy /tmp Unix Socket** (fallback for development)
///    - Example: `/tmp/songbird.sock`
///    - Warning logged when used
///
/// # Arguments
///
/// * `env_var` - Environment variable name to check (e.g., "`SONGBIRD_SOCKET`")
/// * `primal_name` - Primal name for discovery (e.g., "songbird")
/// * `legacy_path` - Legacy `/tmp` path for backward compatibility
///
/// # Returns
///
/// IPC endpoint (Unix socket or TCP) to use for connection.
///
/// # Deep Debt Principles
///
/// - ✅ **Runtime Discovery**: Detects available transports automatically
/// - ✅ **Zero Configuration**: No platform-specific flags needed
/// - ✅ **Platform Agnostic**: Same discovery code for all platforms
/// - ✅ **Primal Autonomy**: Self-discovers optimal transport
#[must_use]
pub fn discover_ipc_endpoint(env_var: &str, primal_name: &str, legacy_path: &str) -> IpcEndpoint {
    discover_ipc_endpoint_with(env_var, primal_name, legacy_path, |key| std::env::var(key))
}

/// Injectable version for concurrent-safe testing
fn discover_ipc_endpoint_with<F>(
    env_var: &str,
    primal_name: &str,
    legacy_path: &str,
    env_reader: F,
) -> IpcEndpoint
where
    F: Fn(&str) -> Result<String, std::env::VarError>,
{
    debug!("🔍 IPC endpoint discovery for {}", primal_name);
    debug!("   Checking: 1) ${}", env_var);
    debug!("            2) Unix socket (XDG)");
    debug!("            3) TCP endpoint (discovery file)");
    debug!("            4) Legacy {}", legacy_path);

    // Priority 1: Environment variable (explicit configuration)
    if let Ok(socket) = env_reader(env_var)
        && !socket.is_empty()
    {
        info!("✅ IPC endpoint via ${}: {}", env_var, socket);
        return IpcEndpoint::UnixSocket(socket);
    }

    // Priority 2: Unix socket via XDG (optimal when available)
    if let Ok(xdg_dir) = env_reader("XDG_RUNTIME_DIR") {
        let family_id = env_reader("FAMILY_ID").unwrap_or_else(|_| String::new());
        let socket_name = if family_id.is_empty() {
            format!("{primal_name}.sock")
        } else {
            format!("{primal_name}-{family_id}.sock")
        };
        let socket_path = PathBuf::from(&xdg_dir).join(BIOMEOS_RUNTIME_SUBDIR).join(&socket_name);
        if socket_path.exists() {
            let path_str = socket_path.to_string_lossy().to_string();
            info!("✅ Unix socket via XDG: {}", path_str);
            return IpcEndpoint::UnixSocket(path_str);
        }
    }

    // Priority 3: TCP endpoint via discovery file (isomorphic fallback)
    if let Some(tcp_addr) = discover_tcp_endpoint(primal_name) {
        info!("✅ TCP endpoint via discovery file: {}", tcp_addr);
        info!("   (Server using isomorphic fallback mode)");
        return IpcEndpoint::TcpLocal(tcp_addr);
    }

    // Priority 4: Legacy /tmp path (development fallback)
    warn!("⚠️  Using legacy /tmp socket: {}", legacy_path);
    warn!("   Consider setting ${} or XDG_RUNTIME_DIR", env_var);
    warn!("   Example: {}=/run/user/$UID/biomeos/{}-$FAMILY_ID.sock", env_var, primal_name);

    IpcEndpoint::UnixSocket(legacy_path.to_string())
}

/// Discover TCP endpoint via discovery file (isomorphic fallback)
///
/// When the server cannot bind Unix sockets (Android/SELinux, Windows),
/// it automatically falls back to TCP localhost and writes a discovery file.
/// This function reads that file to find the TCP endpoint.
///
/// ## Discovery File Locations (XDG-Compliant Priority)
///
/// 1. `$XDG_RUNTIME_DIR/{primal}-ipc-port` (preferred, user-specific)
/// 2. `$HOME/.local/share/{primal}-ipc-port` (fallback, persistent)
/// 3. `/tmp/{primal}-ipc-port` (last resort, system-wide)
///
/// ## File Format
///
/// ```text
/// tcp:127.0.0.1:12345
/// ```
///
/// # Arguments
///
/// * `primal_name` - Primal name (e.g., "songbird", "beardog")
///
/// # Returns
///
/// TCP socket address if discovery file found and parsed, None otherwise.
///
/// # Deep Debt Principles
///
/// - ✅ **Runtime Discovery**: Reads server-written discovery file
/// - ✅ **XDG Compliant**: Follows XDG Base Directory specification
/// - ✅ **Zero Hardcoding**: No hardcoded ports or addresses
/// - ✅ **Platform Agnostic**: Works on any platform with filesystem
fn discover_tcp_endpoint(primal_name: &str) -> Option<std::net::SocketAddr> {
    debug!("   Checking TCP discovery files for {}", primal_name);

    // Discovery file candidates (XDG priority order)
    let candidates = get_tcp_discovery_file_candidates(primal_name);

    for path in candidates {
        debug!("      Trying: {}", path.display());

        if let Ok(content) = std::fs::read_to_string(&path) {
            // Parse format: "tcp:127.0.0.1:12345"
            if let Some(addr_str) = content.strip_prefix("tcp:") {
                if let Ok(addr) = addr_str.trim().parse::<std::net::SocketAddr>() {
                    debug!("      ✅ Found TCP endpoint: {}", addr);
                    return Some(addr);
                }
                warn!("      ⚠️  Invalid TCP address in {}: {}", path.display(), addr_str);
            } else {
                warn!("      ⚠️  Invalid format in {}: {}", path.display(), content.trim());
            }
        }
    }

    debug!("   ❌ No TCP discovery file found");
    None
}

/// Get TCP discovery file candidates in XDG priority order
fn get_tcp_discovery_file_candidates(primal_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let filename = format!("{primal_name}-ipc-port");

    // Priority 1: XDG_RUNTIME_DIR (preferred, user-specific, volatile)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        candidates.push(PathBuf::from(runtime_dir).join(&filename));
    }

    // Priority 2: HOME/.local/share (persistent, user-specific)
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(PathBuf::from(home).join(".local/share").join(&filename));
    }

    // Priority 3: /tmp (last resort, system-wide)
    candidates.push(PathBuf::from(IPC_DISCOVERY_TMP_DIR).join(&filename));

    candidates
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
///     ├── crypto-default.sock
///     ├── songbird-default.sock
///     ├── neural-api-default.sock
///     └── storage-default.sock
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
            // Try common family IDs (canonical default is "default")
            for family in &["default"] {
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
    let socket_path = PathBuf::from(runtime_dir)
        .join(BIOMEOS_RUNTIME_SUBDIR)
        .join(format!("{primal_name}-{family_id}.sock"))
        .to_string_lossy()
        .into_owned();

    debug!("   Checking XDG: {}", socket_path);

    if Path::new(&socket_path).exists() {
        debug!("   ✅ Found XDG socket");
        Some(socket_path)
    } else {
        debug!("   ❌ XDG socket not found");
        None
    }
}

/// Discover socket path with XDG-compliant fallback chain (legacy function)
///
/// **Note**: For new code, use `discover_ipc_endpoint()` which supports
/// isomorphic TCP fallback. This function is kept for backward compatibility.
///
/// # Priority Order
///
/// 1. **Environment Variable** (highest priority)
/// 2. **XDG Runtime Directory** (recommended for production)
/// 3. **Legacy /tmp Path** (fallback for development/testing)
///
/// # Arguments
///
/// * `env_var` - Environment variable name to check (e.g., "`BEARDOG_SOCKET`")
/// * `primal_name` - Primal name for XDG discovery (e.g., "beardog")
/// * `legacy_path` - Legacy `/tmp` path for backward compatibility
///
/// # Returns
///
/// Socket path to use, guaranteed to exist or be the specified fallback.
pub fn discover_socket(env_var: &str, primal_name: &str, legacy_path: &str) -> String {
    // Use new isomorphic discovery and extract Unix socket path
    match discover_ipc_endpoint(env_var, primal_name, legacy_path) {
        IpcEndpoint::UnixSocket(path) => path,
        IpcEndpoint::TcpLocal(_) => {
            // Legacy function doesn't support TCP, fall back to legacy path
            warn!("⚠️  TCP endpoint discovered but legacy discover_socket() doesn't support TCP");
            warn!("   Consider updating to discover_ipc_endpoint() for isomorphic support");
            legacy_path.to_string()
        }
    }
}
///
/// Checks in order:
/// 1. `$BEARDOG_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/beardog-$FAMILY_ID.sock`
/// 3. `/tmp/beardog.sock` (legacy)
#[must_use]
pub fn discover_beardog_socket() -> String {
    discover_socket("BEARDOG_SOCKET", "beardog", BEARDOG_SOCKET_LEGACY)
}

/// Discover Neural API socket with full fallback chain
///
/// Checks in order:
/// 1. `$NEURAL_API_SOCKET` or `$NEURALS_SOCKET`
/// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api-$FAMILY_ID.sock`
/// 3. `/tmp/neural-api-{family_id}.sock` (legacy fallback with env-derived family)
pub fn discover_neural_api_socket() -> String {
    // Check both NEURAL_API_SOCKET and NEURALS_SOCKET
    if let Ok(socket) = std::env::var("NEURAL_API_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $NEURAL_API_SOCKET: {}", socket);
        return socket;
    }

    if let Ok(socket) = std::env::var("NEURALS_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $NEURALS_SOCKET: {}", socket);
        return socket;
    }

    // Try XDG discovery
    if let Some(xdg_socket) = discover_xdg_socket("neural-api") {
        return xdg_socket;
    }

    // Legacy fallback — use env-derived family ID (canonical chain)
    let family_id = std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());
    let socket = format!("{NEURAL_API_SOCKET_LEGACY_PATTERN}{family_id}.sock");
    warn!("⚠️  Using legacy /tmp socket: {}", socket);
    warn!("   Consider setting $NEURAL_API_SOCKET or XDG_RUNTIME_DIR");
    socket
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Create a mock env reader from a `HashMap` (concurrent-safe, no global state)
    fn mock_env(vars: HashMap<&str, &str>) -> impl Fn(&str) -> Result<String, std::env::VarError> {
        let owned: HashMap<String, String> =
            vars.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move |key: &str| owned.get(key).cloned().ok_or(std::env::VarError::NotPresent)
    }

    #[test]
    fn test_env_var_priority() {
        // Explicit env var takes highest priority
        let env = mock_env(HashMap::from([("TEST_SOCKET", "/custom/path.sock")]));
        let endpoint =
            discover_ipc_endpoint_with("TEST_SOCKET", "test-primal", "/tmp/fallback.sock", env);
        assert_eq!(endpoint, IpcEndpoint::UnixSocket("/custom/path.sock".to_string()));
    }

    #[test]
    fn test_legacy_fallback() {
        // No env vars set -> legacy fallback
        let env = mock_env(HashMap::new());
        let endpoint =
            discover_ipc_endpoint_with("TEST_SOCKET", "test-primal", "/tmp/fallback.sock", env);
        assert_eq!(endpoint, IpcEndpoint::UnixSocket("/tmp/fallback.sock".to_string()));
    }

    #[test]
    fn test_xdg_path_construction() {
        // XDG_RUNTIME_DIR with FAMILY_ID - socket won't exist in test,
        // so it falls through to legacy (verifying the logic path)
        let env =
            mock_env(HashMap::from([("XDG_RUNTIME_DIR", "/run/user/1000"), ("FAMILY_ID", "nat0")]));
        let endpoint =
            discover_ipc_endpoint_with("TEST_SOCKET", "test-primal", "/tmp/fallback.sock", env);
        // Socket file doesn't exist, so falls through to legacy
        assert_eq!(endpoint, IpcEndpoint::UnixSocket("/tmp/fallback.sock".to_string()));
    }

    #[test]
    fn test_empty_env_var_ignored() {
        // Empty env var should be ignored, fall back to legacy
        let env = mock_env(HashMap::from([("TEST_SOCKET", "")]));
        let endpoint =
            discover_ipc_endpoint_with("TEST_SOCKET", "test-primal", "/tmp/fallback.sock", env);
        assert_eq!(endpoint, IpcEndpoint::UnixSocket("/tmp/fallback.sock".to_string()));
    }

    #[test]
    fn test_ipc_endpoint_unix() {
        let endpoint = IpcEndpoint::UnixSocket("/tmp/test.sock".to_string());
        assert_eq!(endpoint.display(), "unix:///tmp/test.sock");
    }

    #[test]
    fn test_ipc_endpoint_tcp() {
        use std::net::{IpAddr, Ipv4Addr};
        let addr = std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345);
        let endpoint = IpcEndpoint::TcpLocal(addr);
        assert_eq!(endpoint.display(), "tcp://127.0.0.1:12345");
    }

    #[test]
    fn test_tcp_discovery_file_candidates() {
        // Test with actual env (candidates always include /tmp fallback)
        let candidates = get_tcp_discovery_file_candidates("songbird");
        // Should always have at least the /tmp fallback
        assert!(!candidates.is_empty());
        assert!(candidates.last().unwrap() == &PathBuf::from("/tmp/songbird-ipc-port"));
    }

    #[test]
    fn test_isomorphic_discovery_priority() {
        // No env vars set -> should fall back to legacy Unix socket
        let env = mock_env(HashMap::new());
        let endpoint =
            discover_ipc_endpoint_with("TEST_SOCKET", "test-primal", "/tmp/fallback.sock", env);

        match endpoint {
            IpcEndpoint::UnixSocket(path) => assert_eq!(path, "/tmp/fallback.sock"),
            IpcEndpoint::TcpLocal(_) => panic!("Should not discover TCP without discovery file"),
        }
    }
}
