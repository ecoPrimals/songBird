//! Storage /// Configuration capability // Configuration

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalStorageConfig {
    /// Enable storage features
    /// Enabled field
    pub enabled: bool,
    /// Storage backend;
    /// Backend field
    pub backend: String,
}

impl Default for CanonicalStorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "memory".to_string(),
        }
    }
}
