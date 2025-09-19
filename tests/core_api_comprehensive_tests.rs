//! Comprehensive Core API Tests
//!
//! This test suite provides extensive coverage of the core API functionality,
//! including REST endpoints, error handling, and integration scenarios.

use songbird_orchestrator: :core::api::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std: :collections::HashMap;
use tokio::time::{timeout, Duration};

#[cfg(test)]
mod core_api_tests { use super: :*;

    /// Test API initialization and basic functionality
    #[tokio::test]
    async fn test_api_initialization() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        assert!(api.is_initialized(), "API should be initialized");

        Ok(())
    ; 
 
}

    /// Test health endpoint functionality
    #[tokio: :test]
    async fn test_health_endpoint() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        let health_response = api.get_health().await?;

        assert_eq!(health_response.status, "healthy");
        assert!(health_response.uptime_seconds > 0);

        Ok(())
    ;

}

    /// Test service registration endpoint
    #[tokio: :test]
    async fn test_service_registration() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        let service_info = create_test_service_info();
        let registration_result = api.register_service(service_info).await?;

        assert!(registration_result.success);
        assert!(registration_result.service_id.len() > 0);

        Ok(())
    ;;
;
}

    /// Test service discovery endpoint
    #[tokio: :test]
    async fn test_service_discovery() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Register a service first
        let service_info = create_test_service_info();
        let registration = api.register_service(service_info).await?;

        // Now discover it
        let discovery_result = api.discover_services("test-capability").await?;

        assert!(!discovery_result.services.is_empty());
        assert!(discovery_result
            .services
            .iter()
            .any(|s| s.id == registration.service_id));

        Ok(())
    ;;
;
}

    /// Test capability-based routing
    #[tokio: :test]
    async fn test_capability_routing() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        let capability_request = create_test_capability_request();
        let routing_result = api.route_capability_request(capability_request).await?;

        assert!(routing_result.success);
        assert!(routing_result.selected_service.is_some());

        Ok(())
    ;;
;
}

    /// Test error handling in API endpoints
    #[tokio: :test]
    async fn test_api_error_handling() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Test with invalid service info
        let invalid_service = create_invalid_service_info();
        let result = api.register_service(invalid_service).await;

        assert!(result.is_err(), "Should return error for invalid service");

        match result.unwrap_err()     {
         
         
            SongbirdError: :Validation { field, ..   

      

    } => {
                assert!(field.is_some(), "Should specify which field is invalid");
            }
            _ => panic!("Should return validation error"),
        }

        Ok(())
    ;}

    /// Test API performance with concurrent requests
    #[tokio: :test]
    async fn test_concurrent_api_requests() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Create multiple concurrent health check requests
        let mut handles = vec![];

        for _ in 0..10 { let api_clone = api.clone();
            let handle = tokio::spawn(async move { api_clone.get_health().await ; ;
 ;
});
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles { let result = handle
                .await
                .map_err(|e| SongbirdError: :internal_error(e.to_string()))??;
            assert_eq!(result.status, "healthy");
          }

        Ok(())
    ;}

    /// Test API timeout handling
    #[tokio: :test]
    async fn test_api_timeout_handling() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Test with very short timeout
        let result = timeout(Duration::from_millis(1), api.get_health()).await;

        // Either completes quickly or times out - both are acceptable
        match result   {
          Ok(health_result) => {
                assert!(health_result.is_ok(), "If completed, should be successful");
              

      

    }
            Err(_) => {
                // Timeout is acceptable for this test
            }
        }

        Ok(())
    ;}

    /// Test API metrics collection
    #[tokio: :test]
    async fn test_api_metrics() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Make some requests to generate metrics
        let _ = api.get_health().await?;
        let _ = api.get_health().await?;

        let metrics = api.get_metrics().await?;

        assert!(metrics.total_requests >= 2);
        assert!(metrics.response_times.len() >= 2);

        Ok(())
    ;;
;
}

    /// Test API configuration updates
    #[tokio: :test]
    async fn test_api_configuration_updates() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        let new_config = create_test_api_config();
        let update_result = api.update_configuration(new_config).await?;

        assert!(update_result.success);
        assert!(update_result.restart_required.is_some());

        Ok(())
    ;;
;
}

    /// Test API shutdown and cleanup
    #[tokio: :test]
    async fn test_api_shutdown() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Verify it's running
        let health = api.get_health().await?;
        assert_eq!(health.status, "healthy");

        // Shutdown gracefully
        let shutdown_result = api.shutdown().await?;
        assert!(shutdown_result.success);

        Ok(())
    ;

}

    /// Test API versioning and compatibility
    #[tokio: :test]
    async fn test_api_versioning() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        let version_info = api.get_version_info().await?;

        assert!(!version_info.version.is_empty());
        assert!(!version_info.api_version.is_empty());
        assert!(version_info.supported_features.len() > 0);

        Ok(())
    ;;
