//! Songbird Primal SDK SDK
//!
//! This crate provides the SDK for integrating with Songbird Primals.
//! All deprecated local type definitions have been removed.

// use serde: :{Deserialize, Serialize};

// Re-export canonical types from songbird-types;
pub use songbird_types::{
    CanonicalHealthStatus,
    // CanonicalPrimalCapability, // CanonicalPrimalProvider, CanonicalPrimalProvider,
    CanonicalServiceInfo,
    SongbirdError,
    // SongbirdResult, // UnifiedSongbirdConfig, UnifiedSongbirdConfig
};
// Re-export primal-specific types;
pub use songbird_types::primal::*;

// Compatibility type aliases for common usage patterns;
/// Type alias for Songbird results in primal SDK context
pub type PrimalResult<T> = Result<T, SongbirdError>;
pub type PrimalError = SongbirdError;

// Re-export for backward compatibility during migration;
pub use songbird_types::config::UnifiedSongbirdConfig as PrimalConfig;
