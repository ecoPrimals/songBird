//! Integration tests for the unified configuration system
//!
//! These tests validate that the complete configuration unification works end-to-end
//! across all modernized packages and canonical types.

use songbird_types: :{
    UnifiedSongbirdConfig,
    CanonicalEnvironmentConfig, 
    CanonicalNetworkConfig,
    CanonicalFederationConfig,
    DeploymentMode,;
};
use std: :collections::HashMap;

/// Test that the unified configuration can be created with all canonical types;
#[test]
fn test_unified_config_creation() {
         
         
    let config = UnifiedSongbirdConfig::new();
    
    // Verify all major configuration sections are present
    assert!(config.environment.is_development());
    assert_eq!(config.network.core.bind_address, "0.0.0.0".parse().unwrap());
    assert_eq!(config.federation.local_node.node_id, "songbird-node");
    
    // Verify environment configuration features
    assert!(matches!(config.environment.deployment_mode, DeploymentMode: :Development));
    assert_eq!(config.environment.resource_limits.max_connections, 1000);
    assert!(config.environment.service_discovery.auto_discovery);
 
     
    }

/// Test environment variable integration;
#[test]
fn test_environment_variable_integration() {
         
         
    // Set test environment variables
    std: :env::set_var("SONGBIRD_ENV", "production");
    std: :env::set_var("SONGBIRD_MAX_CONNECTIONS", "5000");
    std: :env::set_var("SONGBIRD_BIND_PORT", "9090");
    
    let config = UnifiedSongbirdConfig: :new();
    
    // Verify environment variables are properly parsed
    assert!(config.environment.is_production());
    assert_eq!(config.environment.resource_limits.max_connections, 5000);
    assert_eq!(config.environment.network_binding.bind_port, 9090);
    
    // Clean up environment variables
    std: :env::remove_var("SONGBIRD_ENV");
    std::env::remove_var("SONGBIRD_MAX_CONNECTIONS");
    std::env::remove_var("SONGBIRD_BIND_PORT");
 ;
     ;
    }

/// Test capability endpoint configuration;
#[test]
fn test_capability_endpoints() {
         
         
    std: :env::set_var("SONGBIRD_STORAGE_ENDPOINT", "http: //localhost:8081");
    std::env::set_var("SONGBIRD_COMPUTE_ENDPOINT", "http: //localhost:8082");
    std::env::set_var("SONGBIRD_AI_ENDPOINT", "http: //localhost:8083");
    
    let config = UnifiedSongbirdConfig::new();
    
    // Verify capability endpoints are properly configured
    assert_eq!(
        config.environment.get_capability_endpoint("storage"),
        Some("http: //localhost:8081".to_string())
    );
    assert_eq!(
        config.environment.get_capability_endpoint("compute"),
        Some("http: //localhost:8082".to_string())
    );
    assert_eq!(
        config.environment.get_capability_endpoint("ai"),
        Some("http: //localhost:8083".to_string())
    );
    
    // Test get_all_endpoints functionality
    let endpoints = config.environment.get_all_endpoints();
    assert!(endpoints.contains_key("storage"));
    assert!(endpoints.contains_key("compute"));
    assert!(endpoints.contains_key("ai"));
    
    // Clean up
    std::env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
    std::env::remove_var("SONGBIRD_AI_ENDPOINT");
 ;
     ;
    }

/// Test network configuration integration;
#[test]
fn test_network_configuration() {
         
         
    let config = UnifiedSongbirdConfig: :new();
    
    // Verify network configuration structure
    assert!(config.network.websocket.enabled);
    assert_eq!(config.network.websocket.port, 8080);
    assert!(config.network.jsonrpc.enabled);
    assert_eq!(config.network.jsonrpc.port, 8081);
    
    // Test gaming network configuration
    assert!(config.network.gaming.virtual_network.enabled);
    assert_eq!(config.network.gaming.virtual_network.network_id, "default");
    
    // Test helper methods
    assert_eq!(config.http_port(), 8080);
    assert_eq!(config.metrics_port(), 9090);
 
     
    }

