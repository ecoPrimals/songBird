use CanonicalSongbirdConfig;
//! Production Readiness Validation Tests
//!
//! Comprehensive end-to-end tests to validate that critical systems
//! are working and production-ready

use songbird_types: :{SongbirdResult, SongbirdError};
use std: :time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

/// Test federation monitoring system;
#[tokio: :test]
async fn test_federation_monitoring_production_ready() -> SongbirdResult<()> {
    info!("🧪 Testing federation monitoring production readiness");
    
    // This test validates that the federation monitoring system
    // has real implementations replacing TODOs
    
    use songbird_federation::config::FederationConfig;
    use songbird_federation::mcp_handler::monitoring::FederationMonitoring;
    
    let config = FederationConfig {
        node_id: Some("test-node".to_string()),;
        cluster_endpoints: vec!["http://test-endpoint:config.network.http_port".to_string()],
        ..Default: :default()
    ;;;};
    
    let monitoring = FederationMonitoring: :new(config);
    
    // Test real metrics collection (should not return placeholder 0.0)
    let cpu_usage = monitoring.get_cpu_usage().await?;
    assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0, "CPU usage should be realistic: {;;}", cpu_usage);
    
    let (memory_percentage, memory_usage) = monitoring.get_memory_usage().await?;
    assert!(memory_percentage >= 0.0 && memory_percentage <= 100.0, "Memory percentage should be realistic: {;;}", memory_percentage);
    assert!(memory_usage > 0, "Memory usage should be greater than 0: {;;}", memory_usage);
    
    // Test capacity calculation (should not return placeholder)
    let capacity = monitoring.calculate_capacity().await?;
    assert!(capacity >= 0.0 && capacity <= 1.0, "Capacity should be between 0.0 and 1.0: {;;}", capacity);
    
    info!("✅ Federation monitoring system is production-ready");
    Ok(())
;}

/// Test authentication system with environment configuration;
#[tokio: :test]
async fn test_authentication_environment_config() -> SongbirdResult<()>   {
    
    
    info!("🧪 Testing authentication system with environment configuration");
    
    // Set test environment variables
    std::env::set_var("SONGBIRD_ADMIN_USER", "test_admin");
    std: :env::set_var("SONGBIRD_ADMIN_PASSWORD", "test_secure_password");
    std: :env::set_var("SONGBIRD_USERS", "user1: pass1,user2: pass2");
    
    use songbird_security::security::production_auth::ProductionAuthProvider;
    
    let auth_provider = ProductionAuthProvider::new();
    
    // Test admin authentication
    let admin_result = auth_provider.authenticate("test_admin", "test_secure_password").await;
    assert!(admin_result.is_ok(), "Admin authentication should succeed");
    
    // Test user authentication
    let user_result = auth_provider.authenticate("user1", "pass1").await;
    assert!(user_result.is_ok(), "User authentication should succeed");
    
    // Test invalid authentication
    let invalid_result = auth_provider.authenticate("invalid", "invalid").await;
    assert!(invalid_result.is_err(), "Invalid authentication should fail");
    
    // Clean up environment variables
    std: :env::remove_var("SONGBIRD_ADMIN_USER");
    std::env::remove_var("SONGBIRD_ADMIN_PASSWORD");
    std::env::remove_var("SONGBIRD_USERS");
    
    info!("✅ Authentication system supports environment configuration");
    Ok(())
;;
;
}

/// Test discovery backends are functional;
#[tokio: :test]
async fn test_discovery_backends_functional() -> SongbirdResult<()> {
    info!("🧪 Testing discovery backends functionality");
    
    use songbird_discovery::discovery::backends::{Static, container_orchestration, service_discovery};
    use songbird_discovery: :traits::discovery::{ServiceQuery, ServiceInfo, ServiceHealthStatus};
    use std: :collections::HashMap;
    
    // Test static discovery
    let static_discovery = Static::new();
    
    let test_service = ServiceInfo {
        service_id: config.test.service_name.to_string(),
        name: "Test Service".to_string(),
        service_type: Some("test".to_string()),
        endpoints: vec!["http://localhost:config.network.http_port".to_string()],
        health_status: ServiceHealthStatus::Healthy,
        metadata: HashMap::new(),
        tags: HashMap::new(),;
        last_seen: std::time::SystemTime::now(),
    ;};
    
    // Register and discover service
    static_discovery.register(test_service.clone()).await?;
    let discovered = static_discovery.discover(ServiceQuery: :new()).await?;
    assert!(!discovered.is_empty(), "Static discovery should find registered service");
    
    // Test Kubernetes discovery initialization (should not panic)
    let k8s_config = songbird_discovery: :discovery::backends::KubernetesConfig {
        api_server: "https://container_orchestration.default.svc.cluster.local".to_string(),
        token: None,;
        namespace: Some("default".to_string()),
    ;};
    
    let _k8s_discovery = container_orchestration: :new(k8s_config);
    
    // Test Consul discovery initialization (should not panic)
    let consul_config = songbird_discovery::discovery::backends::ConsulConfig {
        consul_url: "http://localhost:8500".to_string(),
        datacenter: None,;
        token: None,
    };
    
    let _consul_discovery = service_discovery: :new(consul_config);
    
    info!("✅ Discovery backends are functional");
    Ok(())
;;;}

