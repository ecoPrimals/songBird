// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use songbird_test_utils::ScopedEnv;
use std::collections::HashMap;

#[tokio::test]
async fn test_capability_from_environment() {
    let _env = ScopedEnv::set("CAPABILITY_SECURITY_ENDPOINT", "http://security:8443").await;

    let endpoint = get_capability_endpoint("security").await.expect("security endpoint");
    assert_eq!(endpoint, "http://security:8443");
}

#[tokio::test]
async fn test_capability_not_found() {
    let _env = ScopedEnv::remove_multiple([
        "CAPABILITY_CUSTOM_TEST_ENDPOINT",
        "SERVICE_REGISTRY_ENDPOINT",
    ])
    .await;

    let result = get_capability_endpoint("custom_test").await;
    assert!(result.is_err());
}

#[test]
fn test_capability_type_parsing() {
    assert_eq!("security".parse::<CapabilityType>().expect("parse"), CapabilityType::Security);
    assert_eq!("AUTH".parse::<CapabilityType>().expect("parse"), CapabilityType::Security);
    assert_eq!("encryption".parse::<CapabilityType>().expect("parse"), CapabilityType::Security);
    assert_eq!("database".parse::<CapabilityType>().expect("parse"), CapabilityType::Storage);
    assert_eq!("runtime".parse::<CapabilityType>().expect("parse"), CapabilityType::Compute);
    assert_eq!("intelligence".parse::<CapabilityType>().expect("parse"), CapabilityType::Ai);
    assert_eq!("workflow".parse::<CapabilityType>().expect("parse"), CapabilityType::Orchestration);
    assert_eq!("metrics".parse::<CapabilityType>().expect("parse"), CapabilityType::Observability);
    assert_eq!("mesh".parse::<CapabilityType>().expect("parse"), CapabilityType::Networking);
    assert_eq!("Storage".parse::<CapabilityType>().expect("parse"), CapabilityType::Storage);

    if let CapabilityType::Custom(name) = "my_custom".parse::<CapabilityType>().expect("parse") {
        assert_eq!(name, "my_custom");
    } else {
        panic!("Expected Custom capability");
    }
}

#[test]
fn test_env_var_names() {
    assert_eq!(CapabilityType::Security.env_var_name(), "CAPABILITY_SECURITY_ENDPOINT");
    assert_eq!(
        CapabilityType::Custom("test".to_string()).env_var_name(),
        "CAPABILITY_TEST_ENDPOINT"
    );
    assert_eq!(CapabilityType::Observability.as_str(), "observability");
    assert_eq!(CapabilityType::Networking.as_str(), "networking");
}

#[test]
fn test_capability_type_json_roundtrip() {
    let cap = CapabilityType::Orchestration;
    let json = serde_json::to_string(&cap).expect("serialize");
    let back: CapabilityType = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, cap);
}

#[test]
fn test_discovery_method_serde_roundtrip() {
    let m = DiscoveryMethod::ConfigFile;
    let json = serde_json::to_string(&m).expect("serialize");
    let back: DiscoveryMethod = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(format!("{back:?}"), format!("{m:?}"));
}

#[tokio::test]
async fn test_multiple_endpoints() {
    let _env = ScopedEnv::set_multiple([
        ("CAPABILITY_SECURITY_ENDPOINT", "http://security:8443"),
        ("CAPABILITY_STORAGE_ENDPOINT", "http://storage:9000"),
    ])
    .await;

    let endpoints = get_multiple_endpoints(&["security", "storage"]).await.expect("multiple");

    assert_eq!(endpoints.len(), 2);
    assert_eq!(endpoints[0], "http://security:8443");
    assert_eq!(endpoints[1], "http://storage:9000");
}

#[tokio::test]
async fn test_cache_functionality() {
    let _env = ScopedEnv::set("CAPABILITY_SECURITY_ENDPOINT", "http://security:8443").await;

    let endpoint1 = get_capability_endpoint("security").await.expect("first");
    let endpoint2 = get_capability_endpoint("security").await.expect("second");

    assert_eq!(endpoint1, endpoint2);

    clear_cache();

    let endpoint3 = get_capability_endpoint("security").await.expect("third");

    assert_eq!(endpoint1, endpoint3);
}

#[tokio::test]
async fn static_override_returns_endpoint_without_environment() {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Compute, "http://compute-override:9000".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let ep = resolver.get_endpoint(CapabilityType::Compute).await.expect("static override");
    assert_eq!(ep, "http://compute-override:9000");
    let cached_map = resolver.get_all_cached().await;
    let cached = cached_map.get(&CapabilityType::Compute).expect("cached");
    assert!(matches!(cached.discovery_method, DiscoveryMethod::ConfigFile));
}

#[test]
fn capability_endpoint_serde_roundtrip() {
    let ce = CapabilityEndpoint {
        capability: CapabilityType::Observability,
        endpoint: "http://obs:4317".into(),
        provider_id: Some("prov".into()),
        discovery_method: DiscoveryMethod::ServiceRegistry,
        confidence: 0.85,
        discovered_at: std::time::SystemTime::UNIX_EPOCH,
    };
    let json = serde_json::to_string(&ce).expect("serialize");
    let back: CapabilityEndpoint = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.capability, ce.capability);
    assert_eq!(back.endpoint, ce.endpoint);
    assert_eq!(back.confidence, ce.confidence);
    assert_eq!(back.provider_id, ce.provider_id);
}

#[test]
fn capability_type_custom_stores_normalized_name() {
    let c: CapabilityType = "MyWidget".parse().expect("parse");
    match c {
        CapabilityType::Custom(s) => assert_eq!(s, "mywidget"),
        _ => panic!("expected Custom"),
    }
    assert_eq!(CapabilityType::Custom("foo".into()).env_var_name(), "CAPABILITY_FOO_ENDPOINT");
}

#[test]
fn capability_type_roundtrip_as_str_known_variants() {
    for (ct, expected) in [
        (CapabilityType::Security, "security"),
        (CapabilityType::Storage, "storage"),
        (CapabilityType::Compute, "compute"),
        (CapabilityType::Ai, "ai"),
        (CapabilityType::Orchestration, "orchestration"),
        (CapabilityType::Observability, "observability"),
        (CapabilityType::Networking, "networking"),
    ] {
        assert_eq!(ct.as_str(), expected);
        let parsed: CapabilityType = expected.parse().expect("parse");
        assert_eq!(parsed, ct);
    }
}

#[tokio::test]
async fn resolver_static_override_for_storage_endpoint() {
    let mut overrides = HashMap::new();
    overrides.insert(CapabilityType::Storage, "unix:///tmp/storage.sock".to_string());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(overrides);
    let ep = resolver.get_endpoint(CapabilityType::Storage).await.expect("storage");
    assert_eq!(ep, "unix:///tmp/storage.sock");
}
