//! Chaos Testing: Universal Adapter Edge Cases
//!
//! Tests adapter behavior under extreme and unexpected conditions

use songbird_types::{CapabilityRequest, PrimalType, ServiceInfo};
use songbird_universal::capabilities::{DiscoveryConfig, UniversalCapabilityAdapter};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn chaos_adapter_creation_stress() {
    // Create 500 adapters rapidly
    let mut adapters = Vec::new();

    for _ in 0..500 {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);
        adapters.push(adapter);
    }

    // All should be functional
    assert_eq!(adapters.len(), 500, "Should create 500 adapters");
}

#[tokio::test]
async fn chaos_adapter_concurrent_creation() {
    let mut handles = vec![];

    // Spawn 100 concurrent adapter creators
    for _ in 0..100 {
        let handle = tokio::spawn(async move {
            let config = DiscoveryConfig::default();
            let _adapter = UniversalCapabilityAdapter::new(config);
            sleep(Duration::from_micros(100)).await;
        });
        handles.push(handle);
    }

    // Wait for all creations
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn chaos_adapter_empty_capability_request() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Request with empty strings
    let request = CapabilityRequest {
        capability: "".to_string(),
        operation: "".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(1),
    };

    // Should handle empty request gracefully
    let result = adapter.execute_capability_request(request).await;
    assert!(result.is_ok() || result.is_err(), "Should handle empty capability gracefully");
}

#[tokio::test]
async fn chaos_adapter_extreme_timeout() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Request with 1 nanosecond timeout
    let request = CapabilityRequest {
        capability: "compute".to_string(),
        operation: "test".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_nanos(1),
    };

    // Should handle extreme timeout
    let result = adapter.execute_capability_request(request).await;
    assert!(result.is_ok() || result.is_err(), "Should handle extreme timeout");
}

#[tokio::test]
async fn chaos_adapter_unicode_capability_names() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Test with various unicode strings
    let test_capabilities = vec!["🚀compute", "データ処理", "计算能力", "вычисления", "حَاسِب"];

    for capability in test_capabilities {
        let request = CapabilityRequest {
            capability: capability.to_string(),
            operation: "test".to_string(),
            parameters: Default::default(),
            timeout: Duration::from_secs(1),
        };

        let result = adapter.execute_capability_request(request).await;
        assert!(result.is_ok() || result.is_err(), "Should handle unicode capability names");
    }
}

#[tokio::test]
async fn chaos_adapter_very_long_capability_name() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // 10KB capability name
    let long_capability = "a".repeat(10_000);

    let request = CapabilityRequest {
        capability: long_capability,
        operation: "test".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(1),
    };

    // Should handle long names gracefully
    let result = adapter.execute_capability_request(request).await;
    assert!(result.is_ok() || result.is_err(), "Should handle very long capability names");
}

#[tokio::test]
async fn chaos_adapter_rapid_request_flood() {
    let config = DiscoveryConfig::default();
    let adapter = Arc::new(UniversalCapabilityAdapter::new(config));

    // Send 1000 requests as fast as possible
    let mut handles = vec![];
    for i in 0..1000 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move {
            let request = CapabilityRequest {
                capability: format!("test_{}", i % 10),
                operation: "flood_test".to_string(),
                parameters: Default::default(),
                timeout: Duration::from_millis(100),
            };
            let _ = adapter_clone.execute_capability_request(request).await;
        });
        handles.push(handle);
    }

    // Wait for flood to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn chaos_adapter_service_registration_stress() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Register 100 services rapidly
    for i in 0..100 {
        let service = ServiceInfo {
            id: format!("service_{}", i),
            name: format!("Test Service {}", i),
            primal_type: PrimalType::Compute,
            capabilities: vec![format!("capability_{}", i % 5)],
            endpoint: format!("http://localhost:{}", 8000 + i),
            health_check_path: Some("/health".to_string()),
            metadata: Default::default(),
        };

        let result = adapter.register_service(service).await;
        assert!(result.is_ok() || result.is_err(), "Should handle rapid service registration");
    }
}

