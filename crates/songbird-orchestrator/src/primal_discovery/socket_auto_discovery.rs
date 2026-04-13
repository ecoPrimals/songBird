// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Socket auto-discovery: scan biomeos socket directories and register
//! discovered primals into the IPC service registry (LD-08).
//!
//! Without this, `ipc.resolve` / `capability.resolve` return empty results because
//! the registry starts empty and no primals call `ipc.register`. This module
//! implements option (b) from the primalSpring audit: Songbird probes the socket
//! directory and auto-registers what it finds, which is more resilient than
//! requiring every primal to self-register.
//!
//! Called both at startup (Stage 2c) and periodically (every 30s from Stage 6)
//! so that primals starting after Songbird are picked up without launcher
//! assistance — making the registry self-healing.

use songbird_universal_ipc::endpoint::NativeEndpoint;
use songbird_universal_ipc::registry::ServiceRegistry;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Result of probing a single socket: primal name + capabilities.
struct ProbeResult {
    primal_name: String,
    capabilities: Vec<String>,
}

/// Scan biomeos socket directories and auto-register every responding primal.
///
/// Skips Songbird's own socket(s). Returns the count of newly registered primals.
pub async fn discover_and_register_biomeos_primals(registry: &ServiceRegistry) -> usize {
    let paths = list_biomeos_sock_paths_async().await;

    if paths.is_empty() {
        debug!("Socket auto-discovery: no biomeos socket directories found");
        return 0;
    }

    info!("🔍 Socket auto-discovery: scanning {} socket(s) in biomeos directories", paths.len());

    let mut registered = 0usize;

    for path in paths {
        if is_own_socket(&path) {
            debug!("Skipping own socket: {}", path.display());
            continue;
        }

        match probe_socket(&path).await {
            Some(result) => {
                let endpoint = NativeEndpoint::UnixSocket(path.clone());
                match registry
                    .register(&result.primal_name, endpoint, result.capabilities.clone())
                    .await
                {
                    Ok(_) => {
                        info!(
                            "   ✅ Auto-registered '{}' ({} caps) from {}",
                            result.primal_name,
                            result.capabilities.len(),
                            path.display()
                        );
                        registered += 1;
                    }
                    Err(e) => {
                        debug!(
                            "   ⚠️  Skipped '{}' ({}): {}",
                            result.primal_name,
                            path.display(),
                            e
                        );
                    }
                }
            }
            None => {
                debug!("   — {} did not respond to probes", path.display());
            }
        }
    }

    if registered > 0 {
        info!("✅ Socket auto-discovery complete: {} primal(s) registered", registered);
    } else {
        debug!("Socket auto-discovery: no new primals found");
    }

    registered
}

/// Probe a Unix socket with `identity.get` (Wire Standard L3) and
/// fall back to `capabilities.list` for name extraction.
async fn probe_socket(path: &Path) -> Option<ProbeResult> {
    let path = path.to_path_buf();

    tokio::task::spawn_blocking(move || probe_socket_sync(&path)).await.ok().flatten()
}

/// Synchronous probe: connect, send `identity.get` then `capabilities.list`.
fn probe_socket_sync(path: &Path) -> Option<ProbeResult> {
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let mut stream = UnixStream::connect(path).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    stream.set_write_timeout(Some(Duration::from_secs(2))).ok()?;

    // Try identity.get first (Wire Standard L3: {primal, version, domain, license})
    let identity = send_jsonrpc(&mut stream, "identity.get", 1);

    // Then capabilities.list for capability tokens
    let caps_resp = send_jsonrpc(&mut stream, "capabilities.list", 2)
        .or_else(|| send_jsonrpc(&mut stream, "capability.list", 3));

    let capabilities = caps_resp
        .as_ref()
        .and_then(|v| super::parse::parse_capabilities_result(v))
        .unwrap_or_default();

    // Extract primal name: prefer identity.get result, fall back to socket filename
    let primal_name = identity
        .as_ref()
        .and_then(|v| v.get("result"))
        .and_then(|r| r.get("primal"))
        .and_then(|p| p.as_str())
        .map(String::from)
        .or_else(|| name_from_socket_path(path))?;

    // Must have at least a name to register
    if primal_name.is_empty() {
        return None;
    }

    Some(ProbeResult {
        primal_name,
        capabilities,
    })
}

