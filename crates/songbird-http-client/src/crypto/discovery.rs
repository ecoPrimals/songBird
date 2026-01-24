//! Crypto Capability Discovery
//!
//! Provides runtime discovery of crypto providers, eliminating hardcoded
//! socket paths and enabling capability-based communication.
//!
//! ## Discovery Order
//!
//! 1. `CRYPTO_CAPABILITY_SOCKET` environment variable
//! 2. `BEARDOG_SOCKET` environment variable (backward compatibility)
//! 3. Neural API capability query (future)
//! 4. Well-known default `/tmp/beardog.sock`
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::crypto::discover_crypto_capability;
//!
//! let crypto = discover_crypto_capability().await?;
//! ```

use std::sync::Arc;
use tracing::{debug, info, warn};

use super::beardog_provider::BearDogProvider;
use super::capability::CryptoCapability;
use crate::error::{Error, Result};

/// Well-known default socket paths (in order of preference)
const DEFAULT_SOCKET_PATHS: &[&str] =
    &["/tmp/beardog.sock", "/run/user/1000/beardog-default.sock", "/var/run/beardog.sock"];

/// Discover crypto capability at runtime
///
/// Tries multiple discovery methods in order:
/// 1. Environment variable `CRYPTO_CAPABILITY_SOCKET`
/// 2. Environment variable `BEARDOG_SOCKET` (backward compat)
/// 3. Well-known default socket paths
///
/// # Errors
///
/// Returns error if no provider can be discovered.
///
/// # Example
///
/// ```rust,ignore
/// let crypto = discover_crypto_capability().await?;
/// let (pub_key, priv_key) = crypto.generate_x25519_keypair().await?;
/// ```
pub async fn discover_crypto_capability() -> Result<Arc<dyn CryptoCapability>> {
    info!("🔍 Discovering crypto capability provider...");

    // 1. Try CRYPTO_CAPABILITY_SOCKET env var
    if let Ok(socket_path) = std::env::var("CRYPTO_CAPABILITY_SOCKET") {
        info!("   Found CRYPTO_CAPABILITY_SOCKET: {}", socket_path);
        let provider = BearDogProvider::new(&socket_path);
        if provider.is_available().await {
            info!("✅ Using crypto provider at {} (via CRYPTO_CAPABILITY_SOCKET)", socket_path);
            return Ok(Arc::new(provider));
        }
        warn!("⚠️  CRYPTO_CAPABILITY_SOCKET set but provider not available");
    }

    // 2. Try BEARDOG_SOCKET env var (backward compatibility)
    if let Ok(socket_path) = std::env::var("BEARDOG_SOCKET") {
        info!("   Found BEARDOG_SOCKET: {}", socket_path);
        let provider = BearDogProvider::new(&socket_path);
        if provider.is_available().await {
            info!("✅ Using crypto provider at {} (via BEARDOG_SOCKET)", socket_path);
            return Ok(Arc::new(provider));
        }
        warn!("⚠️  BEARDOG_SOCKET set but provider not available");
    }

    // 3. Try well-known default paths
    for socket_path in DEFAULT_SOCKET_PATHS {
        debug!("   Trying well-known path: {}", socket_path);

        // Check if socket file exists
        if !std::path::Path::new(socket_path).exists() {
            debug!("   Socket not found: {}", socket_path);
            continue;
        }

        let provider = BearDogProvider::new(*socket_path);
        if provider.is_available().await {
            info!("✅ Using crypto provider at {} (well-known path)", socket_path);
            return Ok(Arc::new(provider));
        }
        debug!("   Socket exists but provider not responding: {}", socket_path);
    }

    // 4. Future: Try Neural API capability query
    // TODO: Implement when Neural API is available
    // if let Some(provider) = discover_via_neural_api().await {
    //     return Ok(provider);
    // }

    Err(Error::BearDogRpc(
        "No crypto capability provider found. Set CRYPTO_CAPABILITY_SOCKET or ensure BearDog is running.".to_string()
    ))
}

/// Discover crypto capability with explicit socket path
///
/// Use this when you know the socket path but want the `CryptoCapability` trait.
///
/// # Example
///
/// ```rust,ignore
/// let crypto = discover_crypto_capability_at("/custom/path/crypto.sock").await?;
/// ```
pub async fn discover_crypto_capability_at(socket_path: &str) -> Result<Arc<dyn CryptoCapability>> {
    let provider = BearDogProvider::new(socket_path);

    if provider.is_available().await {
        Ok(Arc::new(provider))
    } else {
        Err(Error::BearDogRpc(format!("Crypto provider at {} is not available", socket_path)))
    }
}

/// Create crypto capability without availability check (for testing)
///
/// Use this in tests where you control the provider lifecycle.
pub fn create_crypto_capability(socket_path: &str) -> Arc<dyn CryptoCapability> {
    Arc::new(BearDogProvider::new(socket_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_crypto_capability() {
        let crypto = create_crypto_capability("/tmp/test.sock");
        assert_eq!(crypto.name(), "BearDog");
    }

    #[tokio::test]
    async fn test_discover_fails_gracefully() {
        // Should fail gracefully when no provider available
        std::env::remove_var("CRYPTO_CAPABILITY_SOCKET");
        std::env::remove_var("BEARDOG_SOCKET");

        // This will fail if no BearDog is running, which is expected in unit tests
        let result = discover_crypto_capability_at("/nonexistent/path.sock").await;
        assert!(result.is_err());
    }
}
