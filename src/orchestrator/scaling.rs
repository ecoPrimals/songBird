//! Gaming-Focused Scaling for Songbird Orchestrator
//!
//! Simple scaling optimized for single-region gaming orchestration.
//! Supports 2-50 players with two modes: Home gaming and LAN party gaming.
//! Multi-region complexity removed - handled by separate Toadstool system.

use crate::errors::Result;
use serde::{Deserialize, Serialize};

/// Gaming-focused scaling modes for single-region excellence
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamingScale {
    /// Home Gaming: 2-8 players, home network gaming setup
    /// Perfect for: Family gaming, small friend groups, retro gaming
    HomeGaming,

    /// LAN Party: 8-50 players, local gaming events  
    /// Perfect for: Gaming tournaments, LAN parties, gaming cafes
    LanParty,

    /// Auto: Detect appropriate scale based on gaming load
    Auto,
}

/// Resource limits for gaming scales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingResourceLimits {
    /// Maximum number of gaming sessions
    pub max_gaming_sessions: usize,

    /// Maximum memory usage in MB
    pub max_memory_mb: usize,

    /// Maximum concurrent players
    pub max_players: usize,

    /// Maximum network connections
    pub max_connections: usize,
}

/// Gaming scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingScalingConfig {
    /// Current gaming scale mode
    pub scale: GamingScale,

    /// Enable automatic scaling based on player load
    pub auto_scaling: bool,

    /// Scale up when this many players are active
    pub scale_up_player_threshold: usize,

    /// Scale down when fewer than this many players
    pub scale_down_player_threshold: usize,

    /// Custom resource overrides
    pub resource_overrides: Option<GamingResourceLimits>,
}

impl Default for GamingScalingConfig {
    fn default() -> Self {
        Self {
            scale: GamingScale::Auto,
            auto_scaling: true,
            scale_up_player_threshold: 6,   // Scale up at 6+ players
            scale_down_player_threshold: 2, // Scale down below 2 players
            resource_overrides: None,
        }
    }
}

impl GamingScale {
    /// Get resource limits for this gaming scale
    pub fn resource_limits(&self) -> GamingResourceLimits {
        match self {
            GamingScale::HomeGaming => GamingResourceLimits {
                max_gaming_sessions: 5,
                max_memory_mb: 512,
                max_players: 8,
                max_connections: 20,
            },
            GamingScale::LanParty => GamingResourceLimits {
                max_gaming_sessions: 20,
                max_memory_mb: 2048,
                max_players: 50,
                max_connections: 100,
            },
            GamingScale::Auto => {
                // Auto mode starts with HomeGaming and adapts
                GamingScale::HomeGaming.resource_limits()
            }
        }
    }

    /// Get a human-readable description of this scale
    pub fn description(&self) -> &'static str {
        match self {
            GamingScale::HomeGaming => "Home network gaming for 2-8 players",
            GamingScale::LanParty => "LAN party gaming for 8-50 players",
            GamingScale::Auto => "Automatically adapts based on player count",
        }
    }

    /// Detect appropriate scale based on player count
    pub fn detect_from_players(player_count: usize) -> GamingScale {
        match player_count {
            0..=8 => GamingScale::HomeGaming,
            _ => GamingScale::LanParty,
        }
    }

    /// Check if this scale can handle the given gaming load
    pub fn can_handle_load(&self, sessions: usize, players: usize, connections: usize) -> bool {
        let limits = self.resource_limits();
        sessions <= limits.max_gaming_sessions
            && players <= limits.max_players
            && connections <= limits.max_connections
    }

    /// Get the next scale up for gaming
    pub fn scale_up(&self) -> Option<GamingScale> {
        match self {
            GamingScale::HomeGaming => Some(GamingScale::LanParty),
            GamingScale::LanParty => None, // Already at max gaming scale
            GamingScale::Auto => Some(GamingScale::LanParty),
        }
    }

    /// Get the next scale down for gaming
    pub fn scale_down(&self) -> Option<GamingScale> {
        match self {
            GamingScale::HomeGaming => None, // Already at min gaming scale
            GamingScale::LanParty => Some(GamingScale::HomeGaming),
            GamingScale::Auto => Some(GamingScale::HomeGaming),
        }
    }
}

/// Gaming scaling manager
pub struct GamingScalingManager {
    config: GamingScalingConfig,
    current_scale: GamingScale,
}

impl GamingScalingManager {
    pub fn new(config: GamingScalingConfig) -> Result<Self> {
        let current_scale = if config.scale == GamingScale::Auto {
            GamingScale::HomeGaming // Start with home gaming
        } else {
            config.scale.clone()
        };

        Ok(Self {
            config,
            current_scale,
        })
    }

    pub fn current_scale(&self) -> &GamingScale {
        &self.current_scale
    }

    pub fn resource_limits(&self) -> GamingResourceLimits {
        if let Some(ref overrides) = self.config.resource_overrides {
            overrides.clone()
        } else {
            self.current_scale.resource_limits()
        }
    }

    /// Evaluate if scaling is needed based on current gaming load
    pub fn evaluate_scaling(&mut self, current_load: &GamingLoadMetrics) -> Option<GamingScale> {
        if !self.config.auto_scaling {
            return None;
        }

        let new_scale = GamingScale::detect_from_players(current_load.active_players);

        if new_scale != self.current_scale {
            self.current_scale = new_scale.clone();
            Some(new_scale)
        } else {
            None
        }
    }
}

/// Gaming load metrics
pub struct GamingLoadMetrics {
    pub active_sessions: usize,
    pub active_players: usize,
    pub active_connections: usize,
    pub memory_usage_mb: usize,
}

impl GamingLoadMetrics {
    /// Calculate utilization percentage for current gaming load
    pub fn calculate_utilization(&self, limits: &GamingResourceLimits) -> f64 {
        let session_util = self.active_sessions as f64 / limits.max_gaming_sessions as f64;
        let player_util = self.active_players as f64 / limits.max_players as f64;
        let connection_util = self.active_connections as f64 / limits.max_connections as f64;

        // Return the highest utilization
        session_util.max(player_util).max(connection_util).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaming_scale_progression() {
        assert_eq!(
            GamingScale::HomeGaming.scale_up(),
            Some(GamingScale::LanParty)
        );
        assert_eq!(GamingScale::LanParty.scale_up(), None);
        assert_eq!(
            GamingScale::LanParty.scale_down(),
            Some(GamingScale::HomeGaming)
        );
        assert_eq!(GamingScale::HomeGaming.scale_down(), None);
    }

    #[test]
    fn test_gaming_detection() {
        assert_eq!(GamingScale::detect_from_players(3), GamingScale::HomeGaming);
        assert_eq!(GamingScale::detect_from_players(15), GamingScale::LanParty);
    }

    #[test]
    fn test_gaming_load_calculation() {
        let limits = GamingScale::HomeGaming.resource_limits();
        let load = GamingLoadMetrics {
            active_sessions: 2,
            active_players: 4,
            active_connections: 10,
            memory_usage_mb: 256,
        };

        let utilization = load.calculate_utilization(&limits);
        assert!((0.0..=1.0).contains(&utilization));
    }
}
