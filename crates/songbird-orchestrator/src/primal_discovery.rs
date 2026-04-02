// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Agnostic Primal Discovery — wateringHole / TRUE PRIMAL
//!
//! Discovers primals by **capability at runtime** via JSON-RPC probes on Unix
//! sockets under `$XDG_RUNTIME_DIR/biomeos/`, with **no** filename- or
//! primal-name-based classification.
//!
//! ## Principles
//!
//! 1. **Self-Knowledge Only**: Songbird knows only itself (see [`get_primal_name`])
//! 2. **Capability-Based**: Classify by `capabilities.list` tokens, not paths
//! 3. **Runtime Discovery**: No compile-time dependencies on other primals
//! 4. **Graceful Degradation**: Features work without optional primals
//!
//! ## Discovery Strategy
//!
//! ```text
//! 1. Environment variables (explicit overrides, preferred)
//!    - Primary: {CAPABILITY}_PROVIDER_SOCKET
//!    - Compatibility: BEARDOG_SOCKET, NESTGATE_SOCKET, etc.
//!
//! 2. TCP discovery files (capability-named, isomorphic fallback)
//!
//! 3. BiomeOS socket scan + probe (no filename classification)
//!    - Enumerate `*.sock` in `$XDG_RUNTIME_DIR/biomeos/`
//!    - For each socket: `health.liveness` then `capabilities.list`
//!    - First socket whose capability list matches the requested role wins
//! ```

use anyhow::{Result, anyhow};
use songbird_types::primal_names;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Capability types for primal discovery (functional roles, not primal names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Cryptographic operations (signing, encryption, hashing)
    Crypto,
    /// Security operations (JWT, auth, trust evaluation)
    Security,
    /// HTTP/HTTPS requests (external API delegation)
    Http,
    /// AI operations (LLM inference, routing)
    Ai,
    /// Storage operations (key-value, blob)
    Storage,
    /// Messaging operations (pub/sub, queues)
    Messaging,
}

impl Capability {
    /// Get environment variable name for this capability
    const fn env_var_name(&self) -> &'static str {
        match self {
            Self::Crypto => "CRYPTO_PROVIDER_SOCKET",
            Self::Security => "SECURITY_PROVIDER_SOCKET",
            Self::Http => "HTTP_PROVIDER_SOCKET",
            Self::Ai => "AI_PROVIDER_SOCKET",
            Self::Storage => "STORAGE_PROVIDER_SOCKET",
            Self::Messaging => "MESSAGING_PROVIDER_SOCKET",
        }
    }

    /// Alternative environment variable names (backward compatibility).
    fn alt_env_vars(&self) -> Vec<&'static str> {
        match self {
            Self::Crypto => vec!["SECURITY_PROVIDER_SOCKET", "BEARDOG_CRYPTO_SOCKET", "BEARDOG_SOCKET"],
            Self::Security => vec!["SECURITY_PROVIDER_SOCKET", "SONGBIRD_SECURITY_PROVIDER", "BEARDOG_SOCKET"],
            Self::Http => vec!["HTTP_CLIENT_SOCKET", "SONGBIRD_SOCKET"],
            Self::Ai => vec!["SQUIRREL_SOCKET", "AI_PROVIDER_SOCKETS"],
            Self::Storage => vec!["NESTGATE_SOCKET", "STORAGE_SOCKET"],
            Self::Messaging => vec!["MESSENGER_SOCKET", "PUBSUB_SOCKET"],
        }
    }

    /// Returns true if a flat `capabilities.list` response satisfies this role.
    #[must_use]
    pub fn matches_capability_tokens(&self, tokens: &[String]) -> bool {
        let lowered: Vec<String> = tokens.iter().map(|t| t.to_ascii_lowercase()).collect();
        match self {
            Self::Crypto => lowered.iter().any(|t| {
                t.contains("crypto.delegate")
                    || t.starts_with("crypto.")
                    || t == "crypto"
                    || t.contains("encryption")
            }),
            Self::Security => lowered.iter().any(|t| {
                t.starts_with("security.")
                    || t.contains("jwt")
                    || t.contains("btsp.")
                    || t == "security"
            }),
            Self::Http => lowered.iter().any(|t| {
                t == "http.request" || t == "http.get" || t == "http.post" || t.starts_with("http.")
            }),
            Self::Ai => lowered.iter().any(|t| {
                t.starts_with("ai.")
                    || t.contains("llm")
                    || t.contains("mcp")
                    || t.contains("inference")
                    || t.contains("model")
            }),
            Self::Storage => {
                lowered.iter().any(|t| t.starts_with("storage.") || t.contains("persist"))
            }
            Self::Messaging => lowered.iter().any(|t| {
                t.contains("messaging")
                    || t.contains("pubsub")
                    || t.contains("queue")
                    || t.starts_with("message.")
            }),
        }
    }
}

