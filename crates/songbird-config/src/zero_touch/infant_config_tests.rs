// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for zero-touch infant configuration system
//!
//! Extracted from `infant_config.rs` for file-size discipline (<1000 lines).

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::canonical::constants::read_process_env;

#[test]
fn test_zero_touch_config_requires_service_port() {
    let result = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" | "PORT" => Err(std::env::VarError::NotPresent),
        _ => read_process_env(key),
    });
    assert!(result.is_err(), "Should require SERVICE_PORT");
}

#[test]
fn test_self_identity_discovery() {
    let identity = ZeroTouchConfig::discover_self_identity(&|key| match key {
        "SERVICE_ID" => Ok("test-service-123".to_string()),
        "SERVICE_CAPABILITIES" => Ok("compute,storage".to_string()),
        _ => Err(std::env::VarError::NotPresent),
    });
    assert_eq!(identity.service_id, "test-service-123");
    assert_eq!(identity.provides_capabilities.len(), 2);
}

#[test]
fn test_zero_touch_config_from_environment_success() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("18080".to_string()),
        _ => read_process_env(key),
    })
    .expect("valid zero-touch config");
    assert_eq!(cfg.network.service_port, 18080);
}

#[test]
fn test_service_port_invalid_errors() {
    let err = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("not-a-port".to_string()),
        _ => read_process_env(key),
    })
    .expect_err("invalid port");
    assert!(matches!(err, SongbirdError::Configuration { .. }));
}

#[test]
#[ignore = "Self-referential test - use hardcoding scanner script instead"]
fn test_no_hardcoded_primal_names() {
    let source = include_str!("infant_config.rs");

    let code_only: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !code_only.to_lowercase().contains("\"beardog\"")
            && !code_only.to_lowercase().contains("'beardog'"),
        "No beardog string literals in code"
    );
    assert!(
        !code_only.to_lowercase().contains("\"toadstool\"")
            && !code_only.to_lowercase().contains("'toadstool'"),
        "No toadstool string literals in code"
    );
    assert!(
        !code_only.to_lowercase().contains("\"nestgate\"")
            && !code_only.to_lowercase().contains("'nestgate'"),
        "No nestgate string literals in code"
    );
    assert!(
        !code_only.to_lowercase().contains("\"squirrel\"")
            && !code_only.to_lowercase().contains("'squirrel'"),
        "No squirrel string literals in code"
    );
}

#[test]
#[ignore = "Requires zero_touch_config.rs file which doesn't exist - use hardcoding scanner script"]
fn test_no_hardcoded_vendor_names() {
    let _source = include_str!("infant_config.rs");
}

#[test]
fn required_capabilities_parses_operations_and_quality() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("security".into()),
        "REQUIRED_OPERATIONS_SECURITY" => Ok("encrypt,decrypt".into()),
        "CAPABILITY_SECURITY_MAX_RESPONSE_MS" => Ok("250".into()),
        "CAPABILITY_SECURITY_MIN_AVAILABILITY" => Ok("0.99".into()),
        "CAPABILITY_SECURITY_SECURITY_LEVEL" => Ok("encrypted".into()),
        "CAPABILITY_SECURITY_FALLBACK" => Ok("fail".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(cfg.required_capabilities.len(), 1);
    let req = &cfg.required_capabilities[0];
    assert_eq!(req.capability_type, "security");
    assert_eq!(req.required_operations, vec!["encrypt", "decrypt"]);
    assert_eq!(req.quality_requirements.max_response_time_ms, Some(250));
    assert_eq!(req.quality_requirements.min_availability, Some(0.99));
    assert_eq!(req.quality_requirements.security_level, SecurityLevel::Encrypted);
    assert!(matches!(req.fallback_behavior, FallbackBehavior::Fail));
}

#[test]
fn optional_capabilities_use_default_quality_and_local_fallback() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "OPTIONAL_CAPABILITIES" => Ok("metrics".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(cfg.optional_capabilities.len(), 1);
    let o = &cfg.optional_capabilities[0];
    assert_eq!(o.capability_type, "metrics");
    assert_eq!(o.required_operations, vec!["*".to_string()]);
    assert!(matches!(o.fallback_behavior, FallbackBehavior::LocalFallback));
}

#[test]
fn create_bootstrap_config_respects_infant_and_fail_flags() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "ENABLE_INFANT_DISCOVERY" | "FAIL_ON_MISSING_CAPABILITIES" => Ok("false".into()),
        "MAX_BOOTSTRAP_SECS" => Ok("120".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert!(!cfg.bootstrap.enable_infant_discovery);
    assert_eq!(cfg.bootstrap.discovery_phases.len(), 2);
    assert!(!cfg.bootstrap.fail_on_missing_required);
    assert_eq!(cfg.bootstrap.max_bootstrap_time, Duration::from_secs(120));
}

#[test]
fn create_discovery_config_adds_http_registry_when_set() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "SERVICE_REGISTRY_ENDPOINT" => Ok("http://registry:8080".into()),
        "DISABLE_DISCOVERY_CACHE" => Ok("1".into()),
        "DISCOVERY_TIMEOUT_SECS" => Ok("45".into()),
        "DISCOVERY_REFRESH_SECS" => Ok("90".into()),
        "DISCOVERY_CACHE_TTL_SECS" => Ok("600".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert!(
        cfg.discovery.methods.iter().any(|m| matches!(m, DiscoveryMethod::HttpRegistry { .. }))
    );
    assert!(!cfg.discovery.enable_cache);
    assert_eq!(cfg.discovery.timeout, Duration::from_secs(45));
    assert_eq!(cfg.discovery.refresh_interval, Duration::from_secs(90));
    assert_eq!(cfg.discovery.cache_ttl, Duration::from_secs(600));
}

