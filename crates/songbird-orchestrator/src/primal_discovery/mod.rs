// SPDX-License-Identifier: AGPL-3.0-or-later
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
pub(crate) mod parse;
#[cfg(unix)]
pub(crate) mod socket_auto_discovery;
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
        let t = vec![String::from("crypto.delegate"), String::from("ipc.jsonrpc")];
        assert!(Capability::Crypto.matches_capability_tokens(&t));
    }

    #[test]
    fn test_matches_security_tokens() {
        let t = vec![String::from("security.verify")];
        assert!(Capability::Security.matches_capability_tokens(&t));
    }

    #[test]
    fn test_matches_http_tokens() {
        let t = vec![String::from("http.request")];
        assert!(Capability::Http.matches_capability_tokens(&t));
    }

    #[test]
    fn test_matches_storage_not_sovereign() {
        let t = vec![String::from("storage.get")];
        assert!(Capability::Storage.matches_capability_tokens(&t));
        assert!(!matches_sovereign_storage_tokens(&t));
    }

    #[test]
    fn test_matches_sovereign_storage() {
        let t = vec![String::from("storage.get"), String::from("edge.sovereign")];
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
        assert_eq!(result, Some(String::from("127.0.0.1:12345")));

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
        assert_eq!(result, Some(String::from("127.0.0.1:33765")));

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

    // ─── capability.rs coverage ──────────────────────────────────────────

    #[test]
    fn capability_matches_ai_tokens() {
        assert!(Capability::Ai.matches_capability_tokens(&["ai.inference".into()]));
        assert!(Capability::Ai.matches_capability_tokens(&["llm.chat".into()]));
        assert!(Capability::Ai.matches_capability_tokens(&["mcp.tool".into()]));
        assert!(Capability::Ai.matches_capability_tokens(&["model.load".into()]));
        assert!(Capability::Ai.matches_capability_tokens(&["local.inference.gpu".into()]));
        assert!(!Capability::Ai.matches_capability_tokens(&["storage.get".into()]));
    }

    #[test]
    fn capability_matches_messaging_tokens() {
        assert!(Capability::Messaging.matches_capability_tokens(&["messaging.send".into()]));
        assert!(Capability::Messaging.matches_capability_tokens(&["pubsub.subscribe".into()]));
        assert!(Capability::Messaging.matches_capability_tokens(&["queue.push".into()]));
        assert!(Capability::Messaging.matches_capability_tokens(&["message.deliver".into()]));
        assert!(!Capability::Messaging.matches_capability_tokens(&["http.request".into()]));
    }

    #[test]
    fn capability_matches_storage_tokens() {
        assert!(Capability::Storage.matches_capability_tokens(&["storage.put".into()]));
        assert!(Capability::Storage.matches_capability_tokens(&["data.persist".into()]));
        assert!(!Capability::Storage.matches_capability_tokens(&["compute.run".into()]));
    }

    #[test]
    fn capability_matches_http_all_variants() {
        assert!(Capability::Http.matches_capability_tokens(&["http.get".into()]));
        assert!(Capability::Http.matches_capability_tokens(&["http.post".into()]));
        assert!(Capability::Http.matches_capability_tokens(&["http.request".into()]));
        assert!(Capability::Http.matches_capability_tokens(&["http.delete".into()]));
        assert!(!Capability::Http.matches_capability_tokens(&["https.verify".into()]));
    }

    #[test]
    fn capability_matches_crypto_all_variants() {
        assert!(Capability::Crypto.matches_capability_tokens(&["crypto.delegate".into()]));
        assert!(Capability::Crypto.matches_capability_tokens(&["crypto.sign".into()]));
        assert!(Capability::Crypto.matches_capability_tokens(&["encryption.aes".into()]));
        assert!(!Capability::Crypto.matches_capability_tokens(&["security.jwt".into()]));
    }

    #[test]
    fn capability_matches_security_all_variants() {
        assert!(Capability::Security.matches_capability_tokens(&["security.verify".into()]));
        assert!(Capability::Security.matches_capability_tokens(&["jwt.issue".into()]));
        assert!(Capability::Security.matches_capability_tokens(&["btsp.negotiate".into()]));
        assert!(Capability::Security.matches_capability_tokens(&["security".into()]));
        assert!(!Capability::Security.matches_capability_tokens(&["crypto.sign".into()]));
    }

    #[test]
    fn capability_matches_empty_tokens_returns_false() {
        assert!(!Capability::Crypto.matches_capability_tokens(&[]));
        assert!(!Capability::Security.matches_capability_tokens(&[]));
        assert!(!Capability::Http.matches_capability_tokens(&[]));
        assert!(!Capability::Ai.matches_capability_tokens(&[]));
        assert!(!Capability::Storage.matches_capability_tokens(&[]));
        assert!(!Capability::Messaging.matches_capability_tokens(&[]));
    }

    #[test]
    fn capability_matches_is_case_insensitive() {
        assert!(Capability::Crypto.matches_capability_tokens(&["CRYPTO.DELEGATE".into()]));
        assert!(Capability::Http.matches_capability_tokens(&["HTTP.Request".into()]));
        assert!(Capability::Ai.matches_capability_tokens(&["AI.Inference".into()]));
    }

    #[test]
    fn capability_from_wire_id_valid() {
        assert_eq!(capability_from_wire_id("crypto").unwrap(), Capability::Crypto);
        assert_eq!(capability_from_wire_id("ai").unwrap(), Capability::Ai);
        assert_eq!(capability_from_wire_id("storage").unwrap(), Capability::Storage);
        assert_eq!(capability_from_wire_id("messaging").unwrap(), Capability::Messaging);
        assert_eq!(capability_from_wire_id("http").unwrap(), Capability::Http);
        assert_eq!(capability_from_wire_id("security").unwrap(), Capability::Security);
    }

    #[test]
    fn capability_from_wire_id_unknown() {
        assert!(capability_from_wire_id("unknown").is_err());
        assert!(capability_from_wire_id("").is_err());
        assert!(capability_from_wire_id("compute").is_err());
    }

    #[test]
    fn capability_alt_env_vars_non_empty() {
        assert!(!Capability::Crypto.alt_env_vars().is_empty());
        assert!(!Capability::Security.alt_env_vars().is_empty());
        assert!(!Capability::Http.alt_env_vars().is_empty());
        assert!(!Capability::Ai.alt_env_vars().is_empty());
        assert!(!Capability::Storage.alt_env_vars().is_empty());
        assert!(!Capability::Messaging.alt_env_vars().is_empty());
    }

    #[test]
    fn sovereign_storage_requires_both_storage_and_sovereign() {
        assert!(matches_sovereign_storage_tokens(&["storage.get".into(), "edge.sovereign".into()]));
        assert!(!matches_sovereign_storage_tokens(&["storage.get".into()]));
        assert!(!matches_sovereign_storage_tokens(&["edge.sovereign".into()]));
    }

    // ─── parse.rs coverage ───────────────────────────────────────────────

    #[test]
    fn parse_capabilities_array_result() {
        let response = serde_json::json!({
            "result": ["http.request", "http.get", "tls.1.3"]
        });
        let caps = parse::parse_capabilities_result(&response).unwrap();
        assert_eq!(caps, vec!["http.request", "http.get", "tls.1.3"]);
    }

    #[test]
    fn parse_capabilities_object_with_capabilities_key() {
        let response = serde_json::json!({
            "result": {
                "capabilities": ["crypto.sign", "crypto.delegate"]
            }
        });
        let caps = parse::parse_capabilities_result(&response).unwrap();
        assert_eq!(caps, vec!["crypto.sign", "crypto.delegate"]);
    }

    #[test]
    fn parse_capabilities_missing_result_key() {
        let response = serde_json::json!({ "error": "not found" });
        assert!(parse::parse_capabilities_result(&response).is_none());
    }

    #[test]
    fn parse_capabilities_null_result() {
        let response = serde_json::json!({ "result": null });
        assert!(parse::parse_capabilities_result(&response).is_none());
    }

    #[test]
    fn parse_capabilities_empty_array() {
        let response = serde_json::json!({ "result": [] });
        let caps = parse::parse_capabilities_result(&response).unwrap();
        assert!(caps.is_empty());
    }

    #[test]
    fn parse_capabilities_filters_non_strings() {
        let response = serde_json::json!({
            "result": ["valid", 42, null, "also_valid", true]
        });
        let caps = parse::parse_capabilities_result(&response).unwrap();
        assert_eq!(caps, vec!["valid", "also_valid"]);
    }

    // ─── tcp_biomeos.rs injectable env coverage ──────────────────────────

    #[test]
    fn discover_with_sync_finds_via_primary_env_var() {
        let env = |name: &str| -> Option<String> {
            if name == "CRYPTO_PROVIDER_SOCKET" {
                Some("/run/crypto.sock".into())
            } else {
                None
            }
        };
        let result = tcp_biomeos::discover_with_sync(Capability::Crypto, env);
        assert_eq!(result.unwrap(), "/run/crypto.sock");
    }

    #[test]
    fn discover_with_sync_finds_via_alt_env_var() {
        let env = |name: &str| -> Option<String> {
            if name == "BEARDOG_SOCKET" {
                Some("/run/beardog.sock".into())
            } else {
                None
            }
        };
        let result = tcp_biomeos::discover_with_sync(Capability::Crypto, env);
        assert_eq!(result.unwrap(), "/run/beardog.sock");
    }

    #[test]
    fn discover_with_sync_no_env_no_fs_fails() {
        let env = |_: &str| -> Option<String> { None };
        let result = tcp_biomeos::discover_with_sync(Capability::Messaging, env);
        assert!(result.is_err());
    }

    #[test]
    fn tcp_discovery_file_ignores_malformed_content() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test-malformed-tcp-discovery");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:not_a_valid_socket_addr").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, None);

        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn tcp_discovery_file_empty_file() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test-empty-tcp-discovery");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, None);

        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn tcp_discovery_file_nonexistent_path() {
        let candidates = vec![std::path::PathBuf::from("/nonexistent/path/to/tcp-port-file")];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, None);
    }

    #[test]
    fn tcp_discovery_ipv6_address() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test-ipv6-tcp-discovery");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:[::1]:9876").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, Some(String::from("[::1]:9876")));

        std::fs::remove_file(file_path).ok();
    }
}
