//! Tests for Network Effects Optimizer
//!
//! Comprehensive tests for sovereignty-aware network optimization

use songbird_universal::sovereignty::network_optimizer::{
    NetworkEffectsOptimizer, OptimizationConfig,
};
use songbird_universal::sovereignty::types::{
    PathSegment, RoutingPath, SecurityCapability, SecurityLevel, SovereigntyLevel,
};
use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo,
};
use std::collections::HashMap;

// Helper function to create a test service
fn create_test_service(id: &str, name: &str, endpoint: &str) -> ServiceInfo {
    ServiceInfo {
        name: name.to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: endpoint.to_string(),
        capabilities: vec![DiscoveredCapability {
            name: "test-capability".to_string(),
            version: "1.0".to_string(),
            description: "Test capability".to_string(),
            provider: id.to_string(),
            endpoint: endpoint.to_string(),
            qos_metrics: QosMetrics {
                latency_ms: Some(10.0),
                throughput_ops_sec: Some(1000.0),
                availability: Some(0.99),
                reliability: Some(0.99),
            },
            health_status: HealthStatus::Healthy,
        }],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}

// Helper to create a test path segment
fn create_test_segment(service_id: &str) -> PathSegment {
    PathSegment {
        service: create_test_service(service_id, "Test Service", "http://localhost:8080"),
        sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        efficiency_score: 0.7,
        security_capabilities: vec![SecurityCapability::Encryption],
        metadata: HashMap::new(),
    }
}

// Helper to create a test routing path
fn create_test_path(num_segments: usize) -> RoutingPath {
    let mut segments = Vec::new();
    for i in 0..num_segments {
        segments.push(create_test_segment(&format!("service-{i}")));
    }

    RoutingPath {
        segments,
        sovereignty_score: 0.6,
        efficiency_score: 0.7,
        combined_score: 0.62,
        security_level: SecurityLevel::Medium,
    }
}

#[test]
fn test_optimizer_new() {
    let optimizer = NetworkEffectsOptimizer::new();

    let debug_str = format!("{optimizer:?}");
    assert!(debug_str.contains("NetworkEffectsOptimizer"));
}

#[test]
fn test_optimizer_default() {
    let optimizer = NetworkEffectsOptimizer::default();

    let debug_str = format!("{optimizer:?}");
    assert!(debug_str.contains("NetworkEffectsOptimizer"));
}

#[test]
fn test_optimization_config_default() {
    let config = OptimizationConfig::default();

    assert!(config.enable_latency_optimization);
    assert!(config.enable_throughput_optimization);
    assert!(config.enable_security_enhancement);
    assert!(!config.enable_cost_optimization); // Conservative default
}

#[test]
fn test_optimization_config_custom() {
    let config = OptimizationConfig {
        enable_latency_optimization: true,
        enable_throughput_optimization: false,
        enable_security_enhancement: true,
        enable_cost_optimization: true,
    };

    assert!(config.enable_latency_optimization);
    assert!(!config.enable_throughput_optimization);
    assert!(config.enable_security_enhancement);
    assert!(config.enable_cost_optimization);
}

#[test]
fn test_optimizer_with_config() {
    let config = OptimizationConfig {
        enable_latency_optimization: false,
        enable_throughput_optimization: true,
        enable_security_enhancement: false,
        enable_cost_optimization: true,
    };

    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let debug_str = format!("{optimizer:?}");
    assert!(debug_str.contains("NetworkEffectsOptimizer"));
}

#[tokio::test]
async fn test_optimize_empty_paths() {
    let optimizer = NetworkEffectsOptimizer::new();
    let paths = vec![];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();
    assert!(optimized_paths.is_empty());
}

#[tokio::test]
async fn test_optimize_single_path() {
    let optimizer = NetworkEffectsOptimizer::new();
    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();
    assert_eq!(optimized_paths.len(), 1);
}

#[tokio::test]
async fn test_optimize_multiple_paths() {
    let optimizer = NetworkEffectsOptimizer::new();
    let paths = vec![create_test_path(1), create_test_path(2), create_test_path(1)];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();
    assert_eq!(optimized_paths.len(), 3);
}

#[tokio::test]
async fn test_optimization_adds_metadata() {
    let optimizer = NetworkEffectsOptimizer::new();
    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();
    assert_eq!(optimized_paths.len(), 1);

    // Check that optimization metadata was added
    let optimized_path = &optimized_paths[0];
    assert!(!optimized_path.segments.is_empty());
    let segment = &optimized_path.segments[0];

    // Should have network optimization metadata
    assert!(segment.metadata.contains_key("network_optimized"));
}

#[tokio::test]
async fn test_latency_optimization() {
    let config = OptimizationConfig {
        enable_latency_optimization: true,
        enable_throughput_optimization: false,
        enable_security_enhancement: false,
        enable_cost_optimization: false,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Check latency optimization metadata
    let segment = &optimized_paths[0].segments[0];
    assert_eq!(segment.metadata.get("latency_optimized"), Some(&"true".to_string()));
}

#[tokio::test]
async fn test_throughput_optimization() {
    let config = OptimizationConfig {
        enable_latency_optimization: false,
        enable_throughput_optimization: true,
        enable_security_enhancement: false,
        enable_cost_optimization: false,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Check throughput optimization metadata
    let segment = &optimized_paths[0].segments[0];
    assert_eq!(segment.metadata.get("throughput_optimized"), Some(&"true".to_string()));
}

#[tokio::test]
async fn test_security_enhancement() {
    let config = OptimizationConfig {
        enable_latency_optimization: false,
        enable_throughput_optimization: false,
        enable_security_enhancement: true,
        enable_cost_optimization: false,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Check security enhancement metadata
    let segment = &optimized_paths[0].segments[0];
    assert_eq!(segment.metadata.get("security_enhanced"), Some(&"true".to_string()));
    assert!(segment.security_capabilities.contains(&SecurityCapability::SovereigntyCompliant));
}

#[tokio::test]
async fn test_cost_optimization() {
    let config = OptimizationConfig {
        enable_latency_optimization: false,
        enable_throughput_optimization: false,
        enable_security_enhancement: false,
        enable_cost_optimization: true,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Check cost optimization metadata
    let segment = &optimized_paths[0].segments[0];
    assert_eq!(segment.metadata.get("cost_optimized"), Some(&"true".to_string()));
}

#[tokio::test]
async fn test_all_optimizations_enabled() {
    let config = OptimizationConfig {
        enable_latency_optimization: true,
        enable_throughput_optimization: true,
        enable_security_enhancement: true,
        enable_cost_optimization: true,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // All optimization metadata should be present
    let segment = &optimized_paths[0].segments[0];
    assert!(segment.metadata.contains_key("network_optimized"));
    assert!(segment.metadata.contains_key("latency_optimized"));
    assert!(segment.metadata.contains_key("throughput_optimized"));
    assert!(segment.metadata.contains_key("security_enhanced"));
    assert!(segment.metadata.contains_key("cost_optimized"));
}

#[tokio::test]
async fn test_no_optimizations_enabled() {
    let config = OptimizationConfig {
        enable_latency_optimization: false,
        enable_throughput_optimization: false,
        enable_security_enhancement: false,
        enable_cost_optimization: false,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Should still have basic network optimization
    let segment = &optimized_paths[0].segments[0];
    assert!(segment.metadata.contains_key("network_optimized"));
}

#[test]
fn test_get_optimization_stats() {
    let optimizer = NetworkEffectsOptimizer::new();

    let stats = optimizer.get_optimization_stats();

    // Default config has 3 strategies enabled (latency, throughput, security)
    assert_eq!(stats.strategies_enabled, 3);
    assert!(stats.optimization_config.enable_latency_optimization);
    assert!(stats.optimization_config.enable_throughput_optimization);
    assert!(stats.optimization_config.enable_security_enhancement);
    assert!(!stats.optimization_config.enable_cost_optimization);
}

#[test]
fn test_get_optimization_stats_all_enabled() {
    let config = OptimizationConfig {
        enable_latency_optimization: true,
        enable_throughput_optimization: true,
        enable_security_enhancement: true,
        enable_cost_optimization: true,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let stats = optimizer.get_optimization_stats();

    assert_eq!(stats.strategies_enabled, 4);
}

#[test]
fn test_get_optimization_stats_none_enabled() {
    let config = OptimizationConfig {
        enable_latency_optimization: false,
        enable_throughput_optimization: false,
        enable_security_enhancement: false,
        enable_cost_optimization: false,
    };
    let optimizer = NetworkEffectsOptimizer::with_config(config);

    let stats = optimizer.get_optimization_stats();

    assert_eq!(stats.strategies_enabled, 0);
}

#[test]
fn test_optimization_config_clone() {
    let config = OptimizationConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enable_latency_optimization, cloned.enable_latency_optimization);
    assert_eq!(config.enable_throughput_optimization, cloned.enable_throughput_optimization);
    assert_eq!(config.enable_security_enhancement, cloned.enable_security_enhancement);
    assert_eq!(config.enable_cost_optimization, cloned.enable_cost_optimization);
}

#[test]
fn test_optimization_config_debug() {
    let config = OptimizationConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("OptimizationConfig"));
}

#[test]
fn test_optimization_stats_debug() {
    let optimizer = NetworkEffectsOptimizer::new();
    let stats = optimizer.get_optimization_stats();

    let debug_str = format!("{stats:?}");
    assert!(debug_str.contains("OptimizationStats"));
    assert!(debug_str.contains("strategies_enabled"));
}

#[test]
fn test_optimization_stats_clone() {
    let optimizer = NetworkEffectsOptimizer::new();
    let stats = optimizer.get_optimization_stats();
    let cloned = stats.clone();

    assert_eq!(stats.strategies_enabled, cloned.strategies_enabled);
}

#[tokio::test]
async fn test_optimize_path_with_multiple_segments() {
    let optimizer = NetworkEffectsOptimizer::new();
    let path = create_test_path(3);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();
    assert_eq!(optimized_paths[0].segments.len(), 3);

    // Each segment should be optimized
    for segment in &optimized_paths[0].segments {
        assert!(segment.metadata.contains_key("network_optimized"));
    }
}

#[tokio::test]
async fn test_optimize_preserves_service_info() {
    let optimizer = NetworkEffectsOptimizer::new();
    let path = create_test_path(1);
    let original_service_name = path.segments[0].service.name.clone();
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Service info should be preserved
    assert_eq!(optimized_paths[0].segments[0].service.name, original_service_name);
}

#[tokio::test]
async fn test_network_optimized_capability_added() {
    let optimizer = NetworkEffectsOptimizer::new();
    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // NetworkOptimized capability should be added
    let segment = &optimized_paths[0].segments[0];
    assert!(segment.security_capabilities.contains(&SecurityCapability::NetworkOptimized));
}

#[tokio::test]
async fn test_optimization_timestamp_added() {
    let optimizer = NetworkEffectsOptimizer::new();
    let path = create_test_path(1);
    let paths = vec![path];

    let result = optimizer.optimize_for_network_effects(&paths).await;

    assert!(result.is_ok());
    let optimized_paths = result.unwrap();

    // Optimization timestamp should be added
    let segment = &optimized_paths[0].segments[0];
    assert!(segment.metadata.contains_key("optimization_timestamp"));
}
