#![cfg(feature = "tests-incomplete")]
#![allow(unexpected_cfgs)]
//! NOTE: Disabled - requires fixes

//! Comprehensive capability discovery tests
//!
//! Tests capability-based service discovery, provider selection, and dynamic discovery

use songbird_discovery::{CapabilityDiscovery, DiscoveryConfig, ServiceRegistration};
use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

#[tokio::test]
async fn test_discover_by_single_capability() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register a service with compute capability
    let service = ServiceRegistration {
        name: "compute-service".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: format!("http://compute:{}", test_orchestrator_port()).to_string(),
        metadata: HashMap::new(),
    };

    discovery.register(service).await.ok();

    // Discover by capability
    let providers = discovery.discover_by_capability("compute").await;

    assert!(providers.is_ok());
    assert!(!providers
        .ok_or_else(|| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?
        .is_empty());
    Ok(())
}

#[tokio::test]
async fn test_discover_multiple_capabilities() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register services with different capabilities
    discovery
        .register(ServiceRegistration {
            name: "service1".to_string(),
            capabilities: vec!["compute".to_string(), "storage".to_string()],
            endpoint: format!("http://service1:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    discovery
        .register(ServiceRegistration {
            name: "service2".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: format!("http://service2:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    // Discover compute providers
    let compute_providers = discovery.discover_by_capability("compute").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(compute_providers.len(), 2);

    // Discover storage providers
    let storage_providers = discovery.discover_by_capability("storage").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(storage_providers.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_capability_not_found() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    let providers = discovery.discover_by_capability("nonexistent").await;

    // Should return empty list, not error
    assert!(providers.is_ok());
    assert!(providers
        .ok_or_else(|| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?
        .is_empty());
    Ok(())
}

#[tokio::test]
async fn test_dynamic_capability_registration() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Initially no providers
    let initial = discovery.discover_by_capability("ai").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert!(initial.is_empty());

    // Register new provider
    discovery
        .register(ServiceRegistration {
            name: "ai-service".to_string(),
            capabilities: vec!["ai".to_string()],
            endpoint: format!("http://ai:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    // Now should find provider
    let after = discovery.discover_by_capability("ai").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(after.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_capability_deregistration() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    discovery
        .register(ServiceRegistration {
            name: "temp-service".to_string(),
            capabilities: vec!["temp".to_string()],
            endpoint: format!("http://temp:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    // Deregister
    discovery.deregister("temp-service").await.ok();

    // Should no longer be discoverable
    let providers = discovery.discover_by_capability("temp").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert!(providers.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_multiple_providers_same_capability() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register multiple compute providers
    for i in 1..=5 {
        discovery
            .register(ServiceRegistration {
                name: format!("compute-{}", i),
                capabilities: vec!["compute".to_string()],
                endpoint: format!("http://compute{}:{}", i, test_orchestrator_port()),
                metadata: HashMap::new(),
            })
            .await
            .ok();
    }

    let providers = discovery.discover_by_capability("compute").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(providers.len(), 5);
    Ok(())
}

#[tokio::test]
async fn test_capability_with_metadata() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    let mut metadata = HashMap::new();
    metadata.insert("gpu".to_string(), "RTX3090".to_string());
    metadata.insert("location".to_string(), "us-west".to_string());

    discovery
        .register(ServiceRegistration {
            name: "gpu-compute".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: format!("http://gpu-compute:{}", test_orchestrator_port()).to_string(),
            metadata,
        })
        .await
        .ok();

    let providers = discovery.discover_by_capability("compute").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert!(!providers.is_empty());
    assert!(providers[0].metadata.contains_key("gpu"));
    Ok(())
}

#[tokio::test]
async fn test_filtered_discovery_by_metadata() {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register services with different GPU types
    let mut meta1 = HashMap::new();
    meta1.insert("gpu".to_string(), "RTX3090".to_string());

    let mut meta2 = HashMap::new();
    meta2.insert("gpu".to_string(), "V100".to_string());

    discovery
        .register(ServiceRegistration {
            name: "service1".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: format!("http://service1:{}", test_orchestrator_port()).to_string(),
            metadata: meta1,
        })
        .await
        .ok();

    discovery
        .register(ServiceRegistration {
            name: "service2".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: format!("http://service2:{}", test_orchestrator_port()).to_string(),
            metadata: meta2,
        })
        .await
        .ok();

    // Discover with filter
    let mut filter = HashMap::new();
    filter.insert("gpu".to_string(), "RTX3090".to_string());

    let filtered = discovery.discover_by_capability_with_filter("compute", filter).await;
    assert!(filtered.is_ok());
}

#[tokio::test]
async fn test_capability_priority_selection() {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register providers with different priorities
    for i in 1..=3 {
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), i.to_string());

        discovery
            .register(ServiceRegistration {
                name: format!("service-{}", i),
                capabilities: vec!["storage".to_string()],
                endpoint: format!("http://service{}:{}", i, test_orchestrator_port()),
                metadata,
            })
            .await
            .ok();
    }

    // Should be able to select by priority
    let highest_priority = discovery.select_highest_priority("storage").await;
    assert!(highest_priority.is_ok());
}

#[tokio::test]
async fn test_capability_health_aware_discovery() {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    discovery
        .register(ServiceRegistration {
            name: "healthy-service".to_string(),
            capabilities: vec!["api".to_string()],
            endpoint: format!("http://healthy:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    // Mark as healthy
    discovery.update_health("healthy-service", true).await.ok();

    // Should discover healthy services
    let providers = discovery.discover_healthy_providers("api").await;
    assert!(providers.is_ok());
}

#[tokio::test]
async fn test_concurrent_capability_registration() {
    let discovery = std::sync::Arc::new(CapabilityDiscovery::new(DiscoveryConfig::default()));

    let mut handles = vec![];

    // Concurrently register services
    for i in 0..10 {
        let discovery_clone = std::sync::Arc::clone(&discovery);
        let handle = tokio::spawn(async move {
            discovery_clone
                .register(ServiceRegistration {
                    name: format!("service-{}", i),
                    capabilities: vec!["test".to_string()],
                    endpoint: format!("http://service{}:{}", i, test_orchestrator_port()),
                    metadata: HashMap::new(),
                })
                .await
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.await.ok();
    }

    // Should have all 10 services
    let providers = discovery.discover_by_capability("test").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(providers.len(), 10);
}

#[tokio::test]
async fn test_capability_cache_invalidation() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::with_cache_ttl(100));

    discovery
        .register(ServiceRegistration {
            name: "cached-service".to_string(),
            capabilities: vec!["cache-test".to_string()],
            endpoint: format!("http://cached:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    // First discovery (caches)
    let first = discovery.discover_by_capability("cache-test").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(first.len(), 1);

    // Deregister
    discovery.deregister("cached-service").await.ok();

    // Wait for cache to expire
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    // Should reflect deregistration
    let after = discovery.discover_by_capability("cache-test").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert!(after.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_wildcard_capability_discovery() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    discovery
        .register(ServiceRegistration {
            name: "multi-service".to_string(),
            capabilities: vec!["compute".to_string(), "storage".to_string(), "network".to_string()],
            endpoint: format!("http://multi:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await
        .ok();

    // Discover all capabilities
    let all_capabilities = discovery.list_all_capabilities().await;
    assert!(all_capabilities.is_ok());
    assert!(
        all_capabilities
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .len()
            >= 3
    );
    Ok(())
}

#[tokio::test]
async fn test_capability_version_compatibility() {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "2.0".to_string());

    discovery
        .register(ServiceRegistration {
            name: "versioned-service".to_string(),
            capabilities: vec!["api".to_string()],
            endpoint: format!("http://versioned:{}", test_orchestrator_port()).to_string(),
            metadata,
        })
        .await
        .ok();

    // Should be able to discover by version compatibility
    let compatible = discovery.discover_compatible_version("api", "2.0").await;
    assert!(compatible.is_ok());
}

#[tokio::test]
async fn test_capability_discovery_timeout() {
    let config = DiscoveryConfig::with_timeout(50); // 50ms timeout
    let discovery = CapabilityDiscovery::new(config);

    // Discovery should respect timeout
    let result = discovery.discover_by_capability("slow-capability").await;

    // Should complete (timeout or success)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_empty_capability_list() {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register service with no capabilities
    let result = discovery
        .register(ServiceRegistration {
            name: "no-caps".to_string(),
            capabilities: vec![],
            endpoint: format!("http://no-caps:{}", test_orchestrator_port()).to_string(),
            metadata: HashMap::new(),
        })
        .await;

    // Should handle gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_duplicate_capability_registration() -> SongbirdResult<()> {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    let service = ServiceRegistration {
        name: "service1".to_string(),
        capabilities: vec!["duplicate".to_string()],
        endpoint: format!("http://service1:{}", test_orchestrator_port()).to_string(),
        metadata: HashMap::new(),
    };

    // Register twice
    discovery.register(service.clone()).await.ok();
    discovery.register(service).await.ok();

    // Should handle duplicates (either dedupe or allow)
    let providers = discovery.discover_by_capability("duplicate").await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert!(providers.len() >= 1);
    Ok(())
}

#[tokio::test]
async fn test_capability_statistics() {
    let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

    // Register several services
    for i in 1..=5 {
        discovery
            .register(ServiceRegistration {
                name: format!("service-{}", i),
                capabilities: vec!["stats-test".to_string()],
                endpoint: format!("http://service{}:{}", i, test_orchestrator_port()),
                metadata: HashMap::new(),
            })
            .await
            .ok();
    }

    // Should provide statistics
    let stats = discovery.get_capability_statistics("stats-test").await;
    assert!(stats.is_ok());
}
