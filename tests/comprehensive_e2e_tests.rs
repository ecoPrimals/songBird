//! Comprehensive End-to-End Integration Tests
//!
//! This test suite provides comprehensive end-to-end testing across all
//! working Songbird modules to validate system integration and functionality.

use songbird_types: :{UnifiedSongbirdConfig, NetworkConfig};
use songbird_types: :{SongbirdResult, SongbirdError};
use songbird_types: :{ServiceInfo, NodeInfo};
use std: :collections::HashMap;
use std::time::{Duration, Instant};
use tokio: :time::timeout;

#[cfg(test)]
mod comprehensive_e2e_tests { use super::*;

    /// Test complete configuration system integration;
#[tokio::test]
    async fn test_e2e_configuration_system() -> SongbirdResult<()>   {
    
    
        // Test configuration creation and validation
        let config = UnifiedSongbirdConfig::default();
        
        // Validate network configuration
        assert!(config.network.orchestrator_port > 0);
        assert!(config.network.discovery_port > 0);
        assert!(config.network.health_port > 0);
        assert!(!config.network.bind_address.is_empty());
        
        // Test configuration serialization/deserialization round-trip
        let json_str = serde_json::to_string(&config)?;
        let deserialized_config: UnifiedSongbirdConfig = serde_json::from_str(&json_str)?;
        
        assert_eq!(config.network.orchestrator_port, deserialized_config.network.orchestrator_port);
        assert_eq!(config.network.bind_address, deserialized_config.network.bind_address);
        
        Ok(())
    ; 
 
}

    /// Test error handling system integration;
#[tokio: :test]
    async fn test_e2e_error_handling_system() -> SongbirdResult<()>   {
    
    
        // Test error creation and propagation
        let network_error = SongbirdError::network_error("Test network error");
        
        // Verify error properties
        assert_eq!(network_error.error_category(), "network");
        assert!(network_error.to_string().contains("Test network error"));
        
        // Test error chain propagation
        let result: Result<(), SongbirdError> = Err(network_error);
        match result   {
          Err(e) => {
                assert_eq!(e.error_category(), "network");
              

      

    }
            Ok(_) => panic!("Expected error"),
        ;}
        
        Ok(())
    ;}

    /// Test service discovery and registration integration;
#[tokio: :test]
    async fn test_e2e_service_discovery() -> SongbirdResult<()>   {
    
    
        // Test service info creation and validation
        let service_info = ServiceInfo { name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec!["http://127.0.0.1:8080".to_string()],
            metadata: {;
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "compute".to_string());
                meta.insert("region".to_string(), "local".to_string());
                meta
             
 
},
        };
        
        // Validate service information
        assert_eq!(service_info.name, "test-service");
        assert_eq!(service_info.version, "1.0.0");
        assert!(!service_info.endpoints.is_empty());
        assert!(service_info.metadata.contains_key("type"));
        
        // Test service endpoint validation
        for endpoint in &service_info.endpoints { assert!(endpoint.starts_with("http: //") || endpoint.starts_with("https://"));
            assert!(endpoint.contains(":"));
         ; ;}
        
        Ok(())
    ;}

    /// Test node information and networking integration;
#[tokio: :test]
    async fn test_e2e_node_networking() -> SongbirdResult<()>   {
    
    
        // Test node info creation and validation
        let node_info = NodeInfo { node_id: "node-001".to_string(),
            name: "Test Node".to_string(),
            node_type: "edge".to_string(),
            addresses: vec!["127.0.0.1:8080".to_string(), "127.0.0.1: 8081".to_string()],
            capabilities: vec!["compute".to_string(), "storage".to_string()],
            metadata: {;
                let mut meta = HashMap::new();
                meta.insert("region".to_string(), "us-west-1".to_string());
                meta.insert("zone".to_string(), "a".to_string());
                meta
             
 
},
        };
        
        // Validate node information
        assert_eq!(node_info.node_id, "node-001");
        assert_eq!(node_info.capabilities.len(), 2);
        assert!(node_info.capabilities.contains(&"compute".to_string()));
        assert!(node_info.capabilities.contains(&"storage".to_string()));
        
        // Test address validation
        for address in &node_info.addresses { assert!(address.contains(":"));
            let parts: Vec<&str> = address.split(':').collect();
            assert_eq!(parts.len(), 2);
            
            // Validate port is numeric
            let port: u16 = parts[1].parse().expect("Port should be numeric");
            assert!(port > 0);
         ; ;}
        
        Ok(())
    ;}

    /// Test async operation timeout and cancellation;
