// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Traits Module
//!
//! **MODERNIZED**: Now uses canonical Provider traits from songbird-types

// Re-export canonical traits instead of defining our own
pub use songbird_types::traits::canonical::{
    Capability, DiscoveryProvider, HealthStatus, Provider, ProviderConfig, ProviderMetadata,
    ServiceProvider,
};

// Re-export core types
pub use songbird_types::{SongbirdError, SongbirdResult};

// Local discovery-specific types and traits
pub mod config;
pub mod discovery;
pub mod feature_flags;
pub mod service;

// Re-export local types
pub use config::*;
pub use discovery::*;
pub use feature_flags::*;
pub use service::*;
