// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use songbird_types::SongbirdError;

#[tokio::test]
async fn test_environment_discovery() {
    let discovery =
        CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |k| {
            if k == "COMPUTE_ENDPOINT" {
                Ok("http://10.0.0.100:8001".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });

    let providers = discovery.discover_compute().await.expect("compute from env");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].url, "http://10.0.0.100:8001");
    assert!(providers[0].capabilities.contains(&"compute".to_string()));
}

#[tokio::test]
async fn test_no_providers_found() {
    let discovery =
        CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |_| {
            Err(std::env::VarError::NotPresent)
        });

    let result = discovery.find_providers_by_capability("nonexistent").await;
    assert!(result.is_err());

    if let Err(SongbirdError::Discovery {
        message,
        ..
    }) = result
    {
        assert!(message.contains("No providers found"));
        assert!(message.contains("NONEXISTENT_ENDPOINT"));
    } else {
        panic!("Expected Discovery error");
    }
}

#[tokio::test]
async fn test_cache_behavior() {
    let discovery =
        CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |k| {
            if k == "TEST_CAPABILITY_ENDPOINT" {
                Ok("http://test:1234".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });

    // First discovery
    let providers1 =
        discovery.find_providers_by_capability("test_capability").await.expect("first discovery");

    // Second discovery should use cache
    let providers2 =
        discovery.find_providers_by_capability("test_capability").await.expect("cached discovery");

    assert_eq!(providers1.len(), providers2.len());

    // Clear cache
    discovery.clear_cache("test_capability").await;
}

#[test]
fn test_parse_toml_config_rejects_invalid_syntax() {
    let err =
        CapabilityDiscovery::parse_toml_config("{{{not_toml", "compute").expect_err("invalid TOML");
    match err {
        SongbirdError::Discovery {
            message,
            backend: Some(b),
            ..
        } => {
            assert_eq!(b, "config_file");
            assert!(message.contains("TOML"), "message: {message}");
        }
        other => panic!("expected Discovery error, got {other:?}"),
    }
}

#[test]
fn test_parse_json_config_rejects_invalid_syntax() {
    let err = CapabilityDiscovery::parse_json_config("{", "compute").expect_err("invalid JSON");
    assert!(
        matches!(err, SongbirdError::Discovery { ref backend, .. } if backend.as_deref() == Some("config_file")),
        "{err:?}"
    );
}

#[test]
fn test_parse_yaml_config_rejects_invalid_syntax() {
    let err =
        CapabilityDiscovery::parse_yaml_config(":\n  -", "compute").expect_err("invalid YAML");
    assert!(
        matches!(err, SongbirdError::Discovery { ref backend, .. } if backend.as_deref() == Some("config_file")),
        "{err:?}"
    );
}

#[test]
fn test_extract_endpoints_from_toml_respects_capability_and_health() {
    let toml = r#"
[services.alpha]
url = "http://alpha:1"
capabilities = ["compute"]
health_score = 0.42

[services.beta]
url = "http://beta:2"
capabilities = ["storage"]
"#;
    let v: toml::Value = toml::from_str(toml).expect("fixture TOML");
    let endpoints = CapabilityDiscovery::extract_endpoints_from_config(&v, "compute");
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].id, "alpha");
    assert_eq!(endpoints[0].url, "http://alpha:1");
    assert!((endpoints[0].health_score - 0.42).abs() < f64::EPSILON);
}

#[test]
fn test_extract_endpoints_from_json_empty_when_capability_missing() {
    let json = r#"{"services":{"only":{"url":"http://x:1","capabilities":["ai"]}}}"#;
    let v: serde_json::Value = serde_json::from_str(json).expect("fixture JSON");
    let endpoints = CapabilityDiscovery::extract_endpoints_from_json(&v, "compute");
    assert!(endpoints.is_empty());
}

#[test]
fn test_extract_endpoints_from_yaml_matches_capability() {
    let yaml = r"
services:
  svc1:
    url: http://y:3
    capabilities:
      - compute
";
    let v: serde_yaml::Value = serde_yaml::from_str(yaml).expect("fixture YAML");
    let endpoints = CapabilityDiscovery::extract_endpoints_from_yaml(&v, "compute");
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].url, "http://y:3");
}

#[tokio::test]
async fn test_discover_via_config_file_rejects_unsupported_extension() {
    let path =
        std::env::temp_dir().join(format!("songbird_cfg_unsup_{}.bin", uuid::Uuid::new_v4()));
    tokio::fs::write(&path, b"{}").await.expect("write fixture");

    let discovery = CapabilityDiscovery::new();
    let path_str = path.to_str().expect("utf8 path");
    let err = discovery
        .discover_via_config_file("compute", path_str)
        .await
        .expect_err("unsupported extension");

    let _ = tokio::fs::remove_file(&path).await;

    match err {
        SongbirdError::Discovery {
            message,
            backend: Some(b),
            ..
        } => {
            assert_eq!(b, "config_file");
            assert!(message.contains("Unsupported"), "message: {message}");
        }
        other => panic!("expected Discovery error, got {other:?}"),
    }
}

#[test]
fn test_capability_discovery_default_impl() {
    let d = CapabilityDiscovery::default();
    assert!(!d.methods.is_empty());
}

#[tokio::test]
async fn test_clear_all_caches() {
    let d = CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |k| {
        if k == "SB_CAP_CLEAR_ENDPOINT" {
            Ok("http://127.0.0.1:1".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    let _ = d.find_providers_by_capability("sb_cap_clear").await;
    d.clear_all_caches().await;
}

#[tokio::test]
async fn test_discover_storage_delegates_to_find() {
    let d = CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |k| {
        if k == "STORAGE_ENDPOINT" {
            Ok("http://store:9".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    let v = d.discover_storage().await.expect("storage from env");
    assert!(!v.is_empty());
}
