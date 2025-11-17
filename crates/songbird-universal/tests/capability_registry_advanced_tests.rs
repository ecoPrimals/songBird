//! Comprehensive tests for Capability Registry and Service Discovery
//!
//! Tests capability registration, lookup, indexing, and service discovery

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{CapabilityRegistry, RegistryStats};

// ============================================================================
// CapabilityRegistry Tests
// ============================================================================

#[test]
fn test_capability_registry_new() -> SongbirdResult<()> {
    let registry = CapabilityRegistry::default();
    assert!(registry.service_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.service_info.is_empty());
    assert!(registry.last_updated.is_empty());
    Ok(())
}

#[test]
fn test_capability_registry_clone() -> SongbirdResult<()> {
    let registry = CapabilityRegistry::default();
    let cloned = registry;

    assert!(cloned.service_capabilities.is_empty());
    assert!(cloned.capability_providers.is_empty());
    Ok(())
}

#[test]
fn test_capability_registry_debug() -> SongbirdResult<()> {
    let registry = CapabilityRegistry::default();
    let debug_str = format!("{registry:?}");
    assert!(debug_str.contains("CapabilityRegistry"));
    Ok(())
}

#[test]
fn test_capability_registry_service_capabilities() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    // Add service capabilities
    registry.service_capabilities.insert("service-1".to_string(), vec![]);

    assert_eq!(registry.service_capabilities.len(), 1);
    assert!(registry.service_capabilities.contains_key("service-1"));
    Ok(())
}

#[test]
fn test_capability_registry_capability_providers() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    // Add capability providers
    registry
        .capability_providers
        .insert("compute".to_string(), vec!["service-1".to_string(), "service-2".to_string()]);

    assert_eq!(registry.capability_providers.len(), 1);
    assert_eq!(
        registry
            .capability_providers
            .get("compute")
            .ok_or_else(|| SongbirdError::configuration("Capability not found".to_string()))?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn test_capability_registry_last_updated() {
    let mut registry = CapabilityRegistry::default();
    let now = chrono::Utc::now();

    registry.last_updated.insert("service-1".to_string(), now);

    assert_eq!(registry.last_updated.len(), 1);
    assert!(registry.last_updated.contains_key("service-1"));
}

#[test]
fn test_capability_registry_multiple_services() {
    let mut registry = CapabilityRegistry::default();
    let now = chrono::Utc::now();

    // Add multiple services
    for i in 1..=5 {
        let service_id = format!("service-{i}");
        registry.service_capabilities.insert(service_id.clone(), vec![]);
        registry.last_updated.insert(service_id, now);
    }

    assert_eq!(registry.service_capabilities.len(), 5);
    assert_eq!(registry.last_updated.len(), 5);
}

#[test]
fn test_capability_registry_multiple_capabilities() {
    let mut registry = CapabilityRegistry::default();

    // Add multiple capabilities
    let capabilities = vec!["compute", "storage", "network", "security"];
    for cap in capabilities {
        registry.capability_providers.insert(cap.to_string(), vec!["provider-1".to_string()]);
    }

    assert_eq!(registry.capability_providers.len(), 4);
}

#[test]
fn test_capability_registry_service_removal() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    registry.service_capabilities.insert("service-1".to_string(), vec![]);
    assert_eq!(registry.service_capabilities.len(), 1);

    registry.service_capabilities.remove("service-1");
    assert_eq!(registry.service_capabilities.len(), 0);
    Ok(())
}

#[test]
fn test_capability_registry_capability_lookup() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    registry
        .capability_providers
        .insert("ai".to_string(), vec!["service-a".to_string(), "service-b".to_string()]);

    let providers = registry.capability_providers.get("ai");
    assert!(providers.is_some());
    assert_eq!(
        providers
            .ok_or_else(|| SongbirdError::configuration("Capability not found".to_string()))?
            .len(),
        2
    );
    Ok(())
}

// ============================================================================
// RegistryStats Tests
// ============================================================================

#[test]
fn test_registry_stats_creation() {
    let stats = RegistryStats {
        total_services: 10,
        total_capabilities: 25,
        healthy_services: 8,
    };

    assert_eq!(stats.total_services, 10);
    assert_eq!(stats.total_capabilities, 25);
    assert_eq!(stats.healthy_services, 8);
}

#[test]
fn test_registry_stats_clone() -> SongbirdResult<()> {
    let stats = RegistryStats {
        total_services: 5,
        total_capabilities: 10,
        healthy_services: 5,
    };

    let cloned = stats.clone();
    assert_eq!(stats.total_services, cloned.total_services);
    assert_eq!(stats.total_capabilities, cloned.total_capabilities);
    Ok(())
}

#[test]
fn test_registry_stats_debug() -> SongbirdResult<()> {
    let stats = RegistryStats {
        total_services: 5,
        total_capabilities: 10,
        healthy_services: 5,
    };

    let debug_str = format!("{stats:?}");
    assert!(debug_str.contains("RegistryStats"));
    Ok(())
}

