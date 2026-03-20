// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use std::collections::HashMap;
use std::sync::Arc;

use songbird_config::canonical::resilience::LoadBalancerConfig as CanonicalLoadBalancerConfig;
use songbird_types::{SongbirdError, SongbirdResult};

use super::*;

#[test]
fn test_consolidated_orchestrator_config_default() {
    use songbird_config::canonical::resilience::LoadBalancingAlgorithm;
    let config = ConsolidatedOrchestratorConfig::default();
    assert_eq!(config.load_balancing.algorithm, LoadBalancingAlgorithm::RoundRobin);
    assert!(!config.load_balancing.sticky_sessions);
    assert_eq!(config.load_balancing.max_connections_per_backend, 100);
}

#[test]
fn test_load_balancing_config_default() {
    let config = CanonicalLoadBalancerConfig::default();
    use songbird_config::canonical::resilience::LoadBalancingAlgorithm;
    assert_eq!(config.algorithm, LoadBalancingAlgorithm::RoundRobin);
    assert!(!config.sticky_sessions);
    assert_eq!(config.max_connections_per_backend, 100);
}

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();
    assert_eq!(config.metrics_interval, 60);
    assert!(config.enable_benchmarking);
    assert_eq!(config.alert_thresholds.get(&Arc::from("cpu_usage")), Some(&80.0));
    assert_eq!(config.alert_thresholds.get(&Arc::from("memory_usage")), Some(&85.0));
    assert_eq!(config.alert_thresholds.get(&Arc::from("response_time")), Some(&1000.0));
}

#[test]
fn test_registry_config_default() {
    let config = RegistryConfig::default();
    assert_eq!(config.discovery_interval, 30);
    assert_eq!(config.service_timeout, 300);
    assert_eq!(config.max_services, 1000);
}

#[test]
fn test_scaling_config_default() {
    let config = ScalingConfig::default();
    assert!(config.enable_auto_scaling);
    assert_eq!(config.scale_up_threshold, 70.0);
    assert_eq!(config.scale_down_threshold, 30.0);
    assert_eq!(config.min_instances, 1);
    assert_eq!(config.max_instances, 10);
}

#[test]
fn test_zero_touch_config_default() {
    let config = ZeroTouchConfig::default();
    assert!(!config.enable_auto_deployment);
    assert_eq!(config.deployment_strategy, DeploymentStrategy::BlueGreen);
    assert!(config.rollback_on_failure);
}

#[test]
fn test_deployment_strategy_equality() {
    assert_eq!(DeploymentStrategy::BlueGreen, DeploymentStrategy::BlueGreen);
    assert_ne!(DeploymentStrategy::BlueGreen, DeploymentStrategy::RollingUpdate);
    assert_ne!(DeploymentStrategy::RollingUpdate, DeploymentStrategy::Canary);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Unhealthy, HealthStatus::Unknown);
}

#[test]
fn test_scaling_config_thresholds_valid() {
    let config = ScalingConfig::default();
    assert!(
        config.scale_up_threshold > config.scale_down_threshold,
        "Scale up threshold should be higher than scale down threshold"
    );
    assert!(
        config.min_instances <= config.max_instances,
        "Min instances should be <= max instances"
    );
}

#[test]
fn test_performance_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceConfig::default();
    let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;
    let deserialized: PerformanceConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.metrics_interval, deserialized.metrics_interval);
    assert_eq!(config.enable_benchmarking, deserialized.enable_benchmarking);
    Ok(())
}

#[test]
fn test_zero_touch_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = ZeroTouchConfig::default();
    let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;
    let deserialized: ZeroTouchConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.enable_auto_deployment, deserialized.enable_auto_deployment);
    assert_eq!(config.deployment_strategy, deserialized.deployment_strategy);
    assert_eq!(config.rollback_on_failure, deserialized.rollback_on_failure);
    Ok(())
}

#[test]
fn test_component_health_construction() {
    let health = ComponentHealth {
        status: HealthStatus::Healthy,
        message: Some(Arc::from("All systems operational")),
        last_check: Some(1234567890),
    };
    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.message.as_deref(), Some("All systems operational"));
    assert_eq!(health.last_check, Some(1234567890));
}

