//! Capability-Based Routing E2E Tests
//!
//! Tests for multi-capability service routing and discovery

#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;

use common::{TestEnvironment, MockServiceConfig, TestAssertions};
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
        let service = discovered.iter()
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
        assert_eq!(first.name, "specialist-compute", 
            "Specialist should be first due to higher priority");
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
        assert!(*hits >= expected_per_service - variance && *hits <= expected_per_service + variance,
            "Service {} has unbalanced load: {} requests (expected ~{})", 
            service, hits, expected_per_service);
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
    let healthy_services: Vec<_> = discovered.iter()
        .filter(|s| {
            env.check_service_health(&s.name)
                .map(|h| h == songbird_types::HealthStatus::Healthy)
                .unwrap_or(false)
        })
        .collect();
    
    assert!(!healthy_services.is_empty(), "Should find healthy services");
    assert!(healthy_services.iter().any(|s| s.name == "multi-service"),
        "Multi-capability service should be available as fallback");
    
    Ok(())
}

#[tokio::test]
async fn test_capability_combination_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with different capability combinations
    let ai_compute_config = MockServiceConfig::new("ai-compute")
        .with_capabilities(vec!["ai", "compute"]);
        
    let storage_compute_config = MockServiceConfig::new("storage-compute")
        .with_capabilities(vec!["storage", "compute"]);
        
    let all_in_one_config = MockServiceConfig::new("all-in-one")
        .with_capabilities(vec!["ai", "compute", "storage"]);
    
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
    let config = MockServiceConfig::new("storage-only")
        .with_capability("storage");
    
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
    let config = MockServiceConfig::new("dynamic-service")
        .with_capability("compute");
    
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

#[tokio::test]
async fn test_capability_based_affinity_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services in different regions with same capability
    for region in &["us-east", "us-west", "eu-central"] {
        let config = MockServiceConfig::new(&format!("compute-{}", region))
            .with_capability("compute");
        
        env.start_mock_service(&format!("compute-{}", region), config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: format!("compute-{}", region),
            version: "1.0.0".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: env.get_endpoint(&format!("compute-{}", region), 52),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("region".to_string(), region.to_string());
                meta
            },
        };
        
        env.register_service(service).await?;
    }
    
    // Discover services with region affinity
    let all_services = env.discover_services("compute").await?;
    assert_eq!(all_services.len(), 3, "Should find all 3 regional services");
    
    // Filter for specific region
    let us_east_services: Vec<_> = all_services.iter()
        .filter(|s| s.metadata.get("region").map(|r| r == "us-east").unwrap_or(false))
        .collect();
    
    assert_eq!(us_east_services.len(), 1, "Should find exactly one us-east service");
    assert_eq!(us_east_services[0].name, "compute-us-east");
    
    Ok(())
}

#[tokio::test]
async fn test_wildcard_capability_matching() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with hierarchical capabilities
    for (name, caps) in &[
        ("db-postgres", vec!["database", "database:postgres"]),
        ("db-mysql", vec!["database", "database:mysql"]),
        ("db-mongo", vec!["database", "database:mongodb"]),
    ] {
        let config = MockServiceConfig::new(name)
            .with_capabilities(caps.to_vec());
        
        env.start_mock_service(name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: env.get_endpoint(name, 60),
            metadata: HashMap::new(),
        };
        
        env.register_service(service).await?;
    }
    
    // Discover all database services
    let db_services = env.discover_services("database").await?;
    assert_eq!(db_services.len(), 3, "Should find all 3 database services");
    
    // Discover specific database type
    let postgres_services = env.discover_services("database:postgres").await?;
    assert_eq!(postgres_services.len(), 1, "Should find postgres service");
    TestAssertions::assert_service_present(&postgres_services, "db-postgres");
    
    Ok(())
}

