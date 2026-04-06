// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Gaming Configuration - Canonical Types Types
//!
//! This module consolidates all gaming-related configuration structures
//! that were previously scattered across songbird-network crate.

mod auto;
mod nat;
mod network;
mod one_touch;
mod performance;
mod security;
mod taxonomy;

pub use auto::*;
pub use nat::*;
pub use network::*;
pub use one_touch::*;
pub use performance::*;
pub use security::*;
pub use taxonomy::*;

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Gaming Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalGamingConfig {
    /// Core gaming settings
    pub core: GamingCoreConfig,
    /// Network configuration for gaming
    pub network: GamingNetworkConfig,
    /// Security configuration for gaming
    pub security: GamingSecurityConfig,
    /// Performance optimization settings
    /// Performance field
    pub performance: GamingPerformanceConfig,
    /// Auto-configuration settings
    pub auto: GamingAutoConfig,
    /// One-touch configuration
    /// One Touch field
    pub one_touch: OneTouchConfig,
}

/// Core gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingCoreConfig {
    /// Enable gaming features
    pub enabled: bool,
    /// Gaming mode
    pub mode: GamingMode,
    /// Default game type
    pub default_game_type: GameType,
}

impl Default for GamingCoreConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: GamingMode::Performance,
            default_game_type: GameType::Fps,
        }
    }
}

#[cfg(test)]
mod gaming_tests;
