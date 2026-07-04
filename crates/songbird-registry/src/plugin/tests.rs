// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;

fn encryption_cap() -> PluginCapability {
    PluginCapability::Encryption {
        algorithms: vec!["chacha20-poly1305".into()],
    }
}

fn discovery_cap() -> PluginCapability {
    PluginCapability::ServiceDiscovery {
        protocols: vec!["birdsong".into()],
    }
}

fn compute_cap(cores: u32, mem: u32) -> PluginCapability {
    PluginCapability::Compute {
        cpu_cores: cores,
        memory_gb: mem,
    }
}

#[tokio::test]
async fn new_registry_is_empty() {
    let reg = DynamicPluginRegistry::new();
    assert!(reg.list_plugins().await.is_empty());
}

#[tokio::test]
async fn register_plugin_returns_id() {
    let reg = DynamicPluginRegistry::new();
    let id = reg
        .register_plugin("crypto-provider".into(), vec![encryption_cap()], vec![])
        .await
        .unwrap();
    assert_eq!(id, "crypto-provider");
}

#[tokio::test]
async fn get_plugin_capabilities_empty_for_unknown() {
    let reg = DynamicPluginRegistry::new();
    let caps = reg.get_plugin_capabilities("nonexistent").await.unwrap();
    assert!(caps.is_empty());
}

#[tokio::test]
async fn discover_plugins_finds_matching() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("enc-1".into(), vec![encryption_cap()], vec![]).await.unwrap();
    reg.register_plugin("disc-1".into(), vec![discovery_cap()], vec![]).await.unwrap();

    let found = reg
        .discover_plugins(vec![PluginRequirement::RequiresEncryption {
            min_key_size: None,
        }])
        .await
        .unwrap();

    assert!(found.iter().any(|id| id.starts_with("enc-1")));
}

#[tokio::test]
async fn discover_plugins_returns_empty_when_none_match() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("disc-1".into(), vec![discovery_cap()], vec![]).await.unwrap();

    let found = reg
        .discover_plugins(vec![PluginRequirement::RequiresCompute {
            min_cpu_cores: 8,
            min_memory_gb: 16,
        }])
        .await
        .unwrap();

    assert!(found.is_empty());
}

#[tokio::test]
async fn discover_optimal_composition_empty_registry() {
    let reg = DynamicPluginRegistry::new();
    let plans = reg
        .discover_optimal_composition(
            "test task",
            vec![encryption_cap()],
            CompositionConstraints::default(),
        )
        .await
        .unwrap();
    assert!(plans.is_empty());
}

#[tokio::test]
async fn discover_optimal_composition_returns_plans() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("enc-a".into(), vec![encryption_cap()], vec![]).await.unwrap();
    reg.register_plugin("enc-b".into(), vec![encryption_cap()], vec![]).await.unwrap();

    let plans = reg
        .discover_optimal_composition(
            "encrypt data",
            vec![encryption_cap()],
            CompositionConstraints::default(),
        )
        .await
        .unwrap();

    assert!(!plans.is_empty());
    assert!(plans.iter().any(|p| p.plugins.len() == 1));
    assert!(plans.iter().any(|p| p.plugins.len() == 2));
}

#[tokio::test]
async fn auto_compose_succeeds_with_matching_plugin() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("net-1".into(), vec![compute_cap(4, 8)], vec![]).await.unwrap();

    let plan = reg.auto_compose(vec![compute_cap(4, 8)]).await.unwrap();
    assert!(!plan.plugins.is_empty());
}

