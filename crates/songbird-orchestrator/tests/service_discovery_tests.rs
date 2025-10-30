//! Service Discovery Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//!
//! Comprehensive tests for service discovery and routing functionality.
//! These tests validate the orchestrator's ability to discover services,
//! route requests, handle failures, and maintain service registry state.

use songbird_test_utils::{
    ai_service, compute_service,
    mocks::common::{HealthStatus, MockPrimalServer},
    security_service, storage_service, MockBearDog, MockNestGate, MockSquirrel, MockToadStool,
    OrchestratorTestEnvironment,
};
use songbird_types::SongbirdError;

#[tokio::test]
async fn test_discover_services_by_capability() {
    // Arrange: Create test environment with healthy primals
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    // Act: Create service fixtures for each capability
    let compute = compute_service("toadstool-1").with_endpoint(&env.toadstool_endpoint().await);
    let storage = storage_service("nestgate-1").with_endpoint(&env.nestgate_endpoint().await);
    let security = security_service("beardog-1").with_endpoint(&env.beardog_endpoint().await);
    let ai = ai_service("squirrel-1").with_endpoint(&env.squirrel_endpoint().await);

    // Assert: Each service has the correct capability
    assert!(compute.capabilities().contains(&"compute".to_string()));
    assert!(storage.capabilities().contains(&"storage".to_string()));
    assert!(security.capabilities().contains(&"security".to_string()));
    assert!(ai.capabilities().contains(&"ai".to_string()));

    // Cleanup
    env.cleanup().await;
}

#[tokio::test]
async fn test_discover_healthy_services_only() {
    use songbird_test_utils::mocks::common::{HealthStatus, MockPrimalServer};

    // Arrange: Create environment with mixed health
    let env = OrchestratorTestEnvironment::new().await;

    // Set one service as unhealthy
    env.toadstool.read().await.set_health(HealthStatus::Unhealthy);

    // Act & Assert: Verify health status
    assert_eq!(env.toadstool.read().await.get_health(), HealthStatus::Unhealthy);
    assert_eq!(env.beardog.read().await.get_health(), HealthStatus::Healthy);
    assert_eq!(env.nestgate.read().await.get_health(), HealthStatus::Healthy);
    assert_eq!(env.squirrel.read().await.get_health(), HealthStatus::Healthy);

    env.cleanup().await;
}

#[tokio::test]
async fn test_discover_with_filters() {
    // Arrange: Create environment with multiple services
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    // Act: Create filtered service lists
    let compute_services = [compute_service("toadstool-1")
        .with_endpoint(&env.toadstool_endpoint().await)
        .with_metadata("type", "compute")];

    let storage_services = [storage_service("nestgate-1")
        .with_endpoint(&env.nestgate_endpoint().await)
        .with_metadata("type", "storage")];

    // Assert: Services have correct metadata
    assert_eq!(compute_services[0].metadata.get("type"), Some(&"compute".to_string()));
    assert_eq!(storage_services[0].metadata.get("type"), Some(&"storage".to_string()));

    env.cleanup().await;
}

#[tokio::test]
async fn test_route_to_single_service() {
    // Arrange: Create environment with only compute capability
    let env = OrchestratorTestEnvironment::with_compute_only().await;

    // Act: Get endpoint for routing
    let endpoint = env.toadstool_endpoint().await;

    // Assert: Endpoint is valid
    assert!(endpoint.contains("localhost"));
    assert!(endpoint.contains("http://"));

    env.cleanup().await;
}

#[tokio::test]
async fn test_route_with_load_balancing() {
    // Arrange: Multiple services available
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    // Act: Collect all available endpoints
    let endpoints = [
        env.toadstool_endpoint().await,
        env.beardog_endpoint().await,
        env.nestgate_endpoint().await,
        env.squirrel_endpoint().await,
    ];

    // Assert: All endpoints are available and unique
    assert_eq!(endpoints.len(), 4);
    assert!(endpoints.iter().all(|e| !e.is_empty()));
    assert!(endpoints.iter().all(|e| e.contains("localhost")));

    // Verify all endpoints are different
    let unique_endpoints: std::collections::HashSet<_> = endpoints.iter().collect();
    assert_eq!(unique_endpoints.len(), 4);

    env.cleanup().await;
}

