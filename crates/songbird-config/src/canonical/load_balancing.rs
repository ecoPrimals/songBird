// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Load balancing strategies and configuration types

#![allow(missing_docs, reason = "strategy enum variants map to well-known balancer modes")]

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Load balancing strategy
///
/// Unified from multiple definitions across:
/// - `crates/songbird-orchestrator/src/core/load_balancer.rs`
/// - `songbird-network/src/balancing/mod.rs`
/// - `songbird-universal/src/load_balancing.rs`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    #[default]
    RoundRobin,
    /// Weighted round-robin with service capacity
    WeightedRoundRobin {
        weights: Vec<u32>,
    },
    /// Least connections strategy
    LeastConnections,
    /// Weighted least connections
    WeightedLeastConnections {
        weights: Vec<u32>,
    },
    /// Random selection
    Random,
    /// Weighted random selection
    WeightedRandom {
        weights: Vec<u32>,
    },
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

impl std::fmt::Display for LoadBalancingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoundRobin => write!(f, "round-robin"),
            Self::WeightedRoundRobin {
                ..
            } => write!(f, "weighted-round-robin"),
            Self::LeastConnections => write!(f, "least-connections"),
            Self::WeightedLeastConnections {
                ..
            } => {
                write!(f, "weighted-least-connections")
            }
            Self::Random => write!(f, "random"),
            Self::WeightedRandom {
                ..
            } => write!(f, "weighted-random"),
            Self::ResponseTime => write!(f, "response-time"),
            Self::ResourceBased => write!(f, "resource-based"),
            Self::ConsistentHashing => write!(f, "consistent-hashing"),
            Self::IpHash => write!(f, "ip-hash"),
            Self::Adaptive => write!(f, "adaptive"),
            Self::Custom {
                name,
                ..
            } => write!(f, "custom-{name}"),
        }
    }
}

impl LoadBalancingStrategy {
    /// Check if this strategy requires weights
    #[must_use]
    pub const fn requires_weights(&self) -> bool {
        matches!(
            self,
            Self::WeightedRoundRobin { .. }
                | Self::WeightedLeastConnections { .. }
                | Self::WeightedRandom { .. }
        )
    }

    /// Check if this strategy supports session affinity
    #[must_use]
    pub const fn supports_affinity(&self) -> bool {
        matches!(self, Self::ConsistentHashing | Self::IpHash)
    }

    /// Check if this strategy adapts to performance metrics
    #[must_use]
    pub const fn is_adaptive(&self) -> bool {
        matches!(self, Self::ResponseTime | Self::ResourceBased | Self::Adaptive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancing_strategy_display() {
        assert_eq!(LoadBalancingStrategy::RoundRobin.to_string(), "round-robin");
        assert_eq!(LoadBalancingStrategy::LeastConnections.to_string(), "least-connections");
        assert_eq!(LoadBalancingStrategy::ResponseTime.to_string(), "response-time");
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
        assert_eq!(LoadBalancingStrategy::default(), LoadBalancingStrategy::RoundRobin);
    }

    #[test]
    fn test_all_strategy_display_formats() {
        assert_eq!(LoadBalancingStrategy::RoundRobin.to_string(), "round-robin");
        assert_eq!(LoadBalancingStrategy::LeastConnections.to_string(), "least-connections");
        assert_eq!(LoadBalancingStrategy::Random.to_string(), "random");
        assert_eq!(LoadBalancingStrategy::ResponseTime.to_string(), "response-time");
        assert_eq!(LoadBalancingStrategy::ResourceBased.to_string(), "resource-based");
        assert_eq!(LoadBalancingStrategy::ConsistentHashing.to_string(), "consistent-hashing");
        assert_eq!(LoadBalancingStrategy::IpHash.to_string(), "ip-hash");
        assert_eq!(LoadBalancingStrategy::Adaptive.to_string(), "adaptive");
    }

    #[test]
    fn test_weighted_strategies_display() {
        let weighted_rr = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2],
        };
        assert_eq!(weighted_rr.to_string(), "weighted-round-robin");

        let weighted_lc = LoadBalancingStrategy::WeightedLeastConnections {
            weights: vec![5, 10],
        };
        assert_eq!(weighted_lc.to_string(), "weighted-least-connections");

        let weighted_rand = LoadBalancingStrategy::WeightedRandom {
            weights: vec![1, 1, 2],
        };
        assert_eq!(weighted_rand.to_string(), "weighted-random");
    }

    #[test]
    fn test_custom_strategy_display() {
        let custom = LoadBalancingStrategy::Custom {
            name: String::from("my-strategy"),
            config: serde_json::json!({"key": "value"}),
        };
        assert_eq!(custom.to_string(), "custom-my-strategy");
    }

