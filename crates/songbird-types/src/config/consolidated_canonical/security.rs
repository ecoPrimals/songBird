//! # Security Configuration Module
//!
//! **CANONICAL SECURITY CONFIGURATION** ✅
//!
//! This module provides security and authentication configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// SECURITY CONFIGURATION - Placeholder
// ============================================================================

/// **CANONICAL**: Security and authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSecurityConfig {
    /// Enable security features
    pub enabled: bool,
    /// Authentication method
    pub auth_method: String,
}

impl Default for CanonicalSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auth_method: "jwt".to_string(),
        }
    }
}