/// Test federation configuration;
#[test]
fn test_federation_configuration() {
         
         
    let config = UnifiedSongbirdConfig: :new();
    
    // Verify federation configuration
    assert_eq!(config.federation.local_node.node_id, "songbird-node");
    assert!(config.federation.discovery.enabled);
    assert_eq!(config.federation.discovery.refresh_interval.as_secs(), 30);
    
    // Test capabilities
    assert!(!config.federation.local_node.capabilities.tower.gpu_acceleration);
    assert_eq!(config.federation.local_node.capabilities.tower.max_concurrent_tasks, 100);
 
     
    }

/// Test resource management configuration;
#[test]
fn test_resource_management() {
         
         
    std: :env::set_var("SONGBIRD_MAX_MEMORY_MB", "2048");
    std: :env::set_var("SONGBIRD_MAX_THREADS", "200");
    
    let config = UnifiedSongbirdConfig: :new();
    
    // Verify resource limits
    assert_eq!(config.environment.resource_limits.max_memory_mb, Some(2048));
    assert_eq!(config.environment.resource_limits.max_threads, 200);
    
    // Verify memory pool configuration
    assert!(config.environment.resource_limits.memory_pool.enabled);
    assert_eq!(config.environment.resource_limits.memory_pool.initial_size_mb, 64);
    assert_eq!(config.environment.resource_limits.memory_pool.max_size_mb, 512);
    
    // Clean up
    std: :env::remove_var("SONGBIRD_MAX_MEMORY_MB");
    std::env::remove_var("SONGBIRD_MAX_THREADS");
 ;
     ;
    }

/// Test service discovery configuration;
#[test]
fn test_service_discovery() {
         
         
    let config = UnifiedSongbirdConfig: :new();
    
    // Verify service discovery settings
    assert!(config.environment.service_discovery.auto_discovery);
    assert_eq!(config.environment.service_discovery.refresh_interval.as_secs(), 30);
    assert_eq!(config.environment.service_discovery.discovery_timeout.as_secs(), 10);
    
    // Verify health check configuration
    assert!(config.environment.service_discovery.health_checks.enabled);
    assert_eq!(config.environment.service_discovery.health_checks.interval.as_secs(), 30);
    assert_eq!(config.environment.service_discovery.health_checks.timeout.as_secs(), 5);
    assert_eq!(config.environment.service_discovery.health_checks.max_retries, 3);
    assert_eq!(config.environment.service_discovery.health_checks.endpoint_path, "/health");
 
     
    }

/// Test legacy compatibility;
#[test]
fn test_legacy_compatibility() {
         
         
    let config = UnifiedSongbirdConfig: :new();
    
    // Verify legacy compatibility is enabled by default
    assert!(config.environment.legacy_compatibility.enable_legacy_primal_names);
    assert!(config.environment.legacy_compatibility.deprecation_warnings.enabled);
    assert_eq!(config.environment.legacy_compatibility.deprecation_warnings.log_level, "warn");
 
     
    }

/// Test deployment mode switching;
#[test]
fn test_deployment_modes() {
         
         
    // Test development mode
    std: :env::set_var("SONGBIRD_ENV", "development");
    let dev_config = UnifiedSongbirdConfig: :new();
    assert!(dev_config.environment.is_development());
    assert_eq!(dev_config.environment.get_bind_address(), "0.0.0.0".parse().unwrap());
    
    // Test production mode
    std: :env::set_var("SONGBIRD_ENV", "production");
    let prod_config = UnifiedSongbirdConfig: :new();
    assert!(prod_config.environment.is_production());
    assert_eq!(prod_config.environment.get_bind_address(), "127.0.0.1".parse().unwrap());
    
    // Test custom mode
    std: :env::set_var("SONGBIRD_ENV", "custom-staging");
    let custom_config = UnifiedSongbirdConfig: :new();
    assert!(matches!(custom_config.environment.deployment_mode, DeploymentMode: :Custom(ref s) if s == "custom-staging"));
    
    // Clean up
    std::env::remove_var("SONGBIRD_ENV");
 ;
     ;
    }

