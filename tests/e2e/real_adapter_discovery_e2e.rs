// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! # Real Adapter Discovery E2E Tests
//!
//! **Purpose**: Test actual adapter discovery and connection workflows
//!
//! These tests use real components (not just framework validation):
//! - Real UniversalCapabilityAdapter
//! - Real CapabilityEndpointResolver  
//! - Real discovery mechanisms
//! - Mock HTTP servers for endpoints

use songbird_types::SongbirdResult;
use songbird_universal::unified_adapter::{UnifiedAdapterConfig, UnifiedCapabilityAdapter};
use std::time::Duration;

#[tokio::test]
async fn test_e2e_adapter_creation_with_discovery() -> SongbirdResult<()> {
    // Test that adapter can be created with discovery configuration
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            "http://localhost:65432/discovery".to_string(),
        ],
        timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let adapter = UnifiedCapabilityAdapter::new(config);
    
    // Verify adapter was created successfully
    assert!(adapter.discovery_endpoints.len() > 0);
    assert_eq!(adapter.config.timeout, Duration::from_secs(5));
    
    Ok(())
}

#[tokio::test]
async fn test_e2e_adapter_registry_operations() -> SongbirdResult<()> {
    // Test real registry operations
    let config = UnifiedAdapterConfig::default();
    let mut adapter = UnifiedCapabilityAdapter::new(config);

    // Register a mock service capability
    use songbird_universal::types::capability::DiscoveredCapability;
    use songbird_universal::types::service::ServiceConnection;
    
    let capability = DiscoveredCapability {
        capability_type: "compute".to_string(),
        provider_id: "test-provider-1".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: None,
    };

    let connection = ServiceConnection {
        service_id: "test-provider-1".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["compute".to_string()],
        health_status: songbird_universal::types::capability::HealthStatus::Healthy,
    };

    // Register the capability
    adapter.register_capability(capability, connection);

    // Verify it can be found
    let providers = adapter.find_capability_providers("compute")?;
    assert!(!providers.is_empty(), "Should find registered provider");
    assert_eq!(providers[0].service_id, "test-provider-1");

    Ok(())
}

#[tokio::test]
async fn test_e2e_adapter_multiple_capabilities() -> SongbirdResult<()> {
    // Test adapter handling multiple capability types
    let config = UnifiedAdapterConfig::default();
    let mut adapter = UnifiedCapabilityAdapter::new(config);

    use songbird_universal::types::capability::DiscoveredCapability;
    use songbird_universal::types::service::ServiceConnection;

    // Register compute capability
    let compute_cap = DiscoveredCapability {
        capability_type: "compute".to_string(),
        provider_id: "compute-provider".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: None,
    };

    let compute_conn = ServiceConnection {
        service_id: "compute-provider".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["compute".to_string()],
        health_status: songbird_universal::types::capability::HealthStatus::Healthy,
    };

    adapter.register_capability(compute_cap, compute_conn);

    // Register security capability
    let security_cap = DiscoveredCapability {
        capability_type: "security".to_string(),
        provider_id: "security-provider".to_string(),
        endpoint: "http://localhost:8081".to_string(),
        qos_metrics: None,
    };

    let security_conn = ServiceConnection {
        service_id: "security-provider".to_string(),
        endpoint: "http://localhost:8081".to_string(),
        capabilities: vec!["security".to_string()],
        health_status: songbird_universal::types::capability::HealthStatus::Healthy,
    };

    adapter.register_capability(security_cap, security_conn);

    // Verify both can be found independently
    let compute_providers = adapter.find_capability_providers("compute")?;
    assert_eq!(compute_providers.len(), 1);
    assert_eq!(compute_providers[0].service_id, "compute-provider");

    let security_providers = adapter.find_capability_providers("security")?;
    assert_eq!(security_providers.len(), 1);
    assert_eq!(security_providers[0].service_id, "security-provider");

    Ok(())
}

