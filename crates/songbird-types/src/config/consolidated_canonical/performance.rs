// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Performance Configuration Module
//!
//! **CANONICAL PERFORMANCE CONFIGURATION** ✅
//!
//! This module provides performance and optimization configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// PERFORMANCE CONFIGURATION
// ============================================================================

/// **CANONICAL**: Performance and optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPerformanceConfig {
    /// Enable performance optimizations
    pub enabled: bool,
    /// Thread pool size
    pub thread_pool_size: usize,
}

impl CanonicalPerformanceConfig {
    /// Check if performance config has meaningful settings (compatibility helper)
    #[must_use]
    pub const fn is_some(&self) -> bool {
        true // Config always exists with this struct
    }
}

impl Default for CanonicalPerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thread_pool_size: 4,
        }
    }
}
