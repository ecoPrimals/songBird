// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo,
};
use std::collections::HashMap;

/// Helper function to create a test service
pub fn create_test_service(name: &str, endpoint: &str, capabilities: Vec<&str>) -> ServiceInfo {
    ServiceInfo {
        name: name.to_string(),
        endpoint: endpoint.to_string(),
        primal_type: PrimalType {
            category: "test".to_string(),
            subcategory: None,
            version: "1.0".to_string(),
        },
        capabilities: capabilities
            .iter()
            .map(|c| DiscoveredCapability {
                name: (*c).to_string(),
                version: "1.0".to_string(),
                description: format!("{c} capability"),
                provider: name.to_string(),
                endpoint: format!("{endpoint}/api/v1/{c}"),
                qos_metrics: QosMetrics {
                    latency_ms: Some(100.0),
                    throughput_ops_sec: Some(1000.0),
                    availability: Some(0.99),
                    reliability: Some(0.99),
                },
                health_status: HealthStatus::Healthy,
            })
            .collect(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}