#[test]
fn test_orchestrator_health_construction() {
    let health = OrchestratorHealth {
        status: HealthStatus::Healthy,
        load_balancer_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: None,
        },
        performance_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: None,
        },
        registry_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: None,
        },
        scaling_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: None,
        },
    };
    assert_eq!(health.status, HealthStatus::Healthy);
}

#[test]
fn test_consolidated_orchestrator_new() {
    let config = ConsolidatedOrchestratorConfig::default();
    let orchestrator = ConsolidatedOrchestrator::new(config);

    assert!(format!("{:?}", orchestrator).contains("ConsolidatedOrchestrator"));
}

#[test]
fn test_consolidated_orchestrator_new_with_custom_config() {
    use songbird_config::canonical::resilience::LoadBalancingAlgorithm;
    let mut config = ConsolidatedOrchestratorConfig::default();
    config.load_balancing.algorithm = LoadBalancingAlgorithm::LeastConnections;
    config.scaling.min_instances = 2;
    config.scaling.max_instances = 20;

    let orchestrator = ConsolidatedOrchestrator::new(config);

    assert!(format!("{:?}", orchestrator).contains("ConsolidatedOrchestrator"));
}

#[tokio::test]
async fn test_consolidated_orchestrator_initialize() {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    let result = orchestrator.initialize().await;
    assert!(result.is_ok(), "Initialize should succeed");
}

#[tokio::test]
async fn test_consolidated_orchestrator_start() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    orchestrator.initialize().await?;

    let result = orchestrator.start().await;
    assert!(result.is_ok(), "Start should succeed");
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_stop() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    orchestrator.initialize().await?;
    orchestrator.start().await?;

    let result = orchestrator.stop().await;
    assert!(result.is_ok(), "Stop should succeed");
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_health_check() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let orchestrator = ConsolidatedOrchestrator::new(config);

    let health = orchestrator.health_check().await?;
    assert_eq!(health.status, HealthStatus::Healthy);
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_full_lifecycle() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    orchestrator.initialize().await?;
    orchestrator.start().await?;

    let health = orchestrator.health_check().await?;
    assert_eq!(health.status, HealthStatus::Healthy);

    orchestrator.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_health_check_components() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let orchestrator = ConsolidatedOrchestrator::new(config);

    let health = orchestrator.health_check().await?;

    assert_eq!(health.load_balancer_health.status, HealthStatus::Healthy);
    assert_eq!(health.performance_health.status, HealthStatus::Healthy);
    assert_eq!(health.registry_health.status, HealthStatus::Healthy);
    assert_eq!(health.scaling_health.status, HealthStatus::Healthy);
    Ok(())
}

#[test]
fn test_consolidated_orchestrator_config_clone() {
    let config = ConsolidatedOrchestratorConfig::default();
    let cloned = config.clone();

    assert_eq!(config.load_balancing.algorithm, cloned.load_balancing.algorithm);
    assert_eq!(config.scaling.enable_auto_scaling, cloned.scaling.enable_auto_scaling);
}

#[test]
fn test_consolidated_orchestrator_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConsolidatedOrchestratorConfig::default();
    let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;
    let deserialized: ConsolidatedOrchestratorConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;

    assert_eq!(config.load_balancing.algorithm, deserialized.load_balancing.algorithm);
    assert_eq!(config.scaling.min_instances, deserialized.scaling.min_instances);
    Ok(())
}

#[test]
fn test_load_balancing_config_custom() {
    use songbird_config::canonical::resilience::LoadBalancingAlgorithm;

    let config = CanonicalLoadBalancerConfig {
        algorithm: LoadBalancingAlgorithm::LeastConnections,
        sticky_sessions: true,
        session_timeout_secs: 600,
        max_connections_per_backend: 200,
        connection_timeout_ms: 60000,
        fail_fast: false,
    };

    assert_eq!(config.algorithm, LoadBalancingAlgorithm::LeastConnections);
    assert!(config.sticky_sessions);
    assert_eq!(config.max_connections_per_backend, 200);
    assert_eq!(config.session_timeout_secs, 600);
}

