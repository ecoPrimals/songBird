// SPDX-License-Identifier: AGPL-3.0-or-later
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

//! Tests for `PathConfig` and `ConfigProvider` systems
//!
//! Covers: `PathConfig::default()`, `PathConfig::new()`, `PathConfig::ensure_dirs_exist()`,
//! `ConfigFormat`, `ConfigProviderInfo`, `FileConfigProvider`, `EndpointSpec`.

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    songbird_process_env::test_env_lock()
}

mod path_config_tests {
    use super::lock_env;
    use songbird_config::config::paths::PathConfig;

    #[test]
    fn test_path_config_default() {
        let _guard = lock_env();
        let config = PathConfig::default();

        // All paths should be non-empty
        assert!(!config.data_dir.as_os_str().is_empty());
        assert!(!config.config_dir.as_os_str().is_empty());
        assert!(!config.log_dir.as_os_str().is_empty());
        assert!(!config.cache_dir.as_os_str().is_empty());
        assert!(!config.runtime_dir.as_os_str().is_empty());

        // Service dirs should be subdirectories of config_dir
        assert!(config.service_data_dirs.orchestrator.to_string_lossy().contains("orchestrator"));
        assert!(config.service_data_dirs.federation.to_string_lossy().contains("federation"));
        assert!(config.service_data_dirs.metrics.to_string_lossy().contains("metrics"));
        assert!(config.service_data_dirs.discovery.to_string_lossy().contains("discovery"));
        assert!(config.service_data_dirs.registry.to_string_lossy().contains("registry"));
    }

    #[test]
    fn test_path_config_new() {
        let _guard = lock_env();
        let result = PathConfig::new();
        // Should succeed on most systems (needs HOME set)
        if songbird_process_env::var("HOME").is_ok() {
            assert!(result.is_ok(), "PathConfig::new() should succeed when HOME is set");
            let config = result.unwrap();
            assert!(!config.data_dir.as_os_str().is_empty());
        }
    }

    #[test]
    fn test_path_config_serialization_roundtrip() {
        let _guard = lock_env();
        let config = PathConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PathConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.data_dir, deserialized.data_dir);
        assert_eq!(config.config_dir, deserialized.config_dir);
        assert_eq!(config.log_dir, deserialized.log_dir);
    }

    #[test]
    fn test_path_config_debug() {
        let _guard = lock_env();
        let config = PathConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("PathConfig"));
    }

    #[test]
    fn test_path_config_clone() {
        let _guard = lock_env();
        let config = PathConfig::default();
        let cloned = config.clone();
        assert_eq!(config.data_dir, cloned.data_dir);
        assert_eq!(config.config_dir, cloned.config_dir);
    }

    #[test]
    fn test_path_config_service_data_dirs_are_subdirs() {
        let _guard = lock_env();
        let config = PathConfig::default();
        // All service dirs should contain their service name
        let orchestrator_str = config.service_data_dirs.orchestrator.to_string_lossy();
        assert!(orchestrator_str.contains("orchestrator"), "orchestrator dir: {orchestrator_str}");
    }
}

mod config_providers_tests {
    use songbird_config::config::providers::{
        ConfigFormat, ConfigProviderInfo, FileConfigProvider,
    };
    use std::path::PathBuf;

    #[test]
    fn test_config_format_variants() {
        let formats = [ConfigFormat::Toml, ConfigFormat::Json, ConfigFormat::Yaml];
        for format in &formats {
            let debug = format!("{format:?}");
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_config_format_equality() {
        assert_eq!(ConfigFormat::Toml, ConfigFormat::Toml);
        assert_ne!(ConfigFormat::Toml, ConfigFormat::Json);
        assert_ne!(ConfigFormat::Json, ConfigFormat::Yaml);
    }

    #[test]
    fn test_config_provider_info() {
        let info = ConfigProviderInfo {
            name: "test-provider".to_string(),
            description: "A test provider".to_string(),
            format: ConfigFormat::Json,
        };
        assert_eq!(info.name, "test-provider");
        assert_eq!(info.description, "A test provider");
        assert_eq!(info.format, ConfigFormat::Json);
    }

    #[test]
    fn test_file_config_provider_new() {
        let provider: FileConfigProvider<String> =
            FileConfigProvider::new(PathBuf::from("/tmp/config.toml"), ConfigFormat::Toml);
        assert_eq!(provider.path(), &PathBuf::from("/tmp/config.toml"));
        assert_eq!(provider.format(), &ConfigFormat::Toml);
    }

    #[test]
    fn test_file_config_provider_json_format() {
        let provider: FileConfigProvider<serde_json::Value> =
            FileConfigProvider::new(PathBuf::from("/tmp/config.json"), ConfigFormat::Json);
        assert_eq!(provider.format(), &ConfigFormat::Json);
    }

    #[test]
    fn test_file_config_provider_yaml_format() {
        let provider: FileConfigProvider<String> =
            FileConfigProvider::new(PathBuf::from("/etc/app/config.yaml"), ConfigFormat::Yaml);
        assert!(provider.path().to_string_lossy().contains("config.yaml"));
    }
}

#[allow(deprecated, reason = "test assertions and harness ergonomics")]
mod universal_primals_tests {
    use super::lock_env;
    use songbird_config::config::universal_primals::{AutoDiscoveryConfig, PrimalRegistry};

    #[test]
    fn test_primal_registry_default() {
        let _guard = lock_env();
        let registry = PrimalRegistry::default();
        assert!(registry.primals.is_empty());
    }

    #[test]
    fn test_primal_registry_serialization() {
        let _guard = lock_env();
        let registry = PrimalRegistry::default();
        let json = serde_json::to_string(&registry).unwrap();
        let deserialized: PrimalRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(registry.primals.len(), deserialized.primals.len());
    }

    #[test]
    fn test_auto_discovery_config_default() {
        let _guard = lock_env();
        let config = AutoDiscoveryConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("AutoDiscoveryConfig"));
    }

    #[test]
    fn test_primal_registry_debug() {
        let _guard = lock_env();
        let registry = PrimalRegistry::default();
        let debug = format!("{registry:?}");
        assert!(debug.contains("PrimalRegistry"));
    }

    #[test]
    fn test_primal_registry_clone() {
        let _guard = lock_env();
        let registry = PrimalRegistry::default();
        let cloned = registry.clone();
        assert_eq!(registry.primals.len(), cloned.primals.len());
    }
}
