//! Load balancing strategies and configuration types

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Load balancing strategy
///
/// Unified from multiple definitions across:
/// - `songbird-core/src/load_balancer/strategies.rs`
/// - `songbird-network/src/balancing/mod.rs`
/// - `songbird-universal/src/load_balancing.rs`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Weighted round-robin with service capacity
    WeightedRoundRobin { weights: Vec<u32> },
    /// Least connections strategy
    LeastConnections,
    /// Weighted least connections
    WeightedLeastConnections { weights: Vec<u32> },
    /// Random selection
    Random,
    /// Weighted random selection
    WeightedRandom { weights: Vec<u32> },
    /// Response time based selection
    ResponseTime,
    /// Resource utilization based selection
    ResourceBased,
    /// Consistent hashing for sticky sessions
    ConsistentHashing,
    /// IP hash for session affinity
    IpHash,
    /// Adaptive strategy that changes based on performance
    Adaptive,
    /// Custom strategy with user-defined logic
    Custom {
        name: String,
        config: serde_json::Value,
    },
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

impl std::fmt::Display for LoadBalancingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadBalancingStrategy::RoundRobin => write!(f, "round-robin"),
            LoadBalancingStrategy::WeightedRoundRobin { .. } => write!(f, "weighted-round-robin"),
            LoadBalancingStrategy::LeastConnections => write!(f, "least-connections"),
            LoadBalancingStrategy::WeightedLeastConnections { .. } => {
                write!(f, "weighted-least-connections")
            }
            LoadBalancingStrategy::Random => write!(f, "random"),
            LoadBalancingStrategy::WeightedRandom { .. } => write!(f, "weighted-random"),
            LoadBalancingStrategy::ResponseTime => write!(f, "response-time"),
            LoadBalancingStrategy::ResourceBased => write!(f, "resource-based"),
            LoadBalancingStrategy::ConsistentHashing => write!(f, "consistent-hashing"),
            LoadBalancingStrategy::IpHash => write!(f, "ip-hash"),
            LoadBalancingStrategy::Adaptive => write!(f, "adaptive"),
            LoadBalancingStrategy::Custom { name, .. } => write!(f, "custom-{name}"),
        }
    }
}

impl LoadBalancingStrategy {
    /// Check if this strategy requires weights
    #[must_use]
    pub fn requires_weights(&self) -> bool {
        matches!(
            self,
            LoadBalancingStrategy::WeightedRoundRobin { .. }
                | LoadBalancingStrategy::WeightedLeastConnections { .. }
                | LoadBalancingStrategy::WeightedRandom { .. }
        )
    }

    /// Check if this strategy supports session affinity
    #[must_use]
    pub fn supports_affinity(&self) -> bool {
        matches!(
            self,
            LoadBalancingStrategy::ConsistentHashing | LoadBalancingStrategy::IpHash
        )
    }

    /// Check if this strategy adapts to performance metrics
    #[must_use]
    pub fn is_adaptive(&self) -> bool {
        matches!(
            self,
            LoadBalancingStrategy::ResponseTime
                | LoadBalancingStrategy::ResourceBased
                | LoadBalancingStrategy::Adaptive
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancing_strategy_display() {
        assert_eq!(LoadBalancingStrategy::RoundRobin.to_string(), "round-robin");
        assert_eq!(
            LoadBalancingStrategy::LeastConnections.to_string(),
            "least-connections"
        );
        assert_eq!(
            LoadBalancingStrategy::ResponseTime.to_string(),
            "response-time"
        );
    }

    #[test]
    fn test_strategy_characteristics() {
        let weighted_rr = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2, 3],
        };
        assert!(weighted_rr.requires_weights());
        assert!(!weighted_rr.supports_affinity());

        let consistent_hash = LoadBalancingStrategy::ConsistentHashing;
        assert!(!consistent_hash.requires_weights());
        assert!(consistent_hash.supports_affinity());

        let adaptive = LoadBalancingStrategy::Adaptive;
        assert!(adaptive.is_adaptive());
    }

    #[test]
    fn test_default_strategy() {
        assert_eq!(
            LoadBalancingStrategy::default(),
            LoadBalancingStrategy::RoundRobin
        );
    }
}
