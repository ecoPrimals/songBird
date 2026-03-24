// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Agnostic Primal Discovery - TRUE PRIMAL Architecture
//!
//! Discovers primals by capability at runtime, with ZERO hardcoding.
//! Songbird only knows itself - discovers other primals via capabilities.
//!
//! ## Principles
//!
//! 1. **Self-Knowledge Only**: Songbird knows only itself
//! 2. **Capability-Based**: Discover by what primals DO, not what they ARE
//! 3. **Runtime Discovery**: No compile-time dependencies on other primals
//! 4. **Graceful Degradation**: Features work without optional primals
//!
//! ## Discovery Strategy
//!
//! ```text
//! 1. Environment Variables (orchestrator-provided, preferred)
//!    - Explicit: {CAPABILITY}_PROVIDER_SOCKET
//!    - Generic: {PRIMAL}_SOCKET
//!    
//! 2. Capability Registry (runtime discovery)
//!    - Query for capability
//!    - Get socket path
//!    
//! 3. Common Socket Patterns (fallback)
//!    - /tmp/{capability}.sock
//!    - /tmp/{primal}-{family}.sock
//!    
//! 4. Socket Scanning (last resort)
//!    - Scan /tmp for matching sockets
//!    - Check socket capabilities via RPC
//! ```

use anyhow::Result;
use songbird_types::primal_names;
use std::path::Path;
use tracing::{debug, info, warn};

/// Capability types for primal discovery
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

    /// Get alternative environment variable names (for compatibility)
    fn alt_env_vars(&self) -> Vec<&'static str> {
        match self {
            Self::Crypto => vec!["BEARDOG_CRYPTO_SOCKET", "BEARDOG_SOCKET"],
            Self::Security => vec!["SONGBIRD_SECURITY_PROVIDER", "BEARDOG_SOCKET"],
            Self::Http => vec!["HTTP_CLIENT_SOCKET", "SONGBIRD_SOCKET"],
            Self::Ai => vec!["SQUIRREL_SOCKET", "AI_PROVIDER_SOCKETS"],
            Self::Storage => vec!["NESTGATE_SOCKET", "STORAGE_SOCKET"],
            Self::Messaging => vec!["MESSENGER_SOCKET", "PUBSUB_SOCKET"],
        }
    }

    /// Get common socket path patterns for this capability.
    ///
    /// Returns capability-named sockets first (e.g., `crypto.sock`),
    /// then known-provider sockets as hints (e.g., `beardog.sock`).
    ///
    /// Priority order:
    /// 1. `$XDG_RUNTIME_DIR/biomeos/{capability}.sock` (capability-first)
    /// 2. `$XDG_RUNTIME_DIR/biomeos/{provider}.sock` (known-provider hint)
    /// 3. `/tmp/biomeos/{capability}.sock` (fallback)
    /// 4. `/tmp/{provider}.sock` (legacy)
    fn socket_patterns(&self) -> Vec<String> {
        self.socket_patterns_with_env(&|k| std::env::var(k).ok())
    }

    /// Same as [`Self::socket_patterns`], but `XDG_RUNTIME_DIR` (and any future lookups)
    /// go through `env_reader` so tests can inject values without mutating process env.
    fn socket_patterns_with_env<F>(&self, env_reader: &F) -> Vec<String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let xdg_base = env_reader("XDG_RUNTIME_DIR")
            .map_or_else(|| "/tmp/biomeos".to_string(), |d| format!("{d}/biomeos"));

        let cap_name: &str = match self {
            Self::Crypto => "crypto",
            Self::Security => "security",
            Self::Http => "http",
            Self::Ai => "ai",
            Self::Storage => "storage",
            Self::Messaging => "messaging",
        };

        vec![
            format!("{xdg_base}/{cap_name}.sock"),
            format!("/tmp/biomeos/{cap_name}.sock"),
            format!("/tmp/{cap_name}.sock"),
        ]
    }
}

/// Discover a primal by capability (functional, no state)
///
/// # Example
///
/// ```rust,no_run
/// use songbird_orchestrator::primal_discovery::{discover, Capability};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Discover crypto provider (could be BearDog, or any primal with crypto capability)
/// let crypto_socket = discover(Capability::Crypto).await?;
/// println!("Crypto provider at: {}", crypto_socket);
///
/// // Discover AI provider (could be Squirrel, or any primal with AI capability)
/// let ai_socket = discover(Capability::Ai).await?;
/// println!("AI provider at: {}", ai_socket);
/// # Ok(())
/// # }
/// ```
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover(capability: Capability) -> Result<String> {
    discover_with(capability, |name| std::env::var(name).ok()).await
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

    // Strategy 3: Common socket patterns (Unix sockets)
    for pattern in capability.socket_patterns_with_env(&env_reader) {
        if Path::new(&pattern).exists() {
            info!("   ✅ Found {:?} provider socket at: {}", capability, pattern);
            return Ok(pattern);
        }
        debug!("   ⏭️  Not found: {}", pattern);
    }

    // Strategy 3.5: TCP discovery files (isomorphic fallback)
    if let Some(tcp_endpoint) = discover_tcp_from_capability(capability) {
        info!("   ✅ Found {:?} provider via TCP discovery file: {}", capability, tcp_endpoint);
        return Ok(tcp_endpoint);
    }

    // Strategy 4: Socket scanning (last resort)
    if let Some(socket_path) = scan_sockets_with_env(capability, &env_reader) {
        info!("   ✅ Found {:?} provider via scanning: {}", capability, socket_path);
        return Ok(socket_path);
    }

    // Not found
    warn!("❌ No {:?} provider found - checked all discovery strategies", capability);
    anyhow::bail!("No {capability:?} provider available")
}

