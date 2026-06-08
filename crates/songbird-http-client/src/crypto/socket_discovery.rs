// SPDX-License-Identifier: AGPL-3.0-or-later
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

use songbird_types::defaults::paths::{BIOMEOS_RUNTIME_SUBDIR, ipc_discovery_primal_port_path};
use songbird_types::primal_names::NEURAL_API;
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
    discover_ipc_endpoint_with(env_var, primal_name, legacy_path, |key| {
        songbird_process_env::var(key)
    })
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
        if let Some(addr_str) = socket.strip_prefix("tcp://")
            && let Ok(addr) = addr_str.parse::<std::net::SocketAddr>()
        {
            info!("   Resolved as TCP endpoint: {}", addr);
            return IpcEndpoint::TcpLocal(addr);
        }
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
/// * `primal_name` - Primal name (e.g., "songbird", "other-primal")
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
    if let Ok(runtime_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        candidates.push(PathBuf::from(runtime_dir).join(&filename));
    }

    // Priority 2: HOME/.local/share (persistent, user-specific)
    if let Ok(home) = songbird_process_env::var("HOME") {
        candidates.push(PathBuf::from(home).join(".local/share").join(&filename));
    }

    // Priority 3: OS temp dir (last resort, system-wide)
    candidates.push(ipc_discovery_primal_port_path(primal_name));

    candidates
}

/// Discover socket in XDG Runtime Directory
///
/// Checks `$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock`
///
/// # Arguments
///
/// * `primal_name` - Primal name (e.g., "other-primal", "songbird")
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
    let runtime_dir = match songbird_process_env::var("XDG_RUNTIME_DIR") {
        Ok(dir) if !dir.is_empty() => dir,
        _ => {
            debug!("   XDG_RUNTIME_DIR not set");
            return None;
        }
    };

    // Get FAMILY_ID (ecoPrimals family identifier)
    let family_id = match songbird_process_env::var("FAMILY_ID") {
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
/// * `env_var` - Environment variable name to check (e.g., "`SECURITY_PROVIDER_SOCKET`")
/// * `primal_name` - Primal name for XDG discovery (e.g., "security")
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
/// Discover security provider socket via capability-based discovery.
///
/// Priority (wateringHole v1.2, aligned with `songbird-crypto-provider`):
/// 1. `$SECURITY_PROVIDER_SOCKET` (capability-standard)
/// 2. `$SECURITY_SOCKET` (capability domain)
/// 3. `$CRYPTO_PROVIDER_SOCKET` (alternate capability name)
/// 4. `$XDG_RUNTIME_DIR/biomeos/security.sock` (capability symlink)
/// 5. `$XDG_RUNTIME_DIR/biomeos/security-{family_id}.sock` (family-scoped)
/// 6. `$XDG_RUNTIME_DIR/biomeos/crypto-{family_id}.sock` (domain socket)
/// 7. `$XDG_RUNTIME_DIR/biomeos/beardog-{family_id}.sock` (legacy on-disk)
/// 8. `$BEARDOG_SOCKET` (legacy env — logged as deprecated)
/// 9. `/var/run/biomeos/security.sock` (VPS fallback — DH-1 compliant)
#[must_use]
pub fn discover_security_socket() -> String {
    if let Ok(socket) = songbird_process_env::var("SECURITY_PROVIDER_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Security provider via $SECURITY_PROVIDER_SOCKET: {socket}");
        return socket;
    }

    if let Ok(socket) = songbird_process_env::var("SECURITY_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Security provider via $SECURITY_SOCKET: {socket}");
        return socket;
    }

    if let Ok(socket) = songbird_process_env::var("CRYPTO_PROVIDER_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Security provider via $CRYPTO_PROVIDER_SOCKET: {socket}");
        return socket;
    }

    if let Ok(xdg_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        let biomeos = PathBuf::from(&xdg_dir).join(BIOMEOS_RUNTIME_SUBDIR);

        let cap_path = biomeos.join("security.sock");
        if cap_path.exists() {
            let path = cap_path.to_string_lossy().to_string();
            info!("✅ Security provider via capability symlink: {path}");
            return path;
        }

        let family_id = songbird_process_env::var("FAMILY_ID").unwrap_or_default();

        if !family_id.is_empty() {
            let family_security = biomeos.join(format!("security-{family_id}.sock"));
            if family_security.exists() {
                let path = family_security.to_string_lossy().to_string();
                info!("✅ Security provider via family-scoped security socket: {path}");
                return path;
            }
        }

        let crypto_name = if family_id.is_empty() {
            "crypto.sock".to_string()
        } else {
            format!("crypto-{family_id}.sock")
        };
        let crypto_path = biomeos.join(&crypto_name);
        if crypto_path.exists() {
            let path = crypto_path.to_string_lossy().to_string();
            info!("✅ Security provider via crypto domain socket: {path}");
            return path;
        }

        if !family_id.is_empty() {
            let legacy_beardog = biomeos.join(format!("beardog-{family_id}.sock"));
            if legacy_beardog.exists() {
                let path = legacy_beardog.to_string_lossy().to_string();
                warn!(
                    "Security provider via legacy on-disk socket: {path} — migrate to security-{{family}}.sock"
                );
                return path;
            }
        }
    }

    if let Ok(socket) = songbird_process_env::var("BEARDOG_SOCKET")
        && !socket.is_empty()
    {
        warn!(
            "DEPRECATED: BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT (capability-first)"
        );
        return socket;
    }

    // VPS fallback (DH-1 compliant — no /tmp writes)
    let fallback = "/var/run/biomeos/security.sock";
    warn!(
        "VPS fallback: {} — set SECURITY_PROVIDER_SOCKET, BEARDOG_SOCKET, or XDG_RUNTIME_DIR",
        fallback
    );
    fallback.to_string()
}

