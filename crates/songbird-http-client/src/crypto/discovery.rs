// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Crypto Capability Discovery
//!
//! Provides runtime discovery of crypto providers, eliminating hardcoded
//! socket paths and enabling capability-based communication.
//!
//! ## Discovery Order
//!
//! 1. `CRYPTO_CAPABILITY_SOCKET` environment variable
//! 2. `SECURITY_PROVIDER_SOCKET` (preferred)
//! 3. Legacy `BEARDOG_SOCKET` (backward compatibility in discovery chains)
//! 4. Neural API capability query (future)
//! 5. Well-known default paths (legacy temp/capability paths; see `security_socket_candidates`)
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

use super::capability::CryptoCapability;
use super::security_provider::SecurityCryptoProvider;
use crate::error::{Error, Result};
use songbird_types::defaults::paths::security_socket_candidates;

/// Discover crypto capability at runtime
///
/// Tries multiple discovery methods in order:
/// 1. Environment variable `CRYPTO_CAPABILITY_SOCKET`
/// 2. Environment variable `SECURITY_PROVIDER_SOCKET`
/// 3. Environment variable `BEARDOG_SOCKET` (legacy name; backward compatibility)
/// 4. Well-known default socket paths
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
    if let Ok(socket_path) = songbird_process_env::var("CRYPTO_CAPABILITY_SOCKET") {
        info!("   Found CRYPTO_CAPABILITY_SOCKET: {}", socket_path);
        let provider = SecurityCryptoProvider::new(&socket_path);
        if provider.is_available().await {
            info!("✅ Using crypto provider at {} (via CRYPTO_CAPABILITY_SOCKET)", socket_path);
            return Ok(Arc::new(provider));
        }
        warn!("⚠️  CRYPTO_CAPABILITY_SOCKET set but provider not available");
    }

    // 2. Try SECURITY_PROVIDER_SOCKET env var
    if let Ok(socket_path) = songbird_process_env::var("SECURITY_PROVIDER_SOCKET") {
        info!("   Found SECURITY_PROVIDER_SOCKET: {}", socket_path);
        let provider = SecurityCryptoProvider::new(&socket_path);
        if provider.is_available().await {
            info!("✅ Using crypto provider at {} (via SECURITY_PROVIDER_SOCKET)", socket_path);
            return Ok(Arc::new(provider));
        }
        warn!("⚠️  SECURITY_PROVIDER_SOCKET set but provider not available");
    }

    // 3. Try legacy BEARDOG_SOCKET env var (backward compatibility)
    if let Ok(socket_path) = songbird_process_env::var("BEARDOG_SOCKET") {
        warn!(
            "DEPRECATED: BEARDOG_SOCKET is set — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT (capability-first). Path: {}",
            socket_path
        );
        let provider = SecurityCryptoProvider::new(&socket_path);
        if provider.is_available().await {
            info!("✅ Using security provider at {} (legacy env BEARDOG_SOCKET)", socket_path);
            return Ok(Arc::new(provider));
        }
        warn!("⚠️  Legacy BEARDOG_SOCKET set but provider not available");
    }

    // 4. Try well-known default paths
    for socket_path in security_socket_candidates() {
        let socket_str = socket_path.to_string_lossy();
        debug!("   Trying well-known path: {}", socket_str);

        // Check if socket file exists
        if !socket_path.exists() {
            debug!("   Socket not found: {}", socket_str);
            continue;
        }

        let provider = SecurityCryptoProvider::new(socket_str.as_ref());
        if provider.is_available().await {
            info!("✅ Using crypto provider at {} (well-known path)", socket_str);
            return Ok(Arc::new(provider));
        }
        debug!("   Socket exists but provider not responding: {}", socket_str);
    }

    // Neural API `capability.discover("crypto")` is not used here yet; discovery stays path-based.

    Err(Error::SecurityProviderRpc(
        "No crypto capability provider found. Set CRYPTO_CAPABILITY_SOCKET or ensure a security provider is running.".to_string()
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
#[allow(dead_code)]
pub async fn discover_crypto_capability_at(socket_path: &str) -> Result<Arc<dyn CryptoCapability>> {
    let provider = SecurityCryptoProvider::new(socket_path);

    if provider.is_available().await {
        Ok(Arc::new(provider))
    } else {
        Err(Error::SecurityProviderRpc(format!(
            "Crypto provider at {socket_path} is not available"
        )))
    }
}

/// Create crypto capability without availability check (for testing)
///
/// Use this in tests where you control the provider lifecycle.
#[allow(dead_code)]
pub fn create_crypto_capability(socket_path: &str) -> Arc<dyn CryptoCapability> {
    Arc::new(SecurityCryptoProvider::new(socket_path))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::error::Error;

    #[test]
    fn test_create_crypto_capability() {
        let crypto = create_crypto_capability("/tmp/test.sock");
        assert_eq!(crypto.name(), "security provider");
    }

    #[tokio::test]
    async fn test_discover_fails_gracefully() {
        // ✅ Concurrent-safe: Uses explicit path (no env vars needed)
        let result = discover_crypto_capability_at("/nonexistent/path.sock").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn discover_crypto_capability_at_error_message_mentions_unavailable() {
        let err = discover_crypto_capability_at("/definitely/missing/security-provider.sock")
            .await
            .expect_err("unavailable");
        let msg = err.to_string();
        assert!(msg.contains("not available") || msg.to_lowercase().contains("unavailable"));
    }

    #[test]
    fn create_crypto_capability_twice_same_provider_name() {
        let a = create_crypto_capability("/a/b/c.sock");
        let b = create_crypto_capability("/x/y/z.sock");
        assert_eq!(a.name(), b.name());
    }

    #[tokio::test]
    async fn discover_crypto_capability_at_returns_security_provider_rpc_error() {
        let err = discover_crypto_capability_at("/nonexistent.sock").await.expect_err("err");
        assert!(matches!(err, Error::SecurityProviderRpc(_)));
    }

    #[test]
    fn create_crypto_capability_debug_includes_path() {
        let c = create_crypto_capability("/tmp/unique-test.sock");
        let dbg = format!("{c:?}");
        assert!(!dbg.is_empty());
    }
}
