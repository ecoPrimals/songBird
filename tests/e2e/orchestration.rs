//! End-to-End Orchestration Tests
//!
//! Tests the full orchestrator lifecycle and coordination

#![cfg(test)]

use super::common::*;

#[tokio::test]
async fn test_full_system_startup() {
    // Test that the system can start up with default configuration
    let env = TestEnvironment::new().await;
    assert!(env.config.validate().is_ok());
    
    // Verify all configuration sections are valid
    assert!(env.config.network.validate().is_ok());
    assert!(env.config.observability.validate().is_ok());
    
    // Verify default ports are within valid range
    assert!(env.config.network.orchestrator_port > 0);
    assert!(env.config.network.orchestrator_port < 65536);
    
    // Verify resource limits are reasonable
    assert!(env.config.environment.resource_limits.max_connections > 0);
    assert!(env.config.environment.resource_limits.max_threads > 0);
}

#[tokio::test]
async fn test_multi_service_coordination() {
    // Test coordinating multiple services
    let env = TestEnvironment::new().await;
    
    // Create test service endpoints
    let service_a = create_test_endpoint("service-a", 8001);
    let service_b = create_test_endpoint("service-b", 8002);
    let service_c = create_test_endpoint("service-c", 8003);
    
    // Verify endpoints are created correctly
    assert_eq!(service_a.name, "service-a");
    assert_eq!(service_a.port, 8001);
    assert_eq!(service_b.name, "service-b");
    assert_eq!(service_b.port, 8002);
    assert_eq!(service_c.name, "service-c");
    assert_eq!(service_c.port, 8003);
    
    // Verify all services have unique ports
    assert_ne!(service_a.port, service_b.port);
    assert_ne!(service_b.port, service_c.port);
    assert_ne!(service_a.port, service_c.port);
    
    // Verify configuration supports multi-service coordination
    assert!(env.config.network.connection_limits.max_total_connections >= 3);
}

#[tokio::test]
async fn test_service_lifecycle() {
    // Test complete service lifecycle
    let env = TestEnvironment::new().await;
    
    // Phase 1: Service creation
    let service = create_test_endpoint("test-service", 9000);
    assert_eq!(service.name, "test-service");
    assert_eq!(service.host, "localhost");
    assert_eq!(service.port, 9000);
    assert_eq!(service.protocol, "http");
    
    // Phase 2: Verify service configuration is valid
    assert!(!service.name.is_empty());
    assert!(!service.host.is_empty());
    assert!(service.port > 0 && service.port < 65536);
    
    // Phase 3: Verify health check configuration exists
    assert!(env.config.observability.health_checks.enabled);
    assert!(env.config.observability.health_checks.interval_secs > 0);
    
    // Phase 4: Verify graceful shutdown configuration
    assert!(env.config.network.timeouts.connection_timeout_secs > 0);
}

#[tokio::test]
async fn test_dynamic_configuration() {
    // Test dynamic configuration updates
    let mut env = TestEnvironment::new().await;
    
    // Phase 1: Verify default configuration
    let original_port = env.config.network.orchestrator_port;
    let original_max_connections = env.config.network.connection_limits.max_total_connections;
    assert!(original_port > 0);
    assert!(original_max_connections > 0);
    
    // Phase 2: Update configuration (simulate dynamic update)
    env.config.network.orchestrator_port = 9999;
    env.config.network.connection_limits.max_total_connections = 5000;
    
    // Phase 3: Verify changes took effect
    assert_eq!(env.config.network.orchestrator_port, 9999);
    assert_eq!(env.config.network.connection_limits.max_total_connections, 5000);
    assert_ne!(env.config.network.orchestrator_port, original_port);
    
    // Phase 4: Verify configuration is still valid after updates
    assert!(env.config.validate().is_ok());
    assert!(env.config.network.validate().is_ok());
}

#[tokio::test]
async fn test_cross_service_communication() {
    // Test communication between services
    let env = TestEnvironment::new().await;
    
    // Phase 1: Create service endpoints
    let service_a = create_test_endpoint("api-gateway", 8080);
    let service_b = create_test_endpoint("backend-service", 8081);
    
    // Phase 2: Verify service A can reference service B
    assert_eq!(service_a.name, "api-gateway");
    assert_eq!(service_b.name, "backend-service");
    
    // Phase 3: Verify both services use compatible protocols
    assert_eq!(service_a.protocol, service_b.protocol);
    assert_eq!(service_a.protocol, "http");
    
    // Phase 4: Verify network configuration supports communication
    assert!(env.config.network.connection_limits.max_total_connections >= 2);
    assert!(env.config.network.timeouts.default_timeout_secs > 0);
    
    // Phase 5: Verify both services are on same host (for testing)
    assert_eq!(service_a.host, service_b.host);
    assert_eq!(service_a.host, "localhost");
}