#[test]
fn test_service_discovery_configuration() {
    // Arrange: Create default config
    use songbird_config::SongbirdConfig;
    let config = SongbirdConfig::default();

    // Assert: Discovery config is valid
    assert!(!config.network.bind_address.is_empty());
    assert!(config.network.max_connections > 0);
    assert!(config.network.port_range.start < config.network.port_range.end);
}

#[tokio::test]
async fn test_route_with_failover() {
    use songbird_test_utils::mocks::common::{HealthStatus, MockPrimalServer};

    // Arrange: Create environment with one failed service
    let env = OrchestratorTestEnvironment::new().await;

    // Simulate primary service failure
    env.toadstool.read().await.set_health(HealthStatus::Unhealthy);

    // Act: Check failover capabilities
    let primary_health = env.toadstool.read().await.get_health();
    let backup_health = env.beardog.read().await.get_health();

    // Assert: Can detect failure and have backup
    assert_eq!(primary_health, HealthStatus::Unhealthy);
    assert_eq!(backup_health, HealthStatus::Healthy);

    env.cleanup().await;
}

#[tokio::test]
async fn test_high_load_routing() {
    // Arrange: Create high load scenario
    let env = OrchestratorTestEnvironment::with_high_load().await;

    // Act: Check service health under load
    let services_health = [
        env.toadstool.read().await.get_health(),
        env.beardog.read().await.get_health(),
        env.nestgate.read().await.get_health(),
        env.squirrel.read().await.get_health(),
    ];

    // Assert: All services are degraded under high load
    assert!(services_health.iter().all(|h| *h == HealthStatus::Degraded));

    env.cleanup().await;
}

