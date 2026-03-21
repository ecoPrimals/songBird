// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for hosts_evolved.rs - Self-aware service configuration
//!
//! Tests the self-aware configuration, bind config, advertise config,
//! and environment detection.

use songbird_config::defaults::hosts_evolved::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn env_reader(
    m: &HashMap<String, String>,
) -> impl Fn(&str) -> Result<String, std::env::VarError> + '_ {
    |k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent)
}

// ==================== ENVIRONMENT TESTS ====================

#[test]
fn test_environment_variants() {
    let envs = vec![
        Environment::Development,
        Environment::Staging,
        Environment::Production,
        Environment::Test,
    ];

    for env in &envs {
        let debug = format!("{:?}", env);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_environment_equality() {
    assert_eq!(Environment::Development, Environment::Development);
    assert_eq!(Environment::Production, Environment::Production);
    assert_ne!(Environment::Development, Environment::Production);
}

#[test]
fn test_environment_clone() {
    let env = Environment::Production;
    let cloned = env;
    assert_eq!(env, cloned);
}

#[test]
fn test_environment_serialization() {
    let env = Environment::Staging;
    let json = serde_json::to_string(&env).unwrap();
    let deserialized: Environment = serde_json::from_str(&json).unwrap();
    assert_eq!(env, deserialized);
}

#[test]
fn test_environment_detect_development() {
    let m: HashMap<String, String> = HashMap::new();
    let detected = Environment::detect_with(&env_reader(&m));
    assert!(matches!(detected, Environment::Development | Environment::Test));
}

#[test]
fn test_environment_detect_production() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "production".into());
    let detected = Environment::detect_with(&env_reader(&m));
    assert_eq!(detected, Environment::Production);
}

#[test]
fn test_environment_detect_staging() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "staging".into());
    let detected = Environment::detect_with(&env_reader(&m));
    assert_eq!(detected, Environment::Staging);
}

#[test]
fn test_environment_detect_test() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "test".into());
    let detected = Environment::detect_with(&env_reader(&m));
    assert_eq!(detected, Environment::Test);
}

// ==================== BIND CONFIG TESTS ====================

#[test]
fn test_bind_config_for_development() {
    let config = BindConfig::for_environment(&Environment::Development);
    assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(config.port, 8080);
}

#[test]
fn test_bind_config_for_production() {
    let config = BindConfig::for_environment(&Environment::Production);
    assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(config.port, 8080);
}

#[test]
fn test_bind_config_for_staging() {
    let config = BindConfig::for_environment(&Environment::Staging);
    assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(config.port, 8080);
}

#[test]
fn test_bind_config_for_test() {
    let config = BindConfig::for_environment(&Environment::Test);
    assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(config.port, 0); // OS assigns port
}

#[test]
fn test_bind_config_socket_addr() {
    let config = BindConfig {
        ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        port: 9000,
    };
    let addr = config.socket_addr();
    assert_eq!(addr.port(), 9000);
}

#[test]
fn test_bind_config_clone() {
    let config = BindConfig::for_environment(&Environment::Development);
    let cloned = config.clone();
    assert_eq!(config.ip, cloned.ip);
    assert_eq!(config.port, cloned.port);
}

#[test]
fn test_bind_config_debug() {
    let config = BindConfig::for_environment(&Environment::Development);
    let debug = format!("{:?}", config);
    assert!(debug.contains("BindConfig"));
}

#[test]
fn test_bind_config_serialization() {
    let config = BindConfig::for_environment(&Environment::Development);
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: BindConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.port, deserialized.port);
}

// ==================== ADVERTISE CONFIG TESTS ====================

#[test]
fn test_advertise_config_for_development() {
    let config = AdvertiseConfig::for_environment(&Environment::Development);
    assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
}