/// Scan socket directories for sockets matching capability.
///
/// Searches using capability terms first (e.g., "crypto"), then
/// known provider names as secondary hints (e.g., "beardog").
///
/// Scan priority: `$XDG_RUNTIME_DIR/biomeos/` → `/tmp/biomeos/` → `/tmp/`
fn scan_sockets(capability: Capability) -> Option<String> {
    scan_sockets_with_env(capability, &|k| std::env::var(k).ok())
}

fn scan_sockets_with_env<F>(capability: Capability, env_reader: &F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    let search_terms = match capability {
        Capability::Crypto => vec!["crypto", "encryption"],
        Capability::Security => vec!["security", "auth"],
        Capability::Http => vec!["http", "gateway"],
        Capability::Ai => vec!["ai", "inference", "ml"],
        Capability::Storage => vec!["storage", "persist", "data"],
        Capability::Messaging => vec!["messaging", "pubsub"],
    };

    // Build directory search order
    let mut dirs_to_scan = Vec::new();

    // Priority 1: XDG_RUNTIME_DIR/biomeos/
    if let Some(xdg_runtime) = env_reader("XDG_RUNTIME_DIR") {
        dirs_to_scan.push(format!("{xdg_runtime}/biomeos"));
    }

    // Priority 2: /tmp/biomeos/
    dirs_to_scan.push("/tmp/biomeos".to_string());

    // Priority 3: /tmp/ (legacy)
    dirs_to_scan.push("/tmp".to_string());

    for dir in dirs_to_scan {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    // Check if filename matches any search term
                    if file_name.to_ascii_lowercase().ends_with(".sock") {
                        for term in &search_terms {
                            if file_name.contains(term) {
                                let path = entry.path();
                                debug!("   Found potential socket: {}", path.display());
                                return Some(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Discover TCP endpoint for a capability (isomorphic fallback support).
///
/// Checks TCP discovery files for primals that provide this capability.
/// Searches capability-named files first (e.g., `crypto-ipc-port`),
/// then known-provider files (e.g., `beardog-ipc-port`).
///
/// # Discovery File Format
///
/// File: `$XDG_RUNTIME_DIR/{name}-ipc-port`\
/// Content: `tcp:127.0.0.1:12345`
///
/// # Arguments
///
/// * `capability` - The capability to discover (e.g., Crypto, Storage)
///
/// # Returns
///
/// Socket descriptor string (e.g., "tcp:127.0.0.1:12345") if found, None otherwise.
fn discover_tcp_from_capability(capability: Capability) -> Option<String> {
    let names: Vec<&str> = match capability {
        Capability::Crypto => vec!["crypto"],
        Capability::Security => vec!["security"],
        Capability::Http => vec!["http"],
        Capability::Ai => vec!["ai"],
        Capability::Storage => vec!["storage"],
        Capability::Messaging => vec!["messaging"],
    };

    for name in names {
        if let Some(tcp_addr) = check_tcp_discovery_file(name) {
            return Some(format!("tcp:{tcp_addr}"));
        }
    }

    None
}

/// Check TCP discovery file for a specific primal
///
/// Checks XDG-compliant locations in priority order:
/// 1. `$XDG_RUNTIME_DIR/{primal}-ipc-port`
/// 2. `$HOME/.local/share/{primal}-ipc-port`
/// 3. `/tmp/{primal}-ipc-port`
///
/// # Arguments
///
/// * `primal_name` - Primal name (e.g., "beardog", "squirrel")
///
/// # Returns
///
/// TCP socket address (e.g., "127.0.0.1:12345") if found, None otherwise.
fn check_tcp_discovery_file(primal_name: &str) -> Option<String> {
    let filename = format!("{primal_name}-ipc-port");
    let mut candidates = Vec::new();

    // Priority 1: XDG_RUNTIME_DIR (preferred)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        candidates.push(std::path::PathBuf::from(runtime_dir).join(&filename));
    }

    // Priority 2: HOME/.local/share (fallback)
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(std::path::PathBuf::from(home).join(".local/share").join(&filename));
    }

    // Priority 3: /tmp (last resort)
    candidates.push(std::path::PathBuf::from(format!("/tmp/{filename}")));

    check_tcp_discovery_from_candidates(&candidates)
}

/// Check TCP discovery from explicit candidate paths (testable, no env vars)
fn check_tcp_discovery_from_candidates(candidates: &[std::path::PathBuf]) -> Option<String> {
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            // Parse format: "tcp:127.0.0.1:12345"
            if let Some(addr_str) = content.strip_prefix("tcp:") {
                let addr_trimmed = addr_str.trim();
                // Validate it's a parseable socket address
                if addr_trimmed.parse::<std::net::SocketAddr>().is_ok() {
                    debug!("   Found TCP discovery file: {} -> {}", path.display(), addr_trimmed);
                    return Some(addr_trimmed.to_string());
                }
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
    std::env::var("PRIMAL_NAME").unwrap_or_else(|_| primal_names::SELF_NAME.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_env_vars() {
        assert_eq!(Capability::Crypto.env_var_name(), "CRYPTO_PROVIDER_SOCKET");
        assert_eq!(Capability::Security.env_var_name(), "SECURITY_PROVIDER_SOCKET");
    }

    #[test]
    fn test_capability_patterns_capability_first() {
        let patterns = Capability::Crypto.socket_patterns();
        assert!(!patterns.is_empty());
        assert!(
            patterns.iter().any(|p| p.contains("crypto.sock")),
            "Should have crypto.sock pattern"
        );
        assert!(
            !patterns.iter().any(|p| p.contains("beardog")),
            "Should not contain primal-specific names"
        );
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
        // ✅ Concurrent-safe: Uses check_tcp_discovery_from_candidates (no env vars)
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test-beardog-ipc-port");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"tcp:127.0.0.1:12345").unwrap();
        drop(file);

        // Directly pass candidate path (no env var needed)
        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, Some("127.0.0.1:12345".to_string()));

        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn test_tcp_discovery_from_explicit_path() {
        // ✅ Concurrent-safe: Tests beardog discovery via explicit candidate paths
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("beardog-ipc-port-test");

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
        // ✅ Concurrent-safe: Uses explicit candidate paths
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("invalid-beardog-ipc-port-test");

        // Write invalid format (missing tcp: prefix)
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"127.0.0.1:12345").unwrap();
        drop(file);

        let candidates = vec![file_path.clone()];
        let result = check_tcp_discovery_from_candidates(&candidates);
        assert_eq!(result, None);

        std::fs::remove_file(file_path).ok();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🧪 XDG SOCKET DISCOVERY TESTS (Feb 4, 2026)
    // ✅ Evolved to concurrent-safe — no env var mutation
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_xdg_socket_patterns_structure() {
        let patterns = Capability::Crypto.socket_patterns();

        assert!(patterns.len() >= 2, "Should have at least 2 patterns");
        for pattern in &patterns {
            assert!(pattern.ends_with(".sock"), "Pattern should end with .sock: {pattern}");
        }
        assert!(
            patterns.iter().any(|p| p.contains("crypto")),
            "Crypto patterns should include capability name"
        );
    }

    #[test]
    fn test_all_capabilities_return_patterns() {
        let capabilities = [
            Capability::Crypto,
            Capability::Security,
            Capability::Http,
            Capability::Ai,
            Capability::Storage,
            Capability::Messaging,
        ];

        for cap in &capabilities {
            let patterns = cap.socket_patterns();
            assert!(!patterns.is_empty(), "{cap:?} should return at least one pattern");
            assert!(
                patterns.iter().all(|p| p.ends_with(".sock")),
                "{cap:?} patterns should all end with .sock"
            );
        }
    }

    #[test]
    fn test_socket_patterns_no_nat0_suffix() {
        let patterns = Capability::Crypto.socket_patterns();
        for pattern in &patterns {
            if pattern.contains("biomeos") {
                assert!(
                    !pattern.contains("-nat0"),
                    "XDG patterns should not have -nat0 suffix: {pattern}"
                );
            }
        }
    }

    #[test]
    fn test_socket_patterns_include_capability_names() {
        // Each capability should include its own capability-named socket
        assert!(Capability::Crypto.socket_patterns().iter().any(|p| p.contains("crypto")));
        assert!(Capability::Http.socket_patterns().iter().any(|p| p.contains("http")));
        assert!(Capability::Ai.socket_patterns().iter().any(|p| p.contains("ai")));
        assert!(Capability::Storage.socket_patterns().iter().any(|p| p.contains("storage")));
        assert!(Capability::Messaging.socket_patterns().iter().any(|p| p.contains("messaging")));
    }

    #[test]
    fn test_socket_patterns_are_capability_only() {
        for cap in [
            Capability::Crypto,
            Capability::Security,
            Capability::Http,
            Capability::Ai,
            Capability::Storage,
            Capability::Messaging,
        ] {
            let patterns = cap.socket_patterns();
            assert!(
                !patterns.iter().any(|p| p.contains("beardog")
                    || p.contains("squirrel")
                    || p.contains("nestgate")
                    || p.contains("toadstool")),
                "{cap:?} patterns should not contain primal-specific names: {patterns:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_discover_with_env_var_override() {
        // ✅ Concurrent-safe: Uses discover_with (injectable env reader, no global state)
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
        // ✅ Concurrent-safe: Uses discover_with (injectable env reader, no global state)
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