#[test]
fn test_performance_config_custom_thresholds() {
    let mut thresholds = HashMap::new();
    thresholds.insert(Arc::from("custom_metric"), 95.0);

    let config = PerformanceConfig {
        metrics_interval: 30,
        alert_thresholds: thresholds,
        enable_benchmarking: false,
    };

    assert_eq!(config.metrics_interval, 30);
    assert!(!config.enable_benchmarking);
    assert_eq!(config.alert_thresholds.get(&Arc::from("custom_metric")), Some(&95.0));
}

#[test]
fn test_registry_config_custom() {
    let config = RegistryConfig {
        discovery_interval: 60,
        service_timeout: 600,
        max_services: 5000,
    };

    assert_eq!(config.discovery_interval, 60);
    assert_eq!(config.service_timeout, 600);
    assert_eq!(config.max_services, 5000);
}

#[test]
fn test_scaling_config_custom() {
    let config = ScalingConfig {
        enable_auto_scaling: false,
        scale_up_threshold: 80.0,
        scale_down_threshold: 20.0,
        min_instances: 3,
        max_instances: 50,
    };

    assert!(!config.enable_auto_scaling);
    assert_eq!(config.scale_up_threshold, 80.0);
    assert_eq!(config.scale_down_threshold, 20.0);
    assert_eq!(config.min_instances, 3);
    assert_eq!(config.max_instances, 50);
}

#[test]
fn test_zero_touch_config_custom() {
    let config = ZeroTouchConfig {
        enable_auto_deployment: true,
        deployment_strategy: DeploymentStrategy::Canary,
        rollback_on_failure: false,
    };

    assert!(config.enable_auto_deployment);
    assert_eq!(config.deployment_strategy, DeploymentStrategy::Canary);
    assert!(!config.rollback_on_failure);
}

#[test]
fn test_deployment_strategy_all_variants() {
    let strategies = [
        DeploymentStrategy::BlueGreen,
        DeploymentStrategy::RollingUpdate,
        DeploymentStrategy::Canary,
    ];

    assert_eq!(strategies.len(), 3);
    assert_eq!(strategies[0], DeploymentStrategy::BlueGreen);
    assert_eq!(strategies[1], DeploymentStrategy::RollingUpdate);
    assert_eq!(strategies[2], DeploymentStrategy::Canary);
}

#[test]
fn test_health_status_all_variants() {
    let statuses = [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];

    assert_eq!(statuses.len(), 4);
    assert_eq!(statuses[0], HealthStatus::Healthy);
    assert_eq!(statuses[1], HealthStatus::Degraded);
    assert_eq!(statuses[2], HealthStatus::Unhealthy);
    assert_eq!(statuses[3], HealthStatus::Unknown);
}

#[test]
fn test_component_health_with_message() {
    let health = ComponentHealth {
        status: HealthStatus::Degraded,
        message: Some(Arc::from("High memory usage")),
        last_check: Some(chrono::Utc::now().timestamp() as u64),
    };

    assert_eq!(health.status, HealthStatus::Degraded);
    assert!(health.message.is_some());
    assert!(health.last_check.is_some());
}

#[test]
fn test_orchestrator_config_debug_format() {
    let config = ConsolidatedOrchestratorConfig::default();
    let debug_string = format!("{:?}", config);

    assert!(debug_string.contains("ConsolidatedOrchestratorConfig"));
    assert!(debug_string.contains("load_balancing"));
    assert!(debug_string.contains("performance"));
    assert!(debug_string.contains("registry"));
    assert!(debug_string.contains("scaling"));
}

#[test]
fn test_component_health_json_roundtrip_preserves_arc_message()
-> Result<(), Box<dyn std::error::Error>> {
    let health = ComponentHealth {
        status: HealthStatus::Degraded,
        message: Some(Arc::from("queue lag")),
        last_check: Some(12345),
    };
    let json = serde_json::to_string(&health)?;
    let back: ComponentHealth = serde_json::from_str(&json)?;
    assert_eq!(back.status, HealthStatus::Degraded);
    assert_eq!(back.message.as_deref(), Some("queue lag"));
    assert_eq!(back.last_check, Some(12345));
    Ok(())
}

