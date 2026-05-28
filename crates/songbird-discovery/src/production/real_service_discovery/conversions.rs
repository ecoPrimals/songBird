// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::traits::service::{ServiceEndpoint, ServiceInfo, ServiceStatus};
    use chrono::Utc;
    use std::collections::HashMap;

    fn sample_instance(
        id: &str,
        name: &str,
        endpoint: &str,
        capabilities: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> ServiceInstance {
        ServiceInstance {
            id: id.to_string(),
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            capabilities,
            health_status: "unknown".to_string(),
            metadata,
        }
    }

    fn sample_service_info(
        service_id: &str,
        name: &str,
        host: &str,
        port: u16,
        endpoint_path: &str,
        tags: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> ServiceInfo {
        ServiceInfo {
            service_id: service_id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: None,
            endpoints: vec![ServiceEndpoint {
                path: endpoint_path.to_string(),
                method: "GET".to_string(),
                description: None,
                parameters: Vec::new(),
                response_schema: None,
                auth_required: false,
                rate_limit: None,
            }],
            health_check_endpoint: None,
            metadata,
            tags,
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: service_id.to_string(),
            host: host.to_string(),
            port,
        }
    }

    #[test]
    fn instance_to_service_info_preserves_id_and_name() {
        let instance =
            sample_instance("svc-1", "Alpha", "http://127.0.0.1:8080", vec![], HashMap::new());

        let info = instance_to_service_info(&instance);

        assert_eq!(info.service_id, "svc-1");
        assert_eq!(info.instance_id, "svc-1");
        assert_eq!(info.name, "Alpha");
    }

    #[test]
    fn instance_to_service_info_reads_version_from_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".into(), "2.3.4".into());
        let instance =
            sample_instance("svc-v", "Versioned", "http://127.0.0.1:1", vec![], metadata);

        let info = instance_to_service_info(&instance);

        assert_eq!(info.version, "2.3.4");
    }

    #[test]
    fn instance_to_service_info_defaults_version_when_missing() {
        let instance =
            sample_instance("svc-d", "DefaultVer", "http://127.0.0.1:2", vec![], HashMap::new());

        let info = instance_to_service_info(&instance);

        assert_eq!(info.version, "0.0.0");
    }

    #[test]
    fn instance_to_service_info_reads_type_from_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("type".into(), "compute".into());
        let instance = sample_instance("svc-t", "Typed", "http://127.0.0.1:3", vec![], metadata);

        let info = instance_to_service_info(&instance);

        assert_eq!(info.service_type, "compute");
    }

    #[test]
    fn instance_to_service_info_defaults_type_when_missing() {
        let instance =
            sample_instance("svc-u", "Untyped", "http://127.0.0.1:4", vec![], HashMap::new());

        let info = instance_to_service_info(&instance);

        assert_eq!(info.service_type, "unknown");
    }

    #[test]
    fn instance_to_service_info_builds_health_check_endpoint() {
        let instance =
            sample_instance("svc-h", "Health", "http://127.0.0.1:8080/api", vec![], HashMap::new());

        let info = instance_to_service_info(&instance);

        assert_eq!(info.health_check_endpoint.as_deref(), Some("http://127.0.0.1:8080/api/health"));
    }

    #[test]
    fn instance_to_service_info_trims_trailing_slash_for_health_check() {
        let instance =
            sample_instance("svc-s", "Slash", "http://127.0.0.1:8080/api/", vec![], HashMap::new());

        let info = instance_to_service_info(&instance);

        assert_eq!(info.health_check_endpoint.as_deref(), Some("http://127.0.0.1:8080/api/health"));
    }

    #[test]
    fn instance_to_service_info_maps_capabilities_to_tags() {
        let instance = sample_instance(
            "svc-c",
            "Caps",
            "http://127.0.0.1:5",
            vec!["security".into(), "metrics".into()],
            HashMap::new(),
        );

        let info = instance_to_service_info(&instance);

        assert_eq!(info.tags, vec!["security", "metrics"]);
    }

    #[test]
    fn service_info_to_instance_uses_first_endpoint_path() {
        let info = sample_service_info(
            "svc-e",
            "Endpoint",
            "ignored-host",
            9999,
            "http://127.0.0.1:9000/v1",
            vec![],
            HashMap::new(),
        );

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.endpoint, "http://127.0.0.1:9000/v1");
    }

    #[test]
    fn service_info_to_instance_builds_from_host_and_port_when_path_empty() {
        let info =
            sample_service_info("svc-p", "PortHost", "127.0.0.1", 4242, "", vec![], HashMap::new());

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.endpoint, "http://127.0.0.1:4242");
    }

    #[test]
    fn service_info_to_instance_uses_http_host_as_is() {
        let info = sample_service_info(
            "svc-http",
            "HttpHost",
            "http://example.com:8080",
            8080,
            "",
            vec![],
            HashMap::new(),
        );

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.endpoint, "http://example.com:8080");
    }

    #[test]
    fn service_info_to_instance_uses_https_host_as_is() {
        let info = sample_service_info(
            "svc-https",
            "HttpsHost",
            "https://secure.example.com",
            443,
            "",
            vec![],
            HashMap::new(),
        );

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.endpoint, "https://secure.example.com");
    }

    #[test]
    fn service_info_to_instance_maps_tags_to_capabilities() {
        let info = sample_service_info(
            "svc-tags",
            "Tagged",
            "127.0.0.1",
            80,
            "http://127.0.0.1:80",
            vec!["ai".into(), "storage".into()],
            HashMap::new(),
        );

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.capabilities, vec!["ai", "storage"]);
    }

    #[test]
    fn service_info_to_instance_converts_string_metadata_values() {
        let mut metadata = HashMap::new();
        metadata.insert("region".into(), serde_json::Value::String("us-west".into()));
        metadata.insert("tier".into(), serde_json::Value::String("prod".into()));

        let info = sample_service_info(
            "svc-meta",
            "Meta",
            "127.0.0.1",
            80,
            "http://127.0.0.1:80",
            vec![],
            metadata,
        );

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.metadata.get("region").map(String::as_str), Some("us-west"));
        assert_eq!(instance.metadata.get("tier").map(String::as_str), Some("prod"));
    }

    #[test]
    fn service_info_to_instance_non_string_metadata_becomes_empty_string() {
        let mut metadata = HashMap::new();
        metadata.insert("count".into(), serde_json::json!(42));

        let info = sample_service_info(
            "svc-num",
            "NumericMeta",
            "127.0.0.1",
            80,
            "http://127.0.0.1:80",
            vec![],
            metadata,
        );

        let instance = service_info_to_instance(&info);

        assert_eq!(instance.metadata.get("count").map(String::as_str), Some(""));
    }
}
