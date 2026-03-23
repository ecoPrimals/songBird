// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Mapping between internal [`ServiceInstance`](crate::discovery::core::ServiceInstance) and
//! the [`ServiceDiscovery`](crate::traits::ServiceDiscovery) trait's `ServiceInfo`.

use crate::discovery::core::ServiceInstance;

/// Convert internal `ServiceInstance` to the trait's `ServiceInfo`
pub(super) fn instance_to_service_info(instance: &ServiceInstance) -> crate::traits::ServiceInfo {
    use crate::traits::{ServiceEndpoint as TraitEndpoint, ServiceStatus};
    use chrono::Utc;

    let id = instance.id.clone();
    let endpoint = TraitEndpoint {
        path: instance.endpoint.clone(),
        method: "GET".to_string(),
        description: None,
        parameters: Vec::new(),
        response_schema: None,
        auth_required: false,
        rate_limit: None,
    };

    crate::traits::ServiceInfo {
        service_id: id.clone(),
        name: instance.name.clone(),
        version: instance.metadata.get("version").cloned().unwrap_or_else(|| "0.0.0".to_string()),
        service_type: instance
            .metadata
            .get("type")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        description: instance.metadata.get("description").cloned(),
        endpoints: vec![endpoint],
        health_check_endpoint: Some(format!("{}/health", instance.endpoint.trim_end_matches('/'))),
        metadata: instance
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect(),
        tags: instance.capabilities.clone(),
        dependencies: Vec::new(),
        status: ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        instance_id: id,
        host: instance.endpoint.clone(),
        port: 0,
    }
}

/// Convert the trait's `ServiceInfo` to internal `ServiceInstance`
pub(super) fn service_info_to_instance(info: &crate::traits::ServiceInfo) -> ServiceInstance {
    let endpoint =
        info.endpoints.first().map(|e| e.path.clone()).filter(|p| !p.is_empty()).unwrap_or_else(
            || {
                if info.host.starts_with("http://") || info.host.starts_with("https://") {
                    info.host.clone()
                } else {
                    format!("http://{}:{}", info.host, info.port)
                }
            },
        );

    ServiceInstance {
        id: info.service_id.clone(),
        name: info.name.clone(),
        endpoint,
        capabilities: info.tags.clone(),
        health_status: "unknown".to_string(),
        metadata: info
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect(),
    }
}