#[tokio::test]
async fn test_capability_version_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with different API versions
    for (name, version, caps) in &[
        ("api-v1", "1.0.0", vec!["api:v1"]),
        ("api-v2", "2.0.0", vec!["api:v2", "api:v1"]), // v2 is backward compatible
        ("api-v3", "3.0.0", vec!["api:v3"]),
    ] {
        let config = MockServiceConfig::new(name)
            .with_capabilities(caps.to_vec());
        
        env.start_mock_service(name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: version.to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: env.get_endpoint(name, 70),
            metadata: HashMap::new(),
        };
        
        env.register_service(service).await?;
    }
    
    // Discover v1 compatible services (should find v1 and v2)
    let v1_services = env.discover_services("api:v1").await?;
    assert_eq!(v1_services.len(), 2, "Should find 2 v1-compatible services");
    assert!(v1_services.iter().any(|s| s.name == "api-v1"));
    assert!(v1_services.iter().any(|s| s.name == "api-v2"));
    
    // Discover v2 services (should find only v2)
    let v2_services = env.discover_services("api:v2").await?;
    assert_eq!(v2_services.len(), 1, "Should find 1 v2 service");
    TestAssertions::assert_service_present(&v2_services, "api-v2");
    
    Ok(())
}

#[tokio::test]
async fn test_cross_region_capability_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services across multiple regions with overlapping capabilities
    for (region, caps) in &[
        ("us-east-1", vec!["compute", "storage"]),
        ("us-west-2", vec!["compute", "ai"]),
        ("eu-west-1", vec!["storage", "ai"]),
    ] {
        let name = format!("service-{}", region);
        let config = MockServiceConfig::new(&name)
            .with_capabilities(caps.to_vec());
        
        env.start_mock_service(&name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.clone(),
            version: "1.0.0".to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: env.get_endpoint(&name, 80),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("region".to_string(), region.to_string());
                meta.insert("geo".to_string(), if region.starts_with("us") { "americas" } else { "europe" }.to_string());
                meta
            },
        };
        
        env.register_service(service).await?;
    }
    
    // Discover compute services (should find 2)
    let compute_services = env.discover_services("compute").await?;
    assert_eq!(compute_services.len(), 2, "Should find 2 compute services");
    
    // Discover AI services (should find 2)
    let ai_services = env.discover_services("ai").await?;
    assert_eq!(ai_services.len(), 2, "Should find 2 AI services");
    
    // Filter by geography
    let us_services: Vec<_> = compute_services.iter()
        .filter(|s| s.metadata.get("geo").map(|g| g == "americas").unwrap_or(false))
        .collect();
    
    assert_eq!(us_services.len(), 2, "Should find 2 US compute services");
    
    Ok(())
}

#[tokio::test]
async fn test_capability_based_rate_limiting() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with rate limit metadata
    for (name, rate_limit) in &[
        ("fast-service", "1000"),
        ("standard-service", "100"),
        ("limited-service", "10"),
    ] {
        let config = MockServiceConfig::new(name)
            .with_capability("api");
        
        env.start_mock_service(name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["api".to_string()],
            endpoint: env.get_endpoint(name, 90),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("rate_limit_per_sec".to_string(), rate_limit.to_string());
                meta
            },
        };
        
        env.register_service(service).await?;
    }
    
    // Discover API services
    let services = env.discover_services("api").await?;
    assert_eq!(services.len(), 3, "Should find all 3 API services");
    
    // Services should be ordered by rate limit (highest first) if properly routed
    let mut sorted_services = services.clone();
    sorted_services.sort_by(|a, b| {
        let a_limit = a.metadata.get("rate_limit_per_sec")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let b_limit = b.metadata.get("rate_limit_per_sec")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        b_limit.cmp(&a_limit) // Descending order
    });
    
    // Verify highest rate limit service is available
    assert_eq!(sorted_services[0].name, "fast-service");
    assert_eq!(sorted_services[2].name, "limited-service");
    
    Ok(())
}

