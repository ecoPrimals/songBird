// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::time::Duration;

use super::*;
use songbird_process_env;

#[test]
fn test_capability_request_builder() {
    let request = CapabilityRequest::new("ai")
        .with_features(&["text-generation", "embeddings"])
        .with_preference("performance");

    assert_eq!(request.capability, "ai");
    assert_eq!(request.required_features.len(), 2);
    assert_eq!(request.preferences.len(), 1);
}

#[test]
fn test_provider_feature_support() {
    let provider = CapabilityProvider {
        name: "test-provider".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://localhost:9200".to_string(),
        protocol: Protocol::Http,
        features: vec!["text-generation".to_string(), "embeddings".to_string()],
        metadata: HashMap::new(),
    };

    assert!(provider.supports_features(&["text-generation".to_string()]));
    assert!(provider.supports_features(&["embeddings".to_string()]));
    assert!(!provider.supports_features(&["image-generation".to_string()]));
}

#[tokio::test]
async fn test_environment_discovery() {
    let resolver = CapabilityResolver::new();
    let request = CapabilityRequest::new("ai");

    let result = resolver.discover_from_environment_with(&request, &|k| {
        if k == "SONGBIRD_AI_PROVIDER_URL" {
            Ok("http://test.local:9200".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert!(result.is_ok());

    let provider = result.expect("Provider discovery should succeed in test");
    assert_eq!(provider.endpoint, "http://test.local:9200");
}

#[test]
fn test_cached_provider_expiry() {
    let provider = CapabilityProvider {
        name: "test".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://test".to_string(),
        protocol: Protocol::Http,
        features: vec![],
        metadata: HashMap::new(),
    };

    let cached = CachedProvider {
        provider,
        discovered_at: std::time::Instant::now()
            .checked_sub(Duration::from_secs(400))
            .expect("instant sub"),
        ttl: Duration::from_secs(300),
    };

    assert!(cached.is_expired());
}

#[test]
fn test_cached_provider_not_expired_when_fresh() {
    let provider = CapabilityProvider {
        name: "p".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://x".to_string(),
        protocol: Protocol::Http,
        features: vec![],
        metadata: HashMap::new(),
    };
    let cached = CachedProvider {
        provider,
        discovered_at: std::time::Instant::now(),
        ttl: Duration::from_secs(3600),
    };
    assert!(!cached.is_expired());
}

#[test]
fn test_protocol_json_roundtrip() {
    let p = Protocol::Custom("coap".to_string());
    let json = serde_json::to_string(&p).expect("serde");
    let back: Protocol = serde_json::from_str(&json).expect("de");
    assert_eq!(format!("{back:?}"), format!("{p:?}"));
}

#[test]
fn test_capability_provider_display() {
    let p = CapabilityProvider {
        name: "prov".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://h:1".to_string(),
        protocol: Protocol::Http,
        features: vec![],
        metadata: HashMap::new(),
    };
    assert_eq!(format!("{p}"), "prov[ai] @ http://h:1");
}

#[test]
fn test_supports_features_empty_required() {
    let p = CapabilityProvider {
        name: "p".to_string(),
        capability: "x".to_string(),
        endpoint: "http://x".to_string(),
        protocol: Protocol::Http,
        features: vec!["a".to_string()],
        metadata: HashMap::new(),
    };
    assert!(p.supports_features(&[]));
}

#[test]
fn test_discover_from_environment_errors_without_var() {
    let resolver = CapabilityResolver::new();
    let req = CapabilityRequest::new("sbunsetcap");
    let err = resolver
        .discover_from_environment_with(&req, &|_| Err(std::env::VarError::NotPresent))
        .expect_err("no env");
    assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
}

#[test]
fn test_capability_request_optional_features_and_sla() {
    let sla = SlaRequirements {
        max_latency_ms: 50,
        min_uptime_percent: 99.9,
        max_error_rate_percent: 0.1,
    };
    let req =
        CapabilityRequest::new("storage").with_optional_features(&["cold-archive"]).with_sla(sla);
    assert_eq!(req.optional_features, vec!["cold-archive"]);
    assert_eq!(req.min_sla.as_ref().expect("sla").max_latency_ms, 50);
}

#[test]
fn test_env_var_name_uppercases_capability_for_discovery() {
    let resolver = CapabilityResolver::new();
    let req = CapabilityRequest::new("compute");
    let out = resolver
        .discover_from_environment_with(&req, &|k| {
            if k == "SONGBIRD_COMPUTE_PROVIDER_URL" {
                Ok("http://compute:9".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .expect("env");
    assert_eq!(out.endpoint, "http://compute:9");
    assert_eq!(out.protocol, Protocol::Http);
    assert_eq!(out.name, "compute-provider-from-env");
}

#[test]
fn test_resolver_default_matches_new() {
    assert_eq!(
        CapabilityResolver::default().discovery_mechanisms.len(),
        CapabilityResolver::new().discovery_mechanisms.len()
    );
}

#[test]
fn test_discovery_mechanism_equality() {
    assert_eq!(DiscoveryMechanism::Environment, DiscoveryMechanism::Environment);
    assert_ne!(DiscoveryMechanism::Environment, DiscoveryMechanism::MDNS);
}

#[test]
fn test_provider_supports_features_requires_all() {
    let p = CapabilityProvider {
        name: "p".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://x".to_string(),
        protocol: Protocol::Https,
        features: vec!["a".to_string()],
        metadata: HashMap::new(),
    };
    assert!(!p.supports_features(&["a".to_string(), "b".to_string()]));
}

#[tokio::test]
async fn test_discover_provider_fails_when_all_mechanisms_fail() {
    let mut resolver = CapabilityResolver {
        discovery_mechanisms: vec![DiscoveryMechanism::Environment],
        provider_cache: HashMap::new(),
    };
    let err = resolver
        .discover_provider(CapabilityRequest::new("missingcap"))
        .await
        .expect_err("no provider");
    assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
}

#[tokio::test]
async fn test_discover_provider_uses_cache_before_mechanisms() {
    let cached = CapabilityProvider {
        name: "cached".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://cached".to_string(),
        protocol: Protocol::Http,
        features: vec![],
        metadata: HashMap::new(),
    };
    let mut resolver = CapabilityResolver {
        discovery_mechanisms: vec![DiscoveryMechanism::Environment],
        provider_cache: HashMap::from([(
            "ai".to_string(),
            CachedProvider {
                provider: cached.clone(),
                discovered_at: std::time::Instant::now(),
                ttl: Duration::from_secs(3600),
            },
        )]),
    };
    let got = resolver.discover_provider(CapabilityRequest::new("ai")).await.expect("cache hit");
    assert_eq!(got.endpoint, "http://cached");
}

#[tokio::test]
async fn test_discover_provider_cache_miss_when_expired() {
    let cached = CapabilityProvider {
        name: "old".to_string(),
        capability: "ai".to_string(),
        endpoint: "http://old".to_string(),
        protocol: Protocol::Http,
        features: vec![],
        metadata: HashMap::new(),
    };
    let mut resolver = CapabilityResolver {
        discovery_mechanisms: vec![DiscoveryMechanism::Environment],
        provider_cache: HashMap::from([(
            "ai".to_string(),
            CachedProvider {
                provider: cached,
                discovered_at: std::time::Instant::now()
                    .checked_sub(Duration::from_secs(400))
                    .expect("sub"),
                ttl: Duration::from_secs(300),
            },
        )]),
    };
    let got =
        resolver.discover_provider(CapabilityRequest::new("ai")).await.expect_err("env missing");
    assert!(matches!(got, SongbirdError::Discovery { .. }));
}

#[test]
fn capability_request_serde_roundtrip() {
    let req = CapabilityRequest::new("storage")
        .with_features(&["a", "b"])
        .with_optional_features(&["c"])
        .with_preference("latency");
    let json = serde_json::to_string(&req).expect("ser");
    let back: CapabilityRequest = serde_json::from_str(&json).expect("de");
    assert_eq!(back.capability, "storage");
    assert_eq!(back.required_features, vec!["a", "b"]);
    assert_eq!(back.optional_features, vec!["c"]);
}

#[test]
fn capability_provider_and_sla_serde_roundtrip() {
    let sla = SlaRequirements {
        max_latency_ms: 10,
        min_uptime_percent: 99.0,
        max_error_rate_percent: 0.5,
    };
    let p = CapabilityProvider {
        name: "n".into(),
        capability: "c".into(),
        endpoint: "http://e".into(),
        protocol: Protocol::Https,
        features: vec!["f".into()],
        metadata: HashMap::from([("k".into(), "v".into())]),
    };
    let pj = serde_json::to_string(&p).expect("ser p");
    let _: CapabilityProvider = serde_json::from_str(&pj).expect("de p");
    let sj = serde_json::to_string(&sla).expect("ser sla");
    let _: SlaRequirements = serde_json::from_str(&sj).expect("de sla");
}

#[test]
fn capability_request_multiple_preferences_and_optional_features() {
    let r = CapabilityRequest::new("x")
        .with_optional_features(&["o1", "o2"])
        .with_preference("p1")
        .with_preference("p2");
    assert_eq!(r.preferences, vec!["p1", "p2"]);
    assert_eq!(r.optional_features, vec!["o1", "o2"]);
}

#[tokio::test]
async fn discover_provider_succeeds_via_environment_only() {
    let key = "SONGBIRD_SERIALAI_PROVIDER_URL";
    songbird_process_env::set_var(key, "http://serial-ai:9200");
    let mut resolver = CapabilityResolver {
        discovery_mechanisms: vec![DiscoveryMechanism::Environment],
        provider_cache: HashMap::new(),
    };
    let p =
        resolver.discover_provider(CapabilityRequest::new("serialai")).await.expect("discovered");
    assert_eq!(p.endpoint, "http://serial-ai:9200");
    assert_eq!(p.protocol, Protocol::Http);
    songbird_process_env::remove_var(key);
}

#[tokio::test]
async fn discover_provider_falls_through_when_first_mechanism_fails_but_second_succeeds() {
    let mut resolver = CapabilityResolver {
        discovery_mechanisms: vec![
            DiscoveryMechanism::Environment,
            DiscoveryMechanism::ServiceRegistry,
        ],
        provider_cache: HashMap::new(),
    };
    let err = resolver
        .discover_provider(CapabilityRequest::new("noregistry"))
        .await
        .expect_err("registry not configured");
    assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
}