#[test]
fn self_identity_metadata_populated_from_env() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "SERVICE_ID" => Ok("svc-meta-1".into()),
        "SERVICE_VERSION" => Ok("1.2.3".into()),
        "ENVIRONMENT" => Ok("staging".into()),
        "REGION" => Ok("us-west".into()),
        "POD_NAME" => Ok("pod-a".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    let m = &cfg.self_identity.metadata;
    assert_eq!(m.get("version").map(String::as_str), Some("1.2.3"));
    assert_eq!(m.get("environment").map(String::as_str), Some("staging"));
    assert_eq!(m.get("region").map(String::as_str), Some("us-west"));
    assert_eq!(m.get("pod_name").map(String::as_str), Some("pod-a"));
}

#[test]
fn network_config_accepts_port_alias_and_parses_timeouts() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "PORT" => Ok("3000".into()),
        "HEALTH_PORT" => Ok("3001".into()),
        "METRICS_PORT" => Ok("3002".into()),
        "MAX_CONNECTIONS" => Ok("500".into()),
        "CONNECTION_TIMEOUT_SECS" => Ok("5".into()),
        "REQUEST_TIMEOUT_SECS" => Ok("15".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(cfg.network.service_port, 3000);
    assert_eq!(cfg.network.health_port, 3001);
    assert_eq!(cfg.network.metrics_port, 3002);
    assert_eq!(cfg.network.connection_limits.max_connections, 500);
    assert_eq!(cfg.network.timeouts.connection_timeout, Duration::from_secs(5));
    assert_eq!(cfg.network.timeouts.request_timeout, Duration::from_secs(15));
}

#[test]
fn quality_requirements_default_matches_documented_defaults() {
    let q = QualityRequirements::default();
    assert_eq!(q.max_response_time_ms, Some(5000));
    assert_eq!(q.min_availability, Some(0.95));
    assert_eq!(q.security_level, SecurityLevel::Basic);
}

#[test]
fn parse_quality_security_aliases_tls_strong_max() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("net".into()),
        "CAPABILITY_NET_SECURITY_LEVEL" => Ok("tls".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(
        cfg.required_capabilities[0].quality_requirements.security_level,
        SecurityLevel::Encrypted
    );

    let cfg2 = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("net2".into()),
        "CAPABILITY_NET2_SECURITY_LEVEL" => Ok("strong".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(
        cfg2.required_capabilities[0].quality_requirements.security_level,
        SecurityLevel::StrongAuth
    );

    let cfg3 = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("net3".into()),
        "CAPABILITY_NET3_SECURITY_LEVEL" => Ok("max".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(
        cfg3.required_capabilities[0].quality_requirements.security_level,
        SecurityLevel::Maximum
    );
}

#[test]
fn parse_fallback_degraded_and_local() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("store".into()),
        "CAPABILITY_STORE_FALLBACK" => Ok("degraded".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    match &cfg.required_capabilities[0].fallback_behavior {
        FallbackBehavior::DegradedMode {
            degraded_operations,
        } => {
            assert!(degraded_operations.contains(&"*".to_string()));
        }
        _ => panic!("expected degraded"),
    }

    let cfg2 = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("cache".into()),
        "CAPABILITY_CACHE_FALLBACK" => Ok("local".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert!(matches!(
        cfg2.required_capabilities[0].fallback_behavior,
        FallbackBehavior::LocalFallback
    ));
}

#[test]
fn network_bind_unspecified_in_production_signals() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("9090".into()),
        "KUBERNETES_SERVICE_HOST" => Ok("10.96.0.1".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert_eq!(cfg.network.bind_address, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn discovery_adds_dns_container_and_network_scan_methods() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "SERVICE_DISCOVERY_DOMAIN" => Ok("example.local".into()),
        "CONTAINER_METADATA_API" => Ok("http://metadata".into()),
        "ENABLE_NETWORK_DISCOVERY" => Ok("true".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    assert!(cfg.discovery.methods.iter().any(|m| matches!(m, DiscoveryMethod::DnsSrv { .. })));
    assert!(
        cfg.discovery
            .methods
            .iter()
            .any(|m| matches!(m, DiscoveryMethod::ContainerMetadata { .. }))
    );
    assert!(cfg.discovery.methods.iter().any(|m| matches!(m, DiscoveryMethod::NetworkScan { .. })));
}

#[test]
fn parse_fallback_retry_explicit() {
    let cfg = ZeroTouchConfig::from_environment_reader(|key| match key {
        "SERVICE_PORT" => Ok("8080".into()),
        "REQUIRED_CAPABILITIES" => Ok("x".into()),
        "CAPABILITY_X_FALLBACK" => Ok("retry".into()),
        _ => Err(std::env::VarError::NotPresent),
    })
    .expect("config");
    match &cfg.required_capabilities[0].fallback_behavior {
        FallbackBehavior::Retry {
            max_attempts,
            backoff_ms,
        } => {
            assert_eq!(*max_attempts, 3);
            assert_eq!(*backoff_ms, 1000);
        }
        _ => panic!("expected retry"),
    }
}
