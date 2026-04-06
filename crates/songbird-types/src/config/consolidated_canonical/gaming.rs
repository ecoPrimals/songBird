// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Gaming Configuration Module
//!
//! **CANONICAL GAMING CONFIGURATION** ✅
//!
//! This module provides gaming protocol configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// GAMING CONFIGURATION
// ============================================================================

/// **CANONICAL**: Gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalGamingConfig {
    /// Enable gaming features
    pub enabled: bool,
    /// Gaming protocol version
    pub protocol_version: String,
}

impl Default for CanonicalGamingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol_version: "1.0".to_string(),
        }
    }
}
