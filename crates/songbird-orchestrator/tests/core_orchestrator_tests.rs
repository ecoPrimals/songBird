//! Tests for core orchestrator functionality

use songbird_orchestrator::core::*;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_consolidated_orchestrator_config_default() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();

    // Should create without panic
    assert!(format!("{config:?}").contains("ConsolidatedOrchestratorConfig"));
    Ok(())
}

#[test]
fn test_consolidated_orchestrator_new() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let orchestrator = ConsolidatedOrchestrator::new(config);

    // Should create without panic
    assert!(format!("{orchestrator:?}").contains("ConsolidatedOrchestrator"));
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_initialize() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    let result = orchestrator.initialize().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_start() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    // Initialize first
    orchestrator.initialize().await.map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    // Then start
    let result = orchestrator.start().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_stop() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    orchestrator.initialize().await.map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    orchestrator.start().await.map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    let result = orchestrator.stop().await;
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_load_balancing_config_default() {
    let config = LoadBalancingConfig::default();

    assert_eq!(config.health_check_interval, 30);
    assert_eq!(config.max_retries, 3);
    assert!(matches!(config.strategy, LoadBalancingStrategy::RoundRobin));
}

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();

    assert_eq!(config.metrics_interval, 60);
    assert!(config.enable_benchmarking);
    assert!(!config.alert_thresholds.is_empty());
}

#[test]
fn test_registry_config_default() -> SongbirdResult<()> {
    let config = RegistryConfig::default();

    // Test that defaults are reasonable
    assert!(config.discovery_interval > 0);
    assert!(config.service_timeout > 0);
    assert!(config.max_services > 0);
    Ok(())
}

#[test]
fn test_scaling_config_default() -> SongbirdResult<()> {
    let config = ScalingConfig::default();

    assert_eq!(config.min_instances, 1);
    assert_eq!(config.max_instances, 10);
    assert!(config.max_instances >= config.min_instances);
    Ok(())
}

#[test]
fn test_config_serialization() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();

    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ConsolidatedOrchestratorConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;

    assert_eq!(config.registry.max_services, deserialized.registry.max_services);
    assert_eq!(config.performance.metrics_interval, deserialized.performance.metrics_interval);
    Ok(())
}

#[test]
fn test_load_balancing_strategy_variants() -> SongbirdResult<()> {
    let strategies = [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::WeightedRoundRobin,
    ];

    assert_eq!(strategies.len(), 3);
    Ok(())
}

#[test]
fn test_api_config_default() -> SongbirdResult<()> {
    let config = ApiConfig::default();

    assert_eq!(config.port, songbird_config::defaults::ports::orchestrator_port());
    Ok(())
}

#[test]
fn test_zero_touch_config_default() -> SongbirdResult<()> {
    let config = ZeroTouchConfig::default();

    // Test that config can be created
    assert!(format!("{config:?}").contains("ZeroTouchConfig"));
    Ok(())
}

#[tokio::test]
async fn test_consolidated_orchestrator_lifecycle() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);

    // Full lifecycle test
    assert!(orchestrator.initialize().await.is_ok());
    assert!(orchestrator.start().await.is_ok());
    assert!(orchestrator.stop().await.is_ok());
    Ok(())
}

#[test]
fn test_orchestrator_with_custom_config() -> SongbirdResult<()> {
    let config = ConsolidatedOrchestratorConfig {
        load_balancing: LoadBalancingConfig::default(),
        performance: PerformanceConfig::default(),
        registry: RegistryConfig::default(),
        scaling: ScalingConfig::default(),
        api: ApiConfig::default(),
        zero_touch: ZeroTouchConfig::default(),
    };

    let orchestrator = ConsolidatedOrchestrator::new(config);
    assert!(format!("{orchestrator:?}").contains("ConsolidatedOrchestrator"));
    Ok(())
}

#[test]
fn test_health_status_enum() {
    use songbird_types::{SongbirdError, SongbirdResult};
    use HealthStatus::*;

    let statuses = [Healthy, Degraded, Unhealthy];
    assert_eq!(statuses.len(), 3);
}

#[test]
fn test_config_clone() {
    let config1 = ConsolidatedOrchestratorConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.registry.max_services, config2.registry.max_services);
}
