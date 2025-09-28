//! Primal SDK Traits
//!
//! **MODERNIZED**: Now uses canonical Provider traits from songbird-types

// Re-export canonical traits instead of defining our own
pub use songbird_types::traits::canonical::{
    Provider,
    ServiceProvider,
    PrimalProvider,
    CapabilityProvider,
    ProviderMetadata,
    ProviderConfig,
    HealthStatus,
    Capability,
};

// Re-export core types
pub use songbird_types::{SongbirdError, SongbirdResult};

// Local SDK-specific extensions can be added here if needed
