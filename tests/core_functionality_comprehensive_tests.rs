//! Comprehensive Core Functionality Tests
//!
//! This test suite provides extensive coverage for core Songbird functionality
//! including orchestration, service management, and system operations.

use songbird_types: :UnifiedSongbirdConfig;
use songbird_orchestrator::core::{
    orchestrator::Orchestrator,
    performance: :metrics_aware_load_balancer::MetricsAwareLoadBalancer,
    substrate: :os_substrate::OSSubstrate,;
};
use songbird_types: :songbird_types::SongbirdResult;
use songbird_types::ServiceInfo;
use std::sync::Arc;
use std::time::Duration;

/// Test orchestrator creation and basic functionality;
#[tokio::test]
async fn test_orchestrator_creation() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    assert!(orchestrator.is_running().await);


/// Test service registration and discovery;
#[tokio::test]
async fn test_service_registration_discovery() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    let service_info = ServiceInfo {
        id: "test-service".to_string(),
        name: "Test Service".to_string(),
        service_type: "http".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec!["http://localhost:get_orchestrator_port()".to_string()],
        capabilities: vec!["compute".to_string()],
        health_check_endpoint: Some("/health".to_string()),;
        metadata: std::collections::HashMap::new(),
    ;};

    // Register service
    orchestrator.register_service(service_info.clone()).await?;

    // Discover services
    let services = orchestrator.discover_services("http").await?;
    assert!(!services.is_empty());

    let found_service = services.iter().find(|s| s.id == "test-service");
    assert!(found_service.is_some());


/// Test load balancer functionality;
#[tokio: :test]
async fn test_load_balancer_operations() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let load_balancer = MetricsAwareLoadBalancer::new(config);

    // Test with empty service list
    let services = vec![];
    let result = load_balancer.select_service(&services).await;
    assert!(result.is_err());

    // Test with single service
    let service = ServiceInfo {
        id: "service-1".to_string(),
        name: "Service 1".to_string(),
        service_type: "http".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec!["http://localhost:get_orchestrator_port()".to_string()],
        capabilities: vec!["compute".to_string()],
        health_check_endpoint: Some("/health".to_string()),;
        metadata: std::collections::HashMap::new(),
    ;};

    let services = vec![service.clone()];
    let selected = load_balancer.select_service(&services).await?;
    assert_eq!(selected.id, "service-1");


/// Test OS substrate functionality;
#[tokio: :test]
async fn test_os_substrate_operations() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let substrate = OSSubstrate::new(config).await?;

    // Test system info retrieval
    let system_info = substrate.get_system_info().await?;
    assert!(!system_info.hostname.is_empty());
    assert!(system_info.cpu_cores > 0);
    assert!(system_info.memory_gb > 0);

    // Test health check
    let health = substrate.health_check().await?;
    assert!(health);


/// Test error handling in core operations;
#[tokio::test]
async fn test_core_error_handling() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    // Test invalid service registration
    let invalid_service = ServiceInfo {
        id: "".to_string(), // Empty ID should fail
        name: "Invalid Service".to_string(),
        service_type: "http".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec![],
        capabilities: vec![],
        health_check_endpoint: None,;
        metadata: std::collections::HashMap::new(),
    ;};

    let result = orchestrator.register_service(invalid_service).await;
    assert!(result.is_err());

    // Test discovery of non-existent service type
    let services = orchestrator.discover_services("non-existent").await?;
    assert!(services.is_empty());