#[test]
fn test_registry_stats_all_healthy() {
    let stats = RegistryStats {
        total_services: 10,
        total_capabilities: 20,
        healthy_services: 10,
    };

    assert_eq!(stats.total_services, stats.healthy_services);
}

#[test]
fn test_registry_stats_partial_healthy() {
    let stats = RegistryStats {
        total_services: 10,
        total_capabilities: 20,
        healthy_services: 6,
    };

    assert!(stats.healthy_services < stats.total_services);
    assert!(stats.healthy_services > 0);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_registry_with_stats() {
    let mut registry = CapabilityRegistry::default();

    // Add some services
    for i in 1..=5 {
        registry.service_capabilities.insert(format!("service-{i}"), vec![]);
    }

    // Create stats
    let stats = RegistryStats {
        total_services: registry.service_capabilities.len(),
        total_capabilities: 0,
        healthy_services: registry.service_capabilities.len(),
    };

    assert_eq!(stats.total_services, 5);
    assert_eq!(stats.healthy_services, 5);
}

#[test]
fn test_capability_provider_mapping() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    // Map capabilities to providers
    registry
        .capability_providers
        .insert("storage".to_string(), vec!["s3-service".to_string(), "local-storage".to_string()]);

    registry.capability_providers.insert(
        "compute".to_string(),
        vec!["docker-service".to_string(), "k8s-service".to_string()],
    );

    // Verify mappings
    assert_eq!(registry.capability_providers.len(), 2);
    assert_eq!(
        registry
            .capability_providers
            .get("storage")
            .ok_or_else(|| SongbirdError::configuration("Capability not found".to_string()))?
            .len(),
        2
    );
    assert_eq!(
        registry
            .capability_providers
            .get("compute")
            .ok_or_else(|| SongbirdError::configuration("Capability not found".to_string()))?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn test_service_discovery_workflow() {
    let mut registry = CapabilityRegistry::default();
    let now = chrono::Utc::now();

    // Discover services
    let services = vec!["service-1", "service-2", "service-3"];
    for service in services {
        registry.service_capabilities.insert(service.to_string(), vec![]);
        registry.last_updated.insert(service.to_string(), now);
    }

    // Register capabilities
    registry
        .capability_providers
        .insert("networking".to_string(), vec!["service-1".to_string(), "service-2".to_string()]);

    // Verify discovery
    assert_eq!(registry.service_capabilities.len(), 3);
    assert!(registry.capability_providers.contains_key("networking"));
}

#[test]
fn test_capability_index_operations() {
    let mut registry = CapabilityRegistry::default();

    // Add service with capabilities
    registry.service_capabilities.insert("service-1".to_string(), vec![]);

    // Add capability provider
    registry.capability_providers.insert("ai".to_string(), vec!["service-1".to_string()]);

    // Update timestamp
    registry.last_updated.insert("service-1".to_string(), chrono::Utc::now());

    // Verify all operations completed
    assert!(registry.service_capabilities.contains_key("service-1"));
    assert!(registry.capability_providers.contains_key("ai"));
    assert!(registry.last_updated.contains_key("service-1"));
}

#[test]
fn test_empty_registry_operations() {
    let registry = CapabilityRegistry::default();

    // Verify empty state
    assert!(registry.service_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.service_info.is_empty());
    assert!(registry.last_updated.is_empty());

    // Verify lookups return None/empty
    assert!(!registry.service_capabilities.contains_key("nonexistent"));
    assert!(!registry.capability_providers.contains_key("nonexistent"));
}

#[test]
fn test_registry_timestamp_tracking() {
    let mut registry = CapabilityRegistry::default();

    let timestamp1 = chrono::Utc::now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let timestamp2 = chrono::Utc::now();

    registry.last_updated.insert("service-1".to_string(), timestamp1);
    registry.last_updated.insert("service-2".to_string(), timestamp2);

    // Verify timestamps are different
    assert!(timestamp2 > timestamp1);
    assert_ne!(registry.last_updated.get("service-1"), registry.last_updated.get("service-2"));
}

#[test]
fn test_capability_provider_updates() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    // Initial provider list
    registry.capability_providers.insert("database".to_string(), vec!["postgres".to_string()]);

    // Update with additional provider
    if let Some(providers) = registry.capability_providers.get_mut("database") {
        providers.push("mysql".to_string());
    }

    assert_eq!(
        registry
            .capability_providers
            .get("database")
            .ok_or_else(|| SongbirdError::configuration("Capability not found".to_string()))?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn test_multi_capability_service() -> SongbirdResult<()> {
    let mut registry = CapabilityRegistry::default();

    let service_id = "multi-service";

    // Register service with multiple capabilities
    let capabilities = vec!["compute", "storage", "network"];
    for capability in &capabilities {
        registry
            .capability_providers
            .entry((*capability).to_string())
            .or_insert_with(Vec::new)
            .push(service_id.to_string());
    }

    // Verify service is registered for all capabilities
    for capability in capabilities {
        let providers = registry
            .capability_providers
            .get(capability)
            .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
        assert!(providers.contains(&service_id.to_string()));
    }
    Ok(())
}
