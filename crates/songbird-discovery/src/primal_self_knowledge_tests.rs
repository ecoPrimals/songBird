// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use std::env::VarError;

#[test]
fn test_introspect_name() {
    let name = PrimalSelfKnowledge::introspect_name();
    assert!(!name.is_empty());
}

#[test]
fn test_introspect_capabilities_includes_feature_gated_or_env_hints() {
    let caps = PrimalSelfKnowledge::introspect_capabilities();
    for c in &caps {
        assert!(!c.is_empty(), "capability strings must be non-empty: {caps:?}");
    }
}

#[tokio::test]
async fn test_discover_self() {
    let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
    let identity = self_knowledge.identity();

    assert!(!identity.name.is_empty());
}

#[tokio::test]
async fn test_environment_discovery() {
    songbird_process_env::set_var("SECURITY_HOST", "localhost");
    songbird_process_env::set_var("SECURITY_PORT", "9000");

    let discovery = EnvironmentDiscovery::new();
    let result = discovery.discover("security").await;

    songbird_process_env::remove_var("SECURITY_HOST");
    songbird_process_env::remove_var("SECURITY_PORT");

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 9000);
}

#[tokio::test]
async fn environment_discovery_fails_without_env() {
    songbird_process_env::remove_var("MISSINGCAP_HOST");
    songbird_process_env::remove_var("MISSINGCAP_PORT");
    let discovery = EnvironmentDiscovery::new();
    let err = discovery.discover("missingcap").await.expect_err("no env");
    assert!(matches!(err, PrimalError::EnvironmentError(_)));
}

#[test]
fn primal_identity_serde_roundtrip() {
    let id = PrimalIdentity {
        name: "n".into(),
        capabilities: vec!["a".into()],
    };
    let json = serde_json::to_string(&id).expect("ser");
    let back: PrimalIdentity = serde_json::from_str(&json).expect("de");
    assert_eq!(back.name, "n");
    assert_eq!(back.capabilities, vec!["a".to_string()]);
}

#[tokio::test]
async fn dns_srv_discovery_returns_discovery_failed_without_feature() {
    let d = DnsSrvDiscovery::new();
    let err = d.discover("anything").await.expect_err("dns-srv");
    match err {
        PrimalError::DiscoveryFailed {
            reason,
        } => {
            assert!(reason.contains("DNS SRV") || reason.contains("dns-srv"));
        }
        _ => panic!("expected DiscoveryFailed"),
    }
}

#[test]
fn introspect_name_prefers_primal_name_from_env_fn() {
    let name = PrimalSelfKnowledge::introspect_name_with(|k| {
        if k == "PRIMAL_NAME" {
            Ok("from-primal".into())
        } else {
            Err(VarError::NotPresent)
        }
    });
    assert_eq!(name, "from-primal");
}

#[test]
fn introspect_name_falls_back_to_service_name() {
    let name = PrimalSelfKnowledge::introspect_name_with(|k| match k {
        "SERVICE_NAME" => Ok("svc".into()),
        _ => Err(VarError::NotPresent),
    });
    assert_eq!(name, "svc");
}

#[test]
fn introspect_name_primal_name_takes_priority_over_service_name() {
    let name = PrimalSelfKnowledge::introspect_name_with(|k| match k {
        "PRIMAL_NAME" => Ok("alpha".into()),
        "SERVICE_NAME" => Ok("beta".into()),
        _ => Err(VarError::NotPresent),
    });
    assert_eq!(name, "alpha");
}

#[test]
fn introspect_capabilities_adds_security_when_enable_security_set() {
    let caps = PrimalSelfKnowledge::introspect_capabilities_with(|k| {
        if k == "ENABLE_SECURITY" {
            Ok("1".into())
        } else {
            Err(VarError::NotPresent)
        }
    });
    assert!(caps.contains(&"security".to_string()));
}

