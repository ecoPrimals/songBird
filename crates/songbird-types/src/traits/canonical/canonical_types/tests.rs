// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use serde_json::Value;
use std::time::Duration;

fn t0() -> std::time::SystemTime {
    std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

fn roundtrip<T>(v: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let a: Value = serde_json::to_value(v).expect("serialize");
    let back: T = serde_json::from_value(a.clone()).expect("deserialize");
    assert_eq!(serde_json::to_value(&back).expect("serialize again"), a);
}

#[test]
fn provider_type_eq_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let a = ProviderType::Observability;
    let b = ProviderType::Custom("x".to_string());
    assert_eq!(a, a);
    assert_ne!(a, b);
    let mut h = DefaultHasher::new();
    a.hash(&mut h);
    assert_ne!(h.finish(), 0);
}

#[test]
fn primal_type_ordering() {
    assert!(PrimalType::AI < PrimalType::Network);
    assert_eq!(PrimalType::Custom("a".to_string()), PrimalType::Custom("a".to_string()));
}

#[test]
fn service_type_eq() {
    assert_eq!(ServiceType::Cache, ServiceType::Cache);
    assert_ne!(ServiceType::WebService, ServiceType::Database);
}

#[test]
fn health_status_eq() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
}

#[test]
fn discovery_criteria_and_query_default() {
    let c = DiscoveryCriteria::default();
    assert!(c.name.is_none());
    let q = DiscoveryQuery::default();
    assert!(!q.watch_changes);
}

#[test]
fn serde_roundtrip_provider_config_and_metadata() {
    let cfg = ProviderConfig {
        settings: std::collections::HashMap::from([("k".to_string(), serde_json::json!(1))]),
        enabled_features: vec!["f".to_string()],
        environment: "staging".to_string(),
    };
    roundtrip(&cfg);
    let meta = ProviderMetadata {
        description: "desc".to_string(),
        tags: vec!["a".to_string()],
        documentation_url: Some("https://x".to_string()),
        support_contact: None,
        created_at: t0(),
        updated_at: t0(),
    };
    roundtrip(&meta);
}

#[test]
fn serde_roundtrip_capability_and_parameter_spec() {
    let cap = Capability {
        name: "cap".to_string(),
        version: "1".to_string(),
        description: "d".to_string(),
        parameters: std::collections::HashMap::from([(
            "p".to_string(),
            ParameterSpec {
                parameter_type: "string".to_string(),
                required: false,
                description: "pd".to_string(),
                default_value: Some(serde_json::json!("v")),
            },
        )]),
    };
    roundtrip(&cap);
}

#[test]
fn serde_roundtrip_service_request_response() {
    let req = ServiceRequest {
        id: "1".to_string(),
        method: "GET".to_string(),
        path: "/".to_string(),
        headers: std::collections::HashMap::new(),
        body: serde_json::json!({}),
        timestamp: t0(),
    };
    roundtrip(&req);
    let res = ServiceResponse {
        id: "1".to_string(),
        status_code: 200,
        headers: std::collections::HashMap::new(),
        body: serde_json::json!({}),
        timestamp: t0(),
    };
    roundtrip(&res);
}

#[test]
fn serde_roundtrip_service_metrics_and_info() {
    let m = ServiceMetrics {
        request_count: 1,
        error_count: 0,
        average_response_time_ms: 1.0,
        uptime_seconds: 2,
        memory_usage_mb: 3.0,
        cpu_usage_percent: 4.0,
    };
    roundtrip(&m);
    let info = ServiceInfo {
        id: "id".to_string(),
        name: "n".to_string(),
        service_type: ServiceType::WebService,
        version: "v".to_string(),
        endpoints: vec![Endpoint {
            protocol: "https".to_string(),
            host: "h".to_string(),
            port: 443,
            path: None,
            metadata: std::collections::HashMap::new(),
        }],
        health: HealthStatus::Healthy,
        metadata: std::collections::HashMap::new(),
        tags: vec![],
        capabilities: vec![],
        last_updated: t0(),
    };
    roundtrip(&info);
}

