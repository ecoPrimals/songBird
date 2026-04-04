// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Agnostic Primal Discovery — wateringHole / TRUE PRIMAL
//!
//! Discovers primals by **capability at runtime** via JSON-RPC probes on Unix
//! sockets under `$XDG_RUNTIME_DIR/biomeos/`, with **no** filename- or
//! primal-name-based classification.
//!
//! Submodules: `capability` (roles and token matching), `tcp_biomeos` (TCP files + socket scan),
//! `unix_transport` (Unix JSON-RPC probe), `parse` (response shaping).

mod capability;
mod parse;
pub(crate) mod tcp_biomeos;
#[cfg(unix)]
mod unix_transport;

pub use capability::{Capability, capability_from_wire_id, matches_sovereign_storage_tokens};
pub use tcp_biomeos::{
    discover_via_biomeos_probe_blocking, discover_via_biomeos_probe_blocking_with, scan_sockets,
};

pub(crate) use tcp_biomeos::{discover_via_biomeos_probe_filtered, discover_with_sync};

use anyhow::Result;
use songbird_types::primal_names;
use tracing::info;

/// Discover a primal by capability.
///
/// Uses `songbird_process_env` as the env reader. The env fast-path runs inline;
/// if no env match, the filesystem/socket slow path runs in `spawn_blocking` to
/// avoid blocking async worker threads.
pub async fn discover(capability: Capability) -> Result<String> {
    let env_reader = |name: &str| songbird_process_env::var(name).ok();

    if let Some(socket_path) = env_reader(capability.env_var_name()) {
        info!("   ✅ Found via {}: {}", capability.env_var_name(), socket_path);
        return Ok(socket_path);
    }
    for alt_var in capability.alt_env_vars() {
        if let Some(socket_path) = env_reader(alt_var) {
            info!("   ✅ Found via {} (compatibility): {}", alt_var, socket_path);
            return Ok(socket_path);
        }
    }

    tokio::task::spawn_blocking(move || discover_with_sync(capability, env_reader)).await?
}

/// Discover a primal by capability with injectable env reader (concurrent-safe, testable).
#[expect(
    clippy::unused_async,
    reason = "async signature preserved for call-site compatibility; I/O is sync"
)]
pub async fn discover_with<F>(capability: Capability, env_reader: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    discover_with_sync(capability, env_reader)
}

/// Discover by string id (e.g. doctor): handles `sovereign-storage` specially.
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
#[expect(
    clippy::unused_async,
    reason = "async signature preserved for call-site compatibility; I/O is sync"
)]
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
    for alt in ["STORAGE_SOCKET", "NESTGATE_SOCKET"] {
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

/// Convenience: discover crypto provider socket path.
pub async fn discover_crypto_provider() -> Result<String> {
    discover(Capability::Crypto).await
}

/// Convenience: discover security provider socket path.
pub async fn discover_security_provider() -> Result<String> {
    discover(Capability::Security).await
}

/// Convenience: discover HTTP provider socket path.
pub async fn discover_http_provider() -> Result<String> {
    discover(Capability::Http).await
}

/// Convenience: discover AI provider socket path.
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::tcp_biomeos::check_tcp_discovery_from_candidates;
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
