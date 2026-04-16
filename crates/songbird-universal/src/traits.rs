// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
#![allow(async_fn_in_trait, reason = "native async trait methods; not used as trait objects")]

//! # 🔧 Universal Adapter Traits
//!
//! **CANONICAL TRAIT SYSTEM** ✅
//!
//! This module provides universal adapter traits that use the canonical
//! trait system from songbird-types.

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
pub trait UniversalAdapter: Provider {
    /// Discover available services
    async fn discover_services(&self) -> SongbirdResult<Vec<crate::types::ServiceInfo>>;

    /// Send request to service
    async fn send_request(
        &self,
        request: crate::types::UniversalRequest,
    ) -> SongbirdResult<crate::types::UniversalResponse>;

    /// Register service provider
    async fn register_provider<P: ServiceProvider + Send + 'static>(
        &mut self,
        provider: Box<P>,
    ) -> SongbirdResult<()>;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use crate::traits::{SongbirdError, SongbirdResult};

    #[test]
    fn songbird_result_ok_unit() {
        let r: SongbirdResult<()> = Ok(());
        assert!(r.is_ok());
    }

    #[test]
    fn songbird_error_configuration_display() {
        let e = SongbirdError::configuration("unit test");
        assert!(!e.to_string().is_empty());
    }
}