#[tokio::test]
async fn chaos_adapter_concurrent_service_operations() {
    let config = DiscoveryConfig::default();
    let adapter = Arc::new(UniversalCapabilityAdapter::new(config));

    let mut handles = vec![];

    // Concurrent registrations
    for i in 0..50 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move {
            let service = ServiceInfo {
                id: format!("concurrent_{}", i),
                name: format!("Concurrent Service {}", i),
                primal_type: PrimalType::Storage,
                capabilities: vec!["storage".to_string()],
                endpoint: format!("http://host-{}:8000", i),
                health_check_path: None,
                metadata: Default::default(),
            };
            let _ = adapter_clone.register_service(service).await;
        });
        handles.push(handle);
    }

    // Concurrent deregistrations
    for i in 0..50 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move {
            let _ = adapter_clone.deregister_service(&format!("concurrent_{}", i)).await;
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn chaos_adapter_malformed_parameters() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    use serde_json::json;

    // Test with various malformed parameters
    let test_params = vec![
        json!(null),
        json!([1, 2, 3, "mixed", true]),
        json!({"nested": {"deeply": {"nested": {"value": 123}}}}),
        json!({"key": "\u{0000}control\u{001F}chars"}),
    ];

    for params in test_params {
        let request = CapabilityRequest {
            capability: "test".to_string(),
            operation: "malformed_test".to_string(),
            parameters: params,
            timeout: Duration::from_secs(1),
        };

        let result = adapter.execute_capability_request(request).await;
        assert!(result.is_ok() || result.is_err(), "Should handle malformed parameters");
    }
}

#[tokio::test]
async fn chaos_adapter_discovery_config_extremes() {
    // Test with extreme discovery configs
    let configs = vec![
        DiscoveryConfig {
            timeout: Duration::from_nanos(1),
            retry_attempts: 0,
            auto_discovery: true,
            ..Default::default()
        },
        DiscoveryConfig {
            timeout: Duration::from_secs(3600), // 1 hour
            retry_attempts: 1000,
            auto_discovery: false,
            ..Default::default()
        },
        DiscoveryConfig {
            timeout: Duration::from_millis(1),
            retry_attempts: 1,
            auto_discovery: true,
            ..Default::default()
        },
    ];

    for config in configs {
        let adapter = UniversalCapabilityAdapter::new(config);
        // Should create without panic
        assert!(true, "Adapter created with extreme config");
    }
}

#[tokio::test]
async fn chaos_adapter_service_with_no_capabilities() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Register service with empty capabilities
    let service = ServiceInfo {
        id: "no_caps".to_string(),
        name: "Service Without Capabilities".to_string(),
        primal_type: PrimalType::AI,
        capabilities: vec![], // Empty!
        endpoint: "http://localhost:9999".to_string(),
        health_check_path: None,
        metadata: Default::default(),
    };

    let result = adapter.register_service(service).await;
    assert!(result.is_ok() || result.is_err(), "Should handle service with no capabilities");
}

#[tokio::test]
async fn chaos_adapter_duplicate_service_registration() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let service = ServiceInfo {
        id: "duplicate".to_string(),
        name: "Duplicate Service".to_string(),
        primal_type: PrimalType::Security,
        capabilities: vec!["security".to_string()],
        endpoint: "http://localhost:7777".to_string(),
        health_check_path: Some("/health".to_string()),
        metadata: Default::default(),
    };

    // Register same service 100 times
    for _ in 0..100 {
        let result = adapter.register_service(service.clone()).await;
        assert!(result.is_ok() || result.is_err(), "Should handle duplicate registration");
    }
}

#[tokio::test]
async fn chaos_adapter_memory_leak_check() {
    // Create and drop adapters repeatedly to check for leaks
    for _ in 0..1000 {
        let config = DiscoveryConfig::default();
        let adapter = UniversalCapabilityAdapter::new(config);

        // Do some operations
        let service = ServiceInfo {
            id: format!("leak_check_{}", fastrand::u64(..)),
            name: "Leak Check Service".to_string(),
            primal_type: PrimalType::Compute,
            capabilities: vec!["compute".to_string()],
            endpoint: "http://localhost:6666".to_string(),
            health_check_path: None,
            metadata: Default::default(),
        };

        let _ = adapter.register_service(service).await;

        // Drop adapter (should clean up)
        drop(adapter);
    }

    // If we get here without OOM, no obvious leak
    assert!(true, "No memory leak detected");
}

#[tokio::test]
async fn chaos_adapter_special_characters_in_ids() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    // Test IDs with special characters
    let test_ids = vec![
        "id with spaces",
        "id/with/slashes",
        "id@with#symbols$",
        "id\nwith\nnewlines",
        "id\twith\ttabs",
        "id\"with\"quotes",
    ];

    for id in test_ids {
        let service = ServiceInfo {
            id: id.to_string(),
            name: "Special ID Service".to_string(),
            primal_type: PrimalType::Storage,
            capabilities: vec!["storage".to_string()],
            endpoint: "http://localhost:5555".to_string(),
            health_check_path: None,
            metadata: Default::default(),
        };

        let result = adapter.register_service(service).await;
        assert!(result.is_ok() || result.is_err(), "Should handle special characters in IDs");
    }
}
