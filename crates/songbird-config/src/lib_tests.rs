// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for core configuration types
//!
//! Comprehensive tests for `PerformanceConfig` and main library functionality.

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
    #![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]

    use super::super::PerformanceConfig;
    use std::collections::HashMap;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();

        assert_eq!(config.connection_pool_size, Some(100));
        assert_eq!(config.request_timeout_ms, Some(30000));
        assert_eq!(config.enable_zero_copy, Some(true));
        assert_eq!(config.batch_size, Some(1000));
        assert!(config.buffer_pool_size.is_none());
        assert!(config.max_memory_mb.is_none());
        assert!(config.worker_threads.is_none());
        assert!(config.custom_params.is_none());
    }

    #[test]
    fn test_performance_config_custom_values() {
        let config = PerformanceConfig {
            buffer_pool_size: Some(256),
            max_memory_mb: Some(2048),
            worker_threads: Some(8),
            connection_pool_size: Some(200),
            request_timeout_ms: Some(60000),
            enable_zero_copy: Some(false),
            batch_size: Some(500),
            custom_params: None,
        };

        assert_eq!(config.buffer_pool_size, Some(256));
        assert_eq!(config.max_memory_mb, Some(2048));
        assert_eq!(config.worker_threads, Some(8));
        assert_eq!(config.connection_pool_size, Some(200));
        assert_eq!(config.request_timeout_ms, Some(60000));
        assert_eq!(config.enable_zero_copy, Some(false));
        assert_eq!(config.batch_size, Some(500));
    }

    #[test]
    fn test_performance_config_with_custom_params() {
        let mut custom_params = HashMap::new();
        custom_params.insert("cache_size".to_string(), serde_json::json!(512));
        custom_params.insert("gc_interval".to_string(), serde_json::json!(300));

        let config = PerformanceConfig {
            buffer_pool_size: None,
            max_memory_mb: None,
            worker_threads: None,
            connection_pool_size: Some(100),
            request_timeout_ms: Some(30000),
            enable_zero_copy: Some(true),
            batch_size: Some(1000),
            custom_params: Some(custom_params.clone()),
        };

        assert!(config.custom_params.is_some());
        let params = config.custom_params.unwrap();
        assert_eq!(params.len(), 2);
        assert!(params.contains_key("cache_size"));
        assert!(params.contains_key("gc_interval"));
    }

    #[test]
    fn test_performance_config_serialization() {
        let config = PerformanceConfig::default();
        let serialized = serde_json::to_string(&config);

        assert!(serialized.is_ok());
    }

    #[test]
    fn test_performance_config_deserialization() {
        let json = r#"{
            "buffer_pool_size": null,
            "max_memory_mb": null,
            "worker_threads": null,
            "connection_pool_size": 100,
            "request_timeout_ms": 30000,
            "enable_zero_copy": true,
            "batch_size": 1000,
            "custom_params": null
        }"#;

        let config: Result<PerformanceConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.connection_pool_size, Some(100));
        assert_eq!(config.request_timeout_ms, Some(30000));
    }

    #[test]
    fn test_performance_config_clone() {
        let config1 = PerformanceConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.connection_pool_size, config2.connection_pool_size);
        assert_eq!(config1.request_timeout_ms, config2.request_timeout_ms);
        assert_eq!(config1.enable_zero_copy, config2.enable_zero_copy);
    }

    #[test]
    fn test_performance_config_partial() {
        let json = r#"{
            "connection_pool_size": 50
        }"#;

        let config: Result<PerformanceConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.connection_pool_size, Some(50));
        assert!(config.buffer_pool_size.is_none());
    }

    #[test]
    fn test_performance_config_empty_json() {
        let json = r"{}";

        let config: Result<PerformanceConfig, _> = serde_json::from_str(json);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(config.buffer_pool_size.is_none());
        assert!(config.max_memory_mb.is_none());
    }

    #[test]
    fn test_performance_config_all_none() {
        let config = PerformanceConfig {
            buffer_pool_size: None,
            max_memory_mb: None,
            worker_threads: None,
            connection_pool_size: None,
            request_timeout_ms: None,
            enable_zero_copy: None,
            batch_size: None,
            custom_params: None,
        };

        assert!(config.buffer_pool_size.is_none());
        assert!(config.connection_pool_size.is_none());
        assert!(config.request_timeout_ms.is_none());
    }

    #[test]
    fn test_performance_config_all_some() {
        let config = PerformanceConfig {
            buffer_pool_size: Some(512),
            max_memory_mb: Some(4096),
            worker_threads: Some(16),
            connection_pool_size: Some(200),
            request_timeout_ms: Some(60000),
            enable_zero_copy: Some(true),
            batch_size: Some(2000),
            custom_params: Some(HashMap::new()),
        };

        assert!(config.buffer_pool_size.is_some());
        assert!(config.max_memory_mb.is_some());
        assert!(config.worker_threads.is_some());
        assert!(config.custom_params.is_some());
    }

    #[test]
    fn test_performance_config_zero_copy_enabled() {
        let config = PerformanceConfig {
            enable_zero_copy: Some(true),
            ..Default::default()
        };

        assert_eq!(config.enable_zero_copy, Some(true));
    }

    #[test]
    fn test_performance_config_zero_copy_disabled() {
        let config = PerformanceConfig {
            enable_zero_copy: Some(false),
            ..Default::default()
        };

        assert_eq!(config.enable_zero_copy, Some(false));
    }

    #[test]
    fn test_performance_config_large_values() {
        let config = PerformanceConfig {
            buffer_pool_size: Some(usize::MAX / 2),
            max_memory_mb: Some(u64::MAX / 2),
            worker_threads: Some(1024),
            connection_pool_size: Some(10000),
            request_timeout_ms: Some(3_600_000), // 1 hour
            enable_zero_copy: Some(true),
            batch_size: Some(100_000),
            custom_params: None,
        };

        assert!(config.buffer_pool_size.unwrap() > 1_000_000);
        assert!(config.max_memory_mb.unwrap() > 1_000_000);
    }

    #[test]
    fn test_performance_config_small_values() {
        let config = PerformanceConfig {
            buffer_pool_size: Some(1),
            max_memory_mb: Some(1),
            worker_threads: Some(1),
            connection_pool_size: Some(1),
            request_timeout_ms: Some(1),
            enable_zero_copy: Some(false),
            batch_size: Some(1),
            custom_params: None,
        };

        assert_eq!(config.buffer_pool_size, Some(1));
        assert_eq!(config.worker_threads, Some(1));
    }

    #[test]
    fn test_performance_config_typical_production() {
        let config = PerformanceConfig {
            buffer_pool_size: Some(1024),
            max_memory_mb: Some(8192),
            worker_threads: Some(16),
            connection_pool_size: Some(500),
            request_timeout_ms: Some(30000),
            enable_zero_copy: Some(true),
            batch_size: Some(5000),
            custom_params: None,
        };

        assert_eq!(config.buffer_pool_size, Some(1024));
        assert_eq!(config.worker_threads, Some(16));
        assert_eq!(config.connection_pool_size, Some(500));
    }

    #[test]
    fn test_performance_config_typical_development() {
        let config = PerformanceConfig {
            buffer_pool_size: Some(64),
            max_memory_mb: Some(512),
            worker_threads: Some(2),
            connection_pool_size: Some(10),
            request_timeout_ms: Some(5000),
            enable_zero_copy: Some(false),
            batch_size: Some(100),
            custom_params: None,
        };

        assert_eq!(config.buffer_pool_size, Some(64));
        assert_eq!(config.worker_threads, Some(2));
        assert_eq!(config.enable_zero_copy, Some(false));
    }

    #[test]
    fn test_performance_config_round_trip_serialization() {
        let original = PerformanceConfig {
            buffer_pool_size: Some(256),
            max_memory_mb: Some(2048),
            worker_threads: Some(8),
            connection_pool_size: Some(200),
            request_timeout_ms: Some(45000),
            enable_zero_copy: Some(true),
            batch_size: Some(1500),
            custom_params: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PerformanceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.buffer_pool_size, deserialized.buffer_pool_size);
        assert_eq!(original.max_memory_mb, deserialized.max_memory_mb);
        assert_eq!(original.worker_threads, deserialized.worker_threads);
    }

    #[test]
    fn test_performance_config_with_complex_custom_params() {
        let mut custom_params = HashMap::new();
        custom_params.insert(
            "nested".to_string(),
            serde_json::json!({
                "level1": {
                    "level2": {
                        "value": 42
                    }
                }
            }),
        );

        let config = PerformanceConfig {
            custom_params: Some(custom_params),
            ..Default::default()
        };

        assert!(config.custom_params.is_some());
        let params = config.custom_params.unwrap();
        assert!(params.contains_key("nested"));
    }

    #[test]
    fn test_performance_config_debug_trait() {
        let config = PerformanceConfig::default();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("PerformanceConfig"));
    }

    #[test]
    fn test_performance_config_multiple_clones() {
        let original = PerformanceConfig::default();
        let clone1 = original.clone();
        let clone2 = clone1;
        let clone3 = clone2;

        assert_eq!(original.connection_pool_size, clone3.connection_pool_size);
    }
}
