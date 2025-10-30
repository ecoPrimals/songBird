//! Tests for core orchestrator functionality

use songbird_orchestrator::core::*;

#[test]
fn test_consolidated_orchestrator_config_default() {
    let config = ConsolidatedOrchestratorConfig::default();
    
    // Should create without panic
    assert!(format!("{:?}", config).contains("ConsolidatedOrchestratorConfig"));
}

#[test]
fn test_consolidated_orchestrator_new() {
    let config = ConsolidatedOrchestratorConfig::default();
    let orchestrator = ConsolidatedOrchestrator::new(config);
    
    // Should create without panic
    assert!(format!("{:?}", orchestrator).contains("ConsolidatedOrchestrator"));
}

#[tokio::test]
async fn test_consolidated_orchestrator_initialize() {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);
    
    let result = orchestrator.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_consolidated_orchestrator_start() {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);
    
    // Initialize first
    orchestrator.initialize().await.unwrap();
    
    // Then start
    let result = orchestrator.start().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_consolidated_orchestrator_stop() {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);
    
    orchestrator.initialize().await.unwrap();
    orchestrator.start().await.unwrap();
    
    let result = orchestrator.stop().await;
    assert!(result.is_ok());
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
    assert!(config.alert_thresholds.len() > 0);
}

#[test]
fn test_registry_config_default() {
    let config = RegistryConfig::default();
    
    // Test that defaults are reasonable
    assert!(config.discovery_interval > 0);
    assert!(config.service_timeout > 0);
    assert!(config.max_services > 0);
}

#[test]
fn test_scaling_config_default() {
    let config = ScalingConfig::default();
    
    assert_eq!(config.min_instances, 1);
    assert_eq!(config.max_instances, 10);
    assert!(config.max_instances >= config.min_instances);
}

#[test]
fn test_config_serialization() {
    let config = ConsolidatedOrchestratorConfig::default();
    
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: ConsolidatedOrchestratorConfig = 
        serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(config.registry.max_services, deserialized.registry.max_services);
    assert_eq!(config.performance.metrics_interval, deserialized.performance.metrics_interval);
}

#[test]
fn test_load_balancing_strategy_variants() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::WeightedRoundRobin,
    ];
    
    assert_eq!(strategies.len(), 3);
}

#[test]
fn test_api_config_default() {
    let config = ApiConfig::default();
    
    assert_eq!(config.port, 8080);
}

#[test]
fn test_zero_touch_config_default() {
    let config = ZeroTouchConfig::default();
    
    // Test that config can be created
    assert!(format!("{:?}", config).contains("ZeroTouchConfig"));
}

#[tokio::test]
async fn test_consolidated_orchestrator_lifecycle() {
    let config = ConsolidatedOrchestratorConfig::default();
    let mut orchestrator = ConsolidatedOrchestrator::new(config);
    
    // Full lifecycle test
    assert!(orchestrator.initialize().await.is_ok());
    assert!(orchestrator.start().await.is_ok());
    assert!(orchestrator.stop().await.is_ok());
}


#[test]
fn test_orchestrator_with_custom_config() {
    let config = ConsolidatedOrchestratorConfig {
        load_balancing: LoadBalancingConfig::default(),
        performance: PerformanceConfig::default(),
        registry: RegistryConfig::default(),
        scaling: ScalingConfig::default(),
        api: ApiConfig::default(),
        zero_touch: ZeroTouchConfig::default(),
    };
    
    let orchestrator = ConsolidatedOrchestrator::new(config);
    assert!(format!("{:?}", orchestrator).contains("ConsolidatedOrchestrator"));
}

#[test]
fn test_health_status_enum() {
    use HealthStatus::*;
    
    let statuses = vec![Healthy, Degraded, Unhealthy];
    assert_eq!(statuses.len(), 3);
}

#[test]
fn test_config_clone() {
    let config1 = ConsolidatedOrchestratorConfig::default();
    let config2 = config1.clone();
    
    assert_eq!(config1.registry.max_services, config2.registry.max_services);
}

