//! `BearDog` Integration Traits
//!
//! Modern, idiomatic Rust traits for integrating `BearDog` security with Songbird.
//!
//! **Architecture**: Songbird defines the traits, `BearDog` implements them.
//! **Pattern**: Dependency inversion - Songbird doesn't depend on `BearDog` code.

pub mod birdsong;
pub mod genesis;
pub mod lineage;
pub mod relay;

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
    #[cfg(test)]
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
            // TODO: Create actual HTTP BearDog client implementation
            // For now, return no-op that explicitly errors (graceful degradation)
            tracing::warn!("BearDog discovered but HTTP client not yet implemented - using NoOp provider");
            return Ok(Some(Self::create_noop()));
        }

        Ok(None)
    }

    async fn discover_via_env() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // Check BEARDOG_URL or SECURITY_URL environment variable
        if std::env::var("BEARDOG_URL").is_ok() || std::env::var("SECURITY_URL").is_ok() {
            let url = std::env::var("BEARDOG_URL").or_else(|_| std::env::var("SECURITY_URL"))?;
            tracing::info!("Found BearDog via environment at: {}", url);
            // TODO: Create actual HTTP BearDog client implementation
            tracing::warn!("BearDog URL configured but HTTP client not yet implemented - using NoOp provider");
            return Ok(Some(Self::create_noop()));
        }

        Ok(None)
    }

    async fn discover_via_wellknown() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // Development fallback: Try localhost:8200 (only in debug builds)
        #[cfg(debug_assertions)]
        {
            let default_url = "http://[::]:8200";
            tracing::warn!("Using development fallback for BearDog: {}", default_url);
            tracing::warn!("Set BEARDOG_URL or SECURITY_URL for production");
            // TODO: Create actual HTTP BearDog client implementation
            tracing::warn!("Development mode: HTTP client not yet implemented - using NoOp provider");
            Ok(Some(Self::create_noop()))
        }

        #[cfg(not(debug_assertions))]
        {
            // Production: No fallback
            tracing::error!(
                "BearDog not found. Set BEARDOG_URL or SECURITY_URL environment variable"
            );
            Ok(None)
        }
    }
}

// No-Op implementation for production when BearDog unavailable
pub mod noop;

// Mock implementation - TEST ONLY
#[cfg(test)]
pub mod mock;
