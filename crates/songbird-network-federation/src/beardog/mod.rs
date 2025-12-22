//! BearDog Integration Traits
//!
//! Modern, idiomatic Rust traits for integrating BearDog security with Songbird.
//!
//! **Architecture**: Songbird defines the traits, BearDog implements them.
//! **Pattern**: Dependency inversion - Songbird doesn't depend on BearDog code.

pub mod birdsong;
pub mod genesis;
pub mod lineage;
pub mod relay;

pub use birdsong::{BirdSongCrypto, BroadcastKey, EncryptedBirdSong, LineageHint};
pub use lineage::{LineageChain, LineageLink, LineageProof, LineageProvider};
pub use relay::{AccessLevel, LineageRelay, RelaySession};

/// BearDog provider that combines all capabilities
///
/// This is the main interface Songbird uses to interact with BearDog.
/// BearDog will implement this trait, providing all three capabilities.
#[async_trait::async_trait]
pub trait BearDogProvider: LineageProvider + BirdSongCrypto + LineageRelay + Send + Sync {
    /// Check if BearDog is available and operational
    async fn is_available(&self) -> bool;

    /// Get BearDog version for compatibility checking
    fn version(&self) -> &str;

    /// Graceful shutdown
    async fn shutdown(&self) -> anyhow::Result<()>;
}

/// Factory for discovering and creating BearDog providers
///
/// Supports multiple discovery strategies:
/// 1. UPA (Universal Port Authority) - query for "security" capability
/// 2. Environment variable - BEARDOG_URL
/// 3. Well-known port - localhost:8200
/// 4. Mock provider - for testing without BearDog
pub struct BearDogProviderFactory;

impl BearDogProviderFactory {
    /// Discover BearDog via multiple strategies
    ///
    /// Returns None if BearDog is not available (graceful degradation)
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

    /// Create mock provider for testing
    pub fn create_mock() -> Box<dyn BearDogProvider> {
        use crate::beardog::mock::MockBearDogProvider;
        Box::new(MockBearDogProvider::new())
    }

    async fn discover_via_upa() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // TODO: Query UPA for "security" capability
        Ok(None)
    }

    async fn discover_via_env() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // TODO: Check BEARDOG_URL environment variable
        Ok(None)
    }

    async fn discover_via_wellknown() -> anyhow::Result<Option<Box<dyn BearDogProvider>>> {
        // TODO: Try localhost:8200
        Ok(None)
    }
}

pub mod mock;