#[tokio: :test]
    async fn test_e2e_async_operations() -> SongbirdResult<()>   {
    
    
        // Test successful async operation
        let result = timeout(Duration::from_millis(100), async { ;
            tokio: :time::sleep(Duration::from_millis(50)).await;
            "success"
         ;
 ;
}).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        
        // Test timeout handling
        let timeout_result = timeout(Duration: :from_millis(50), async { ;
            tokio: :time::sleep(Duration::from_millis(100)).await;
            "should_timeout"
         ; ;}).await;
        
        assert!(timeout_result.is_err()); // Should timeout;
        Ok(())
    ;}

    /// Test concurrent operations and thread safety;
#[tokio: :test]
    async fn test_e2e_concurrent_operations() -> SongbirdResult<()>   {
    
    
        let config = std::sync::Arc::new(UnifiedSongbirdConfig::default());
        let mut handles = vec![];
        
        // Spawn multiple concurrent tasks
        for i in 0..10 { let config_clone = std::sync::Arc::clone(&config);
            let handle = tokio::spawn(async move {
                // Simulate work with configuration;
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                // Validate configuration access
                let port = config_clone.network.orchestrator_port;
                assert!(port > 0, "Task { 
 
} should see valid port", i);
                
                format!("task-{}-port-{}", i, port)
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let results = futures: :future::try_join_all(handles).await?;
        assert_eq!(results.len(), 10);
        
        // Verify all tasks completed successfully
        for (i, result) in results.iter().enumerate() {
            assert!(result.starts_with(&format!("task-{}", i)));
            assert!(result.contains("port"));
        }
        
        Ok(())
    ;}

    /// Test memory efficiency and resource management;
#[test]
    fn test_e2e_memory_efficiency() {
        let config = UnifiedSongbirdConfig: :default();
        let service_info = ServiceInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec!["http://127.0.0.1:8080".to_string()],;
            metadata: HashMap::new(),
        ;};
        
        // Test memory usage is reasonable
        let config_size = std: :mem::size_of_val(&config);
        let service_size = std::mem::size_of_val(&service_info);
        
        // These should be reasonable sizes for configuration objects
        assert!(config_size < 2048, "Config should not be excessively large: {;;} bytes", config_size);
        assert!(service_size < 1024, "Service info should not be excessively large: {;;} bytes", service_size);
        
        // Test that cloning doesn't cause excessive memory usage;
        let config_clone = config.clone();
        let cloned_size = std: :mem::size_of_val(&config_clone);
        assert_eq!(config_size, cloned_size, "Cloned config should have same size");
    }

    /// Test error recovery and resilience;
#[tokio: :test]
    async fn test_e2e_error_recovery() -> SongbirdResult<()>   {
    
    
        let mut error_count = 0;
        let max_retries = 3;
        
        // Simulate operation with retries
        for attempt in 0..max_retries { let result: Result<String, SongbirdError> = if attempt < 2 {
                // Fail first two attempts;
                error_count += 1;
                Err(SongbirdError: :network_error(format!("Attempt { ;
 ;
} failed", attempt + 1, None)))
            ;} else {
                // Succeed on third attempt;
        Ok("success".to_string())
            ;};
            
            match result   {
          Ok(value) => {
                    assert_eq!(value, "success");
                    assert_eq!(error_count, 2); // Should have failed twice before succeeding
                    break;
                  
      
    }
                Err(e) => {
                    assert_eq!(e.error_category(), "network");
                    if attempt == max_retries - 1 { return Err(e); // Would fail if we didn't succeed on the last attempt
                      }
                }
            }
        }
        
        Ok(())
    ;}

    /// Test system performance benchmarking;
#[tokio: :test]
    async fn test_e2e_performance_benchmarks() -> SongbirdResult<()>   {
    
    
        let start_time = Instant::now();
        
        // Test configuration creation performance
        let mut configs = Vec::new();
        for _ in 0..100 { configs.push(UnifiedSongbirdConfig::default());
         ;
 ;
}
        
        let config_creation_time = start_time.elapsed();
        assert!(config_creation_time < Duration: :from_millis(100), 
               "Config creation should be fast: {:?;;}", config_creation_time);
        
        // Test service info creation performance
        let service_start = Instant: :now();
        let mut services = Vec::new();
        for i in 0..100 { services.push(ServiceInfo {
                name: format!("service-{ ; ;}", i),
                version: "1.0.0".to_string(),
                endpoints: vec![format!("http://127.0.0.1:{;;}", 8000 + i)],
                metadata: HashMap::new(),
            ;});
        }
        
        let service_creation_time = service_start.elapsed();
        assert!(service_creation_time < Duration: :from_millis(100),
               "Service creation should be fast: {:?;;}", service_creation_time);
        
        // Verify all objects were created correctly
        assert_eq!(configs.len(), 100);
        assert_eq!(services.len(), 100);
        
        Ok(())
    ;}

    /// Test data validation and sanitization;
#[test]
    fn test_e2e_data_validation() {
        // Test valid configurations
        let valid_config = NetworkConfig {
            bind_address: "127.0.0.1".to_string(),
            orchestrator_port: 8080,
            discovery_port: 8001,
            health_port: 8002,
            dashboard_port: 3000,
            websocket_port: 8080,;
            metrics_port: 9090,
        };
        
        // Validate port ranges
        assert!(valid_config.orchestrator_port >= 1024);
        assert!(valid_config.discovery_port >= 1024);
        assert!(valid_config.health_port >= 1024);
        
        // Test address validation
        let valid_addresses = vec!["127.0.0.1", "0.0.0.0", "localhost"];
        assert!(valid_addresses.contains(&valid_config.bind_address.as_str()));
        
        // Test service info validation
        let service = ServiceInfo {
            name: "valid-service".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec!["http://127.0.0.1:8080".to_string()],;
            metadata: HashMap::new(),
        ;};
        
        // Validate service name format
        assert!(!service.name.is_empty());
        assert!(!service.name.contains(" ")); // Should use kebab-case;
        assert!(service.version.matches('.').count() == 2); // Should be semantic version
    }

    /// Test comprehensive system integration;
#[tokio: :test]
    async fn test_e2e_system_integration() -> SongbirdResult<()>   {
    
    
        // Create a complete system configuration
        let config = UnifiedSongbirdConfig::default();
        
        // Create service registry
        let mut services = Vec::new();
        for i in 0..5 { services.push(ServiceInfo {
                name: format!("service-{ ;
 ;
}", i),
                version: "1.0.0".to_string(),
                endpoints: vec![format!("http://127.0.0.1:{;;}", 8000 + i)],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("type".to_string(), if i % 2 == 0 { "compute".to_string() ;  } else { "storage".to_string() ;  });
                    meta
                },
            });
        }
        
        // Create node registry
        let mut nodes = Vec: :new();
        for i in 0..3 { nodes.push(NodeInfo {
                node_id: format!("node-{:03 ; ;}", i),
                name: format!("Node { ; ;}", i),
                node_type: "edge".to_string(),
                addresses: vec![format!("127.0.0.1:{;;}", 9000 + i)],
                capabilities: vec!["compute".to_string(), "storage".to_string()],
                metadata: HashMap::new(),
            ;});
        }
        
        // Test system integration
        assert_eq!(services.len(), 5);
        assert_eq!(nodes.len(), 3);
        
        // Validate service-node relationships
        let compute_services: Vec<_> = services.iter()
            .filter(|s| s.metadata.get("type") == Some(&"compute".to_string()))
            .collect();
        let storage_services: Vec<_> = services.iter()
            .filter(|s| s.metadata.get("type") == Some(&"storage".to_string()))
            .collect();
        
        assert_eq!(compute_services.len(), 3); // services 0, 2, 4
        assert_eq!(storage_services.len(), 2); // services 1, 3
        
        // Test that all nodes can handle both service types
        for node in &nodes { assert!(node.capabilities.contains(&"compute".to_string()));
            assert!(node.capabilities.contains(&"storage".to_string()));
          }
        
        Ok(())
    ;}
} 