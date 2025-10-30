//! # Performance Configuration Module
//!
//! **CANONICAL PERFORMANCE CONFIGURATION** ✅
//!
//! This module provides performance and optimization configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// PERFORMANCE CONFIGURATION - Placeholder
// ============================================================================

/// **CANONICAL**: Performance and optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPerformanceConfig {
    /// Enable performance optimizations
    pub enabled: bool,
    /// Thread pool size
    pub thread_pool_size: usize,
}

impl Default for CanonicalPerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thread_pool_size: 4,
        }
    }
}
