// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;
use std::collections::HashMap;

// ============================================================================
// SERVICE INFO TESTS
// ============================================================================

#[test]
fn test_service_info_creation() {
    let service = ServiceInfo {
        name: "test-service".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: format!("http://localhost:{}", test_orchestrator_port()),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.name, "test-service");
    assert_eq!(service.health, HealthStatus::Healthy);
}

#[test]
fn test_service_info_with_capabilities() {
    let cap = DiscoveredCapability {
        name: "test-cap".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        provider: "test".to_string(),
        endpoint: "http://test".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let service = ServiceInfo {
        name: "service-with-cap".to_string(),
        primal_type: PrimalType::new("ai"),
        endpoint: "http://ai:9000".to_string(),
        capabilities: vec![cap],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.capabilities.len(), 1);
}

#[test]
fn test_service_info_serialization() -> SongbirdResult<()> {
    let service = ServiceInfo {
        name: "serialize-test".to_string(),
        primal_type: PrimalType::new("storage"),
        endpoint: format!("http://storage:{}", test_orchestrator_port()),
        capabilities: vec![],
        health: HealthStatus::Degraded,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&service)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: ServiceInfo = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.name, service.name);
    Ok(())
}