/// `sovereign-storage`: storage role plus an explicit sovereign / edge token.
#[must_use]
pub fn matches_sovereign_storage_tokens(tokens: &[String]) -> bool {
    Capability::Storage.matches_capability_tokens(tokens)
        && tokens.iter().any(|t| t.to_ascii_lowercase().contains("sovereign"))
}

/// Map doctor / CLI capability keys to [`Capability`] (excludes sovereign-storage).
pub fn capability_from_wire_id(id: &str) -> Result<Capability> {
    match id {
        "crypto" => Ok(Capability::Crypto),
        "ai" => Ok(Capability::Ai),
        "storage" => Ok(Capability::Storage),
        "messaging" => Ok(Capability::Messaging),
        "http" => Ok(Capability::Http),
        "security" => Ok(Capability::Security),
        other => Err(anyhow!("Unknown capability id for discovery: {other}")),
    }
}

/// Discover a primal by capability (functional, no state)
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover(capability: Capability) -> Result<String> {
    discover_with(capability, |name| songbird_process_env::var(name).ok()).await
}

/// Discover a primal by capability with injectable env reader (concurrent-safe, testable)
/// # Errors
///
/// Returns an error if the operation fails.
#[expect(
    clippy::unused_async,
    reason = "async signature required by Axum, trait objects, or future I/O"
)]
pub async fn discover_with<F>(capability: Capability, env_reader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    info!("🔍 Discovering {:?} provider (capability-based discovery)...", capability);

    // Strategy 1: Environment variable (orchestrator-provided, preferred)
    if let Some(socket_path) = env_reader(capability.env_var_name()) {
        info!("   ✅ Found via {}: {}", capability.env_var_name(), socket_path);
        return Ok(socket_path);
    }

    // Strategy 2: Alternative environment variables (compatibility)
    for alt_var in capability.alt_env_vars() {
        if let Some(socket_path) = env_reader(alt_var) {
            info!("   ✅ Found via {} (compatibility): {}", alt_var, socket_path);
            return Ok(socket_path);
        }
    }

    // Strategy 3: TCP discovery files (isomorphic fallback; uses same `env_reader` as biomeos scan)
    if let Some(tcp_endpoint) = discover_tcp_from_capability(capability, &env_reader) {
        info!("   ✅ Found {:?} provider via TCP discovery file: {}", capability, tcp_endpoint);
        return Ok(tcp_endpoint);
    }

    // Strategy 4: BiomeOS `*.sock` scan + JSON-RPC capability probe (no filename classification)
    if let Some(socket_path) = discover_via_biomeos_probe(capability, &env_reader) {
        info!("   ✅ Found {:?} provider via biomeos probe: {}", capability, socket_path);
        return Ok(socket_path);
    }

    warn!("❌ No {:?} provider found - checked all discovery strategies", capability);
    anyhow::bail!("No {capability:?} provider available")
}

/// Discover by string id (e.g. doctor): handles `sovereign-storage` specially.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_for_capability_id_with<F>(id: &str, env_reader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    if id == "sovereign-storage" {
        return discover_sovereign_storage_with(env_reader).await;
    }
    let cap = capability_from_wire_id(id)?;
    discover_with(cap, env_reader).await
}

