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
