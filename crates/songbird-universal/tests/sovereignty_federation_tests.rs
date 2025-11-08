//! Tests for Sovereignty Federation Manager
//!
//! Comprehensive tests for the sovereignty-aware federation system

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::sovereignty::federation::SovereigntyFederationManager;
use songbird_universal::sovereignty::types::FederationCapability;
use songbird_universal::types::UniversalRequest;
use std::collections::HashMap;

#[test]
fn test_federation_manager_new() {
    let manager = SovereigntyFederationManager::new();

    assert!(manager.federation_capabilities.is_empty());
    assert!(manager.network_effects.is_empty());
}

#[test]
fn test_federation_manager_default() {
    let manager = SovereigntyFederationManager::default();

    assert!(manager.federation_capabilities.is_empty());
    assert!(manager.network_effects.is_empty());
}

#[test]
fn test_register_capability() {
    let mut manager = SovereigntyFederationManager::new();

    let capability = FederationCapability {
        capability_id: "cap-001".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::CrossNodeCommunication,
        availability_score: 0.95,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 10.0,
                throughput_ops_per_sec: 1000.0,
                reliability_score: 0.99,
            },
    };

    manager.register_capability(capability);

    assert_eq!(manager.federation_capabilities.len(), 1);
}

#[test]
fn test_register_multiple_capabilities() {
    let mut manager = SovereigntyFederationManager::new();

    let cap1 = FederationCapability {
        capability_id: "cap-001".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::CrossNodeCommunication,
        availability_score: 0.95,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 10.0,
                throughput_ops_per_sec: 1000.0,
                reliability_score: 0.99,
            },
    };

    let cap2 = FederationCapability {
        capability_id: "cap-002".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::ConsensusParticipation,
        availability_score: 0.90,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 20.0,
                throughput_ops_per_sec: 800.0,
                reliability_score: 0.95,
            },
    };

    manager.register_capability(cap1);
    manager.register_capability(cap2);

    assert_eq!(manager.federation_capabilities.len(), 2);
}

#[test]
fn test_get_capabilities_empty() {
    let manager = SovereigntyFederationManager::new();

    let capabilities = manager.get_capabilities();

    assert!(capabilities.is_empty());
}

#[test]
fn test_get_capabilities_non_empty() {
    let mut manager = SovereigntyFederationManager::new();

    let capability = FederationCapability {
        capability_id: "cap-001".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::DataReplication,
        availability_score: 0.98,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 15.0,
                throughput_ops_per_sec: 900.0,
                reliability_score: 0.97,
            },
    };

    manager.register_capability(capability);

    let capabilities = manager.get_capabilities();
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0].capability_id, "cap-001");
}

#[tokio::test]
async fn test_coordinate_request_success() -> SongbirdResult<()> {
    let manager = SovereigntyFederationManager::new();

    let request = UniversalRequest {
        request_id: "req-001".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test-action".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result = manager.coordinate_request(&request).await;

    assert!(result.is_ok());
    let response = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(response.request_id, "req-001");
    Ok(())
}

#[tokio::test]
async fn test_coordinate_request_returns_success_status() -> SongbirdResult<()> {
    let manager = SovereigntyFederationManager::new();

    let request = UniversalRequest {
        request_id: "req-002".to_string(),
        source: "federation-test".to_string(),
        target: "federation-coordinator".to_string(),
        action: "coordinate".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result = manager.coordinate_request(&request).await;

    assert!(result.is_ok());
    let response = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert!(matches!(response.status, songbird_universal::types::ResponseStatus::Success));
    Ok(())
}

#[tokio::test]
async fn test_coordinate_request_with_capabilities() {
    let mut manager = SovereigntyFederationManager::new();

    // Register capabilities first
    let capability = FederationCapability {
        capability_id: "cap-003".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::LoadDistribution,
        availability_score: 0.99,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 5.0,
                throughput_ops_per_sec: 2000.0,
                reliability_score: 0.999,
            },
    };
    manager.register_capability(capability);

    let request = UniversalRequest {
        request_id: "req-003".to_string(),
        source: "client".to_string(),
        target: "lb-service".to_string(),
        action: "balance".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result = manager.coordinate_request(&request).await;

    assert!(result.is_ok());
}

#[test]
fn test_federation_manager_debug() -> SongbirdResult<()> {
    let manager = SovereigntyFederationManager::new();
    let debug_str = format!("{manager:?}");

    assert!(debug_str.contains("SovereigntyFederationManager"));
    Ok(())
}

#[test]
fn test_capability_registration_preserves_order() {
    let mut manager = SovereigntyFederationManager::new();

    let cap1 = FederationCapability {
        capability_id: "cap-first".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::CrossNodeCommunication,
        availability_score: 0.95,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 10.0,
                throughput_ops_per_sec: 1000.0,
                reliability_score: 0.99,
            },
    };

    let cap2 = FederationCapability {
        capability_id: "cap-second".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::ConsensusParticipation,
        availability_score: 0.90,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 20.0,
                throughput_ops_per_sec: 800.0,
                reliability_score: 0.95,
            },
    };

    manager.register_capability(cap1);
    manager.register_capability(cap2);

    let capabilities = manager.get_capabilities();
    assert_eq!(capabilities[0].capability_id, "cap-first");
    assert_eq!(capabilities[1].capability_id, "cap-second");
}

