// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for Canonical Load Balancing Configuration
//!
//! This test suite provides thorough coverage of load balancing
//! strategies and configuration types.

use songbird_config::canonical::load_balancing::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ========== LoadBalancingStrategy Tests ==========

#[test]
fn test_round_robin_strategy() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    assert!(matches!(strategy, LoadBalancingStrategy::RoundRobin));
}

#[test]
fn test_weighted_round_robin_strategy() {
    let strategy = LoadBalancingStrategy::WeightedRoundRobin {
        weights: vec![1, 2, 3],
    };

    if let LoadBalancingStrategy::WeightedRoundRobin {
        weights,
    } = strategy
    {
        assert_eq!(weights.len(), 3);
        assert_eq!(weights[0], 1);
        assert_eq!(weights[2], 3);
    } else {
        panic!("Expected WeightedRoundRobin");
    }
}

#[test]
fn test_least_connections_strategy() {
    let strategy = LoadBalancingStrategy::LeastConnections;
    assert!(matches!(strategy, LoadBalancingStrategy::LeastConnections));
}

#[test]
fn test_weighted_least_connections_strategy() {
    let strategy = LoadBalancingStrategy::WeightedLeastConnections {
        weights: vec![5, 10, 15],
    };

    if let LoadBalancingStrategy::WeightedLeastConnections {
        weights,
    } = strategy
    {
        assert_eq!(weights.len(), 3);
        assert_eq!(weights[1], 10);
    } else {
        panic!("Expected WeightedLeastConnections");
    }
}

#[test]
fn test_random_strategy() {
    let strategy = LoadBalancingStrategy::Random;
    assert!(matches!(strategy, LoadBalancingStrategy::Random));
}

#[test]
fn test_weighted_random_strategy() {
    let strategy = LoadBalancingStrategy::WeightedRandom {
        weights: vec![100, 200, 300],
    };

    if let LoadBalancingStrategy::WeightedRandom {
        weights,
    } = strategy
    {
        assert_eq!(weights.len(), 3);
    } else {
        panic!("Expected WeightedRandom");
    }
}

#[test]
fn test_response_time_strategy() {
    let strategy = LoadBalancingStrategy::ResponseTime;
    assert!(matches!(strategy, LoadBalancingStrategy::ResponseTime));
}

#[test]
fn test_resource_based_strategy() {
    let strategy = LoadBalancingStrategy::ResourceBased;
    assert!(matches!(strategy, LoadBalancingStrategy::ResourceBased));
}

#[test]
fn test_consistent_hashing_strategy() {
    let strategy = LoadBalancingStrategy::ConsistentHashing;
    assert!(matches!(strategy, LoadBalancingStrategy::ConsistentHashing));
}

#[test]
fn test_ip_hash_strategy() {
    let strategy = LoadBalancingStrategy::IpHash;
    assert!(matches!(strategy, LoadBalancingStrategy::IpHash));
}

#[test]
fn test_adaptive_strategy() {
    let strategy = LoadBalancingStrategy::Adaptive;
    assert!(matches!(strategy, LoadBalancingStrategy::Adaptive));
}

#[test]
fn test_custom_strategy() {
    let strategy = LoadBalancingStrategy::Custom {
        name: "my-custom-strategy".to_string(),
        config: serde_json::json!({"param": "value"}),
    };

    if let LoadBalancingStrategy::Custom {
        name,
        config,
    } = strategy
    {
        assert_eq!(name, "my-custom-strategy");
        assert!(!config.is_null());
    } else {
        panic!("Expected Custom strategy");
    }
}

#[test]
fn test_strategy_default() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::default();
    assert!(matches!(strategy, LoadBalancingStrategy::RoundRobin));
    Ok(())
}

#[test]
fn test_strategy_clone() -> SongbirdResult<()> {
    let original = LoadBalancingStrategy::LeastConnections;
    let cloned = original.clone();
    assert_eq!(original, cloned);
    Ok(())
}

#[test]
fn test_strategy_equality() -> SongbirdResult<()> {
    let s1 = LoadBalancingStrategy::RoundRobin;
    let s2 = LoadBalancingStrategy::RoundRobin;
    let s3 = LoadBalancingStrategy::Random;

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
    Ok(())
}