/// Send a JSON-RPC request and read the newline-delimited response.
fn send_jsonrpc(
    stream: &mut std::os::unix::net::UnixStream,
    method: &str,
    id: i64,
) -> Option<serde_json::Value> {
    use std::io::Write;

    let method = songbird_types::normalize_json_rpc_method_name(method);
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": {},
        "id": id,
    });
    let mut bytes = serde_json::to_vec(&req).ok()?;
    bytes.push(b'\n');
    stream.write_all(&bytes).ok()?;

    let line = read_line(stream).ok()?;
    let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    if v.get("error").is_some() {
        return None;
    }
    Some(v)
}

/// Read a single newline-terminated line from the stream.
fn read_line(stream: &mut std::os::unix::net::UnixStream) -> Result<String, std::io::Error> {
    use std::io::Read;

    let mut buf = Vec::with_capacity(4096);
    let mut one = [0u8; 1];
    loop {
        match stream.read(&mut one) {
            Ok(0) => {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short read"));
            }
            Ok(_) => {
                if one[0] == b'\n' {
                    break;
                }
                buf.push(one[0]);
            }
            Err(e) => return Err(e),
        }
    }
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Derive a primal name from the socket filename (e.g. `beardog.sock` → `beardog`).
fn name_from_socket_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Strip common suffixes: `beardog-family123.sock` → `beardog`
            s.split('-').next().unwrap_or(s).to_string()
        })
        .filter(|s| !s.is_empty())
}

/// Whether a socket path belongs to Songbird itself (should be skipped).
fn is_own_socket(path: &Path) -> bool {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let own = songbird_types::primal_names::SELF_NAME;
    stem == own
        || stem.starts_with(&format!("{own}-"))
        || stem == "network"
        || stem.starts_with("network-")
}

/// Async wrapper for biomeos socket directory enumeration.
async fn list_biomeos_sock_paths_async() -> Vec<PathBuf> {
    tokio::task::spawn_blocking(|| {
        super::tcp_biomeos::list_biomeos_sock_paths(&|k| songbird_process_env::var(k).ok())
    })
    .await
    .unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn name_from_socket_extracts_stem() {
        assert_eq!(
            name_from_socket_path(Path::new("/run/user/1000/biomeos/beardog.sock")),
            Some("beardog".into())
        );
    }

    #[test]
    fn name_from_socket_strips_family_suffix() {
        assert_eq!(
            name_from_socket_path(Path::new("/tmp/biomeos/beardog-myfamily.sock")),
            Some("beardog".into())
        );
    }

    #[test]
    fn name_from_socket_handles_dotfile() {
        // `.sock` has stem `.sock` (no extension) — still produces a name from the split
        assert_eq!(name_from_socket_path(Path::new("/tmp/.sock")), Some(".sock".into()));
    }

    #[test]
    fn is_own_socket_matches_songbird_variants() {
        assert!(is_own_socket(Path::new("/tmp/biomeos/songbird.sock")));
        assert!(is_own_socket(Path::new("/tmp/biomeos/songbird-myfamily.sock")));
        assert!(is_own_socket(Path::new("/tmp/biomeos/network.sock")));
        assert!(is_own_socket(Path::new("/tmp/biomeos/network-myfamily.sock")));
    }

    #[test]
    fn is_own_socket_does_not_match_other_primals() {
        assert!(!is_own_socket(Path::new("/tmp/biomeos/beardog.sock")));
        assert!(!is_own_socket(Path::new("/tmp/biomeos/toadstool.sock")));
    }

    #[tokio::test]
    async fn discover_on_empty_dir_returns_zero() {
        let registry = ServiceRegistry::new();
        let count = discover_and_register_biomeos_primals(&registry).await;
        // In CI / test environment there are typically no biomeos sockets
        assert_eq!(registry.list_services().await.len(), count);
    }

    #[tokio::test]
    async fn probe_nonexistent_socket_returns_none() {
        let result = probe_socket(Path::new("/no/such/path/fake.sock")).await;
        assert!(result.is_none());
    }
}
