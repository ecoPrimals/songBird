// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Runtime Capability-Based Discovery
//!
//! **Core Principle**: Each primal knows ONLY itself. Discovery happens at runtime.
//!
//! This module implements zero-hardcoding, capability-based service discovery:
//! 1. Environment variables (primary)
//! 2. DNS-SD / mDNS (local network)
//! 3. Central registry (if available)
//! 4. Capability announcements (peer-to-peer)
//!
//! Submodules: `types`, `engine`, `announcement`, `convenience`.

mod announcement;
mod constants;
mod convenience;
mod engine;
mod types;

pub use convenience::{discover_ai, discover_compute, discover_security, discover_storage};
pub use engine::RuntimeDiscoveryEngine;
pub use types::{DiscoveredService, DiscoveryMethod};

pub(crate) use convenience::discover_by_capability_timed;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_process_env;

    #[test]
    fn test_discovery_method_display() {
        assert_eq!(DiscoveryMethod::Environment.to_string(), "Environment Variable");
        assert_eq!(DiscoveryMethod::MDNS.to_string(), "mDNS");
        assert_eq!(DiscoveryMethod::Registry.to_string(), "Central Registry");
        assert_eq!(DiscoveryMethod::Announcement.to_string(), "Peer Announcement");
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        let result = RuntimeDiscoveryEngine::from_environment_with("test", &|k| {
            if k == "TEST_ENDPOINT" {
                Ok("http://test.example.com:8080".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert!(result.is_ok());

        let service = result.expect("env set");
        assert_eq!(service.capability, "test");
        assert_eq!(service.endpoint, "http://test.example.com:8080");
        assert_eq!(service.discovered_via, DiscoveryMethod::Environment);
    }

    #[test]
    fn test_from_environment_errors_when_var_missing() {
        let err =
            RuntimeDiscoveryEngine::from_environment_with("no_such_var_for_sb_rtdisc", &|_| {
                Err(std::env::VarError::NotPresent)
            })
            .expect_err("missing env var");
        assert!(matches!(err, songbird_types::SongbirdError::Configuration { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn test_check_cache_misses_expired_entry() {
        let engine = RuntimeDiscoveryEngine::new();
        let service = DiscoveredService {
            capability: "exp".to_string(),
            endpoint: "http://old".to_string(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
        };
        engine.update_cache("exp", &service).await;
        assert!(engine.check_cache("exp").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let engine = RuntimeDiscoveryEngine::new();

        let service = DiscoveredService {
            capability: "test".to_string(),
            endpoint: "http://test.example.com:8080".to_string(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        };

        engine.update_cache("test", &service).await;

        let cached = engine.check_cache("test").await;
        assert!(cached.is_some());

        let cached_service = cached.expect("cached");
        assert_eq!(cached_service.capability, "test");
        assert_eq!(cached_service.endpoint, "http://test.example.com:8080");
    }

    #[test]
    fn engine_default_matches_new() {
        let _ = RuntimeDiscoveryEngine::default();
        let _ = RuntimeDiscoveryEngine::new();
    }

    #[test]
    fn with_capabilities_constructs_engine() {
        let _e = RuntimeDiscoveryEngine::with_capabilities(vec!["a".into(), "b".into()]);
    }

    #[tokio::test]
    async fn discover_by_capability_uses_environment_variable() {
        let cap = "sbserialrtcap";
        let var = format!("{}_ENDPOINT", cap.to_uppercase());
        songbird_process_env::set_var(&var, "http://rt-env:8080");
        let engine = RuntimeDiscoveryEngine::new();
        let s = engine.discover_by_capability(cap).await.expect("from env");
        assert_eq!(s.endpoint, "http://rt-env:8080");
        assert_eq!(s.discovered_via, DiscoveryMethod::Environment);
        songbird_process_env::remove_var(&var);
    }

    #[tokio::test]
    async fn discover_by_capability_returns_cached_before_env() {
        let cap = "sbcachedcap";
        let var = format!("{}_ENDPOINT", cap.to_uppercase());
        songbird_process_env::set_var(&var, "http://should-not-be-used");
        let engine = RuntimeDiscoveryEngine::new();
        let fresh = DiscoveredService {
            capability: cap.to_string(),
            endpoint: "http://cached-first".to_string(),
            discovered_via: DiscoveryMethod::Environment,
            health_score: 1.0,
            last_seen: std::time::SystemTime::now(),
        };
        engine.update_cache(cap, &fresh).await;
        let s = engine.discover_by_capability(cap).await.expect("cache");
        assert_eq!(s.endpoint, "http://cached-first");
        songbird_process_env::remove_var(&var);
    }

    #[tokio::test]
    async fn discover_compute_errors_without_configuration() {
        use songbird_types::SongbirdError;

        let err = RuntimeDiscoveryEngine::with_timeout(std::time::Duration::from_millis(1))
            .discover_by_capability("compute")
            .await
            .expect_err("no compute");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn discover_ai_errors_without_configuration() {
        use songbird_types::SongbirdError;

        let err = RuntimeDiscoveryEngine::with_timeout(std::time::Duration::from_millis(1))
            .discover_by_capability("ai")
            .await
            .expect_err("no ai");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn discover_storage_errors_without_configuration() {
        use songbird_types::SongbirdError;

        let err = RuntimeDiscoveryEngine::with_timeout(std::time::Duration::from_millis(1))
            .discover_by_capability("storage")
            .await
            .expect_err("no storage");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn discover_security_errors_without_configuration() {
        use songbird_types::SongbirdError;

        let err = RuntimeDiscoveryEngine::with_timeout(std::time::Duration::from_millis(1))
            .discover_by_capability("security")
            .await
            .expect_err("no security");
        assert!(matches!(err, SongbirdError::Discovery { .. }), "{err:?}");
    }

    #[test]
    fn discovered_service_clone_and_debug() {
        let s = DiscoveredService {
            capability: "c".into(),
            endpoint: "e".into(),
            discovered_via: DiscoveryMethod::Registry,
            health_score: 0.5,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
        };
        let _ = format!("{:?}", &s);
        assert_eq!(s.health_score, 0.5);
    }

    #[test]
    fn from_environment_uppercases_capability_for_env_suffix() {
        let svc = RuntimeDiscoveryEngine::from_environment_with("MyCap", &|k| {
            if k == "MYCAP_ENDPOINT" {
                Ok("http://example:1".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        })
        .expect("env");
        assert_eq!(svc.capability, "MyCap");
        assert_eq!(svc.endpoint, "http://example:1");
    }

    #[test]
    fn discovery_method_copy_and_partial_eq() {
        let a = DiscoveryMethod::Environment;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, DiscoveryMethod::Registry);
    }

    #[test]
    fn discovered_service_debug_includes_capability() {
        let s = DiscoveredService {
            capability: "compute".into(),
            endpoint: "http://c:1".into(),
            discovered_via: DiscoveryMethod::MDNS,
            health_score: 1.0,
            last_seen: std::time::SystemTime::UNIX_EPOCH,
        };
        let d = format!("{s:?}");
        assert!(d.contains("compute") && d.contains("MDNS"));
    }
}
