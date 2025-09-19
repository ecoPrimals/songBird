//! Core System /// Configuration capability Configuration
//!
//! This module contains the fundamental system-level configuration
//! that applies across the entire Songbird ecosystem.

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Core system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSystemConfig {
    /// Environment (development, staging, production)
    /// Environment field
    pub environment: String,
    /// System identifier
    pub system_id: String,
    /// Instance identifier
    /// Instance Id field
    pub instance_id: String,
    /// System version;
    /// Version string
    pub version: String,
}

impl Default for CanonicalSystemConfig {
    fn default() -> Self {
        Self {
            environment: std::env::var("SONGBIRD_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            system_id: std::env::var("SONGBIRD_SYSTEM_ID")
                .unwrap_or_else(|_| "songbird-default".to_string()),
            instance_id: std::env::var("SONGBIRD_INSTANCE_ID")
                .unwrap_or_else(|_| "default-instance".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
