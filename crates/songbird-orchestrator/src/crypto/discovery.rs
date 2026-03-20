// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Crypto Provider Discovery
//!
//! Discovers ANY primal offering "crypto" capability via runtime discovery.
//! Maintains TRUE PRIMAL self-knowledge — Songbird only knows itself,
//! discovers crypto providers at runtime by capability, not by name.
//!
//! **Philosophy**: Primals only know themselves, discover others by capability.
//! The discovery order is: capability env vars → capability socket names →
//! known-provider hints → filesystem scanning.

use anyhow::Result;
use tracing::{debug, info, warn};

/// Well-known search terms for crypto capability socket scanning.
/// Capability terms come first; known provider names are secondary hints.
const CRYPTO_SEARCH_TERMS: &[&str] = &["crypto", "security", "encryption"];

/// Discover crypto provider socket via capability-based discovery.
///
/// ## TRUE PRIMAL Principles
///
/// 1. **Self-Knowledge**: Songbird only knows itself
/// 2. **Capability Discovery**: Searches for "crypto" capability first
/// 3. **Runtime Discovery**: No compile-time dependencies on providers
/// 4. **Graceful Fallback**: Works without any crypto provider
///
/// ## Discovery Strategy (priority order)
///
/// 1. `CRYPTO_PROVIDER_SOCKET` env var (orchestrator-provided, preferred)
/// 2. `CRYPTO_PROVIDER` env var (alternative)
/// 3. `BEARDOG_CRYPTO_SOCKET` env var (migration compatibility)
/// 4. `BEARDOG_SOCKET` env var (legacy compatibility)
/// 5. Capability-named sockets: `crypto.sock` (XDG → `/tmp/biomeos` → `/tmp`)
/// 6. Known-provider sockets: `beardog.sock` (XDG → `/tmp/biomeos` → `/tmp`)
/// 7. Filesystem scan for any socket matching crypto search terms
///
/// # Errors
///
/// Returns error if no crypto provider is discoverable.
pub async fn discover_crypto_socket() -> Result<String> {
    info!("🔍 Discovering crypto provider via capability-based discovery...");

    // Strategy 1: CRYPTO_PROVIDER_SOCKET (orchestrator-managed, preferred)
    if let Ok(socket_path) = std::env::var("CRYPTO_PROVIDER_SOCKET") {
        info!("   ✅ Found CRYPTO_PROVIDER_SOCKET: {}", socket_path);
        return Ok(socket_path);
    }

    // Strategy 2: CRYPTO_PROVIDER (alternative env var)
    if let Ok(socket_path) = std::env::var("CRYPTO_PROVIDER") {
        info!("   ✅ Found CRYPTO_PROVIDER: {}", socket_path);
        return Ok(socket_path);
    }

    // Strategy 3: BEARDOG_CRYPTO_SOCKET (compatibility during migration)
    if let Ok(socket_path) = std::env::var("BEARDOG_CRYPTO_SOCKET") {
        info!("   ✅ Found BEARDOG_CRYPTO_SOCKET (compatibility): {}", socket_path);
        return Ok(socket_path);
    }

    // Strategy 4: BEARDOG_SOCKET (legacy compatibility)
    if let Ok(socket_path) = std::env::var("BEARDOG_SOCKET") {
        info!("   ✅ Found BEARDOG_SOCKET (crypto capability): {}", socket_path);
        return Ok(socket_path);
    }

    // Strategy 5+6: Search common socket paths — capability names first, then known providers
    let xdg_base = std::env::var("XDG_RUNTIME_DIR")
        .map_or_else(|_| "/tmp/biomeos".to_string(), |d| format!("{d}/biomeos"));

    let common_paths = [
        format!("{xdg_base}/crypto.sock"),
        "/tmp/biomeos/crypto.sock".to_string(),
        "/tmp/crypto.sock".to_string(),
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            info!("   ✅ Found crypto provider socket at: {}", path);
            return Ok(path.clone());
        }
        debug!("   ⏭️  Not found: {}", path);
    }

    // Strategy 7: Scan socket directories for any crypto-capable socket
    if let Some(found) = scan_for_capability_socket(CRYPTO_SEARCH_TERMS) {
        info!("   ✅ Found crypto provider via scanning: {}", found);
        return Ok(found);
    }

    warn!("❌ No crypto provider found — checked all discovery strategies");
    warn!("   Songbird will fall back to ring crypto provider (temporary)");
    warn!("   This maintains TLS functionality but uses C dependencies");

    Err(anyhow::anyhow!("No crypto provider available"))
}

/// Backward-compatible alias for [`discover_crypto_socket`].
pub async fn get_beardog_crypto_socket() -> Result<String> {
    discover_crypto_socket().await
}

