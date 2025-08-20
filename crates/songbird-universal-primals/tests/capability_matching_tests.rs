//! Capability Matching Tests - Canonical Modernized Version
//!
//! This test suite validates capability-based discovery and matching logic
//! using the modernized universal primal architecture.

use chrono::Utc;
use songbird_universal_primals::errors::PrimalResult;
use songbird_universal_primals::universal_registry::{
    CircuitBreakerConfig, ComplianceLevel, ContactInfo, HealthStatus, IntegrationPreferences,
    LoadBalancingStrategy, RateLimitConfig, RateLimitStrategy, ResourceSpec, ServiceCategory,
    ServiceEndpoint, ServiceFilter, ServiceLifecycleStage, ServiceMetadata, ServicePriority,
    StorageType,
};
use songbird_universal_primals::{
    MemoryServiceRegistry, ServiceCapability, UniversalServiceRegistration,
    UniversalServiceRegistry,
};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_capability_based_service_discovery() -> PrimalResult<()> {
    // Create a modernized service registry
    let registry = MemoryServiceRegistry::new();

    // Register a compute service with proper structure
    let compute_registration = UniversalServiceRegistration {
        service_id: Uuid::new_v4(),
        metadata: ServiceMetadata {
            name: "compute-service".to_string(),
            description: "Container orchestration service".to_string(),
            category: ServiceCategory::Compute,
            tags: vec!["docker".to_string(), "containers".to_string()],
            documentation_url: None,
            contact: ContactInfo {
                maintainer: "test-team".to_string(),
                email: Some("test@example.com".to_string()),
                documentation: None,
                support: None,
            },
            lifecycle_stage: ServiceLifecycleStage::Production,
            compliance_level: ComplianceLevel::Basic,
        },
        capabilities: vec![ServiceCapability::Compute {
            cpu_cores: Some(8.0),
            memory_gb: Some(32.0),
            gpu_support: false,
            container_runtime: Some("docker".to_string()),
        }],
        resources: ResourceSpec::default(),
        endpoints: vec![ServiceEndpoint {
            name: "api".to_string(),
            url: "http://localhost:8080".to_string(),
            health_check: None,
            authentication_required: false,
            rate_limit: None,
            circuit_breaker: None,
        }],
        integration: create_default_integration_preferences(),
        extensions: HashMap::new(),
        registration_timestamp: Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: "compute-001".to_string(),
    };

    let _handle = registry.register_service(compute_registration).await?;

    // Register a storage service
    let storage_registration = UniversalServiceRegistration {
        service_id: Uuid::new_v4(),
        metadata: ServiceMetadata {
            name: "storage-service".to_string(),
            description: "Object storage service".to_string(),
            category: ServiceCategory::Storage,
            tags: vec!["object-storage".to_string()],
            documentation_url: None,
            contact: ContactInfo {
                maintainer: "storage-team".to_string(),
                email: Some("storage@example.com".to_string()),
                documentation: None,
                support: None,
            },
            lifecycle_stage: ServiceLifecycleStage::Production,
            compliance_level: ComplianceLevel::Basic,
        },
        capabilities: vec![ServiceCapability::Storage {
            storage_gb: Some(1000.0),
            storage_type: StorageType::ObjectStorage,
            backup_support: true,
            encryption_support: true,
        }],
        resources: ResourceSpec::default(),
        endpoints: vec![ServiceEndpoint {
            name: "api".to_string(),
            url: "http://localhost:8081".to_string(),
            health_check: None,
            authentication_required: false,
            rate_limit: None,
            circuit_breaker: None,
        }],
        integration: create_default_integration_preferences(),
        extensions: HashMap::new(),
        registration_timestamp: Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: "storage-001".to_string(),
    };

    let _handle = registry.register_service(storage_registration).await?;

    // Test capability-based discovery - look for exact match for now
    // (In a production system, this would use more sophisticated matching)
    let compute_capability = ServiceCapability::Compute {
        cpu_cores: Some(8.0),  // Exact match with registered service
        memory_gb: Some(32.0), // Exact match with registered service
        gpu_support: false,
        container_runtime: Some("docker".to_string()),
    };

    let compute_services = registry
        .find_services_by_capability(vec![compute_capability])
        .await?;

    assert!(!compute_services.is_empty(), "Should find compute services");
    assert_eq!(
        compute_services[0].registration.metadata.name,
        "compute-service"
    );

    Ok(())
}

