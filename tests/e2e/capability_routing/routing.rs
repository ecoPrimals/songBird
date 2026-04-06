// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core discovery: multi-capability services, priority, load spread, fallback, and dynamic updates.

use super::common::{MockServiceConfig, TestAssertions, TestEnvironment};
use std::collections::HashMap;

#[tokio::test]
async fn test_route_by_multiple_capabilities() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create a service with multiple capabilities
    let config = MockServiceConfig::new("multi-cap-service")
        .with_capabilities(vec!["compute", "storage", "ai"]);

    env.start_mock_service("multi-cap-service", config).await?;

    let service = songbird_types::ServiceInfo {
        name: "multi-cap-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec![
            "compute".to_string(),
            "storage".to_string(),
            "ai".to_string(),
        ],
        endpoint: env.get_endpoint("multi-cap-service", 20),
        metadata: HashMap::new(),
    };

    env.register_service(service).await?;

    // Test routing by each capability
    for capability in &["compute", "storage", "ai"] {
        let discovered = env.discover_services(capability).await?;

        TestAssertions::assert_services_found(&discovered);
        TestAssertions::assert_service_present(&discovered, "multi-cap-service");

        // Verify the service has the requested capability
        let service = discovered
            .iter()
            .find(|s| s.name == "multi-cap-service")
            .expect("Service should be found");

        TestAssertions::assert_has_capability(service, capability);
    }

    Ok(())
}

