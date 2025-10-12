//! # Primals Configuration Module
//!
//! **CANONICAL PRIMALS CONFIGURATION** ✅
//!
//! This module provides primal provider configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// PRIMALS CONFIGURATION - Placeholder
// ============================================================================

/// **CANONICAL**: Primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrimalConfig {
    /// Enable primal features
    pub enabled: bool,
    /// Primal discovery method
    pub discovery_method: String,
}

impl Default for CanonicalPrimalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_method: "universal".to_string(),
        }
    }
}