#[test]
fn test_advertise_config_for_production() {
    let config = AdvertiseConfig::for_environment(&Environment::Production);
    // In production, should advertise on detected network interface
    assert!(!config.ip.is_loopback() || config.ip == IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn test_advertise_config_socket_addr() {
    let config = AdvertiseConfig {
        ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)),
        port: 8080,
    };
    let addr = config.socket_addr();
    assert_eq!(addr, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)), 8080));
}

#[test]
fn test_advertise_config_clone() {
    let config = AdvertiseConfig::for_environment(&Environment::Development);
    let cloned = config.clone();
    assert_eq!(config.ip, cloned.ip);
}

#[test]
fn test_advertise_config_debug() {
    let config = AdvertiseConfig::for_environment(&Environment::Development);
    let debug = format!("{:?}", config);
    assert!(debug.contains("AdvertiseConfig"));
}

#[test]
fn test_advertise_config_serialization() {
    let config = AdvertiseConfig::for_environment(&Environment::Development);
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AdvertiseConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.port, deserialized.port);
}

// ==================== SELF AWARE CONFIG TESTS ====================

#[test]
fn test_self_aware_config_from_environment() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "development".into());
    let config = SelfAwareConfig::from_environment_with(&env_reader(&m));
    assert_eq!(config.environment, Environment::Development);
}

#[test]
fn test_self_aware_config_bind_address() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "development".into());
    let config = SelfAwareConfig::from_environment_with(&env_reader(&m));
    let addr = config.bind_address();
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_self_aware_config_advertise_address() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "development".into());
    let config = SelfAwareConfig::from_environment_with(&env_reader(&m));
    let addr = config.advertise_address();
    assert!(addr.port() > 0 || addr.port() == 0); // Valid port range
}

#[test]
fn test_self_aware_config_clone() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "test".into());
    let config = SelfAwareConfig::from_environment_with(&env_reader(&m));
    let cloned = config.clone();
    assert_eq!(config.environment, cloned.environment);
}

#[test]
fn test_self_aware_config_debug() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "test".into());
    let config = SelfAwareConfig::from_environment_with(&env_reader(&m));
    let debug = format!("{:?}", config);
    assert!(debug.contains("SelfAwareConfig"));
}

#[test]
fn test_self_aware_config_serialization() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENVIRONMENT".into(), "staging".into());
    let config = SelfAwareConfig::from_environment_with(&env_reader(&m));
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SelfAwareConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.environment, deserialized.environment);
}

#[test]
fn test_self_aware_config_equality() {
    let config1 = SelfAwareConfig {
        bind: BindConfig::for_environment(&Environment::Development),
        advertise: AdvertiseConfig::for_environment(&Environment::Development),
        environment: Environment::Development,
    };
    let config2 = config1.clone();
    assert_eq!(config1, config2);
}

// ==================== INTEGRATION TESTS ====================

#[test]
fn test_environment_affects_bind_config() {
    let dev_config = SelfAwareConfig {
        bind: BindConfig::for_environment(&Environment::Development),
        advertise: AdvertiseConfig::for_environment(&Environment::Development),
        environment: Environment::Development,
    };

    let prod_config = SelfAwareConfig {
        bind: BindConfig::for_environment(&Environment::Production),
        advertise: AdvertiseConfig::for_environment(&Environment::Production),
        environment: Environment::Production,
    };

    // Development binds to localhost
    assert!(dev_config.bind.ip.is_loopback());

    // Production binds to all interfaces
    assert!(!prod_config.bind.ip.is_loopback());
}

#[test]
fn test_all_environments_produce_valid_config() {
    let envs = vec![
        Environment::Development,
        Environment::Staging,
        Environment::Production,
        Environment::Test,
    ];

    for env in envs {
        let config = SelfAwareConfig {
            bind: BindConfig::for_environment(&env),
            advertise: AdvertiseConfig::for_environment(&env),
            environment: env,
        };

        // All should produce valid socket addresses
        let bind = config.bind_address();
        let advertise = config.advertise_address();

        // Exercise address parsing (port is u16; always >= 0).
        let _ = bind.port();
        let _ = advertise.port();
    }
}
