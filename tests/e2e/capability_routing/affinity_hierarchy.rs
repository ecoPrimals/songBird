// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Region affinity, hierarchical capability names, API versions, and cross-region routing.

use super::common::{MockServiceConfig, TestAssertions, TestEnvironment};
use std::collections::HashMap;

#[tokio::test]
async fn test_capability_based_affinity_routing() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;

    // Create services in different regions with same capability
    for region in &["us-east", "us-west", "eu-central"] {
        let config = MockServiceConfig::new(&format!("compute-{}", region)).with_capability("compute");

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
    let us_east_services: Vec<_> = all_services
        .iter()
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
        let config = MockServiceConfig::new(name).with_capabilities(caps.to_vec());

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
        let config = MockServiceConfig::new(name).with_capabilities(caps.to_vec());

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
        let config = MockServiceConfig::new(&name).with_capabilities(caps.to_vec());

        env.start_mock_service(&name, config).await?;

        let service = songbird_types::ServiceInfo {
            name: name.clone(),
            version: "1.0.0".to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: env.get_endpoint(&name, 80),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("region".to_string(), region.to_string());
                meta.insert(
                    "geo".to_string(),
                    if region.starts_with("us") {
                        "americas"
                    } else {
                        "europe"
                    }
                    .to_string(),
                );
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
    let us_services: Vec<_> = compute_services
        .iter()
        .filter(|s| s.metadata.get("geo").map(|g| g == "americas").unwrap_or(false))
        .collect();

    assert_eq!(us_services.len(), 2, "Should find 2 US compute services");

    Ok(())
}