/// Discover sovereign storage: env overrides, then biomeos probe with [`matches_sovereign_storage_tokens`].
pub async fn discover_sovereign_storage_with<F>(env_reader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(p) = env_reader("SONGBIRD_SOVEREIGN_STORAGE_PROVIDER_SOCKET") {
        return Ok(p);
    }
    if let Some(p) = env_reader("STORAGE_PROVIDER_SOCKET") {
        return Ok(p);
    }
    for alt in ["NESTGATE_SOCKET", "STORAGE_SOCKET"] {
        if let Some(p) = env_reader(alt) {
            return Ok(p);
        }
    }
    if let Some(p) =
        discover_via_biomeos_probe_filtered(&env_reader, matches_sovereign_storage_tokens)
    {
        return Ok(p);
    }
    anyhow::bail!("No sovereign-storage provider available")
}

/// True when `ft` is a Unix domain socket ([`std::fs::FileType`] is not a regular file for those).
#[cfg(unix)]
#[inline]
fn is_unix_socket_filetype(ft: &std::fs::FileType) -> bool {
    std::os::unix::fs::FileTypeExt::is_socket(ft)
}

#[cfg(not(unix))]
#[inline]
fn is_unix_socket_filetype(_ft: &std::fs::FileType) -> bool {
    false
}