#[tokio::test]
async fn test_capability_priority_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create services with different capability priorities
    let specialist_config = MockServiceConfig::new("specialist-compute")
        .with_capabilities(vec!["compute"])
        .with_priority(10);

    let generalist_config = MockServiceConfig::new("generalist")
        .with_capabilities(vec!["compute", "storage", "ai"])
        .with_priority(5);

    env.start_mock_service("specialist-compute", specialist_config).await?;
    env.start_mock_service("generalist", generalist_config).await?;

    // Register both services
    let specialist = songbird_types::ServiceInfo {
        name: "specialist-compute".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("specialist-compute", 21),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("priority".to_string(), "10".to_string());
            meta
        },
    };

    let generalist = songbird_types::ServiceInfo {
        name: "generalist".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec![
            "compute".to_string(),
            "storage".to_string(),
            "ai".to_string(),
        ],
        endpoint: env.get_endpoint("generalist", 22),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("priority".to_string(), "5".to_string());
            meta
        },
    };

    env.register_service(specialist).await?;
    env.register_service(generalist).await?;

    // Discover compute services - specialist should be prioritized
    let discovered = env.discover_services("compute").await?;

    TestAssertions::assert_services_found(&discovered);
    assert_eq!(discovered.len(), 2, "Should find both services");

    // First service should be the specialist (higher priority)
    if let Some(first) = discovered.first() {
        assert_eq!(
            first.name, "specialist-compute",
            "Specialist should be first due to higher priority"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_capability_based_load_balancing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create multiple services with the same capability
    for i in 1..=3 {
        let config = MockServiceConfig::new(&format!("compute-service-{}", i))
            .with_capability("compute");

        env.start_mock_service(&format!("compute-service-{}", i), config).await?;

        let service = songbird_types::ServiceInfo {
            name: format!("compute-service-{}", i),
            version: "1.0.0".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: env.get_endpoint(&format!("compute-service-{}", i), 20 + i),
            metadata: HashMap::new(),
        };

        env.register_service(service).await?;
    }

    // Make multiple requests and track which services are used
    let mut service_hits: HashMap<String, usize> = HashMap::new();
    let num_requests = 30;

    for _ in 0..num_requests {
        let discovered = env.discover_services("compute").await?;

        // Simulate load balancing by picking a service
        if let Some(service) = discovered.first() {
            *service_hits.entry(service.name.clone()).or_insert(0) += 1;
        }
    }

    // Verify load is distributed across all services
    assert_eq!(service_hits.len(), 3, "All services should receive traffic");

    // Each service should get approximately equal load (within 50% variance)
    let expected_per_service = num_requests / 3;
    for (service, hits) in &service_hits {
        assert!(*hits > 0, "Service {} should receive requests", service);

        // Allow for some variance in distribution (rough load balancing)
        let variance = expected_per_service / 2;
        assert!(
            *hits >= expected_per_service - variance && *hits <= expected_per_service + variance,
            "Service {} has unbalanced load: {} requests (expected ~{})",
            service,
            hits,
            expected_per_service
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_fallback_to_multi_capability_service() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create a specialist that's unhealthy and a generalist that's healthy
    let specialist_config = MockServiceConfig::new("compute-specialist")
        .with_capability("compute")
        .with_health(songbird_types::HealthStatus::Unhealthy);

    let generalist_config = MockServiceConfig::new("multi-service")
        .with_capabilities(vec!["compute", "storage"])
        .with_health(songbird_types::HealthStatus::Healthy);

    env.start_mock_service("compute-specialist", specialist_config).await?;
    env.start_mock_service("multi-service", generalist_config).await?;

    let specialist = songbird_types::ServiceInfo {
        name: "compute-specialist".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("compute-specialist", 30),
        metadata: HashMap::new(),
    };

    let generalist = songbird_types::ServiceInfo {
        name: "multi-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        endpoint: env.get_endpoint("multi-service", 31),
        metadata: HashMap::new(),
    };

    env.register_service(specialist).await?;
    env.register_service(generalist).await?;

    // Discover compute services
    let discovered = env.discover_services("compute").await?;

    TestAssertions::assert_services_found(&discovered);

    // Should include both, but healthy generalist should be preferred for routing
    let healthy_services: Vec<_> = discovered
        .iter()
        .filter(|s| {
            env.check_service_health(&s.name)
                .map(|h| h == songbird_types::HealthStatus::Healthy)
                .unwrap_or(false)
        })
        .collect();

    assert!(!healthy_services.is_empty(), "Should find healthy services");
    assert!(
        healthy_services.iter().any(|s| s.name == "multi-service"),
        "Multi-capability service should be available as fallback"
    );

    Ok(())
}

#[tokio::test]
async fn test_capability_combination_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create services with different capability combinations
    let ai_compute_config = MockServiceConfig::new("ai-compute").with_capabilities(vec!["ai", "compute"]);

    let storage_compute_config =
        MockServiceConfig::new("storage-compute").with_capabilities(vec!["storage", "compute"]);

    let all_in_one_config =
        MockServiceConfig::new("all-in-one").with_capabilities(vec!["ai", "compute", "storage"]);

    env.start_mock_service("ai-compute", ai_compute_config).await?;
    env.start_mock_service("storage-compute", storage_compute_config).await?;
    env.start_mock_service("all-in-one", all_in_one_config).await?;

    // Register services
    for (name, caps) in &[
        ("ai-compute", vec!["ai", "compute"]),
        ("storage-compute", vec!["storage", "compute"]),
        ("all-in-one", vec!["ai", "compute", "storage"]),
    ] {
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: env.get_endpoint(name, 40),
            metadata: HashMap::new(),
        };
        env.register_service(service).await?;
    }

    // Test finding services with specific capability combinations

    // Services with AI capability
    let ai_services = env.discover_services("ai").await?;
    assert_eq!(ai_services.len(), 2, "Should find 2 services with AI");
    assert!(ai_services.iter().any(|s| s.name == "ai-compute"));
    assert!(ai_services.iter().any(|s| s.name == "all-in-one"));

    // Services with storage capability
    let storage_services = env.discover_services("storage").await?;
    assert_eq!(storage_services.len(), 2, "Should find 2 services with storage");
    assert!(storage_services.iter().any(|s| s.name == "storage-compute"));
    assert!(storage_services.iter().any(|s| s.name == "all-in-one"));

    // Services with compute capability (all of them)
    let compute_services = env.discover_services("compute").await?;
    assert_eq!(compute_services.len(), 3, "Should find 3 services with compute");

    Ok(())
}

#[tokio::test]
async fn test_capability_mismatch_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create services with specific capabilities
    let config = MockServiceConfig::new("storage-only").with_capability("storage");

    env.start_mock_service("storage-only", config).await?;

    let service = songbird_types::ServiceInfo {
        name: "storage-only".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["storage".to_string()],
        endpoint: env.get_endpoint("storage-only", 50),
        metadata: HashMap::new(),
    };

    env.register_service(service).await?;

    // Try to discover services with a non-existent capability
    let ai_services = env.discover_services("ai").await?;

    // Should find no services
    assert!(ai_services.is_empty(), "Should find no services with 'ai' capability");

    // Storage services should still be discoverable
    let storage_services = env.discover_services("storage").await?;
    TestAssertions::assert_services_found(&storage_services);
    TestAssertions::assert_service_present(&storage_services, "storage-only");

    Ok(())
}

#[tokio::test]
async fn test_dynamic_capability_updates() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create a service with initial capabilities
    let config = MockServiceConfig::new("dynamic-service").with_capability("compute");

    env.start_mock_service("dynamic-service", config).await?;

    let mut service = songbird_types::ServiceInfo {
        name: "dynamic-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("dynamic-service", 51),
        metadata: HashMap::new(),
    };

    env.register_service(service.clone()).await?;

    // Verify initial capability
    let compute_services = env.discover_services("compute").await?;
    TestAssertions::assert_service_present(&compute_services, "dynamic-service");

    // Update service with additional capability
    service.capabilities.push("ai".to_string());
    env.update_service(service).await?;

    // Verify new capability is discoverable
    let ai_services = env.discover_services("ai").await?;
    TestAssertions::assert_service_present(&ai_services, "dynamic-service");

    // Verify old capability still works
    let compute_services = env.discover_services("compute").await?;
    TestAssertions::assert_service_present(&compute_services, "dynamic-service");

    Ok(())
}
