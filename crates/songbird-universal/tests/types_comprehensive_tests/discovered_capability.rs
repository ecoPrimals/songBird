// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// DISCOVERED CAPABILITY TESTS
// ============================================================================

#[test]
fn test_discovered_capability_creation() {
    let cap = DiscoveredCapability {
        name: "encryption".to_string(),
        version: "2.0.0".to_string(),
        description: "AES encryption capability".to_string(),
        provider: "security-primal".to_string(),
        endpoint: "https://security.example.com".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    assert_eq!(cap.name, "encryption");
    assert_eq!(cap.version, "2.0.0");
    assert_eq!(cap.provider, "security-primal");
}

#[test]
fn test_discovered_capability_clone() {
    let cap1 = DiscoveredCapability {
        name: "test-cap".to_string(),
        version: "1.0.0".to_string(),
        description: "Test capability".to_string(),
        provider: "test-provider".to_string(),
        endpoint: format!("http://localhost:{}", test_orchestrator_port()),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let cap2 = cap1.clone();
    assert_eq!(cap1.name, cap2.name);
    assert_eq!(cap1.version, cap2.version);
}

#[test]
fn test_discovered_capability_serialization() -> SongbirdResult<()> {
    let cap = DiscoveredCapability {
        name: "storage".to_string(),
        version: "1.5.0".to_string(),
        description: "Object storage".to_string(),
        provider: "storage-primal".to_string(),
        endpoint: "https://storage.example.com".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let json = serde_json::to_string(&cap)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: DiscoveredCapability = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.name, cap.name);
    Ok(())
}
