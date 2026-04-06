// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_environment_detection() {
    let env = Environment::detect();
    assert!(matches!(
        env,
        Environment::Development
            | Environment::Test
            | Environment::Staging
            | Environment::Production
    ));
}

#[test]
fn test_self_aware_config_development() {
    let config = SelfAwareConfig::from_environment_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("development".to_string())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(config.environment, Environment::Development);
    assert!(config.bind_address().ip().is_loopback());
}

#[test]
fn test_bind_config_production() {
    let config = BindConfig::for_environment(&Environment::Production);
    assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn test_bind_config_development() {
    let config = BindConfig::for_environment(&Environment::Development);
    assert!(config.ip.is_loopback());
}

#[test]
fn test_environment_production_like() {
    assert!(Environment::Production.is_production_like());
    assert!(Environment::Staging.is_production_like());
    assert!(!Environment::Development.is_production_like());
    assert!(!Environment::Test.is_production_like());
}

#[test]
fn test_environment_development_like() {
    assert!(Environment::Development.is_development_like());
    assert!(Environment::Test.is_development_like());
    assert!(!Environment::Production.is_development_like());
    assert!(!Environment::Staging.is_development_like());
}

#[test]
fn test_service_locator_creation() {
    let locator = super::super::service_locator::ServiceLocator::new();
    assert!(
        locator.self_config().bind_address().port() > 0
            || locator.self_config().environment == Environment::Test
    );
}

#[tokio::test]
async fn test_capability_discovery() {
    let locator = super::super::service_locator::ServiceLocator::new();
    let _services = locator.discover_by_capability("compute");
}

#[test]
fn detect_with_explicit_production() {
    let e = Environment::detect_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("production".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Production);
}

#[test]
fn detect_with_explicit_staging_alias() {
    let e = Environment::detect_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("stage".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Staging);
}

#[test]
fn detect_with_explicit_test() {
    let e = Environment::detect_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("test".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Test);
}

#[test]
fn detect_with_unknown_songbird_env_defaults_to_development() {
    let e = Environment::detect_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("experimental-lab".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Development);
}

#[test]
fn detect_with_kubernetes_host_is_production() {
    let e = Environment::detect_with(&|k| {
        if k == "KUBERNETES_SERVICE_HOST" {
            Ok("10.0.0.1".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Production);
}

#[test]
fn detect_with_rust_test_threads_is_test() {
    let e = Environment::detect_with(&|k| {
        if k == "RUST_TEST_THREADS" {
            Ok("8".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Test);
}

#[test]
fn bind_config_for_test_uses_ephemeral_port() {
    let b = BindConfig::for_environment(&Environment::Test);
    assert!(b.ip.is_loopback());
    assert_eq!(b.port, 0);
}

#[test]
fn bind_config_for_staging_binds_unspecified() {
    let b = BindConfig::for_environment(&Environment::Staging);
    assert_eq!(b.ip, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn self_aware_config_bind_and_advertise_consistent_in_development() {
    let c = SelfAwareConfig::from_environment_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("development".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(c.bind_address(), c.advertise_address());
}

#[test]
fn advertise_config_for_development_is_loopback() {
    let a = AdvertiseConfig::for_environment(&Environment::Development);
    assert!(a.ip.is_loopback());
    assert_eq!(a.port, 8080);
}

#[test]
fn advertise_config_for_test_uses_loopback_ephemeral_port() {
    let a = AdvertiseConfig::for_environment(&Environment::Test);
    assert!(a.ip.is_loopback());
    assert_eq!(a.port, 0);
}

#[test]
fn detect_with_explicit_prod_alias() {
    let e = Environment::detect_with(&|k| {
        if k == "SONGBIRD_ENVIRONMENT" {
            Ok("prod".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Production);
}

#[test]
fn detect_with_ecs_metadata_uri_implies_production() {
    let e = Environment::detect_with(&|k| {
        if k == "ECS_CONTAINER_METADATA_URI" {
            Ok("http://169.254.170.2/v3/abc".into())
        } else {
            Err(std::env::VarError::NotPresent)
        }
    });
    assert_eq!(e, Environment::Production);
}

#[test]
fn bind_config_socket_addr_round_trips() {
    let b = BindConfig::for_environment(&Environment::Development);
    assert_eq!(b.socket_addr().port(), 8080);
    assert!(b.socket_addr().ip().is_loopback());
}

#[test]
fn environment_serialization_round_trip() {
    for env in
        [Environment::Development, Environment::Test, Environment::Staging, Environment::Production]
    {
        let json = serde_json::to_string(&env).expect("serialize");
        let back: Environment = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(env, back);
    }
}