#[test]
fn test_capability_with_metadata() {
    let mut manager = SovereigntyFederationManager::new();

    let capability = FederationCapability {
        capability_id: "cap-meta".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::DataReplication,
        availability_score: 0.92,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 25.0,
                throughput_ops_per_sec: 750.0,
                reliability_score: 0.94,
            },
    };

    manager.register_capability(capability);

    let capabilities = manager.get_capabilities();
    assert!((capabilities[0].availability_score - 0.92).abs() < 0.001);
}

#[tokio::test]
async fn test_coordinate_request_with_custom_metadata() -> SongbirdResult<()> {
    let manager = SovereigntyFederationManager::new();

    let mut parameters = HashMap::new();
    parameters.insert("custom_key".to_string(), serde_json::json!("custom_value"));

    let request = UniversalRequest {
        request_id: "req-meta".to_string(),
        source: "meta-source".to_string(),
        target: "meta-target".to_string(),
        action: "test-op".to_string(),
        parameters,
        security_context: None,
    };

    let result = manager.coordinate_request(&request).await;

    assert!(result.is_ok());
    let response = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(response.request_id, "req-meta");
    Ok(())
}

#[test]
fn test_multiple_capabilities_same_type() {
    let mut manager = SovereigntyFederationManager::new();

    let cap1 = FederationCapability {
        capability_id: "cap-comm-1".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::CrossNodeCommunication,
        availability_score: 0.96,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 8.0,
                throughput_ops_per_sec: 1200.0,
                reliability_score: 0.98,
            },
    };

    let cap2 = FederationCapability {
        capability_id: "cap-comm-2".to_string(),
        capability_type:
            songbird_universal::sovereignty::types::FederationCapabilityType::CrossNodeCommunication,
        availability_score: 0.94,
        performance_characteristics:
            songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: 12.0,
                throughput_ops_per_sec: 1100.0,
                reliability_score: 0.96,
            },
    };

    manager.register_capability(cap1);
    manager.register_capability(cap2);

    let capabilities = manager.get_capabilities();
    assert_eq!(capabilities.len(), 2);
    // Both have same type but different IDs
    assert_eq!(capabilities[0].capability_id, "cap-comm-1");
    assert_eq!(capabilities[1].capability_id, "cap-comm-2");
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn test_capabilities_different_sovereignty_levels() {
    let mut manager = SovereigntyFederationManager::new();

    let availability_scores = vec![0.95, 0.90, 0.85];

    for (i, score) in availability_scores.into_iter().enumerate() {
        let cap = FederationCapability {
            capability_id: format!("cap-{i}"),
            capability_type: songbird_universal::sovereignty::types::FederationCapabilityType::CrossNodeCommunication,
            availability_score: score,
            performance_characteristics: songbird_universal::sovereignty::types::PerformanceCharacteristics {
                latency_ms: (i as f64).mul_add(5.0, 10.0),
                throughput_ops_per_sec: (i as f64).mul_add(-100.0, 1000.0),
                reliability_score: (i as f64).mul_add(-0.02, 0.99),
            },
        };
        manager.register_capability(cap);
    }

    assert_eq!(manager.get_capabilities().len(), 3);
}

#[tokio::test]
async fn test_coordinate_request_response_contains_data() -> SongbirdResult<()> {
    let manager = SovereigntyFederationManager::new();

    let request = UniversalRequest {
        request_id: "req-data".to_string(),
        source: "data-test".to_string(),
        target: "data-processor".to_string(),
        action: "process".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result = manager.coordinate_request(&request).await;

    assert!(result.is_ok());
    let response = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert!(response.data.is_some());
    assert!(response.error.is_none());
    Ok(())
}