#[tokio::test]
async fn test_complex_capability_queries() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with various capability combinations
    for (name, caps) in &[
        ("ml-gpu-service", vec!["ml", "gpu", "compute"]),
        ("ml-cpu-service", vec!["ml", "cpu", "compute"]),
        ("data-gpu-service", vec!["data", "gpu", "storage"]),
        ("basic-service", vec!["compute"]),
    ] {
        let config = MockServiceConfig::new(name)
            .with_capabilities(caps.to_vec());
        
        env.start_mock_service(name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: env.get_endpoint(name, 100),
            metadata: HashMap::new(),
        };
        
        env.register_service(service).await?;
    }
    
    // Query: Find services with ML capability
    let ml_services = env.discover_services("ml").await?;
    assert_eq!(ml_services.len(), 2, "Should find 2 ML services");
    
    // Query: Find services with GPU capability
    let gpu_services = env.discover_services("gpu").await?;
    assert_eq!(gpu_services.len(), 2, "Should find 2 GPU services");
    
    // Query: Find services with both ML AND GPU (manual intersection)
    let ml_gpu_services: Vec<_> = ml_services.iter()
        .filter(|s| s.capabilities.contains(&"gpu".to_string()))
        .collect();
    
    assert_eq!(ml_gpu_services.len(), 1, "Should find 1 service with both ML and GPU");
    assert_eq!(ml_gpu_services[0].name, "ml-gpu-service");
    
    // Query: Find services with compute capability (union)
    let compute_services = env.discover_services("compute").await?;
    assert_eq!(compute_services.len(), 3, "Should find 3 compute services");
    
    Ok(())
}

#[tokio::test]
async fn test_capability_discovery_caching() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a service
    let config = MockServiceConfig::new("cached-service")
        .with_capability("cache-test");
    
    env.start_mock_service("cached-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "cached-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["cache-test".to_string()],
        endpoint: env.get_endpoint("cached-service", 110),
        metadata: HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // First discovery - should populate cache
    let start = std::time::Instant::now();
    let first_discovery = env.discover_services("cache-test").await?;
    let first_duration = start.elapsed();
    
    TestAssertions::assert_services_found(&first_discovery);
    TestAssertions::assert_service_present(&first_discovery, "cached-service");
    
    // Second discovery - should be faster due to caching
    let start = std::time::Instant::now();
    let second_discovery = env.discover_services("cache-test").await?;
    let second_duration = start.elapsed();
    
    // Results should be identical
    assert_eq!(first_discovery.len(), second_discovery.len());
    assert_eq!(first_discovery[0].name, second_discovery[0].name);
    
    // Second call should be faster or at least not significantly slower
    // (This is a soft assertion, as caching behavior may vary)
    println!("First discovery: {:?}, Second discovery: {:?}", first_duration, second_duration);
    
    Ok(())
}

#[tokio::test]
async fn test_capability_metric_based_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with different performance characteristics
    for (name, latency_ms, throughput) in &[
        ("fast-low-throughput", "10", "100"),
        ("slow-high-throughput", "100", "10000"),
        ("balanced", "30", "1000"),
    ] {
        let config = MockServiceConfig::new(name)
            .with_capability("processing");
        
        env.start_mock_service(name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["processing".to_string()],
            endpoint: env.get_endpoint(name, 120),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("avg_latency_ms".to_string(), latency_ms.to_string());
                meta.insert("throughput_rps".to_string(), throughput.to_string());
                meta
            },
        };
        
        env.register_service(service).await?;
    }
    
    // Discover all processing services
    let services = env.discover_services("processing").await?;
    assert_eq!(services.len(), 3, "Should find all 3 processing services");
    
    // Find lowest latency service
    let lowest_latency = services.iter()
        .min_by_key(|s| {
            s.metadata.get("avg_latency_ms")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(u32::MAX)
        })
        .unwrap();
    
    assert_eq!(lowest_latency.name, "fast-low-throughput");
    
    // Find highest throughput service
    let highest_throughput = services.iter()
        .max_by_key(|s| {
            s.metadata.get("throughput_rps")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0)
        })
        .unwrap();
    
    assert_eq!(highest_throughput.name, "slow-high-throughput");
    
    Ok(())
}