#[test]
fn serde_roundtrip_primal_context_response_dependency() {
    let ctx = PrimalContext {
        user_id: "u".to_string(),
        device_id: "d".to_string(),
        environment: "e".to_string(),
        security_level: "low".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    roundtrip(&ctx);
    let pr = PrimalResponse {
        success: true,
        data: serde_json::json!({}),
        metadata: std::collections::HashMap::new(),
        execution_time_ms: 5,
    };
    roundtrip(&pr);
    let dep = PrimalDependency {
        service_name: "s".to_string(),
        required_version: "^1".to_string(),
        optional: false,
        capabilities: vec![],
    };
    roundtrip(&dep);
}

#[test]
fn serde_roundtrip_integration_and_primal_info() {
    let ir = IntegrationResult {
        success: true,
        shared_capabilities: vec!["c".to_string()],
        communication_channels: vec![],
        metadata: std::collections::HashMap::new(),
    };
    roundtrip(&ir);
    let pi = PrimalInfo {
        id: "i".to_string(),
        name: "n".to_string(),
        primal_type: PrimalType::Compute,
        version: "v".to_string(),
        capabilities: vec![],
        endpoints: vec![],
        health: HealthStatus::Degraded,
        metadata: std::collections::HashMap::new(),
    };
    roundtrip(&pi);
}

#[test]
fn serde_roundtrip_discovery_and_service_event() {
    roundtrip(&DiscoveryCriteria {
        name: Some("n".to_string()),
        service_type: Some(ServiceType::Cache),
        version: None,
        tags: vec![],
        capabilities: vec![],
        metadata: std::collections::HashMap::new(),
        health_status: Some(HealthStatus::Unknown),
        limit: Some(10),
    });
    roundtrip(&DiscoveryQuery {
        criteria: DiscoveryCriteria::default(),
        watch_changes: true,
        include_metadata: true,
    });
    roundtrip(&ServiceEvent::Unregistered {
        service_id: "s".to_string(),
    });
}

#[test]
fn serde_roundtrip_capability_metadata_credentials_tokens() {
    roundtrip(&CapabilityMetadata {
        name: "c".to_string(),
        description: "d".to_string(),
        version: "1".to_string(),
        parameters: std::collections::HashMap::new(),
        examples: vec![serde_json::json!([])],
    });
    roundtrip(&Credentials {
        credential_type: "api_key".to_string(),
        data: std::collections::HashMap::new(),
    });
    roundtrip(&AuthToken {
        token: "t".to_string(),
        token_type: "Bearer".to_string(),
        expires_at: Some(t0()),
        metadata: std::collections::HashMap::new(),
    });
    roundtrip(&TokenClaims {
        subject: "sub".to_string(),
        audience: vec!["a".to_string()],
        expires_at: None,
        custom_claims: std::collections::HashMap::new(),
    });
    roundtrip(&TokenValidation {
        valid: false,
        claims: None,
        error: Some("e".to_string()),
    });
}

#[test]
fn serde_roundtrip_deployment_spec_and_result() {
    roundtrip(&DeploymentSpec {
        name: "dep".to_string(),
        image: "img".to_string(),
        replicas: 2,
        resources: ResourceRequirements {
            cpu_limit: None,
            memory_limit: None,
            cpu_request: None,
            memory_request: None,
        },
        environment: std::collections::HashMap::new(),
        ports: vec![PortSpec {
            name: "http".to_string(),
            port: 80,
            target_port: 8080,
            protocol: "TCP".to_string(),
        }],
    });
    roundtrip(&DeploymentResult {
        deployment_id: "d".to_string(),
        status: DeploymentStatus::Running,
        endpoints: vec![],
        message: "ok".to_string(),
    });
    roundtrip(&DeploymentInfo {
        id: "d".to_string(),
        name: "n".to_string(),
        status: DeploymentStatus::Pending,
        replicas: 1,
        ready_replicas: 0,
        created_at: t0(),
        updated_at: t0(),
    });
}

#[test]
fn serde_roundtrip_deployment_status_and_span_metric_system_health() {
    roundtrip(&DeploymentStatus::Failed);
    roundtrip(&SpanContext {
        trace_id: "t".to_string(),
        span_id: "s".to_string(),
        parent_span_id: None,
        baggage: std::collections::HashMap::new(),
    });
    roundtrip(&MetricQuery {
        metric_name: "m".to_string(),
        start_time: t0(),
        end_time: t0() + Duration::from_secs(60),
        labels: std::collections::HashMap::new(),
        aggregation: Some("sum".to_string()),
    });
    roundtrip(&MetricResult {
        metric_name: "m".to_string(),
        timestamp: t0(),
        value: 1.0,
        labels: std::collections::HashMap::new(),
    });
    roundtrip(&SystemHealth {
        overall_status: HealthStatus::Unhealthy,
        components: std::collections::HashMap::from([("db".to_string(), HealthStatus::Healthy)]),
        metrics: std::collections::HashMap::from([("lat".to_string(), 12.0)]),
        last_check: t0(),
    });
}

#[test]
fn serde_roundtrip_provider_primal_service_type_variants() {
    roundtrip(&ProviderType::Primal);
    roundtrip(&PrimalType::Custom("x".to_string()));
    roundtrip(&ServiceType::Custom("svc".to_string()));
}
