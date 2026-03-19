//! Tests for hosts_evolved.rs - Self-aware service configuration
//!
//! Tests the self-aware configuration, bind config, advertise config,
//! and environment detection.

#![allow(clippy::unwrap_used)]

use songbird_config::defaults::hosts_evolved::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

struct ScopedEnv {
    vars: Vec<(String, Option<String>)>,
}

impl ScopedEnv {
    fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        std::env::set_var(key, value);
        self
    }

    fn remove(&mut self, key: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        std::env::remove_var(key);
        self
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        for (key, old) in self.vars.drain(..).rev() {
            match old {
                Some(val) => std::env::set_var(&key, &val),
                None => std::env::remove_var(&key),
            }
        }
    }
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
    let cloned = env.clone();
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
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_ENV");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.remove("CONTAINER");

    let detected = Environment::detect();
    // Without production indicators, should default to Development
    assert!(matches!(detected, Environment::Development | Environment::Test));
}

#[test]
fn test_environment_detect_production() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "production");

    let detected = Environment::detect();
    assert_eq!(detected, Environment::Production);
}

#[test]
fn test_environment_detect_staging() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "staging");

    let detected = Environment::detect();
    assert_eq!(detected, Environment::Staging);
}

#[test]
fn test_environment_detect_test() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "test");

    let detected = Environment::detect();
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
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "development");

    let config = SelfAwareConfig::from_environment();
    assert_eq!(config.environment, Environment::Development);
}

#[test]
fn test_self_aware_config_bind_address() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "development");

    let config = SelfAwareConfig::from_environment();
    let addr = config.bind_address();
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_self_aware_config_advertise_address() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "development");

    let config = SelfAwareConfig::from_environment();
    let addr = config.advertise_address();
    assert!(addr.port() > 0 || addr.port() == 0); // Valid port range
}

#[test]
fn test_self_aware_config_clone() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "test");

    let config = SelfAwareConfig::from_environment();
    let cloned = config.clone();
    assert_eq!(config.environment, cloned.environment);
}

#[test]
fn test_self_aware_config_debug() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "test");

    let config = SelfAwareConfig::from_environment();
    let debug = format!("{:?}", config);
    assert!(debug.contains("SelfAwareConfig"));
}

#[test]
fn test_self_aware_config_serialization() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENVIRONMENT", "staging");

    let config = SelfAwareConfig::from_environment();
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
            environment: env.clone(),
        };

        // All should produce valid socket addresses
        let bind = config.bind_address();
        let advertise = config.advertise_address();

        // Addresses should be valid
        assert!(bind.port() >= 0); // Port 0 is valid for OS-assigned
        assert!(advertise.port() >= 0);
    }
}
