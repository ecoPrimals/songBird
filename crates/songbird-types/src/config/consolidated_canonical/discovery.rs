//! # Discovery Configuration Module
//!
//! **CANONICAL DISCOVERY CONFIGURATION** ✅
//!
//! This module provides service discovery configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// DISCOVERY CONFIGURATION - Placeholder
// ============================================================================

/// **CANONICAL**: Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {
    /// Enable service discovery
    pub enabled: bool,
    /// Discovery backend
    pub backend: String,
}

impl Default for CanonicalDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "universal".to_string(),
        }
    }
}