#[test]
fn test_strategy_debug() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::Adaptive;
    let debug_str = format!("{strategy:?}");
    assert!(debug_str.contains("Adaptive"));
    Ok(())
}

#[test]
fn test_strategy_display_round_robin() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    assert_eq!(format!("{strategy}"), "round-robin");
}

#[test]
fn test_strategy_display_weighted_round_robin() {
    let strategy = LoadBalancingStrategy::WeightedRoundRobin {
        weights: vec![1],
    };
    assert_eq!(format!("{strategy}"), "weighted-round-robin");
}

#[test]
fn test_strategy_display_least_connections() {
    let strategy = LoadBalancingStrategy::LeastConnections;
    assert_eq!(format!("{strategy}"), "least-connections");
}

#[test]
fn test_strategy_display_random() {
    let strategy = LoadBalancingStrategy::Random;
    assert_eq!(format!("{strategy}"), "random");
}

#[test]
fn test_strategy_display_response_time() {
    let strategy = LoadBalancingStrategy::ResponseTime;
    assert_eq!(format!("{strategy}"), "response-time");
}

#[test]
fn test_strategy_display_consistent_hashing() {
    let strategy = LoadBalancingStrategy::ConsistentHashing;
    assert_eq!(format!("{strategy}"), "consistent-hashing");
}

#[test]
fn test_strategy_display_ip_hash() {
    let strategy = LoadBalancingStrategy::IpHash;
    assert_eq!(format!("{strategy}"), "ip-hash");
}

#[test]
fn test_strategy_display_adaptive() {
    let strategy = LoadBalancingStrategy::Adaptive;
    assert_eq!(format!("{strategy}"), "adaptive");
}

#[test]
fn test_strategy_display_custom() {
    let strategy = LoadBalancingStrategy::Custom {
        name: "special".to_string(),
        config: serde_json::Value::Null,
    };
    assert_eq!(format!("{strategy}"), "custom-special");
}

#[test]
fn test_strategy_serialization() {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let json = serde_json::to_string(&strategy);
    assert!(json.is_ok());
}

#[test]
fn test_strategy_deserialization() {
    let json = r#""RoundRobin""#;
    let result: Result<LoadBalancingStrategy, _> = serde_json::from_str(json);
    assert!(result.is_ok());
}

#[test]
fn test_weighted_strategy_with_empty_weights() {
    let strategy = LoadBalancingStrategy::WeightedRandom {
        weights: vec![],
    };

    if let LoadBalancingStrategy::WeightedRandom {
        weights,
    } = strategy
    {
        assert!(weights.is_empty());
    } else {
        panic!("Expected WeightedRandom");
    }
}

#[test]
fn test_weighted_strategy_with_many_weights() -> SongbirdResult<()> {
    let weights = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let strategy = LoadBalancingStrategy::WeightedRoundRobin {
        weights: weights.clone(),
    };

    if let LoadBalancingStrategy::WeightedRoundRobin {
        weights: w,
    } = strategy
    {
        assert_eq!(w.len(), 10);
        assert_eq!(w, weights);
    } else {
        panic!("Expected WeightedRoundRobin");
    }
    Ok(())
}

#[test]
fn test_strategy_round_trip_serialization() -> SongbirdResult<()> {
    let original = LoadBalancingStrategy::WeightedLeastConnections {
        weights: vec![50, 100, 150],
    };

    let json = serde_json::to_string(&original).map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let deserialized: LoadBalancingStrategy =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {e}"),
            debug_info: None,
        })?;

    assert_eq!(original, deserialized);
    Ok(())
}

#[test]
fn test_custom_strategy_with_complex_config() {
    let config = serde_json::json!({
        "algorithm": "least-busy",
        "threshold": 0.75,
        "metrics": ["cpu", "memory", "connections"]
    });

    let strategy = LoadBalancingStrategy::Custom {
        name: "advanced".to_string(),
        config,
    };

    if let LoadBalancingStrategy::Custom {
        name,
        config,
    } = strategy
    {
        assert_eq!(name, "advanced");
        assert!(config.is_object());
    } else {
        panic!("Expected Custom strategy");
    }
}

#[test]
fn test_all_strategies() -> SongbirdResult<()> {
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
        let debug = format!("{strategy:?}");
        assert!(!debug.is_empty());

        let display = format!("{strategy}");
        assert!(!display.is_empty());
    }
    Ok(())
}

