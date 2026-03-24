// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn provider_type_roundtrips_json() {
    let v = ProviderType::Discovery;
    let s = serde_json::to_string(&v).unwrap();
    let back: ProviderType = serde_json::from_str(&s).unwrap();
    assert_eq!(v, back);
}

#[test]
fn provider_type_custom_variant_preserves_label() {
    let v = ProviderType::Custom("edge".into());
    let s = serde_json::to_string(&v).unwrap();
    let back: ProviderType = serde_json::from_str(&s).unwrap();
    assert_eq!(v, back);
}

#[test]
fn primal_type_distinct_variants() {
    assert_eq!(PrimalType::Security, PrimalType::Security);
    assert_ne!(PrimalType::AI, PrimalType::Storage);
}

#[test]
fn service_type_web_roundtrip() {
    let t = ServiceType::WebService;
    let json = serde_json::to_string(&t).unwrap();
    let back: ServiceType = serde_json::from_str(&json).unwrap();
    assert_eq!(t, back);
}

#[test]
fn discovery_criteria_default_is_empty() {
    let c = DiscoveryCriteria::default();
    assert!(c.name.is_none());
    assert!(c.capabilities.is_empty());
}

#[test]
fn discovery_query_wraps_criteria() {
    let mut q = DiscoveryQuery::default();
    q.criteria.name = Some("svc".into());
    assert_eq!(q.criteria.name.as_deref(), Some("svc"));
}

#[test]
fn service_event_registered_roundtrip() {
    let info = ServiceInfo {
        id: "i".into(),
        name: "n".into(),
        service_type: ServiceType::WebService,
        version: "1".into(),
        endpoints: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
        tags: vec![],
        capabilities: vec![],
        last_updated: UNIX_EPOCH + Duration::from_secs(1),
    };
    let ev = ServiceEvent::Registered(info);
    let js = serde_json::to_string(&ev).unwrap();
    let back: ServiceEvent = serde_json::from_str(&js).unwrap();
    match back {
        ServiceEvent::Registered(i) => assert_eq!(i.id, "i"),
        _ => panic!("wrong variant"),
    }
}

#[test]
fn endpoint_host_port_serde() {
    let e = Endpoint {
        protocol: "https".into(),
        host: "h".into(),
        port: 443,
        path: Some("/p".into()),
        metadata: HashMap::new(),
    };
    let v = serde_json::to_value(&e).unwrap();
    assert_eq!(v["port"], 443);
}

#[test]
fn provider_config_environment_roundtrip() {
    let mut settings = HashMap::new();
    settings.insert("k".into(), serde_json::json!(1));
    let c = ProviderConfig {
        settings,
        enabled_features: vec!["a".into()],
        environment: "dev".into(),
    };
    let s = serde_json::to_string(&c).unwrap();
    let back: ProviderConfig = serde_json::from_str(&s).unwrap();
    assert_eq!(back.environment, "dev");
}

#[test]
fn capability_parameter_spec_optional_default() {
    let p = ParameterSpec {
        parameter_type: "string".into(),
        required: false,
        description: "d".into(),
        default_value: Some(serde_json::json!("x")),
    };
    assert!(!p.required);
}

#[test]
fn token_validation_roundtrip() {
    let tv = TokenValidation {
        valid: false,
        claims: None,
        error: Some("e".into()),
    };
    let s = serde_json::to_string(&tv).unwrap();
    let back: TokenValidation = serde_json::from_str(&s).unwrap();
    assert!(!back.valid);
}

#[test]
fn metric_query_labels_serialize() {
    let mut labels = HashMap::new();
    labels.insert("job".into(), "songbird".into());
    let q = MetricQuery {
        metric_name: "m".into(),
        start_time: UNIX_EPOCH,
        end_time: UNIX_EPOCH + Duration::from_secs(60),
        labels,
        aggregation: None,
    };
    let v = serde_json::to_value(&q).unwrap();
    assert_eq!(v["metric_name"], "m");
}

#[test]
fn deployment_status_variants_distinct() {
    let labels = [
        format!("{:?}", DeploymentStatus::Pending),
        format!("{:?}", DeploymentStatus::Running),
        format!("{:?}", DeploymentStatus::Failed),
        format!("{:?}", DeploymentStatus::Terminated),
    ];
    for (i, a) in labels.iter().enumerate() {
        for (j, b) in labels.iter().enumerate() {
            if i != j {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn primal_response_roundtrip() {
    let r = PrimalResponse {
        success: true,
        data: serde_json::json!({"k": 1}),
        metadata: HashMap::new(),
        execution_time_ms: 42,
    };
    let s = serde_json::to_string(&r).unwrap();
    let back: PrimalResponse = serde_json::from_str(&s).unwrap();
    assert!(back.success);
    assert_eq!(back.execution_time_ms, 42);
}

#[test]
fn system_health_overall_and_components() {
    let mut components = HashMap::new();
    components.insert("db".into(), HealthStatus::Healthy);
    let h = SystemHealth {
        overall_status: HealthStatus::Degraded,
        components,
        metrics: HashMap::new(),
        last_check: UNIX_EPOCH,
    };
    let v = serde_json::to_value(&h).unwrap();
    assert_eq!(v["overall_status"], "Degraded");
}

#[test]
fn span_context_baggage_serde() {
    let mut baggage = HashMap::new();
    baggage.insert("trace".into(), "t1".into());
    let sc = SpanContext {
        trace_id: "tr".into(),
        span_id: "sp".into(),
        parent_span_id: Some("p".into()),
        baggage,
    };
    let js = serde_json::to_string(&sc).unwrap();
    let back: SpanContext = serde_json::from_str(&js).unwrap();
    assert_eq!(back.trace_id, "tr");
    assert_eq!(back.baggage.get("trace").map(String::as_str), Some("t1"));
}

#[test]
fn resource_requirements_optional_limits() {
    let r = ResourceRequirements {
        cpu_limit: Some("500m".into()),
        memory_limit: None,
        cpu_request: None,
        memory_request: Some("256Mi".into()),
    };
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["cpu_limit"], "500m");
}

#[test]
fn deployment_result_message_preserved() {
    let dr = DeploymentResult {
        deployment_id: "d1".into(),
        status: DeploymentStatus::Running,
        endpoints: vec![],
        message: "ok".into(),
    };
    let s = serde_json::to_string(&dr).unwrap();
    assert!(s.contains("ok"));
}

#[test]
fn primal_info_capabilities_list() {
    let p = PrimalInfo {
        id: "p1".into(),
        name: "n".into(),
        primal_type: PrimalType::Network,
        version: "1".into(),
        capabilities: vec!["a".into(), "b".into()],
        endpoints: vec![],
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };
    assert_eq!(p.capabilities.len(), 2);
    let t = serde_json::to_string(&p).unwrap();
    assert!(t.contains("Network"));
}

#[test]
fn service_type_custom_roundtrip() {
    let t = ServiceType::Custom("grpc".into());
    let js = serde_json::to_string(&t).unwrap();
    let back: ServiceType = serde_json::from_str(&js).unwrap();
    assert_eq!(t, back);
}
