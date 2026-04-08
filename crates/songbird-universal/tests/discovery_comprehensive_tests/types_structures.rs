// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::common::create_test_service;
use songbird_test_utils::test_orchestrator_port;
use songbird_universal::types::{DiscoveredCapability, HealthStatus, QosMetrics};

#[tokio::test]
async fn test_service_info_structure() {
    // Verify ServiceInfo can be created with all required fields

    // ARRANGE & ACT: Create a complete ServiceInfo
    let endpoint = format!("http://localhost:{}", test_orchestrator_port());
    let service = create_test_service("test-service", &endpoint, vec!["compute", "storage"]);

    // ASSERT: All fields are properly set
    assert_eq!(service.name, "test-service");
    assert_eq!(service.endpoint, endpoint);
    assert_eq!(service.capabilities.len(), 2);
    assert!(matches!(service.health, HealthStatus::Healthy));
}

#[tokio::test]
async fn test_capability_structure() {
    // Verify Capability structure contains all required information

    // ARRANGE & ACT: Create a capability
    let capability = DiscoveredCapability {
        name: "compute".to_string(),
        version: "1.0.0".to_string(),
        description: "Compute capability".to_string(),
        provider: "test-provider".to_string(),
        endpoint: "http://localhost:8080/compute".to_string(),
        qos_metrics: QosMetrics {
            latency_ms: Some(50.0),
            throughput_ops_sec: Some(1000.0),
            availability: Some(0.99),
            reliability: Some(0.99),
        },
        health_status: HealthStatus::Healthy,
    };

    // ASSERT: All fields accessible and correct
    assert_eq!(capability.name, "compute");
    assert_eq!(capability.version, "1.0.0");
    assert_eq!(capability.provider, "test-provider");
    assert!(matches!(capability.health_status, HealthStatus::Healthy));
}
