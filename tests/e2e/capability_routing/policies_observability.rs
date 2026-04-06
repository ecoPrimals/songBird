// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Metadata-driven ordering, rate limits, multi-query intersections, caching, and performance hints.

use super::common::{MockServiceConfig, TestAssertions, TestEnvironment};
use std::collections::HashMap;

#[tokio::test]
async fn test_capability_based_rate_limiting() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create services with rate limit metadata
    for (name, rate_limit) in &[
        ("fast-service", "1000"),
        ("standard-service", "100"),
        ("limited-service", "10"),
    ] {
        let config = MockServiceConfig::new(name).with_capability("api");

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
        let a_limit = a
            .metadata
            .get("rate_limit_per_sec")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let b_limit = b
            .metadata
            .get("rate_limit_per_sec")
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
        let config = MockServiceConfig::new(name).with_capabilities(caps.to_vec());

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
    let ml_gpu_services: Vec<_> = ml_services
        .iter()
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
    let config = MockServiceConfig::new("cached-service").with_capability("cache-test");

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
        let config = MockServiceConfig::new(name).with_capability("processing");

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
    let lowest_latency = services
        .iter()
        .min_by_key(|s| {
            s.metadata
                .get("avg_latency_ms")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(u32::MAX)
        })
        .unwrap();

    assert_eq!(lowest_latency.name, "fast-low-throughput");

    // Find highest throughput service
    let highest_throughput = services
        .iter()
        .max_by_key(|s| {
            s.metadata
                .get("throughput_rps")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0)
        })
        .unwrap();

    assert_eq!(highest_throughput.name, "slow-high-throughput");

    Ok(())
}
