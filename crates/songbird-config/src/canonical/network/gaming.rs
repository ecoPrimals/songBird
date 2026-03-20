// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Gaming Network Configuration
//!
//! Gaming-specific network configuration including ports, scales,
//! and LAN discovery settings.

use serde::{Deserialize, Serialize};

/// Gaming-specific network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    pub starcraft_port: u16,
    pub aoe2_port: u16,
    pub ipx_port: u16,
    pub udp_port: u16,
    pub enable_lan_discovery: bool,
    pub max_players_per_game: usize,
}

impl Default for GamingNetworkConfig {
    fn default() -> Self {
        Self {
            starcraft_port: 6112,
            aoe2_port: 6113,
            ipx_port: 6112,
            udp_port: 6114,
            enable_lan_discovery: true,
            max_players_per_game: 8,
        }
    }
}

/// **CANONICAL**: Gaming network scale configuration
///
/// Defines the scale and capacity of gaming network deployments.
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::network::GamingScale;
///
/// let scale = GamingScale::Home;
/// assert_eq!(scale.max_players(), 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GamingScale {
    /// Home gaming setup (1-4 players)
    #[default]
    Home,
    /// LAN party setup (5-16 players)
    LanParty,
    /// Tournament setup (17-64 players)
    Tournament,
    /// Professional setup (65+ players)
    Professional,
}

impl GamingScale {
    /// Get the maximum recommended players for this scale
    #[must_use]
    pub const fn max_players(&self) -> usize {
        match self {
            Self::Home => 4,
            Self::LanParty => 16,
            Self::Tournament => 64,
            Self::Professional => 256,
        }
    }

    /// Get the recommended bandwidth in Mbps for this scale
    #[must_use]
    pub const fn recommended_bandwidth_mbps(&self) -> u64 {
        match self {
            Self::Home => 10,
            Self::LanParty => 50,
            Self::Tournament => 200,
            Self::Professional => 1000,
        }
    }

    /// Get the recommended concurrent connections for this scale
    #[must_use]
    pub const fn recommended_connections(&self) -> usize {
        match self {
            Self::Home => 10,
            Self::LanParty => 50,
            Self::Tournament => 200,
            Self::Professional => 1000,
        }
    }
}

impl std::fmt::Display for GamingScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Home => write!(f, "home"),
            Self::LanParty => write!(f, "lan-party"),
            Self::Tournament => write!(f, "tournament"),
            Self::Professional => write!(f, "professional"),
        }
    }
}