/// List `*.sock` paths under biomeos runtime dirs (XDG, then legacy `/tmp/biomeos` if no XDG).
///
/// Includes **real** Unix sockets (not only regular files): [`Path::is_file`] is false for
/// socket inodes, so we use [`std::fs::DirEntry::file_type`] and accept `is_file` or
/// (on Unix) `is_socket`.
fn list_biomeos_sock_paths<F>(env_reader: &F) -> Vec<PathBuf>
where
    F: Fn(&str) -> Option<String>,
{
    let mut dirs = Vec::new();
    if let Some(xdg) = env_reader("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join(songbird_types::primal_names::BIOMEOS_DIR));
    }
    dirs.push(PathBuf::from("/tmp/biomeos"));

    let mut out = Vec::new();
    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sock")) {
                    continue;
                }
                let Ok(ft) = entry.file_type() else {
                    continue;
                };
                if ft.is_file() || is_unix_socket_filetype(&ft) {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Probe every biomeos socket until `predicate` returns true on capability tokens.
#[cfg(unix)]
fn discover_via_biomeos_probe_filtered<F, P>(env_reader: &F, predicate: P) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
    P: Fn(&[String]) -> bool,
{
    for path in list_biomeos_sock_paths(env_reader) {
        if let Some(tokens) = unix::probe_capabilities_list(&path)
            && predicate(&tokens)
        {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

#[cfg(not(unix))]
fn discover_via_biomeos_probe_filtered<F, P>(_env_reader: &F, _predicate: P) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
    P: Fn(&[String]) -> bool,
{
    None
}

fn discover_via_biomeos_probe<F>(capability: Capability, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    discover_via_biomeos_probe_filtered(env_reader, |tokens| {
        capability.matches_capability_tokens(tokens)
    })
}

/// Synchronous entry for non-async callers (e.g. JWT path discovery). Uses blocking Unix I/O only.
#[must_use]
pub fn discover_via_biomeos_probe_blocking(capability: Capability) -> Option<String> {
    discover_via_biomeos_probe(capability, &|k| songbird_process_env::var(k).ok())
}

/// Injectable env reader variant (tests).
#[must_use]
pub fn discover_via_biomeos_probe_blocking_with<F>(
    capability: Capability,
    env_reader: &F,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    discover_via_biomeos_probe(capability, env_reader)
}

/// Scan socket directories for sockets matching capability — **deprecated path**: use [`discover_via_biomeos_probe`].
///
/// Kept for internal/binary compatibility; implements the same probe-based logic (no filename heuristics).
#[must_use]
pub fn scan_sockets(capability: Capability) -> Option<String> {
    discover_via_biomeos_probe_blocking(capability)
}

/// Discover TCP endpoint for a capability (isomorphic fallback support).
fn discover_tcp_from_capability<F>(capability: Capability, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let names: Vec<&str> = match capability {
        Capability::Crypto => vec!["crypto"],
        Capability::Security => vec!["security"],
        Capability::Http => vec!["http"],
        Capability::Ai => vec!["ai"],
        Capability::Storage => vec!["storage"],
        Capability::Messaging => vec!["messaging"],
    };

    for name in names {
        if let Some(tcp_addr) = check_tcp_discovery_file(name, env_reader) {
            return Some(format!("tcp:{tcp_addr}"));
        }
    }

    None
}

fn check_tcp_discovery_file<F>(primal_name: &str, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let filename = format!("{primal_name}-ipc-port");
    let mut candidates = Vec::new();

    if let Some(runtime_dir) = env_reader("XDG_RUNTIME_DIR") {
        candidates.push(std::path::PathBuf::from(runtime_dir).join(&filename));
    }

    if let Some(home) = env_reader("HOME") {
        candidates.push(std::path::PathBuf::from(home).join(".local/share").join(&filename));
    }

    candidates.push(std::path::PathBuf::from(format!("/tmp/{filename}")));

    check_tcp_discovery_from_candidates(&candidates)
}

fn check_tcp_discovery_from_candidates(candidates: &[std::path::PathBuf]) -> Option<String> {
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path)
            && let Some(addr_str) = content.strip_prefix("tcp:")
        {
            let addr_trimmed = addr_str.trim();
            if addr_trimmed.parse::<std::net::SocketAddr>().is_ok() {
                debug!("   Found TCP discovery file: {} -> {}", path.display(), addr_trimmed);
                return Some(addr_trimmed.to_string());
            }
        }
    }

    None
}

/// Convenience function: Discover crypto provider
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_crypto_provider() -> Result<String> {
    discover(Capability::Crypto).await
}

/// Convenience function: Discover security provider
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_security_provider() -> Result<String> {
    discover(Capability::Security).await
}

/// Convenience function: Discover HTTP provider
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_http_provider() -> Result<String> {
    discover(Capability::Http).await
}

/// Convenience function: Discover AI provider
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_ai_provider() -> Result<String> {
    discover(Capability::Ai).await
}

/// Get family ID from environment (delegates to canonical `env_config::family_id()`)
#[must_use]
pub fn get_family_id() -> String {
    crate::env_config::family_id()
}

/// Get primal name from environment (self-knowledge)
#[must_use]
pub fn get_primal_name() -> String {
    songbird_process_env::var("PRIMAL_NAME").unwrap_or_else(|_| primal_names::SELF_NAME.to_string())
}

#[cfg(unix)]
mod unix {
    use super::parse_capabilities_result;
    use std::io::{Read, Write};
    use std::path::Path;
    use std::time::Duration;

    /// `health.liveness` then `capabilities.list` / `capability.list`; returns flat token list.
    pub(super) fn probe_capabilities_list(path: &Path) -> Option<Vec<String>> {
        let mut stream = std::os::unix::net::UnixStream::connect(path).ok()?;
        stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
        stream.set_write_timeout(Some(Duration::from_secs(2))).ok()?;

        let liveness_ok = jsonrpc_request_response(&mut stream, "health.liveness", 1).is_ok()
            || jsonrpc_request_response_raw(&mut stream, "ping", 11).is_ok();
        if !liveness_ok {
            return None;
        }

        let caps_resp = jsonrpc_request_response(&mut stream, "capabilities.list", 2)
            .or_else(|_| jsonrpc_request_response(&mut stream, "capability.list", 3))
            .ok()?;

        parse_capabilities_result(&caps_resp)
    }

    /// Legacy `ping` must be sent literally — [`songbird_types::normalize_json_rpc_method_name`]
    /// maps `ping` → `health.liveness`, which would not help when liveness failed.
    fn jsonrpc_request_response_raw(
        stream: &mut std::os::unix::net::UnixStream,
        method: &str,
        id: i64,
    ) -> Result<serde_json::Value, std::io::Error> {
        jsonrpc_request_response_inner(stream, method, id)
    }

    fn jsonrpc_request_response(
        stream: &mut std::os::unix::net::UnixStream,
        method: &str,
        id: i64,
    ) -> Result<serde_json::Value, std::io::Error> {
        let method = songbird_types::normalize_json_rpc_method_name(method);
        jsonrpc_request_response_inner(stream, method, id)
    }

    fn jsonrpc_request_response_inner(
        stream: &mut std::os::unix::net::UnixStream,
        method: &str,
        id: i64,
    ) -> Result<serde_json::Value, std::io::Error> {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": serde_json::json!({}),
            "id": id,
        });
        let mut bytes = serde_json::to_vec(&req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        bytes.push(b'\n');
        stream.write_all(&bytes)?;
        let line = read_line(stream)?;
        let v: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if v.get("error").is_some() {
            return Err(std::io::Error::other("jsonrpc error"));
        }
        Ok(v)
    }

    fn read_line(stream: &mut std::os::unix::net::UnixStream) -> Result<String, std::io::Error> {
        let mut buf = Vec::new();
        let mut one = [0u8; 1];
        loop {
            match stream.read(&mut one) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "short read",
                    ));
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
}

fn parse_capabilities_result(response: &serde_json::Value) -> Option<Vec<String>> {
    let result = response.get("result")?;
    if let Some(arr) = result.as_array() {
        return Some(
            arr.iter().filter_map(|v| v.as_str().map(std::string::ToString::to_string)).collect(),
        );
    }
    if let Some(obj) = result.as_object()
        && let Some(arr) = obj.get("capabilities").and_then(|c| c.as_array())
    {
        return Some(
            arr.iter().filter_map(|v| v.as_str().map(std::string::ToString::to_string)).collect(),
        );
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_capability_env_vars() {
        assert_eq!(Capability::Crypto.env_var_name(), "CRYPTO_PROVIDER_SOCKET");
        assert_eq!(Capability::Security.env_var_name(), "SECURITY_PROVIDER_SOCKET");
    }

    #[test]
    fn test_matches_crypto_tokens() {
        let t = vec!["crypto.delegate".to_string(), "ipc.jsonrpc".to_string()];
        assert!(Capability::Crypto.matches_capability_tokens(&t));
    }

    #[test]
    fn test_matches_security_tokens() {
        let t = vec!["security.verify".to_string()];
        assert!(Capability::Security.matches_capability_tokens(&t));
    }

    #[test]
    fn test_matches_http_tokens() {
        let t = vec!["http.request".to_string()];
        assert!(Capability::Http.matches_capability_tokens(&t));
    }

    #[test]
    fn test_matches_storage_not_sovereign() {
        let t = vec!["storage.get".to_string()];
        assert!(Capability::Storage.matches_capability_tokens(&t));
        assert!(!matches_sovereign_storage_tokens(&t));
    }

    #[test]
    fn test_matches_sovereign_storage() {
        let t = vec!["storage.get".to_string(), "edge.sovereign".to_string()];
        assert!(matches_sovereign_storage_tokens(&t));
    }

    #[test]
    fn test_normalize_json_rpc_used() {
        assert_eq!(songbird_types::normalize_json_rpc_method_name("ping"), "health.liveness");
    }

    #[test]
    fn test_family_id_default() {
        let family_id = get_family_id();
        assert!(!family_id.is_empty());
    }

    #[test]
    fn test_primal_name_default() {
        let primal_name = get_primal_name();
        assert_eq!(primal_name, "songbird");
    }

    #[test]
    fn test_tcp_discovery_file_parsing() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test-crypto-ipc-port");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:127.0.0.1:12345").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, Some("127.0.0.1:12345".to_string()));

        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_tcp_discovery_from_explicit_path() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("crypto-ipc-port-test");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:127.0.0.1:33765").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, Some("127.0.0.1:33765".to_string()));

        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_tcp_discovery_invalid_format() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("invalid-tcp-ipc-port-test");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"127.0.0.1:12345").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, None);

        std::fs::remove_file(file_path).ok();
    }

    #[tokio::test]
    async fn test_discover_with_env_var_override() {
        let custom_path = "/custom/path/http-provider.sock";
        let mock_env = |name: &str| -> Option<String> {
            if name == "HTTP_PROVIDER_SOCKET" {
                Some(custom_path.to_string())
            } else {
                None
            }
        };

        let result = discover_with(Capability::Http, mock_env).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path);
    }

    #[tokio::test]
    async fn test_discover_returns_env_var_priority() {
        let custom_path = "/test/custom/ai-provider.sock";
        let mock_env = |name: &str| -> Option<String> {
            if name == "AI_PROVIDER_SOCKET" {
                Some(custom_path.to_string())
            } else {
                None
            }
        };

        let result = discover_with(Capability::Ai, mock_env).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), custom_path);
    }
}
