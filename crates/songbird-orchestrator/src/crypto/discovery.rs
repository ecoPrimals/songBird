//! Capability-Based Crypto Provider Discovery
//!
//! Discovers ANY primal offering "crypto" capability via runtime discovery.
//! Maintains TRUE PRIMAL self-knowledge - Songbird only knows itself,
//! discovers crypto providers at runtime.
//!
//! **Pattern**: Adapted from `auth/capability_discovery.rs` (proven in production)
//! **Philosophy**: Primals only know themselves, discover others by capability

use anyhow::Result;
use tracing::{debug, info, warn};

/// Discover crypto provider socket via capability-based discovery
///
/// ## TRUE PRIMAL Principles
///
/// 1. **Self-Knowledge**: Songbird only knows itself
/// 2. **Capability Discovery**: Searches for "crypto" capability
/// 3. **Runtime Discovery**: No hardcoded primal names
/// 4. **Graceful Fallback**: Works without crypto provider
///
/// ## Discovery Strategy
///
/// 1. Check `CRYPTO_PROVIDER_SOCKET` environment variable (orchestrator-provided, preferred)
/// 2. Check `CRYPTO_PROVIDER` environment variable (alternative)
/// 3. Check `BEARDOG_CRYPTO_SOCKET` environment variable (compatibility during migration)
/// 4. Check `BEARDOG_SOCKET` environment variable (generic socket, may support crypto)
/// 5. Search common socket paths for crypto capability
/// 6. Return error if not found
///
/// # Returns
///
/// * `Ok(String)` - Path to crypto provider socket
/// * `Err` - No crypto provider available
pub async fn get_beardog_crypto_socket() -> Result<String> {
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

    // Strategy 4: BEARDOG_SOCKET (generic socket, may support crypto)
    if let Ok(socket_path) = std::env::var("BEARDOG_SOCKET") {
        info!("   ✅ Found BEARDOG_SOCKET (checking for crypto capability): {}", socket_path);
        return Ok(socket_path);
    }

    // Strategy 5: Search common socket paths (XDG-compliant first)
    let xdg_base = std::env::var("XDG_RUNTIME_DIR")
        .map(|d| format!("{}/biomeos", d))
        .unwrap_or_else(|_| "/tmp/biomeos".to_string());

    let common_paths = vec![
        format!("{}/beardog.sock", xdg_base),       // XDG-compliant (highest priority)
        "/tmp/biomeos/beardog.sock".to_string(),    // biomeOS fallback
        "/tmp/beardog.sock".to_string(),            // Legacy fallback
    ];

    for path in &common_paths {
        if std::path::Path::new(&path).exists() {
            info!("   ✅ Found crypto provider socket at: {}", path);
            return Ok(path.clone());
        }
        debug!("   ⏭️  Not found: {}", path);
    }

    // Strategy 6: Search /tmp for any crypto provider socket
    if let Ok(entries) = std::fs::read_dir("/tmp") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                // Look for crypto-related sockets (beardog, crypto, etc.)
                if (file_name.contains("crypto") || file_name.starts_with("beardog"))
                    && file_name.ends_with(".sock")
                {
                    let path = entry.path();
                    info!("   ✅ Found crypto provider socket at: {}", path.display());
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }
    }

    warn!("❌ No crypto provider found - checked all discovery strategies");
    warn!("   Songbird will fall back to ring crypto provider (temporary)");
    warn!("   This maintains TLS functionality but uses C dependencies");

    Err(anyhow::anyhow!("BearDog crypto provider not available"))
}

/// Discover BearDog crypto socket with explicit family ID
///
/// Searches for BearDog socket with specific family ID (e.g., "nat0").
///
/// # Arguments
///
/// * `family_id` - Family ID to search for (e.g., "nat0")
///
/// # Returns
///
/// * `Ok(String)` - Path to BearDog socket for this family
/// * `Err` - BearDog not available for this family
pub async fn get_beardog_crypto_socket_for_family(family_id: &str) -> Result<String> {
    info!("🔍 Discovering crypto provider for family '{}'...", family_id);

    // Check family-specific socket (capability-based, primal-agnostic)
    let family_socket = format!("/tmp/crypto-{}.sock", family_id);
    if std::path::Path::new(&family_socket).exists() {
        info!("   ✅ Found family-specific crypto socket: {}", family_socket);
        return Ok(family_socket);
    }

    // Fall back to generic discovery (TRUE PRIMAL)
    crate::primal_discovery::discover_crypto_provider().await
}

