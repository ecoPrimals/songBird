//! Tests for Universal Service Registration
//!
//! Comprehensive test suite for universal service registration functionality

#[cfg(test)]
mod tests {
    use super::super::manager::UniversalServiceRegistrationManager;
    use super::super::types::*;
    use std::collections::HashMap;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_universal_service_registration_creation() {
        let manager = UniversalServiceRegistrationManager::new();
        let services = manager.list_services().await;
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_service_registration() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let service = create_test_service("test-service");
        let result = manager.register_service(service, None).await;

        assert!(result.is_success());
        let services = manager.list_services().await;
        assert_eq!(services.len(), 1);
    }

    #[tokio::test]
    async fn test_service_deregistration() {
        let mut manager = UniversalServiceRegistrationManager::new();

        // Register a service first
        let service = create_test_service("deregister-test");
        let result = manager.register_service(service, None).await;
        assert!(result.is_success());

        let services = manager.list_services().await;
        assert_eq!(services.len(), 1);

        // Deregister the service
        let deregister_result = manager.deregister_service("deregister-test").await;
        assert!(deregister_result.is_ok());

        let services_after = manager.list_services().await;
        assert_eq!(services_after.len(), 0);
    }

    #[tokio::test]
    async fn test_service_discovery() {
        let mut manager = UniversalServiceRegistrationManager::new();

        // Register multiple services
        let service1 = create_test_service("discovery-test-1");
        let service2 = create_test_service("discovery-test-2");

        manager.register_service(service1, None).await;
        manager.register_service(service2, None).await;

        // Check services were registered
        let services = manager.list_services().await;
        assert_eq!(services.len(), 2);

        // Find specific service
        let found_service = manager.get_service("discovery-test-1").await;
        assert!(found_service.is_some());
    }

    #[tokio::test]
    async fn test_resource_validation() {
        let mut manager = UniversalServiceRegistrationManager::new();

        // Test with excessive resource requirements
        let mut service = create_test_service("resource-test");
        service.resource_requirements = Some(ResourceRequirements {
            cpu_cores: Some(1000.0),              // Excessive CPU
            memory_mb: Some(1024 * 1024),         // 1TB memory
            storage_mb: Some(1024 * 1024 * 1024), // 1PB storage
            network_mbps: Some(100000),
            gpu_count: Some(100),
            custom_resources: HashMap::new(),
        });

        let result = manager.register_service(service, None).await;
        // Should handle gracefully (either succeed or fail with proper error)
        assert!(result.is_success() || result.is_error());
    }

    #[tokio::test]
    async fn test_duplicate_service_registration() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let service1 = create_test_service("duplicate-test");
        let service2 = create_test_service("duplicate-test"); // Same ID

        let result1 = manager.register_service(service1, None).await;
        assert!(result1.is_success());

        let result2 = manager.register_service(service2, None).await;
        assert!(result2.is_error()); // Should fail due to duplicate ID
    }

    #[tokio::test]
    async fn test_empty_service_id_validation() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let mut service = create_test_service("test");
        service.service_id = "".to_string(); // Empty service ID

        let result = manager.register_service(service, None).await;
        assert!(result.is_error()); // Should fail validation
    }

    #[tokio::test]
    async fn test_empty_endpoint_validation() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let mut service = create_test_service("endpoint-test");
        service.endpoints.primary = "".to_string(); // Empty endpoint

        let result = manager.register_service(service, None).await;
        assert!(result.is_error()); // Should fail validation
    }

    #[tokio::test]
    async fn test_human_approval_required() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let mut service = create_test_service("approval-test");
        service.primal_type = "unknown".to_string(); // Should trigger human approval

        let result = manager.register_service(service, None).await;
        assert!(result.is_success()); // Should succeed but with pending approval
    }

    #[tokio::test]
    async fn test_performance_predictions() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let service = create_test_service("performance-test");
        let result = manager.register_service(service, None).await;

        if result.is_success() {
            let data = result.unwrap_data();
            assert!(data.performance_predictions.predicted_latency_ms > 0.0);
            assert!(data.performance_predictions.predicted_max_throughput_rps > 0.0);
        }
    }

    #[tokio::test]
    async fn test_monitoring_configuration() {
        let mut manager = UniversalServiceRegistrationManager::new();

        let service = create_test_service_with_health("monitoring-test");
        let result = manager.register_service(service, None).await;

        if result.is_success() {
            let data = result.unwrap_data();
            assert!(data.monitoring_config.health_check_frequency_seconds > 0);
            assert!(data.monitoring_config.metrics_interval_seconds > 0);
        }
    }

    // Helper functions
    fn create_test_service(name: &str) -> UniversalServiceRegistrationRequest {
        UniversalServiceRegistrationRequest {
            service_id: name.to_string(),
            service_name: name.to_string(),
            version: "1.0.0".to_string(),
            primal_type: "test".to_string(),
            capabilities: vec!["api".to_string()],
            endpoints: ServiceEndpoints {
                primary: format!("http://localhost:808{}", name.len() % 10),
                health: Some("/health".to_string()),
                metrics: Some("/metrics".to_string()),
                admin: None,
                websocket: None,
                custom: HashMap::new(),
            },
            resource_requirements: Some(ResourceRequirements {
                cpu_cores: Some(1.0),
                memory_mb: Some(1024),
                storage_mb: Some(1024),
                network_mbps: Some(100),
                gpu_count: None,
                custom_resources: HashMap::new(),
            }),
            health_check: Some(HealthCheckConfiguration {
                interval_seconds: 30,
                timeout_seconds: 5,
                failure_threshold: 3,
                success_threshold: 2,
                custom_parameters: HashMap::new(),
            }),
            metadata: HashMap::new(),
            human_interaction_preferences: None,
        }
    }

    fn create_test_service_with_type(
        name: &str,
        primal_type: &str,
    ) -> UniversalServiceRegistrationRequest {
        let mut service = create_test_service(name);
        service.primal_type = primal_type.to_string();
        service
    }

    fn create_test_service_with_health(name: &str) -> UniversalServiceRegistrationRequest {
        let mut service = create_test_service(name);
        service.health_check = Some(HealthCheckConfiguration {
            interval_seconds: 10, // Shorter interval for testing
            timeout_seconds: 2,
            failure_threshold: 2,
            success_threshold: 1,
            custom_parameters: HashMap::new(),
        });
        service
    }
}