#[tokio::test]
async fn auto_compose_fails_with_no_matching_plugins() {
    let reg = DynamicPluginRegistry::new();
    let result = reg.auto_compose(vec![encryption_cap()]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn execute_composition_produces_system() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("p1".into(), vec![encryption_cap()], vec![]).await.unwrap();

    let plan = CompositionPlan {
        plugins: vec!["p1".into()],
        integration_order: vec![],
        shared_config: serde_json::Value::Null,
        estimated_performance: PerformanceEstimate {
            latency_ms: 10.0,
            throughput_rps: 500.0,
            memory_usage_mb: 128.0,
            cpu_utilization_percent: 10.0,
        },
    };

    let system = reg.execute_composition(plan).await.unwrap();
    assert_eq!(system.active_plugins, vec!["p1"]);
    assert!(!system.system_id.is_empty());
}

#[tokio::test]
async fn execute_composition_unhealthy_when_plugin_not_in_plugins_map() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("registered".into(), vec![], vec![]).await.unwrap();

    let plan = CompositionPlan {
        plugins: vec!["registered".into(), "ghost".into()],
        integration_order: vec![],
        shared_config: serde_json::Value::Null,
        estimated_performance: PerformanceEstimate {
            latency_ms: 10.0,
            throughput_rps: 500.0,
            memory_usage_mb: 128.0,
            cpu_utilization_percent: 10.0,
        },
    };

    let system = reg.execute_composition(plan).await.unwrap();
    assert!(!system.system_health.overall_healthy);
}

#[test]
fn requirement_to_capability_maps_correctly() {
    let enc =
        DynamicPluginRegistry::requirement_to_capability(&PluginRequirement::RequiresEncryption {
            min_key_size: Some(256),
        });
    assert!(matches!(enc, PluginCapability::Encryption { .. }));

    let disc = DynamicPluginRegistry::requirement_to_capability(
        &PluginRequirement::RequiresServiceDiscovery,
    );
    assert!(matches!(disc, PluginCapability::ServiceDiscovery { .. }));

    let compute =
        DynamicPluginRegistry::requirement_to_capability(&PluginRequirement::RequiresCompute {
            min_cpu_cores: 2,
            min_memory_gb: 4,
        });
    assert!(matches!(
        compute,
        PluginCapability::Compute {
            cpu_cores: 2,
            memory_gb: 4
        }
    ));

    let net =
        DynamicPluginRegistry::requirement_to_capability(&PluginRequirement::RequiresNetwork {
            min_bandwidth_mbps: 100,
            max_latency_ms: 5,
        });
    assert!(matches!(
        net,
        PluginCapability::Network {
            bandwidth_mbps: 100,
            latency_ms: 5
        }
    ));
}

#[tokio::test]
async fn composition_constraints_default_has_limits() {
    let c = CompositionConstraints::default();
    assert_eq!(c.max_latency_ms, Some(1000.0));
    assert_eq!(c.max_memory_mb, Some(1024.0));
    assert_eq!(c.max_plugins, Some(10));
}

#[tokio::test]
async fn generate_combinations_single_and_pairs() {
    let reg = DynamicPluginRegistry::new();
    let plugins = vec!["a".into(), "b".into(), "c".into()];
    let combos = reg.generate_combinations(&plugins, &[]).await.unwrap();
    assert_eq!(combos.len(), 6); // 3 singles + 3 pairs
}

#[tokio::test]
async fn generate_combinations_single_plugin() {
    let reg = DynamicPluginRegistry::new();
    let plugins = vec!["only".into()];
    let combos = reg.generate_combinations(&plugins, &[]).await.unwrap();
    assert_eq!(combos.len(), 1);
    assert_eq!(combos[0], vec!["only".to_string()]);
}

#[tokio::test]
async fn performance_estimate_scales_with_plugins() {
    let reg = DynamicPluginRegistry::new();
    reg.register_plugin("p1".into(), vec![encryption_cap()], vec![]).await.unwrap();
    reg.register_plugin("p2".into(), vec![encryption_cap()], vec![]).await.unwrap();

    let plans = reg
        .discover_optimal_composition(
            "test",
            vec![encryption_cap()],
            CompositionConstraints::default(),
        )
        .await
        .unwrap();

    let single = plans.iter().find(|p| p.plugins.len() == 1).unwrap();
    let pair = plans.iter().find(|p| p.plugins.len() == 2).unwrap();

    assert!(pair.estimated_performance.latency_ms > single.estimated_performance.latency_ms);
    assert!(
        pair.estimated_performance.throughput_rps < single.estimated_performance.throughput_rps
    );
}
