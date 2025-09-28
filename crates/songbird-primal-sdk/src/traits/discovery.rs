//! Primal discovery and registry traits

use super::provider::PrimalProviderEnum;
use songbird_types::{errors::SongbirdResult, CanonicalPrimalType};

/// Trait for discovering primal services (now uses concrete enum)
pub trait PrimalDiscovery: Send + Sync  {/// Discover primals by type
    fn discover_by_type(
        &self)
        primal_type: CanonicalPrimalType,
    ) -> impl std::future::Future<Output = SongbirdResult<Vec<PrimalProviderEnum>>> + Send;
}
pub trait PrimalRegistry: Send + Sync {
    /// Register a primal provider
    fn register(&self) -> impl std::future::Future<Output = SongbirdResult<()>> + Send;
}