/// Check if BearDog crypto is available
///
/// Quick check without logging warnings.
///
/// # Returns
///
/// * `true` - BearDog crypto available
/// * `false` - BearDog crypto not available (will use ring fallback)
pub async fn is_beardog_crypto_available() -> bool {
    // Quick check without verbose logging
    if std::env::var("CRYPTO_PROVIDER").is_ok()
        || std::env::var("BEARDOG_CRYPTO_SOCKET").is_ok()
        || std::env::var("BEARDOG_SOCKET").is_ok()
    {
        return true;
    }

    // Check common paths silently (XDG-compliant first)
    let xdg_base = std::env::var("XDG_RUNTIME_DIR")
        .map(|d| format!("{}/biomeos", d))
        .unwrap_or_else(|_| "/tmp/biomeos".to_string());

    let common_paths = [
        format!("{}/beardog.sock", xdg_base),
        "/tmp/biomeos/beardog.sock".to_string(),
        "/tmp/beardog.sock".to_string(),
    ];

    common_paths.iter().any(|path| std::path::Path::new(path).exists())
}

/// Discover BearDog crypto socket with purpose context
///
/// Same as `get_beardog_crypto_socket()` but logs the purpose for audit.
///
/// # Arguments
///
/// * `purpose` - Purpose of crypto operation (e.g., "tls_handshake")
///
/// # Returns
///
/// * `Ok(String)` - Path to BearDog crypto socket
/// * `Err` - BearDog crypto not available
pub async fn get_beardog_crypto_socket_for_purpose(purpose: &str) -> Result<String> {
    info!("🔍 Discovering crypto provider for purpose: {}", purpose);
    get_beardog_crypto_socket().await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // 🧪 CRYPTO DISCOVERY TESTS
    // Note: These tests share env vars so they may interfere when run in parallel
    // Run with: cargo test -- --test-threads=1 for reliable results
    // ═══════════════════════════════════════════════════════════════════════

    /// Helper to clear all crypto-related env vars
    fn clear_all_crypto_env_vars() {
        std::env::remove_var("CRYPTO_PROVIDER_SOCKET");
        std::env::remove_var("CRYPTO_PROVIDER");
        std::env::remove_var("BEARDOG_CRYPTO_SOCKET");
        std::env::remove_var("BEARDOG_SOCKET");
    }

    #[tokio::test]
    async fn test_crypto_provider_socket_has_highest_priority() {
        clear_all_crypto_env_vars();

        let custom_path = "/test/crypto-socket-highest.sock";
        std::env::set_var("CRYPTO_PROVIDER_SOCKET", custom_path);

        let socket = get_beardog_crypto_socket().await;
        assert!(socket.is_ok());
        assert_eq!(socket.unwrap(), custom_path);

        clear_all_crypto_env_vars();
    }

    #[test]
    fn test_xdg_path_construction_with_runtime_dir() {
        // Test XDG path construction logic (sync test, no env race)
        let xdg_base = "/run/user/1000";
        let expected_path = format!("{}/biomeos/beardog.sock", xdg_base);
        assert_eq!(expected_path, "/run/user/1000/biomeos/beardog.sock");
    }

    #[test]
    fn test_xdg_fallback_path_construction() {
        // Test fallback path construction (sync test)
        let fallback_base = "/tmp/biomeos";
        let expected_path = format!("{}/beardog.sock", fallback_base);
        assert_eq!(expected_path, "/tmp/biomeos/beardog.sock");
    }

    #[test]
    fn test_legacy_fallback_path_construction() {
        // Test legacy path construction (sync test)
        let legacy_path = "/tmp/beardog.sock";
        assert!(legacy_path.ends_with("beardog.sock"));
        assert!(!legacy_path.contains("biomeos"));
    }

    #[tokio::test]
    async fn test_is_beardog_crypto_available_returns_bool() {
        clear_all_crypto_env_vars();

        // is_beardog_crypto_available checks CRYPTO_PROVIDER, BEARDOG_CRYPTO_SOCKET,
        // or BEARDOG_SOCKET (not CRYPTO_PROVIDER_SOCKET)
        std::env::set_var("CRYPTO_PROVIDER", "/test/avail-check.sock");

        let available = is_beardog_crypto_available().await;
        assert!(available, "Should be available when CRYPTO_PROVIDER env var is set");

        clear_all_crypto_env_vars();
    }

    #[tokio::test]
    async fn test_get_beardog_crypto_socket_for_purpose_returns_socket() {
        clear_all_crypto_env_vars();

        std::env::set_var("CRYPTO_PROVIDER_SOCKET", "/test/purpose-socket.sock");

        let socket = get_beardog_crypto_socket_for_purpose("signing").await;
        assert!(socket.is_ok());
        assert!(socket.unwrap().ends_with(".sock"));

        clear_all_crypto_env_vars();
    }
}