/// Deprecated alias for [`discover_security_socket`].
#[deprecated(
    since = "0.3.0",
    note = "Use discover_security_socket; prefer CAPABILITY_* or SECURITY_PROVIDER_* env vars (capability-first)"
)]
#[must_use]
pub fn discover_security_provider_socket() -> String {
    discover_security_socket()
}

/// Discover Neural API socket with full fallback chain (DH-1 compliant)
///
/// Checks in order:
/// 1. `$NEURAL_API_SOCKET` or `$NEURALS_SOCKET`
/// 2. `$SECURITY_PROVIDER_SOCKET` (capability-first naming)
/// 3. `$SECURITY_PROVIDER_ENDPOINT` (set by `--security-socket` CLI flag)
/// 4. `$BEARDOG_SOCKET` (backward-compatible — standard on southGate)
/// 5. `$XDG_RUNTIME_DIR/biomeos/neural-api-$FAMILY_ID.sock`
/// 6. `/var/run/biomeos/neural-api.sock` (VPS fallback — no `/tmp` writes)
pub fn discover_neural_api_socket() -> String {
    if let Ok(socket) = songbird_process_env::var("NEURAL_API_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $NEURAL_API_SOCKET: {}", socket);
        return socket;
    }

    if let Ok(socket) = songbird_process_env::var("NEURALS_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $NEURALS_SOCKET: {}", socket);
        return socket;
    }

    if let Ok(socket) = songbird_process_env::var("SECURITY_PROVIDER_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $SECURITY_PROVIDER_SOCKET: {}", socket);
        return socket;
    }

    if let Ok(socket) = songbird_process_env::var("SECURITY_PROVIDER_ENDPOINT")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $SECURITY_PROVIDER_ENDPOINT: {}", socket);
        return socket;
    }

    if let Ok(socket) = songbird_process_env::var("BEARDOG_SOCKET")
        && !socket.is_empty()
    {
        info!("✅ Socket discovered via $BEARDOG_SOCKET: {}", socket);
        return socket;
    }

    // Try XDG discovery
    if let Some(xdg_socket) = discover_xdg_socket(NEURAL_API) {
        return xdg_socket;
    }

    // VPS fallback (DH-1 compliant — no /tmp writes)
    let fallback = "/var/run/biomeos/neural-api.sock".to_string();
    warn!("⚠️  Using VPS fallback Neural API socket: {}", fallback);
    warn!("   Consider setting $NEURAL_API_SOCKET, $BEARDOG_SOCKET, or XDG_RUNTIME_DIR");
    fallback
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

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

    #[test]
    fn discover_ipc_endpoint_tcp_from_tmp_discovery_file() {
        let primal = "songbird_sockdisc_unit";
        let path = PathBuf::from("/tmp").join(format!("{primal}-ipc-port"));
        std::fs::write(&path, b"tcp:127.0.0.1:12345").unwrap();
        let env = mock_env(HashMap::new());
        let endpoint = discover_ipc_endpoint_with("NO_SUCH_ENV", primal, "/tmp/fallback.sock", env);
        let _ = std::fs::remove_file(&path);
        match endpoint {
            IpcEndpoint::TcpLocal(addr) => {
                assert_eq!(addr.ip().to_string(), "127.0.0.1");
                assert_eq!(addr.port(), 12345);
            }
            IpcEndpoint::UnixSocket(p) => panic!("expected tcp discovery, got unix {p}"),
        }
    }

    #[test]
    fn discover_ipc_endpoint_invalid_tcp_file_falls_back_to_legacy() {
        let primal = "songbird_sockdisc_badfmt";
        let path = PathBuf::from("/tmp").join(format!("{primal}-ipc-port"));
        std::fs::write(&path, b"not-tcp-format").unwrap();
        let env = mock_env(HashMap::new());
        let endpoint = discover_ipc_endpoint_with("NO_SUCH_ENV", primal, "/tmp/fallback.sock", env);
        let _ = std::fs::remove_file(&path);
        assert_eq!(endpoint, IpcEndpoint::UnixSocket("/tmp/fallback.sock".to_string()));
    }

    #[test]
    fn ipc_endpoint_unix_display_escapes_nothing_special() {
        let ep = IpcEndpoint::UnixSocket("/path/with space.sock".to_string());
        assert!(ep.display().contains("unix://"));
    }

    #[test]
    fn tcp_discovery_candidates_order_includes_tmp_last() {
        let c = get_tcp_discovery_file_candidates("z");
        assert!(c.last().is_some_and(|p| p.ends_with("z-ipc-port")));
    }

    #[test]
    fn xdg_runtime_set_prefers_unix_when_socket_exists() {
        let dir = tempfile::tempdir().unwrap();
        let biome = dir.path().join("biomeos");
        std::fs::create_dir_all(&biome).unwrap();
        let sock = biome.join("p.sock");
        std::fs::write(&sock, b"").unwrap();
        let xdg = dir.path().to_string_lossy().to_string();
        let sock_path = sock.to_string_lossy().to_string();
        let env = mock_env(HashMap::from([("XDG_RUNTIME_DIR", xdg.as_str()), ("FAMILY_ID", "")]));
        // No env var for "UNUSED_SOCKET" — XDG path with existing `p.sock` wins.
        let endpoint = discover_ipc_endpoint_with("UNUSED_SOCKET", "p", "/tmp/legacy.sock", env);
        assert_eq!(endpoint, IpcEndpoint::UnixSocket(sock_path));
    }
}