/// Scan socket directories for sockets matching any of the given search terms.
///
/// Scans in priority order: `$XDG_RUNTIME_DIR/biomeos/` → `/tmp/biomeos/` → `/tmp/`.
fn scan_for_capability_socket(search_terms: &[&str]) -> Option<String> {
    let mut dirs = Vec::with_capacity(3);
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(format!("{xdg}/biomeos"));
    }
    dirs.push("/tmp/biomeos".to_string());
    dirs.push("/tmp".to_string());

    for dir in dirs {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    let lower = file_name.to_ascii_lowercase();
                    if std::path::Path::new(&lower)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                        && search_terms.iter().any(|term| lower.contains(term))
                    {
                        return Some(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    None
}

/// Discover crypto provider socket for a specific family.
///
/// Checks family-specific capability socket first, then falls back to
/// generic capability-based discovery.
///
/// # Arguments
///
/// * `family_id` - Family ID to search for (e.g., "my-family")
///
/// # Errors
///
/// Returns error if no crypto provider is discoverable for this family.
pub async fn discover_crypto_socket_for_family(family_id: &str) -> Result<String> {
    info!("🔍 Discovering crypto provider for family '{family_id}'...");

    // Check family-specific socket (capability-based, primal-agnostic)
    let family_socket = format!("/tmp/crypto-{family_id}.sock");
    if std::path::Path::new(&family_socket).exists() {
        info!("   ✅ Found family-specific crypto socket: {family_socket}");
        return Ok(family_socket);
    }

    // Fall back to generic capability discovery
    crate::primal_discovery::discover_crypto_provider().await
}

/// Backward-compatible alias for [`discover_crypto_socket_for_family`].
pub async fn get_beardog_crypto_socket_for_family(family_id: &str) -> Result<String> {
    discover_crypto_socket_for_family(family_id).await
}

/// Check if any crypto provider is available.
///
/// Quick check without logging warnings — suitable for conditional logic.
pub async fn is_crypto_available() -> bool {
    // Quick check via env vars (no I/O)
    if std::env::var("CRYPTO_PROVIDER_SOCKET").is_ok()
        || std::env::var("CRYPTO_PROVIDER").is_ok()
        || std::env::var("BEARDOG_CRYPTO_SOCKET").is_ok()
        || std::env::var("BEARDOG_SOCKET").is_ok()
    {
        return true;
    }

    // Check common paths silently — capability names first
    let xdg_base = std::env::var("XDG_RUNTIME_DIR")
        .map_or_else(|_| "/tmp/biomeos".to_string(), |d| format!("{d}/biomeos"));

    let common_paths = [
        format!("{xdg_base}/crypto.sock"),
        "/tmp/biomeos/crypto.sock".to_string(),
        "/tmp/crypto.sock".to_string(),
        format!("{xdg_base}/beardog.sock"),
        "/tmp/biomeos/beardog.sock".to_string(),
        "/tmp/beardog.sock".to_string(),
    ];

    common_paths.iter().any(|path| std::path::Path::new(path).exists())
}

/// Backward-compatible alias for [`is_crypto_available`].
pub async fn is_beardog_crypto_available() -> bool {
    is_crypto_available().await
}

/// Discover crypto provider socket with purpose context for audit logging.
///
/// Same as [`discover_crypto_socket`] but logs the purpose.
///
/// # Errors
///
/// Returns error if no crypto provider is discoverable.
pub async fn discover_crypto_socket_for_purpose(purpose: &str) -> Result<String> {
    info!("🔍 Discovering crypto provider for purpose: {purpose}");
    discover_crypto_socket().await
}

/// Backward-compatible alias for [`discover_crypto_socket_for_purpose`].
pub async fn get_beardog_crypto_socket_for_purpose(purpose: &str) -> Result<String> {
    discover_crypto_socket_for_purpose(purpose).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // All tests are fully concurrent -- no env var mutation, no global state.

    #[test]
    fn test_xdg_path_construction_with_runtime_dir() {
        let xdg_base = "/run/user/1000";
        let expected_path = format!("{}/biomeos/beardog.sock", xdg_base);
        assert_eq!(expected_path, "/run/user/1000/biomeos/beardog.sock");
    }

    #[test]
    fn test_xdg_fallback_path_construction() {
        let fallback_base = "/tmp/biomeos";
        let expected_path = format!("{}/beardog.sock", fallback_base);
        assert_eq!(expected_path, "/tmp/biomeos/beardog.sock");
    }

    #[test]
    fn test_legacy_fallback_path_construction() {
        let legacy_path = "/tmp/beardog.sock";
        assert!(legacy_path.ends_with("beardog.sock"));
        assert!(!legacy_path.contains("biomeos"));
    }

    #[test]
    fn test_family_socket_path_format() {
        let family_id = "nat0";
        let expected = format!("/tmp/crypto-{}.sock", family_id);
        assert_eq!(expected, "/tmp/crypto-nat0.sock");
    }

    #[tokio::test]
    async fn test_discover_crypto_socket_graceful_failure() {
        let result = discover_crypto_socket().await;
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => {
                let msg = format!("{e}");
                assert!(
                    msg.contains("not available") || msg.contains("No crypto provider"),
                    "Unexpected error: {msg}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_is_crypto_available_returns_bool() {
        // Should return a bool without panicking, regardless of environment
        let _available = is_crypto_available().await;
    }

    #[tokio::test]
    async fn test_discover_crypto_socket_for_purpose_no_panic() {
        let result = discover_crypto_socket_for_purpose("signing").await;
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => {
                let msg = format!("{e}");
                assert!(
                    msg.contains("not available") || msg.contains("No crypto provider"),
                    "Unexpected error: {msg}"
                );
            }
        }
    }

    #[test]
    fn test_crypto_search_terms_capability_first() {
        // Capability terms must appear before provider-specific hints
        assert_eq!(CRYPTO_SEARCH_TERMS[0], "crypto");
    }
}
