//! Discovery Traits Module
//!
//! **MODERNIZED**: Now uses canonical Provider traits from songbird-types

// Re-export canonical traits instead of defining our own
pub use songbird_types::traits::canonical::{
    Provider,
    ServiceProvider,
    DiscoveryProvider,
    ProviderMetadata,
    ProviderConfig,
    HealthStatus,
    Capability,
};

// Re-export core types
pub use songbird_types::{SongbirdError, SongbirdResult};

// Local discovery-specific types
pub mod config;
pub mod feature_flags;

// Re-export local types
pub use config::*;
pub use feature_flags::*;
