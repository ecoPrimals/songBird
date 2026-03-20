// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` Integration Traits
//!
//! Modern, idiomatic Rust traits for integrating `BearDog` security with Songbird.
//!
//! **Architecture**: Songbird defines the traits, `BearDog` implements them.
//! **Pattern**: Dependency inversion - Songbird doesn't depend on `BearDog` code.

pub mod birdsong;
pub mod genesis;
pub mod lineage;
pub mod noop;
pub mod production;
pub mod relay;

#[cfg(any(test, feature = "test-mocks"))]
pub mod mock;

pub use birdsong::{BirdSongCrypto, BroadcastKey, EncryptedBirdSong, LineageHint};
pub use lineage::{LineageChain, LineageLink, LineageProof, LineageProvider};
pub use relay::{AccessLevel, LineageRelay, RelaySession};

/// `BearDog` provider that combines all capabilities
///
/// This is the main interface Songbird uses to interact with `BearDog`.
/// `BearDog` will implement this trait, providing all three capabilities.
#[async_trait::async_trait]
pub trait BearDogProvider: LineageProvider + BirdSongCrypto + LineageRelay + Send + Sync {
    /// Check if `BearDog` is available and operational
    async fn is_available(&self) -> bool;

    /// Get `BearDog` version for compatibility checking
    fn version(&self) -> &str;

    /// Graceful shutdown
    async fn shutdown(&self) -> anyhow::Result<()>;
}

/// Factory for discovering and creating `BearDog` providers
///
/// Supports multiple discovery strategies:
/// 1. UPA (Universal Port Authority) - query for "security" capability
/// 2. Environment variable - `BEARDOG_URL`
/// 3. Well-known port - localhost:8200
/// 4. Mock provider - for testing without `BearDog`
pub struct BearDogProviderFactory;

impl BearDogProviderFactory {
    /// Discover `BearDog` via multiple strategies
    ///
    /// Returns None if `BearDog` is not available (graceful degradation)
    pub async fn discover() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // Strategy 1: Check UPA for "security" capability
        if let Ok(Some(provider)) = Self::discover_via_upa().await {
            tracing::info!("🐻 BearDog discovered via UPA");
            return Ok(Some(provider));
        }

        // Strategy 2: Check environment variable
        if let Ok(Some(provider)) = Self::discover_via_env().await {
            tracing::info!("🐻 BearDog discovered via environment");
            return Ok(Some(provider));
        }

        // Strategy 3: Check well-known port
        if let Ok(Some(provider)) = Self::discover_via_wellknown().await {
            tracing::info!("🐻 BearDog discovered via well-known port");
            return Ok(Some(provider));
        }

        // No BearDog available - graceful degradation
        tracing::warn!("🐻 BearDog not available, running without encryption");
        Ok(None)
    }

    /// Create no-op provider when `BearDog` is unavailable
    ///
    /// This is NOT a mock - it returns clear errors for all operations.
    /// Use in production for graceful degradation when security features are optional.
    #[must_use]
    pub fn create_noop() -> Box<dyn BearDogProvider> {
        use crate::beardog::noop::NoOpBearDogProvider;
        Box::new(NoOpBearDogProvider::new())
    }

    /// Create mock provider for testing
    #[cfg(any(test, feature = "test-mocks"))]
    #[must_use]
    pub fn create_mock() -> Box<dyn BearDogProvider> {
        use crate::beardog::mock::MockBearDogProvider;
        Box::new(MockBearDogProvider::new())
    }

    async fn discover_via_upa() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        use songbird_config::discovery_helpers::discover_primal;
        use songbird_types::CanonicalPrimalType;

        // Query capability registry for "security" capability (BearDog)
        if let Ok(endpoint) = discover_primal(CanonicalPrimalType::Security).await {
            tracing::info!("Discovered BearDog via capability discovery at: {}", endpoint.url);

            // Extract Unix socket path from URL
            if let Some(socket_path) = endpoint.url.strip_prefix("unix://") {
                match crate::beardog::production::ProductionBearDogProvider::new(socket_path).await
                {
                    Ok(provider) => {
                        tracing::info!("✅ Connected to BearDog via Unix socket: {}", socket_path);
                        return Ok(Some(Box::new(provider)));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to discovered BearDog: {}", e);
                    }
                }
            } else {
                tracing::warn!("BearDog endpoint is not a Unix socket URL: {}", endpoint.url);
            }
        }

        Ok(None)
    }

    async fn discover_via_env() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // Check BEARDOG_SOCKET first (preferred for Unix sockets)
        if let Ok(socket_path) = std::env::var("BEARDOG_SOCKET") {
            tracing::info!("Using BearDog socket from BEARDOG_SOCKET: {}", socket_path);
            match crate::beardog::production::ProductionBearDogProvider::new(&socket_path).await {
                Ok(provider) => return Ok(Some(Box::new(provider))),
                Err(e) => tracing::warn!("Failed to connect to BEARDOG_SOCKET: {}", e),
            }
        }

        // Check SECURITY_SOCKET (generic)
        if let Ok(socket_path) = std::env::var("SECURITY_SOCKET") {
            tracing::info!("Using BearDog socket from SECURITY_SOCKET: {}", socket_path);
            match crate::beardog::production::ProductionBearDogProvider::new(&socket_path).await {
                Ok(provider) => return Ok(Some(Box::new(provider))),
                Err(e) => tracing::warn!("Failed to connect to SECURITY_SOCKET: {}", e),
            }
        }

        // Legacy: Check URL-based env vars (convert to socket if possible)
        if std::env::var("BEARDOG_URL").is_ok() || std::env::var("SECURITY_URL").is_ok() {
            let url = std::env::var("BEARDOG_URL").or_else(|_| std::env::var("SECURITY_URL"))?;
            tracing::info!("Found BearDog via environment at: {}", url);

            // Try to extract Unix socket path from URL
            if let Some(socket_path) = url.strip_prefix("unix://") {
                match crate::beardog::production::ProductionBearDogProvider::new(socket_path).await
                {
                    Ok(provider) => return Ok(Some(Box::new(provider))),
                    Err(e) => tracing::warn!("Failed to connect via URL: {}", e),
                }
            } else {
                tracing::warn!("BearDog URL is not a Unix socket URL: {}", url);
                tracing::warn!("Use BEARDOG_SOCKET for Unix socket paths, or prefix with unix://");
            }
        }

        Ok(None)
    }

    async fn discover_via_wellknown() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // Development fallback: Try /tmp/beardog.sock (only in debug builds)
        #[cfg(debug_assertions)]
        {
            let default_socket = "/tmp/beardog.sock";
            if std::path::Path::new(default_socket).exists() {
                tracing::warn!("Using development fallback for BearDog: {}", default_socket);
                tracing::warn!("Set BEARDOG_SOCKET or SECURITY_SOCKET for production");
                match crate::beardog::production::ProductionBearDogProvider::new(default_socket)
                    .await
                {
                    Ok(provider) => return Ok(Some(Box::new(provider))),
                    Err(e) => tracing::warn!("Failed to connect to default socket: {}", e),
                }
            }
        }

        #[cfg(not(debug_assertions))]
        {
            // Production: No fallback
            tracing::error!(
                "BearDog not found. Set BEARDOG_SOCKET or SECURITY_SOCKET environment variable"
            );
        }

        Ok(None)
    }
}