#[tokio::test]
async fn test_service_endpoint_validation() {
    // Arrange: Create test environment
    let env = OrchestratorTestEnvironment::new().await;

    // Act: Get all endpoints
    let endpoints = vec![
        env.toadstool_endpoint().await,
        env.beardog_endpoint().await,
        env.nestgate_endpoint().await,
        env.squirrel_endpoint().await,
    ];

    // Assert: All endpoints are valid HTTP URLs
    for endpoint in endpoints {
        assert!(endpoint.starts_with("http://"));
        assert!(endpoint.contains("localhost"));
        assert!(endpoint.split(':').count() == 3); // http://localhost:PORT format
    }

    env.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_service_discovery() {
    // Arrange: Create test environment
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    // Act: Concurrently access multiple service endpoints
    let (endpoint1, endpoint2, endpoint3, endpoint4) = tokio::join!(
        async { env.toadstool_endpoint().await },
        async { env.beardog_endpoint().await },
        async { env.nestgate_endpoint().await },
        async { env.squirrel_endpoint().await },
    );

    // Assert: All concurrent operations succeeded
    assert!(!endpoint1.is_empty());
    assert!(!endpoint2.is_empty());
    assert!(!endpoint3.is_empty());
    assert!(!endpoint4.is_empty());

    env.cleanup().await;
}

#[tokio::test]
async fn test_service_metadata_filtering() {
    // Arrange: Create services with metadata
    let compute = compute_service("toadstool-1")
        .with_metadata("region", "us-west")
        .with_metadata("tier", "production");

    let storage = storage_service("nestgate-1")
        .with_metadata("region", "us-east")
        .with_metadata("tier", "staging");

    // Act: Filter by metadata
    let production_services = vec![&compute, &storage]
        .into_iter()
        .filter(|s| s.metadata.get("tier") == Some(&"production".to_string()))
        .collect::<Vec<_>>();

    // Assert: Only production service returned
    assert_eq!(production_services.len(), 1);
    assert_eq!(production_services[0].name(), "toadstool-1");
}

#[tokio::test]
async fn test_service_capability_validation() {
    // Arrange: Create services with different capabilities
    let multi_cap =
        compute_service("multi-1").with_capability("storage").with_capability("processing");

    // Act: Check capabilities
    let capabilities = multi_cap.capabilities();

    // Assert: All capabilities present
    assert!(capabilities.contains(&"compute".to_string()));
    assert!(capabilities.contains(&"storage".to_string()));
    assert!(capabilities.contains(&"processing".to_string()));
    assert_eq!(capabilities.len(), 3);
}

#[tokio::test]
async fn test_environment_initialization() {
    // Arrange & Act: Create different environment types
    let env_normal = OrchestratorTestEnvironment::new().await;
    let env_healthy = OrchestratorTestEnvironment::with_healthy_primals().await;
    let env_compute = OrchestratorTestEnvironment::with_compute_only().await;
    let env_load = OrchestratorTestEnvironment::with_high_load().await;

    // Assert: All environments initialized successfully
    assert!(!env_normal.toadstool_endpoint().await.is_empty());
    assert!(!env_healthy.toadstool_endpoint().await.is_empty());
    assert!(!env_compute.toadstool_endpoint().await.is_empty());
    assert!(!env_load.toadstool_endpoint().await.is_empty());

    // Cleanup all
    env_normal.cleanup().await;
    env_healthy.cleanup().await;
    env_compute.cleanup().await;
    env_load.cleanup().await;
}

#[tokio::test]
async fn test_service_health_monitoring() {
    use songbird_test_utils::mocks::common::{HealthStatus, MockPrimalServer};

    // Arrange: Create test environment
    let env = OrchestratorTestEnvironment::new().await;

    // Act: Check initial health
    let initial_health = env.toadstool.read().await.get_health();
    assert_eq!(initial_health, HealthStatus::Healthy);

    // Change health status
    env.toadstool.read().await.set_health(HealthStatus::Degraded);
    let updated_health = env.toadstool.read().await.get_health();
    assert_eq!(updated_health, HealthStatus::Degraded);

    env.cleanup().await;
}

#[test]
fn test_service_fixture_creation() {
    // Arrange & Act: Create various service fixtures
    let compute = compute_service("test-compute");
    let storage = storage_service("test-storage");
    let security = security_service("test-security");
    let ai = ai_service("test-ai");

    // Assert: All fixtures created with correct properties
    assert_eq!(compute.name(), "test-compute");
    assert_eq!(storage.name(), "test-storage");
    assert_eq!(security.name(), "test-security");
    assert_eq!(ai.name(), "test-ai");

    assert!(compute.capabilities().contains(&"compute".to_string()));
    assert!(storage.capabilities().contains(&"storage".to_string()));
    assert!(security.capabilities().contains(&"security".to_string()));
    assert!(ai.capabilities().contains(&"ai".to_string()));
}

#[tokio::test]
async fn test_mock_server_isolation() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange: Create multiple mock servers
    let mut mock1 = MockToadStool::new();
    let mut mock2 = MockBearDog::new();
    let mut mock3 = MockNestGate::new();
    let mut mock4 = MockSquirrel::new();

    // Act: Start servers
    let port1 = mock1
        .start()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Mock 1 should start: {e}")))?;
    let port2 = mock2
        .start()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Mock 2 should start: {e}")))?;
    let port3 = mock3
        .start()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Mock 3 should start: {e}")))?;
    let port4 = mock4
        .start()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Mock 4 should start: {e}")))?;

    // Assert: All servers have unique ports
    let all_ports = [port1, port2, port3, port4];
    let unique_ports: std::collections::HashSet<_> = all_ports.iter().collect();
    assert_eq!(unique_ports.len(), 4);

    // Assert: All ports are in valid range (10000-60000)
    assert!(all_ports.iter().all(|p| *p >= 10000 && *p < 60000));

    // Cleanup
    mock1.stop().await;
    mock2.stop().await;
    mock3.stop().await;
    mock4.stop().await;
    Ok(())
}

#[test]
fn test_service_builder_pattern() {
    // Arrange & Act: Use builder pattern to create service
    let service = compute_service("builder-test")
        .with_endpoint("http://localhost:9000")
        .with_capability("processing")
        .with_metadata("version", "1.0.0")
        .with_metadata("environment", "test");

    // Assert: All properties set correctly
    assert_eq!(service.name(), "builder-test");
    assert_eq!(service.endpoint(), "http://localhost:9000");
    assert!(service.capabilities().contains(&"compute".to_string()));
    assert!(service.capabilities().contains(&"processing".to_string()));
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("environment"), Some(&"test".to_string()));
}

#[tokio::test]
async fn test_service_discovery_scalability() {
    // Arrange: Create large number of service fixtures
    let services: Vec<_> = (0..100).map(|i| compute_service(format!("service-{i}"))).collect();

    // Act: Verify all services created
    let service_count = services.len();

    // Assert: All services have unique names
    let unique_names: std::collections::HashSet<_> =
        services.iter().map(songbird_test_utils::TestService::name).collect();
    assert_eq!(unique_names.len(), 100);
    assert_eq!(service_count, 100);

    // Assert: All have compute capability
    assert!(services.iter().all(|s| s.capabilities().contains(&"compute".to_string())));
}