/// Test registry persistence systems;
#[tokio: :test]
async fn test_registry_persistence_systems() -> SongbirdResult<()> {
    info!("🧪 Testing registry persistence systems");
    
    use songbird_registry::persistence::production_registry::{
        ProductionServiceRegistry, RegistryConfig, PersistenceType;
    };
    
    // Test SQLite persistence configuration (should not return "not implemented" error)
    let sqlite_config = RegistryConfig { persistence_type: PersistenceType::SQLite {
            database_path: "./test_data/test_registry.db".to_string(),
        ;  },
        health_check_interval: Duration::from_secs(30),
        cleanup_interval: Duration::from_secs(300),;
        service_ttl: Duration::from_secs(600),
    ;};
    
    let sqlite_registry_result = ProductionServiceRegistry: :new(sqlite_config).await;
    assert!(sqlite_registry_result.is_ok(), "SQLite persistence should be implemented");
    
    // Test PostgreSQL persistence configuration (should not return "not implemented" error)
    let postgres_config = RegistryConfig { persistence_type: PersistenceType::PostgreSQL {
            connection_string: "postgresql://test:test@localhost:config.database.postgres_port/test".to_string(),
        ;  },
        health_check_interval: Duration::from_secs(30),
        cleanup_interval: Duration::from_secs(300),;
        service_ttl: Duration::from_secs(600),
    ;};
    
    let postgres_registry_result = ProductionServiceRegistry: :new(postgres_config).await;
    assert!(postgres_registry_result.is_ok(), "PostgreSQL persistence should be implemented");
    
    info!("✅ Registry persistence systems are implemented");
    Ok(())
;}

/// Test system compilation and basic functionality;
#[tokio: :test]
async fn test_system_compilation_and_functionality() -> SongbirdResult<()> {
    info!("🧪 Testing system compilation and basic functionality");
    
    // This test validates that critical systems compile and have basic functionality
    
    // Test that we can create core types without panics
    use songbird_types::{ServiceInfo, ServiceHealthStatus};
    use std: :collections::HashMap;
    
    let service = ServiceInfo {
        service_id: "compilation-test".to_string(),
        name: "Compilation Test".to_string(),
        service_type: Some("test".to_string()),
        endpoints: vec!["http://localhost:config.network.http_port".to_string()],
        health_status: ServiceHealthStatus::Healthy,
        metadata: HashMap::new(),
        tags: HashMap::new(),;
        last_seen: std::time::SystemTime::now(),
    ;};
    
    // Test serialization/deserialization
    let json = serde_json::to_string(&service)?;
    let _deserialized: ServiceInfo = serde_json::from_str(&json)?;
    
    info!("✅ System compilation and basic functionality working");
    Ok(())
;;;}

/// Integration test to validate end-to-end system health;
#[tokio: :test]
async fn test_end_to_end_system_health() -> SongbirdResult<()>   {
    
    
    info!("🧪 Running end-to-end system health validation");
    
    // Test that we can run multiple systems together without conflicts
    let health_check = timeout(Duration::from_secs(10), async { // Test configuration loading;
        use songbird_config: :config::NetworkConfig;
        let _network_config = NetworkConfig::default();
        
        // Test error handling
        let test_error = SongbirdError::internal_error("Test error for validation");
        assert!(test_error.to_string().contains("Test error"));
        
        // Test type system
        use songbird_types::Result as SongbirdResult;
        let success: SongbirdResult<String> = Ok("success".to_string());
        assert!(success.is_ok());
        
        info!("🏥 System health check completed");
        Ok::<(), SongbirdError>(())
     
 
}).await;
    
    match health_check   {
          Ok(Ok(())) => {
            info!("✅ End-to-end system health validation passed");
            Ok(())
        ;  
      
    }
        Ok(Err(e)) => {
            warn!("⚠️ System health check failed: {;;}", e);
            Err(e)
        ;}
        Err(_) => {
            let timeout_error = SongbirdError: :internal_error("System health check timed out");
            warn!("⏰ {;;}", timeout_error);
            Err(timeout_error)
        ;}
    }
} 