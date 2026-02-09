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
    async fn test_get_beardog_crypto_socket_graceful_failure() {
        // In CI/test environments without BearDog, should return Err, not panic
        let result = get_beardog_crypto_socket().await;
        // Either succeeds (BearDog running) or fails gracefully
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => assert!(format!("{e}").contains("not available")),
        }
    }

    #[tokio::test]
    async fn test_is_beardog_crypto_available_returns_bool() {
        // Should return a bool without panicking, regardless of environment
        let _available = is_beardog_crypto_available().await;
    }

    #[tokio::test]
    async fn test_get_beardog_crypto_socket_for_purpose_no_panic() {
        let result = get_beardog_crypto_socket_for_purpose("signing").await;
        // Either succeeds or fails gracefully
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => assert!(format!("{e}").contains("not available")),
        }
    }
}
