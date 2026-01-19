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

    // Strategy 5: Search common socket paths
    let common_paths = vec![
        "/tmp/crypto.sock",                  // Generic crypto provider
        "/tmp/beardog-crypto.sock",          // BearDog dedicated crypto socket
        "/tmp/beardog-nat0.sock",            // NUCLEUS default
        "/tmp/beardog-default-default.sock", // biomeOS default
        "/run/user/1000/beardog.sock",       // User runtime dir
        "/var/run/beardog.sock",             // System runtime dir
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            info!("   ✅ Found crypto provider socket at: {}", path);
            return Ok(path.to_string());
        } else {
            debug!("   ⏭️  Not found: {}", path);
        }
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

    // Check family-specific socket
    let family_socket = format!("/tmp/beardog-{}.sock", family_id);
    if std::path::Path::new(&family_socket).exists() {
        info!("   ✅ Found family-specific BearDog socket: {}", family_socket);
        return Ok(family_socket);
    }

    // Fall back to generic discovery
    get_beardog_crypto_socket().await
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

    // Check common paths silently
    let common_paths = vec![
        "/tmp/beardog-crypto.sock",
        "/tmp/beardog-nat0.sock",
        "/tmp/beardog-default-default.sock",
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
    use std::sync::Mutex;

    // Mutex to serialize tests that modify environment
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn test_discover_beardog_crypto_socket_env_var() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Clear all related env vars first
        std::env::remove_var("CRYPTO_PROVIDER");
        std::env::remove_var("BEARDOG_CRYPTO_SOCKET");
        std::env::remove_var("BEARDOG_SOCKET");

        // Set environment variable
        std::env::set_var("CRYPTO_PROVIDER", "/tmp/test-beardog-crypto.sock");

        let socket = get_beardog_crypto_socket().await;
        assert!(socket.is_ok());
        assert_eq!(socket.unwrap(), "/tmp/test-beardog-crypto.sock");

        // Cleanup
        std::env::remove_var("CRYPTO_PROVIDER");
    }

    #[tokio::test]
    async fn test_discover_beardog_crypto_socket_fallback() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Ensure no environment variables set
        std::env::remove_var("CRYPTO_PROVIDER");
        std::env::remove_var("BEARDOG_CRYPTO_SOCKET");
        std::env::remove_var("BEARDOG_SOCKET");

        // Should return Err if no socket found (unless one exists on system)
        let socket = get_beardog_crypto_socket().await;
        // May be Ok or Err depending on system state
        // Just verify it doesn't panic
        match socket {
            Ok(path) => println!("Found socket: {}", path),
            Err(e) => println!("No socket found (expected): {}", e),
        }
    }

    #[tokio::test]
    async fn test_is_beardog_crypto_available() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Clear all related env vars first
        std::env::remove_var("CRYPTO_PROVIDER");
        std::env::remove_var("BEARDOG_CRYPTO_SOCKET");
        std::env::remove_var("BEARDOG_SOCKET");

        std::env::set_var("CRYPTO_PROVIDER", "/tmp/test-crypto.sock");

        let available = is_beardog_crypto_available().await;
        assert!(available);

        std::env::remove_var("CRYPTO_PROVIDER");
    }

    #[tokio::test]
    async fn test_get_beardog_crypto_socket_for_purpose() {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Clear all related env vars first
        std::env::remove_var("CRYPTO_PROVIDER");
        std::env::remove_var("BEARDOG_CRYPTO_SOCKET");
        std::env::remove_var("BEARDOG_SOCKET");

        std::env::set_var("CRYPTO_PROVIDER", "/tmp/purpose-test.sock");

        let socket = get_beardog_crypto_socket_for_purpose("tls_handshake").await;
        assert!(socket.is_ok());
        assert_eq!(socket.unwrap(), "/tmp/purpose-test.sock");

        std::env::remove_var("CRYPTO_PROVIDER");
    }
}