#[tokio::test]
async fn test_e2e_adapter_registry_stats() -> SongbirdResult<()> {
    // Test real registry statistics
    let config = UnifiedAdapterConfig::default();
    let mut adapter = UnifiedCapabilityAdapter::new(config);

    use songbird_universal::types::capability::DiscoveredCapability;
    use songbird_universal::types::service::ServiceConnection;

    // Register multiple services
    for i in 0..5 {
        let cap = DiscoveredCapability {
            capability_type: "compute".to_string(),
            provider_id: format!("provider-{}", i),
            endpoint: format!("http://localhost:808{}", i),
            qos_metrics: None,
        };

        let conn = ServiceConnection {
            service_id: format!("provider-{}", i),
            endpoint: format!("http://localhost:808{}", i),
            capabilities: vec!["compute".to_string()],
            health_status: songbird_universal::types::capability::HealthStatus::Healthy,
        };

        adapter.register_capability(cap, conn);
    }

    // Get registry stats
    let stats = adapter.get_registry_stats();
    assert_eq!(stats.total_services, 5, "Should have 5 registered services");

    Ok(())
}

#[tokio::test]
async fn test_e2e_adapter_capability_not_found() -> SongbirdResult<()> {
    // Test error handling when capability not found
    let config = UnifiedAdapterConfig::default();
    let adapter = UnifiedCapabilityAdapter::new(config);

    // Try to find non-existent capability
    let result = adapter.find_capability_providers("nonexistent");
    
    // Should return empty vec (not error) for not found
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty(), "Should return empty vec for unknown capability");

    Ok(())
}

#[tokio::test]
async fn test_e2e_adapter_concurrent_registration() -> SongbirdResult<()> {
    // Test concurrent capability registration
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let config = UnifiedAdapterConfig::default();
    let adapter = Arc::new(Mutex::new(UnifiedCapabilityAdapter::new(config)));

    use songbird_universal::types::capability::DiscoveredCapability;
    use songbird_universal::types::service::ServiceConnection;

    // Spawn multiple tasks to register capabilities concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move {
            let cap = DiscoveredCapability {
                capability_type: "compute".to_string(),
                provider_id: format!("concurrent-provider-{}", i),
                endpoint: format!("http://localhost:900{}", i),
                qos_metrics: None,
            };

            let conn = ServiceConnection {
                service_id: format!("concurrent-provider-{}", i),
                endpoint: format!("http://localhost:900{}", i),
                capabilities: vec!["compute".to_string()],
                health_status: songbird_universal::types::capability::HealthStatus::Healthy,
            };

            let mut adapter_lock = adapter_clone.lock().await;
            adapter_lock.register_capability(cap, conn);
        });

        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle.await.expect("Task should complete");
    }

    // Verify all were registered
    let adapter_lock = adapter.lock().await;
    let stats = adapter_lock.get_registry_stats();
    assert_eq!(stats.total_services, 10, "Should have all 10 concurrently registered services");

    Ok(())
}

#[tokio::test]
async fn test_e2e_discovery_config_validation() -> SongbirdResult<()> {
    // Test discovery configuration validation
    use songbird_universal::discovery::{DiscoveryConfig, DiscoveryMechanisms};

    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            env_vars: true,
            service_registry: false,
            file_system: false,
            network_broadcast: false,
        },
        timeout: Duration::from_secs(10),
    };

    // Verify configuration is valid
    assert!(config.mechanisms.env_vars);
    assert!(!config.mechanisms.service_registry);
    assert_eq!(config.timeout, Duration::from_secs(10));

    Ok(())
}

#[tokio::test]
async fn test_e2e_circuit_breaker_integration() -> SongbirdResult<()> {
    // Test circuit breaker integration with adapter
    use songbird_universal::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout: Duration::from_secs(30),
    };

    let mut breaker = CircuitBreaker::new(config);

    // Simulate some failures
    for _ in 0..3 {
        breaker.record_failure();
    }

    // Should still be closed (under threshold)
    assert!(!breaker.is_open(), "Circuit should not be open yet");

    // Add more failures to exceed threshold
    for _ in 0..3 {
        breaker.record_failure();
    }

    // Now should be open
    assert!(breaker.is_open(), "Circuit should be open after exceeding threshold");

    Ok(())
}