#[tokio::test]
async fn test_service_health_monitoring() -> PrimalResult<()> {
    let registry = MemoryServiceRegistry::new();

    // Register a service
    let service_id = Uuid::new_v4();
    let registration = create_test_registration(service_id, "health-test-service");
    let _handle = registry.register_service(registration).await?;

    // Update health status
    registry
        .update_health_status(service_id, HealthStatus::Healthy)
        .await?;

    // Record heartbeat
    registry.heartbeat(service_id).await?;

    // Verify service is registered and healthy
    let service_info = registry.get_service(service_id).await?;
    assert!(service_info.is_some());

    let service = service_info.unwrap();
    assert_eq!(service.health_status, HealthStatus::Healthy);
    assert_eq!(service.registration.metadata.name, "health-test-service");

    Ok(())
}

#[tokio::test]
async fn test_service_filtering() -> PrimalResult<()> {
    let registry = MemoryServiceRegistry::new();

    // Register multiple services with different categories
    for (i, category) in [
        ServiceCategory::Compute,
        ServiceCategory::Storage,
        ServiceCategory::Security,
    ]
    .iter()
    .enumerate()
    {
        let service_id = Uuid::new_v4();
        let mut registration = create_test_registration(service_id, &format!("service-{i}"));
        registration.metadata.category = category.clone();
        registry.register_service(registration).await?;
    }

    // Test filtering by category
    let filter = ServiceFilter {
        categories: Some(vec![ServiceCategory::Compute]),
        health_status: None,
        tags: None,
        lifecycle_stages: None,
        compliance_levels: None,
        capabilities: None,
    };

    let compute_services = registry.list_services(Some(filter)).await?;
    assert_eq!(compute_services.len(), 1);
    assert_eq!(
        compute_services[0].registration.metadata.category,
        ServiceCategory::Compute
    );

    Ok(())
}

#[tokio::test]
async fn test_service_lifecycle() -> PrimalResult<()> {
    let registry = MemoryServiceRegistry::new();
    let service_id = Uuid::new_v4();

    // Register service
    let registration = create_test_registration(service_id, "lifecycle-test");
    let _handle = registry.register_service(registration.clone()).await?;

    // Verify registration
    let service = registry.get_service(service_id).await?;
    assert!(service.is_some());

    // Update service
    let mut updated_registration = registration;
    updated_registration.service_version = "2.0.0".to_string();
    registry
        .update_service(service_id, updated_registration)
        .await?;

    // Verify update
    let updated_service = registry.get_service(service_id).await?;
    assert_eq!(
        updated_service.unwrap().registration.service_version,
        "2.0.0"
    );

    // Deregister service
    registry.deregister_service(service_id).await?;

    // Verify deregistration
    let deregistered_service = registry.get_service(service_id).await?;
    assert!(deregistered_service.is_none());

    Ok(())
}

/// Helper function to create a test registration
fn create_test_registration(service_id: Uuid, name: &str) -> UniversalServiceRegistration {
    UniversalServiceRegistration {
        service_id,
        metadata: ServiceMetadata {
            name: name.to_string(),
            description: format!("Test service: {name}"),
            category: ServiceCategory::Compute,
            tags: vec!["test".to_string()],
            documentation_url: None,
            contact: ContactInfo {
                maintainer: "test-team".to_string(),
                email: Some("test@example.com".to_string()),
                documentation: None,
                support: None,
            },
            lifecycle_stage: ServiceLifecycleStage::Development,
            compliance_level: ComplianceLevel::Basic,
        },
        capabilities: vec![ServiceCapability::Compute {
            cpu_cores: Some(2.0),
            memory_gb: Some(4.0),
            gpu_support: false,
            container_runtime: Some("docker".to_string()),
        }],
        resources: ResourceSpec::default(),
        endpoints: vec![ServiceEndpoint {
            name: "api".to_string(),
            url: format!("http://localhost:808{}", service_id.as_u128() % 100),
            health_check: None,
            authentication_required: false,
            rate_limit: None,
            circuit_breaker: None,
        }],
        integration: create_default_integration_preferences(),
        extensions: HashMap::new(),
        registration_timestamp: Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: format!("{name}-001"),
    }
}

/// Helper function to create default integration preferences
fn create_default_integration_preferences() -> IntegrationPreferences {
    IntegrationPreferences {
        preferred_protocols: vec!["http".to_string()],
        load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        circuit_breaker: CircuitBreakerConfig {
            failure_threshold: 5,
            timeout_duration: Duration::from_secs(30),
            half_open_max_calls: 3,
        },
        rate_limiting: RateLimitConfig {
            strategy: RateLimitStrategy::TokenBucket,
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            burst_size: Some(10),
        },
        timeout_ms: 5000,
        retry_count: 3,
        priority: ServicePriority::Normal,
    }
}
