//! # Environment Configuration Module
//!
//! **CANONICAL ENVIRONMENT CONFIGURATION** ✅
//!
//! This module provides environment and deployment configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// ENVIRONMENT CONFIGURATION - Placeholder
// ============================================================================

/// **CANONICAL**: Environment configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalEnvironmentConfig {
    /// Environment name
    pub name: String,
    /// Deployment mode
    pub deployment_mode: String,
}

impl CanonicalEnvironmentConfig {
    /// Check if environment name is empty (compatibility helper)
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
}

impl Default for CanonicalEnvironmentConfig {
    fn default() -> Self {
        Self {
            name: "development".to_string(),
            deployment_mode: "standalone".to_string(),
        }
    }
}