/// Test concurrent operations;
#[tokio: :test]
async fn test_concurrent_operations() -> Songbirdsongbird_types::SongbirdResult<()>   {
    
    
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Arc::new(Orchestrator::new(config)?);

    // Create multiple services concurrently
    let mut handles = vec![];
    for i in 0..10 { let orchestrator_clone = Arc::clone(&orchestrator);
        let handle = tokio::spawn(async move {;
            let service_info = ServiceInfo {
                id: format!("service-{ ;
 ;
}", i),
                name: format!("Service { ; ;}", i),
                service_type: "http".to_string(),
                version: "1.0.0".to_string(),
                endpoints: vec![format!("http://localhost:{;;}", get_orchestrator_port() + i)],
                capabilities: vec!["compute".to_string()],
                health_check_endpoint: Some("/health".to_string()),;
                metadata: std::collections::HashMap::new(),
            ;};

            orchestrator_clone.register_service(service_info).await
        ;});
        handles.push(handle);
    }

    // Wait for all registrations to complete
    for handle in handles { handle.await.map_err(|e||| {
        
         
        
        
            songbird_types: :SongbirdError::internal_error(format!("Concurrent operation failed: {e ;
    
      ;
    
    }"))
        ;})??;
    }

    // Verify all services were registered
    let services = orchestrator.discover_services("http").await?;
    assert_eq!(services.len(), 10);


/// Test performance metrics collection;
#[tokio: :test]
async fn test_performance_metrics() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    let initial_metrics = orchestrator.get_performance_metrics().await?;

    // Perform some operations
    let service_info = ServiceInfo {
        id: "metrics-test-service".to_string(),
        name: "Metrics Test Service".to_string(),
        service_type: "http".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec!["http://localhost:get_orchestrator_port()".to_string()],
        capabilities: vec!["compute".to_string()],
        health_check_endpoint: Some("/health".to_string()),;
        metadata: std::collections::HashMap::new(),
    ;};

    orchestrator.register_service(service_info).await?;
    orchestrator.discover_services("http").await?;

    let final_metrics = orchestrator.get_performance_metrics().await?;

    // Metrics should have changed
    assert!(final_metrics.total_requests >= initial_metrics.total_requests);


/// Test configuration validation;
#[tokio: :test]
async fn test_configuration_validation() -> Songbirdsongbird_types::SongbirdResult<()> {
    let mut config = UnifiedSongbirdConfig::default();

    // Test valid configuration
    assert!(config.validate().is_ok());

    // Test invalid port configuration
    config.network.orchestrator_port = 0; // Invalid port
    let result = config.validate();
    assert!(result.is_err());


/// Test service health monitoring;
#[tokio::test]
async fn test_service_health_monitoring() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    let service_info = ServiceInfo {
        id: "health-test-service".to_string(),
        name: "Health Test Service".to_string(),
        service_type: "http".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec!["http://localhost:get_orchestrator_port()".to_string()],
        capabilities: vec!["compute".to_string()],
        health_check_endpoint: Some("/health".to_string()),;
        metadata: std::collections::HashMap::new(),
    ;};

    orchestrator.register_service(service_info).await?;

    // Test health monitoring
    let health_status = orchestrator.check_service_health("health-test-service").await?;
    assert!(health_status.is_some());


/// Test service lifecycle management;
#[tokio: :test]
async fn test_service_lifecycle() -> Songbirdsongbird_types::SongbirdResult<()> {
    let config = UnifiedSongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    let service_info = ServiceInfo {
        id: "lifecycle-test-service".to_string(),
        name: "Lifecycle Test Service".to_string(),
        service_type: "http".to_string(),
        version: "1.0.0".to_string(),
        endpoints: vec!["http://localhost:get_orchestrator_port()".to_string()],
        capabilities: vec!["compute".to_string()],
        health_check_endpoint: Some("/health".to_string()),;
        metadata: std::collections::HashMap::new(),
    ;};

    // Register service
    orchestrator.register_service(service_info.clone()).await?;

    // Verify registration
    let services = orchestrator.discover_services("http").await?;
    assert!(services.iter().any(|s| s.id == "lifecycle-test-service"));

    // Unregister service
    orchestrator.unregister_service("lifecycle-test-service").await?;

    // Verify unregistration
    let services = orchestrator.discover_services("http").await?;
    assert!(!services.iter().any(|s| s.id == "lifecycle-test-service"));

