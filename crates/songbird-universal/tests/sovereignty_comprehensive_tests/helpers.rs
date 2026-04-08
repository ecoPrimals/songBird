// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

use crate::imports::*;

/// Create a test service with specified sovereignty level
pub(crate) fn create_test_service_with_sovereignty(
    name: &str,
    endpoint: &str,
    sovereignty_level: SovereigntyLevel,
) -> ServiceInfo {
    let mut metadata = HashMap::new();
    metadata.insert("sovereignty_level".to_string(), format!("{sovereignty_level:?}"));

    ServiceInfo {
        name: name.to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: endpoint.to_string(),
        capabilities: vec![DiscoveredCapability {
            name: "test_capability".to_string(),
            version: "1.0".to_string(),
            description: "Test capability".to_string(),
            provider: name.to_string(),
            endpoint: format!("{endpoint}/api/v1/test"),
            qos_metrics: QosMetrics::default(),
            health_status: HealthStatus::Healthy,
        }],
        health: HealthStatus::Healthy,
        metadata,
    }
}

/// Create a test request
pub(crate) fn create_test_request() -> UniversalRequest {
    UniversalRequest {
        request_id: "test-req-001".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "process".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    }
}
