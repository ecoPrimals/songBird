//! Provider abstraction layer for discovery services
//!
//! **MODERNIZED**: Now uses canonical Provider traits from songbird-types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Use canonical traits instead of duplicating them
pub use songbird_types::traits::canonical::{
    Provider,
    DiscoveryProvider,
    ProviderMetadata,
    ProviderConfig,
    HealthStatus,
};
pub use songbird_types::{SongbirdError, SongbirdResult};

// Re-export service types
pub use crate::types::ServiceInfo;

/// Provider metadata for discovery services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryProviderMetadata {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub healthy: bool,
    pub load_score: f64,
}

impl Default for DiscoveryProviderMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown Provider".to_string(),
            version: "0.0.0".to_string(),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            healthy: false,
            load_score: 1.0,
        }
    }
}

// Note: The canonical DiscoveryProvider trait is now used from songbird-types
// All implementations should use the canonical trait hierarchy