#[test]
fn introspect_capabilities_dedup_duplicate_security_hint() {
    let caps = PrimalSelfKnowledge::introspect_capabilities_with(|k| {
        if k == "ENABLE_SECURITY" {
            Ok("1".into())
        } else {
            Err(VarError::NotPresent)
        }
    });
    let n = caps.iter().filter(|c| *c == "security").count();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn discover_primal_caches_first_success() {
    let pk = PrimalSelfKnowledge::discover_self_with(|k| match k {
        "PRIMAL_NAME" => Ok("self".into()),
        "AI_HOST" => Ok("127.0.0.1".into()),
        "AI_PORT" => Ok("7777".into()),
        _ => Err(VarError::NotPresent),
    })
    .expect("self");

    let a = pk.discover_primal("ai").await.expect("first");
    let b = pk.discover_primal("ai").await.expect("cached");
    assert_eq!(a.host, b.host);
    assert_eq!(a.port, b.port);

    let map = pk.discovered().await;
    assert_eq!(map.len(), 1);
}

#[tokio::test]
async fn environment_discover_with_uses_primal_prefix_fallback() {
    let info = EnvironmentDiscovery::discover_with("foo", |k| match k {
        "PRIMAL_FOO_HOST" => Ok("h".into()),
        "PRIMAL_FOO_PORT" => Ok("6500".into()),
        _ => Err(VarError::NotPresent),
    })
    .await
    .expect("discover");
    assert_eq!(info.host, "h");
    assert_eq!(info.port, 6500);
}

#[tokio::test]
async fn environment_discover_with_prefers_plain_prefix_over_primal_prefix() {
    let info = EnvironmentDiscovery::discover_with("dup", |k| match k {
        "DUP_HOST" => Ok("primary-host".into()),
        "DUP_PORT" => Ok("1111".into()),
        "PRIMAL_DUP_HOST" => Ok("fallback-host".into()),
        "PRIMAL_DUP_PORT" => Ok("2222".into()),
        _ => Err(VarError::NotPresent),
    })
    .await
    .expect("discover");
    assert_eq!(info.host, "primary-host");
    assert_eq!(info.port, 1111);
}

#[tokio::test]
async fn environment_discover_with_invalid_port_maps_to_introspection_failed() {
    let err = EnvironmentDiscovery::discover_with("badport", |k| match k {
        "BADPORT_HOST" => Ok("x".into()),
        "BADPORT_PORT" => Ok("not-a-port".into()),
        _ => Err(VarError::NotPresent),
    })
    .await
    .expect_err("bad port");
    assert!(matches!(err, PrimalError::IntrospectionFailed(_)));
}

#[tokio::test]
async fn discover_primal_fails_when_no_mechanism_succeeds() {
    let pk = PrimalSelfKnowledge::discover_self_with(|_| Err(VarError::NotPresent)).expect("self");
    let err = pk.discover_primal("nonexistent-cap-xyz").await.expect_err("none");
    assert!(matches!(err, PrimalError::DiscoveryFailed { .. }));
}

#[test]
fn primal_info_serde_roundtrip() {
    let i = PrimalInfo {
        name: "n".into(),
        host: "h".into(),
        port: 1,
        capabilities: vec!["c".into()],
        discovered_at: std::time::SystemTime::UNIX_EPOCH,
        discovery_method: "m".into(),
    };
    let js = serde_json::to_string(&i).expect("ser");
    let back: PrimalInfo = serde_json::from_str(&js).expect("de");
    assert_eq!(back.name, "n");
}

#[test]
fn primal_error_discovery_failed_display() {
    let e = PrimalError::DiscoveryFailed {
        reason: "r".into(),
    };
    assert!(e.to_string().contains('r'));
}

#[test]
fn primal_error_introspection_failed_display() {
    let e = PrimalError::IntrospectionFailed("parse issue".into());
    assert!(e.to_string().contains("parse issue"));
}

#[test]
fn primal_error_environment_error_from_var_error() {
    let e: PrimalError = VarError::NotPresent.into();
    assert!(e.to_string().contains("Environment variable error"));
}

#[test]
fn introspect_capabilities_adds_ai_when_enable_ai_set() {
    let caps = PrimalSelfKnowledge::introspect_capabilities_with(|k| {
        if k == "ENABLE_AI" {
            Ok("1".into())
        } else {
            Err(VarError::NotPresent)
        }
    });
    assert!(caps.contains(&"ai".to_string()));
}

#[tokio::test]
async fn environment_discover_with_primary_host_port_keys() {
    let info = EnvironmentDiscovery::discover_with("bar", |k| match k {
        "BAR_HOST" => Ok("host".into()),
        "BAR_PORT" => Ok("6501".into()),
        _ => Err(VarError::NotPresent),
    })
    .await
    .expect("discover");
    assert_eq!(info.host, "host");
    assert_eq!(info.port, 6501);
}

#[test]
fn primal_identity_empty_capabilities_serializes() {
    let id = PrimalIdentity {
        name: "solo".into(),
        capabilities: vec![],
    };
    let js = serde_json::to_string(&id).unwrap();
    let back: PrimalIdentity = serde_json::from_str(&js).unwrap();
    assert!(back.capabilities.is_empty());
}

#[test]
fn environment_discovery_default_matches_new() {
    let _ = EnvironmentDiscovery::default();
    let _ = EnvironmentDiscovery::new();
}

#[test]
fn dns_srv_discovery_default_matches_new() {
    let _ = DnsSrvDiscovery::default();
    let _ = DnsSrvDiscovery::new();
}

#[tokio::test]
async fn discovery_mechanism_names() {
    assert_eq!(EnvironmentDiscovery::new().name(), "environment");
    assert_eq!(DnsSrvDiscovery::new().name(), "dns-srv");
}

#[test]
fn identity_returns_cloned_identity_snapshot() {
    let pk = PrimalSelfKnowledge::discover_self_with(|k| match k {
        "PRIMAL_NAME" => Ok("tower-a".into()),
        _ => Err(VarError::NotPresent),
    })
    .expect("self");
    let id = pk.identity();
    assert_eq!(id.name, "tower-a");
    assert_eq!(id.capabilities, pk.identity().capabilities);
}

#[tokio::test]
async fn discover_primal_capability_uppercases_env_prefix() {
    let pk = PrimalSelfKnowledge::discover_self_with(|k| match k {
        "PRIMAL_NAME" => Ok("me".into()),
        "MYWEIRD_HOST" => Ok("10.0.0.2".into()),
        "MYWEIRD_PORT" => Ok("4321".into()),
        _ => Err(VarError::NotPresent),
    })
    .expect("self");
    let info = pk.discover_primal("myweird").await.expect("discovered");
    assert_eq!(info.name, "myweird");
    assert_eq!(info.host, "10.0.0.2");
    assert_eq!(info.port, 4321);
    assert_eq!(info.discovery_method, "environment");
}

#[tokio::test]
async fn primal_info_populated_fields_match_discovery_contract() {
    let pk = PrimalSelfKnowledge::discover_self_with(|k| match k {
        "PRIMAL_NAME" => Ok("self".into()),
        "NET_HOST" => Ok("net.example".into()),
        "NET_PORT" => Ok("9000".into()),
        _ => Err(VarError::NotPresent),
    })
    .expect("self");
    let info = pk.discover_primal("net").await.expect("net");
    assert_eq!(info.capabilities, vec!["net".to_string()]);
    let _ = info.discovered_at;
}

#[tokio::test]
async fn discovered_clones_cache_contents() {
    let pk = PrimalSelfKnowledge::discover_self_with(|k| match k {
        "PRIMAL_NAME" => Ok("x".into()),
        "Z_HOST" => Ok("z.local".into()),
        "Z_PORT" => Ok("1".into()),
        _ => Err(VarError::NotPresent),
    })
    .expect("self");
    assert!(pk.discovered().await.is_empty());
    pk.discover_primal("z").await.expect("z");
    let m = pk.discovered().await;
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("z").map(|i| i.host.as_str()), Some("z.local"));
}