#[test]
fn test_performance_config_json_roundtrip_preserves_threshold_keys()
-> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceConfig::default();
    let json = serde_json::to_string(&config)?;
    let back: PerformanceConfig = serde_json::from_str(&json)?;
    assert_eq!(back.alert_thresholds.get(&Arc::from("cpu_usage")), Some(&80.0));
    assert_eq!(back.alert_thresholds.get(&Arc::from("response_time")), Some(&1000.0));
    Ok(())
}

#[test]
fn orchestrator_health_roundtrip_json() -> Result<(), Box<dyn std::error::Error>> {
    let health = OrchestratorHealth {
        status: HealthStatus::Degraded,
        load_balancer_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some(Arc::from("ok")),
            last_check: Some(1),
        },
        performance_health: ComponentHealth {
            status: HealthStatus::Unknown,
            message: None,
            last_check: None,
        },
        registry_health: ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: Some(Arc::from("down")),
            last_check: Some(2),
        },
        scaling_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: None,
        },
    };
    let json = serde_json::to_string(&health)?;
    let back: OrchestratorHealth = serde_json::from_str(&json)?;
    assert_eq!(back.status, HealthStatus::Degraded);
    assert_eq!(back.registry_health.message.as_deref(), Some("down"));
    Ok(())
}

#[test]
fn scaling_config_extreme_instances_still_serializes() -> Result<(), Box<dyn std::error::Error>> {
    let config = ScalingConfig {
        enable_auto_scaling: false,
        scale_up_threshold: 99.0,
        scale_down_threshold: 1.0,
        min_instances: 100,
        max_instances: 100,
    };
    let json = serde_json::to_string(&config)?;
    let back: ScalingConfig = serde_json::from_str(&json)?;
    assert_eq!(back.min_instances, 100);
    assert_eq!(back.max_instances, 100);
    Ok(())
}

#[test]
fn registry_config_zero_max_services_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig {
        discovery_interval: 0,
        service_timeout: 0,
        max_services: 0,
    };
    let json = serde_json::to_string(&config)?;
    let back: RegistryConfig = serde_json::from_str(&json)?;
    assert_eq!(back.max_services, 0);
    Ok(())
}

#[test]
fn performance_config_empty_alert_thresholds_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let config = PerformanceConfig {
        metrics_interval: 10,
        alert_thresholds: HashMap::new(),
        enable_benchmarking: false,
    };
    let json = serde_json::to_string(&config)?;
    let back: PerformanceConfig = serde_json::from_str(&json)?;
    assert!(back.alert_thresholds.is_empty());
    assert_eq!(back.metrics_interval, 10);
    Ok(())
}

#[test]
fn orchestrator_health_mixed_component_statuses() {
    let health = OrchestratorHealth {
        status: HealthStatus::Degraded,
        load_balancer_health: ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: Some(1),
        },
        performance_health: ComponentHealth {
            status: HealthStatus::Unknown,
            message: Some(Arc::from("metrics delayed")),
            last_check: None,
        },
        registry_health: ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: Some(Arc::from("stale")),
            last_check: Some(2),
        },
        scaling_health: ComponentHealth {
            status: HealthStatus::Degraded,
            message: None,
            last_check: Some(3),
        },
    };
    assert_eq!(health.status, HealthStatus::Degraded);
    assert_eq!(health.registry_health.status, HealthStatus::Unhealthy);
}

#[test]
fn consolidated_orchestrator_config_preserves_zero_touch_api() {
    let mut c = ConsolidatedOrchestratorConfig::default();
    c.api.port = 9090;
    c.api.enable_cors = false;
    c.zero_touch.enable_auto_deployment = true;
    let json = serde_json::to_string(&c).expect("serialize config");
    let back: ConsolidatedOrchestratorConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.zero_touch.enable_auto_deployment, true);
    assert_eq!(back.api.port, 9090);
    assert_eq!(back.api.enable_cors, false);
}