    #[test]
    fn test_requires_weights_all_weighted() {
        let weighted_rr = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1],
        };
        assert!(weighted_rr.requires_weights());

        let weighted_lc = LoadBalancingStrategy::WeightedLeastConnections {
            weights: vec![1],
        };
        assert!(weighted_lc.requires_weights());

        let weighted_rand = LoadBalancingStrategy::WeightedRandom {
            weights: vec![1],
        };
        assert!(weighted_rand.requires_weights());
    }

    #[test]
    fn test_requires_weights_non_weighted() {
        assert!(!LoadBalancingStrategy::RoundRobin.requires_weights());
        assert!(!LoadBalancingStrategy::LeastConnections.requires_weights());
        assert!(!LoadBalancingStrategy::Random.requires_weights());
        assert!(!LoadBalancingStrategy::ResponseTime.requires_weights());
        assert!(!LoadBalancingStrategy::ResourceBased.requires_weights());
        assert!(!LoadBalancingStrategy::ConsistentHashing.requires_weights());
        assert!(!LoadBalancingStrategy::IpHash.requires_weights());
        assert!(!LoadBalancingStrategy::Adaptive.requires_weights());
    }

    #[test]
    fn test_supports_affinity_positive() {
        assert!(LoadBalancingStrategy::ConsistentHashing.supports_affinity());
        assert!(LoadBalancingStrategy::IpHash.supports_affinity());
    }

    #[test]
    fn test_supports_affinity_negative() {
        assert!(!LoadBalancingStrategy::RoundRobin.supports_affinity());
        assert!(!LoadBalancingStrategy::LeastConnections.supports_affinity());
        assert!(!LoadBalancingStrategy::Random.supports_affinity());
        assert!(!LoadBalancingStrategy::ResponseTime.supports_affinity());
        assert!(!LoadBalancingStrategy::ResourceBased.supports_affinity());
        assert!(!LoadBalancingStrategy::Adaptive.supports_affinity());
    }

    #[test]
    fn test_is_adaptive_positive() {
        assert!(LoadBalancingStrategy::ResponseTime.is_adaptive());
        assert!(LoadBalancingStrategy::ResourceBased.is_adaptive());
        assert!(LoadBalancingStrategy::Adaptive.is_adaptive());
    }

    #[test]
    fn test_is_adaptive_negative() {
        assert!(!LoadBalancingStrategy::RoundRobin.is_adaptive());
        assert!(!LoadBalancingStrategy::LeastConnections.is_adaptive());
        assert!(!LoadBalancingStrategy::Random.is_adaptive());
        assert!(!LoadBalancingStrategy::ConsistentHashing.is_adaptive());
        assert!(!LoadBalancingStrategy::IpHash.is_adaptive());
    }

    #[test]
    fn test_strategy_equality() {
        assert_eq!(LoadBalancingStrategy::RoundRobin, LoadBalancingStrategy::RoundRobin);
        assert_ne!(LoadBalancingStrategy::RoundRobin, LoadBalancingStrategy::Random);
    }

    #[test]
    fn test_weighted_strategy_equality() {
        let w1 = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2, 3],
        };
        let w2 = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2, 3],
        };
        let w3 = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2],
        };

        assert_eq!(w1, w2);
        assert_ne!(w1, w3);
    }

    #[test]
    fn test_custom_strategy_equality() {
        let c1 = LoadBalancingStrategy::Custom {
            name: String::from("test"),
            config: serde_json::json!({"a": 1}),
        };
        let c2 = LoadBalancingStrategy::Custom {
            name: String::from("test"),
            config: serde_json::json!({"a": 1}),
        };
        let c3 = LoadBalancingStrategy::Custom {
            name: String::from("other"),
            config: serde_json::json!({"a": 1}),
        };

        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_strategy_clone() {
        let original = LoadBalancingStrategy::Adaptive;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let weighted = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![5, 10],
        };
        let cloned_weighted = weighted.clone();
        assert_eq!(weighted, cloned_weighted);
    }

    #[test]
    fn test_serialization_round_trip() {
        let strategies = vec![
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::Random,
            LoadBalancingStrategy::ResponseTime,
            LoadBalancingStrategy::ResourceBased,
            LoadBalancingStrategy::ConsistentHashing,
            LoadBalancingStrategy::IpHash,
            LoadBalancingStrategy::Adaptive,
        ];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).expect("Failed to serialize");
            let deserialized: LoadBalancingStrategy =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(strategy, deserialized);
        }
    }

    #[test]
    fn test_weighted_serialization() {
        let weighted = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2, 3],
        };
        let json = serde_json::to_string(&weighted).expect("Failed to serialize");
        let deserialized: LoadBalancingStrategy =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(weighted, deserialized);
    }

    #[test]
    fn test_custom_serialization() {
        let custom = LoadBalancingStrategy::Custom {
            name: String::from("custom-algo"),
            config: serde_json::json!({"threshold": 100, "enabled": true}),
        };
        let json = serde_json::to_string(&custom).expect("Failed to serialize");
        let deserialized: LoadBalancingStrategy =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(custom, deserialized);
    }

    #[test]
    fn test_weighted_empty_weights() {
        let weighted = LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![],
        };
        assert!(weighted.requires_weights());
        assert_eq!(weighted.to_string(), "weighted-round-robin");
    }

    #[test]
    fn test_strategy_debug_format() {
        let strategy = LoadBalancingStrategy::RoundRobin;
        let debug_str = format!("{strategy:?}");
        assert!(debug_str.contains("RoundRobin"));

        let weighted = LoadBalancingStrategy::WeightedRandom {
            weights: vec![1, 2],
        };
        let weighted_debug = format!("{weighted:?}");
        assert!(weighted_debug.contains("WeightedRandom"));
        assert!(weighted_debug.contains("weights"));
    }

    #[test]
    fn test_combined_characteristics() {
        let strategy = LoadBalancingStrategy::ResponseTime;
        assert!(strategy.is_adaptive());
        assert!(!strategy.requires_weights());
        assert!(!strategy.supports_affinity());

        let ip_hash = LoadBalancingStrategy::IpHash;
        assert!(!ip_hash.is_adaptive());
        assert!(!ip_hash.requires_weights());
        assert!(ip_hash.supports_affinity());
    }
}