#[test]
fn test_weighted_strategies_with_zero_weights() {
    let strategy = LoadBalancingStrategy::WeightedRandom {
        weights: vec![0, 0, 0],
    };

    if let LoadBalancingStrategy::WeightedRandom {
        weights,
    } = strategy
    {
        assert_eq!(weights.len(), 3);
        assert!(weights.iter().all(|&w| w == 0));
    } else {
        panic!("Expected WeightedRandom");
    }
}

#[test]
fn test_weighted_strategies_with_large_weights() {
    let strategy = LoadBalancingStrategy::WeightedLeastConnections {
        weights: vec![u32::MAX, u32::MAX / 2, 1_000_000],
    };

    if let LoadBalancingStrategy::WeightedLeastConnections {
        weights,
    } = strategy
    {
        assert_eq!(weights[0], u32::MAX);
        assert_eq!(weights[1], u32::MAX / 2);
        assert_eq!(weights[2], 1_000_000);
    } else {
        panic!("Expected WeightedLeastConnections");
    }
}

#[test]
fn test_strategy_equality_with_weights() {
    let s1 = LoadBalancingStrategy::WeightedRandom {
        weights: vec![1, 2, 3],
    };
    let s2 = LoadBalancingStrategy::WeightedRandom {
        weights: vec![1, 2, 3],
    };
    let s3 = LoadBalancingStrategy::WeightedRandom {
        weights: vec![1, 2, 4],
    };

    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

#[test]
fn test_custom_strategy_equality() {
    let s1 = LoadBalancingStrategy::Custom {
        name: "test".to_string(),
        config: serde_json::json!({"a": 1}),
    };
    let s2 = LoadBalancingStrategy::Custom {
        name: "test".to_string(),
        config: serde_json::json!({"a": 1}),
    };

    assert_eq!(s1, s2);
}

#[test]
fn test_strategy_serialization_round_trip_all() -> SongbirdResult<()> {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::WeightedRoundRobin {
            weights: vec![1, 2],
        },
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::WeightedLeastConnections {
            weights: vec![3, 4],
        },
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::WeightedRandom {
            weights: vec![5, 6],
        },
        LoadBalancingStrategy::ResponseTime,
        LoadBalancingStrategy::ResourceBased,
        LoadBalancingStrategy::ConsistentHashing,
        LoadBalancingStrategy::IpHash,
        LoadBalancingStrategy::Adaptive,
        LoadBalancingStrategy::Custom {
            name: "test".to_string(),
            config: serde_json::json!({"key": "value"}),
        },
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy).map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        let deserialized: LoadBalancingStrategy =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {e}"),
                debug_info: None,
            })?;
        assert_eq!(strategy, deserialized);
    }
    Ok(())
}

#[test]
fn test_strategy_json_format() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::Custom {
        name: "my-strategy".to_string(),
        config: serde_json::json!({"type": "advanced"}),
    };

    let json = serde_json::to_string_pretty(&strategy).map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert!(json.contains("Custom"));
    assert!(json.contains("my-strategy"));
    Ok(())
}

#[test]
fn test_weighted_strategy_single_weight() {
    let strategy = LoadBalancingStrategy::WeightedRoundRobin {
        weights: vec![100],
    };

    if let LoadBalancingStrategy::WeightedRoundRobin {
        weights,
    } = strategy
    {
        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0], 100);
    } else {
        panic!("Expected WeightedRoundRobin");
    }
}

#[test]
fn test_strategy_display_all_variants() {
    let test_cases = vec![
        (LoadBalancingStrategy::RoundRobin, "round-robin"),
        (LoadBalancingStrategy::LeastConnections, "least-connections"),
        (LoadBalancingStrategy::Random, "random"),
        (LoadBalancingStrategy::ResponseTime, "response-time"),
        (LoadBalancingStrategy::ResourceBased, "resource-based"),
        (LoadBalancingStrategy::ConsistentHashing, "consistent-hashing"),
        (LoadBalancingStrategy::IpHash, "ip-hash"),
        (LoadBalancingStrategy::Adaptive, "adaptive"),
    ];

    for (strategy, expected) in test_cases {
        assert_eq!(format!("{strategy}"), expected);
    }
}
