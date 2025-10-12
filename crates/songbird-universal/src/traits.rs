//! # 🔧 Universal Adapter Traits
//!
//! **CANONICAL TRAIT SYSTEM** ✅
//!
//! This module provides universal adapter traits that use the canonical
//! trait system from songbird-types.

use async_trait::async_trait;

// Re-export canonical traits from songbird-types
pub use songbird_types::traits::canonical::{
    CapabilityProvider, DiscoveryProvider, ObservabilityProvider, OrchestrationProvider,
    PrimalProvider, Provider, SecurityProvider, ServiceProvider,
};

// Re-export canonical types
pub use songbird_types::{SongbirdError, SongbirdResult};

/// Universal adapter trait for protocol-agnostic communication
///
/// This trait extends the canonical Provider trait with universal adapter functionality.
#[async_trait]
pub trait UniversalAdapter: Provider {
    /// Discover available services
    async fn discover_services(&self) -> SongbirdResult<Vec<crate::types::ServiceInfo>>;

    /// Send request to service
    async fn send_request(
        &self,
        request: crate::types::UniversalRequest,
    ) -> SongbirdResult<crate::types::UniversalResponse>;

    /// Register service provider
    async fn register_provider(&mut self, provider: Box<dyn ServiceProvider>)
        -> SongbirdResult<()>;
}
