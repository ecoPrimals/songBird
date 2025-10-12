//! # Configuration Factory Module
//!
//! **CANONICAL CONFIGURATION FACTORY** ✅
//!
//! This module provides configuration factory and builder patterns for the Songbird ecosystem.

use super::CanonicalSongbirdConfig;
use serde::{Deserialize, Serialize};

// ============================================================================
// CONFIGURATION FACTORY - Placeholder
// ============================================================================

/// **CANONICAL**: Configuration factory for creating canonical configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalConfigFactory;

impl CanonicalConfigFactory {
    /// Create a new configuration factory
    pub fn new() -> Self {
        Self
    }

    /// Create a default configuration
    pub fn create_default() -> CanonicalSongbirdConfig {
        CanonicalSongbirdConfig::default()
    }

    /// Create a configuration for the given environment
    pub fn create_for_environment(env: &str) -> CanonicalSongbirdConfig {
        let mut config = CanonicalSongbirdConfig::default();
        config.system.environment = env.to_string();
        config
    }
}

impl Default for CanonicalConfigFactory {
    fn default() -> Self {
        Self::new()
    }
}