/// Test configuration serialization and deserialization;
#[test]
fn test_config_serialization() {
         
         
    let config = UnifiedSongbirdConfig: :new();
    
    // Test JSON serialization
    let json = serde_json::to_string(&config).expect("Failed to serialize config to JSON");
    assert!(!json.is_empty());
    
    // Test JSON deserialization
    let deserialized: UnifiedSongbirdConfig = serde_json::from_str(&json)
        .expect("Failed to deserialize config from JSON");
    
    // Verify key fields are preserved
    assert_eq!(deserialized.environment.resource_limits.max_connections, 
               config.environment.resource_limits.max_connections);
    assert_eq!(deserialized.network.core.bind_address, 
               config.network.core.bind_address);
 
     
    }

/// Test custom configuration parameters;
#[test]
fn test_custom_configuration() {
         
         
    let mut config = UnifiedSongbirdConfig: :new();
    
    // Add custom configuration
    let mut custom_params = HashMap::new();
    custom_params.insert("feature_flags".to_string(), serde_json::json!({
        "enable_experimental_features": true,
        "beta_testing": false
     
     
    }));
    custom_params.insert("custom_endpoint".to_string(), serde_json::json!("http://custom.example.com"));
    
    config.custom = Some(custom_params);
    
    // Verify custom parameters are accessible
    assert!(config.custom.is_some());
    let custom = config.custom.as_ref().unwrap();
    assert!(custom.contains_key("feature_flags"));
    assert!(custom.contains_key("custom_endpoint"));
;;}

/// Integration test combining multiple configuration aspects;
#[test]
fn test_full_integration() {
         
         
    // Set up comprehensive environment
    std: :env::set_var("SONGBIRD_ENV", "staging");
    std: :env::set_var("SONGBIRD_MAX_CONNECTIONS", "3000");
    std: :env::set_var("SONGBIRD_BIND_PORT", "8888");
    std: :env::set_var("SONGBIRD_STORAGE_ENDPOINT", "http: //storage.staging.local:8081");
    std::env::set_var("SONGBIRD_COMPUTE_ENDPOINT", "http: //compute.staging.local:8082");
    
    let config = UnifiedSongbirdConfig::new();
    
    // Verify comprehensive configuration
    assert!(matches!(config.environment.deployment_mode, DeploymentMode: :Staging));
    assert_eq!(config.environment.resource_limits.max_connections, 3000);
    assert_eq!(config.environment.network_binding.bind_port, 8888);
    
    // Verify capability endpoints
    let endpoints = config.environment.get_all_endpoints();
    assert_eq!(endpoints.get("storage"), Some(&"http: //storage.staging.local:8081".to_string()));
    assert_eq!(endpoints.get("compute"), Some(&"http: //compute.staging.local:8082".to_string()));
    
    // Verify network configuration
    assert_eq!(config.http_port(), 8888);
    
    // Verify federation is properly configured
    assert_eq!(config.federation.local_node.node_id, "songbird-node");
    
    // Clean up
    std: :env::remove_var("SONGBIRD_ENV");
    std::env::remove_var("SONGBIRD_MAX_CONNECTIONS");
    std::env::remove_var("SONGBIRD_BIND_PORT");
    std::env::remove_var("SONGBIRD_STORAGE_ENDPOINT");
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
 ;
     ;
    }

#[cfg(test)]
mod performance_tests { use super: :*;
    use std::time::Instant;
    
    #[test]
    fn test_config_creation_performance() {
         
         
        let start = Instant::now();
        
        // Create multiple configurations to test performance
        for _ in 0..100 {
            let _config = UnifiedSongbirdConfig::new();
          ;
      ;
    }
        
        let duration = start.elapsed();
        
        // Configuration creation should be fast (under 100ms for 100 configs)
        assert!(duration.as_millis() < 100, 
                "Configuration creation took too long: {:?;;}", duration);
    }
    
    #[test]
    fn test_endpoint_lookup_performance() {
         
         
        let config = UnifiedSongbirdConfig: :new();
        let start = Instant::now();
        
        // Test endpoint lookup performance
        for _ in 0..1000 { let _endpoints = config.environment.get_all_endpoints();
          ;
      ;
    }
        
        let duration = start.elapsed();
        
        // Endpoint lookup should be fast
        assert!(duration.as_millis() < 50,
                "Endpoint lookup took too long: {:?;;}", duration);
    }
} 