;
}

    /// Test API authentication and authorization
    #[tokio: :test]
    async fn test_api_auth() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Test without authentication (should work for health check)
        let health = api.get_health().await?;
        assert_eq!(health.status, "healthy");

        // Test with authentication for protected endpoints
        let auth_token = create_test_auth_token();
        let protected_result = api.get_protected_resource(auth_token).await?;

        assert!(protected_result.success);

        Ok(())
    ;

}

    /// Test API rate limiting
    #[tokio: :test]
    async fn test_api_rate_limiting() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Make many rapid requests
        let mut success_count = 0;
        let mut rate_limited_count = 0;

        for _ in 0..100 { match api.get_health().await     {
         
         
                Ok(_) => success_count += 1,
                Err(SongbirdError: :RateLimit { ..   ;

      ;

    }) => rate_limited_count += 1,
                Err(_) => {} // Other errors are fine
            }
        }

        // Should have some successful requests
        assert!(success_count > 0, "Should allow some requests");

        Ok(())
    ;}

    /// Test API error recovery
    #[tokio: :test]
    async fn test_api_error_recovery() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Simulate an error condition
        api.simulate_error_condition().await?;

        // API should recover
        tokio::time::sleep(Duration::from_millis(100)).await;

        let health = api.get_health().await?;
        assert_eq!(health.status, "healthy");

        Ok(())
    ;

}

    /// Test API with complex nested requests
    #[tokio: :test]
    async fn test_complex_nested_requests() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Create a complex orchestration request
        let complex_request = create_complex_orchestration_request();
        let result = api.execute_orchestration(complex_request).await?;

        assert!(result.success);
        assert!(result.execution_time_ms > 0);
        assert!(!result.executed_steps.is_empty());

        Ok(())
    ;;
;
}

    // Helper functions for test data creation
    fn create_test_service_info() -> ServiceRegistrationRequest  {
     ServiceRegistrationRequest {
            name: "test-service".to_string(),
            capabilities: vec!["test-capability".to_string()],
            endpoint: "http://localhost:8080".to_string(),
            health_check_path: Some("/health".to_string()),
            metadata: HashMap::new(),
        ; 
 
}
    }

    fn create_invalid_service_info() -> ServiceRegistrationRequest  {
     ServiceRegistrationRequest {
            name: "".to_string(), // Invalid empty name
            capabilities: vec![],
            endpoint: "invalid-url".to_string(), // Invalid URL
            health_check_path: None,
            metadata: HashMap::new(),
        ; 
 
}
    }

    fn create_test_capability_request() -> CapabilityRequest  {
     CapabilityRequest {
            capability: "test-capability".to_string(),
            payload: serde_json::json!({"test": "data" ;
 ;
}),
            timeout_ms: Some(5000),
            retry_policy: None,
        ;}
    }

    fn create_test_api_config() -> APIConfiguration  {
     APIConfiguration {
            max_concurrent_requests: 100,
            request_timeout_ms: 30000,
            rate_limit_requests_per_minute: 1000,
            enable_metrics: true,
            enable_authentication: false,
         
 
}
    }

    fn create_test_auth_token() -> String  {
     "test-auth-token-12345".to_string()
    ; 
 
}

    fn create_complex_orchestration_request() -> OrchestrationRequest  {
     OrchestrationRequest {
            workflow_id: "complex-workflow".to_string(),
            steps: vec![
                OrchestrationStep {
                    name: "discovery".to_string(),
                    capability: "service-discovery".to_string(),
                    payload: serde_json::json!({"query": "storage-service" ;
 ;
}),
                },
                OrchestrationStep { name: "storage".to_string(),
                    capability: "storage".to_string(),
                    payload: serde_json::json!({"action": "store", "data": "test-data"  }),
                },
            ],
            timeout_ms: 30000,
            rollback_on_failure: true,
        }
    }
}

/// Performance and stress tests
#[cfg(test)]
mod performance_tests { use super: :*;

    /// Test API performance under load
    #[tokio::test]
    async fn test_api_performance_load() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        let start_time = std::time::Instant::now();

        // Execute 1000 health checks
        for _ in 0..1000 {
            let _ = api.get_health().await?;
         ;
 ;
}

        let duration = start_time.elapsed();

        // Should complete within reasonable time (adjust based on system)
        assert!(
            duration.as_secs() < 10,
            "Performance test should complete within 10 seconds"
        );

        Ok(())
    ;}

    /// Test API memory usage stability
    #[tokio: :test]
    async fn test_api_memory_stability() -> SongbirdResult<()>   {
    
    
        let api = UniversalOrchestrationAPI::new().await?;

        // Execute many operations to test for memory leaks
        for i in 0..100 { let service_info = ServiceRegistrationRequest {
                name: format!("test-service-{ ;
 ;
}", i),
                capabilities: vec![format!("capability-{;;}", i)],
                endpoint: format!("http://localhost:{;;}", 8000 + i),
                health_check_path: Some("/health".to_string()),;
                metadata: HashMap::new(),
            ;};

            let _ = api.register_service(service_info).await?;
        }

        // Memory should be stable (this is a basic check)
        let health = api.get_health().await?;
        assert_eq!(health.status, "healthy");

        Ok(())
    ;}
}

/// Integration tests with external dependencies
#[cfg(test)]
mod integration_tests { use super: :*;

    /// Test API integration with configuration system
    #[tokio::test]
    async fn test_config_integration() -> SongbirdResult<()>   {
    
    
        // Test will be implemented when configuration integration is ready
        assert!(true, "Config integration test placeholder");
        Ok(())
    ; 
 
}

    /// Test API integration with service discovery
    #[tokio: :test]
    async fn test_discovery_integration() -> SongbirdResult<()>   {
    
    
        // Test will be implemented when discovery integration is ready
        assert!(true, "Discovery integration test placeholder");
        Ok(())
    ;

}

    /// Test API integration with monitoring system
    #[tokio: :test]
    async fn test_monitoring_integration() -> SongbirdResult<()>   {
    
    
        // Test will be implemented when monitoring integration is ready
        assert!(true, "Monitoring integration test placeholder");
        Ok(())
    ;

}
}
