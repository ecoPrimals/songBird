// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for canonical observability configuration
//!
//! Phase 3 Test Coverage Expansion - Week 1
//! Target: 0% → 80%+ coverage for observability.rs

use super::*;

#[test]
fn test_unified_observability_config_default() {
    let config = UnifiedObservabilityConfig::default();

    assert!(config.dashboard.enabled);
    assert!(config.logging.enabled);
    assert!(!config.tracing.enabled);
}

#[test]
fn test_unified_observability_config_clone() {
    let config = UnifiedObservabilityConfig::default();
    let cloned = config.clone();

    assert_eq!(config.dashboard.enabled, cloned.dashboard.enabled);
    assert_eq!(config.logging.level, cloned.logging.level);
    assert_eq!(config.tracing.sample_rate, cloned.tracing.sample_rate);
}

#[test]
fn test_unified_observability_config_debug() {
    let config = UnifiedObservabilityConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("UnifiedObservabilityConfig"));
    assert!(debug_str.contains("dashboard"));
    assert!(debug_str.contains("logging"));
    assert!(debug_str.contains("tracing"));
}

// ============================================================================
// DASHBOARD CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_dashboard_config_default() {
    let config = DashboardConfig::default();

    assert!(config.enabled);
    assert_eq!(config.host, songbird_types::constants::PRODUCTION_BIND_ADDRESS);
    assert_eq!(config.port, songbird_types::defaults::ports::DEFAULT_DASHBOARD_PORT);
    assert!(config.realtime_updates);
    assert_eq!(config.update_interval_secs, 5);
}

#[test]
fn test_dashboard_config_custom() {
    let config = DashboardConfig {
        enabled: false,
        host: "127.0.0.1".to_string(),
        port: 4000,
        realtime_updates: false,
        update_interval_secs: 10,
    };

    assert!(!config.enabled);
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 4000);
    assert!(!config.realtime_updates);
    assert_eq!(config.update_interval_secs, 10);
}

#[test]
fn test_dashboard_config_clone() {
    let config = DashboardConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.host, cloned.host);
    assert_eq!(config.port, cloned.port);
    assert_eq!(config.realtime_updates, cloned.realtime_updates);
    assert_eq!(config.update_interval_secs, cloned.update_interval_secs);
}

#[test]
fn test_dashboard_config_debug() {
    let config = DashboardConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("DashboardConfig"));
    assert!(debug_str.contains("enabled"));
    assert!(debug_str.contains("host"));
    assert!(debug_str.contains("port"));
}

#[test]
fn test_dashboard_config_serialization() {
    let config = DashboardConfig::default();

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    assert!(json.contains("enabled"));
    assert!(json.contains("host"));
    assert!(json.contains("port"));

    // Test deserialization
    let deserialized: DashboardConfig = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.host, deserialized.host);
    assert_eq!(config.port, deserialized.port);
}

// ============================================================================
// LOGGING CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_logging_config_default() {
    let config = LoggingConfig::default();

    assert!(config.enabled);
    assert_eq!(config.level, "info");
    assert_eq!(config.format, "pretty");
    assert!(config.rotation.enabled);
}

#[test]
fn test_logging_config_custom_levels() {
    let levels = vec!["trace", "debug", "info", "warn", "error"];

    for level in levels {
        let config = LoggingConfig {
            enabled: true,
            level: level.to_string(),
            format: "json".to_string(),
            rotation: LogRotationConfig::default(),
        };

        assert_eq!(config.level, level);
    }
}

#[test]
fn test_logging_config_formats() {
    let formats = vec!["json", "pretty", "compact"];

    for format in formats {
        let config = LoggingConfig {
            enabled: true,
            level: "info".to_string(),
            format: format.to_string(),
            rotation: LogRotationConfig::default(),
        };

        assert_eq!(config.format, format);
    }
}

#[test]
fn test_logging_config_clone() {
    let config = LoggingConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.level, cloned.level);
    assert_eq!(config.format, cloned.format);
}

#[test]
fn test_logging_config_serialization() {
    let config = LoggingConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: LoggingConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.level, deserialized.level);
    assert_eq!(config.format, deserialized.format);
}

// ============================================================================
// LOG ROTATION CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_log_rotation_config_default() {
    let config = LogRotationConfig::default();

    assert!(config.enabled);
    assert_eq!(config.max_size_mb, 100);
    assert_eq!(config.max_files, 10);
}

#[test]
fn test_log_rotation_config_custom() {
    let config = LogRotationConfig {
        enabled: false,
        max_size_mb: 50,
        max_files: 5,
    };

    assert!(!config.enabled);
    assert_eq!(config.max_size_mb, 50);
    assert_eq!(config.max_files, 5);
}

#[test]
fn test_log_rotation_config_disabled() {
    let config = LogRotationConfig {
        enabled: false,
        max_size_mb: 0,
        max_files: 0,
    };

    assert!(!config.enabled);
}

#[test]
fn test_log_rotation_config_large_files() {
    let config = LogRotationConfig {
        enabled: true,
        max_size_mb: 1000,
        max_files: 100,
    };

    assert_eq!(config.max_size_mb, 1000);
    assert_eq!(config.max_files, 100);
}

#[test]
fn test_log_rotation_config_clone() {
    let config = LogRotationConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.max_size_mb, cloned.max_size_mb);
    assert_eq!(config.max_files, cloned.max_files);
}

#[test]
fn test_log_rotation_config_serialization() {
    let config = LogRotationConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: LogRotationConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.max_size_mb, deserialized.max_size_mb);
    assert_eq!(config.max_files, deserialized.max_files);
}

// ============================================================================
// TRACING CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_tracing_config_default() {
    let config = TracingConfig::default();

    assert!(!config.enabled);
    assert!(config.endpoint.is_none());
    assert_eq!(config.sample_rate, 0.1);
}

#[test]
fn test_tracing_config_enabled_with_endpoint() {
    let config = TracingConfig {
        enabled: true,
        endpoint: Some("http://jaeger:14268".to_string()),
        sample_rate: 1.0,
    };

    assert!(config.enabled);
    assert_eq!(config.endpoint, Some("http://jaeger:14268".to_string()));
    assert_eq!(config.sample_rate, 1.0);
}

#[test]
fn test_tracing_config_sample_rates() {
    let rates = vec![0.0, 0.1, 0.5, 1.0];

    for rate in rates {
        let config = TracingConfig {
            enabled: true,
            endpoint: Some("http://localhost:14268".to_string()),
            sample_rate: rate,
        };

        assert_eq!(config.sample_rate, rate);
        assert!(config.sample_rate >= 0.0 && config.sample_rate <= 1.0);
    }
}

#[test]
fn test_tracing_config_various_endpoints() {
    let endpoints = vec!["http://jaeger:14268", "http://zipkin:9411", "http://otel-collector:4318"];

    for endpoint in endpoints {
        let config = TracingConfig {
            enabled: true,
            endpoint: Some(endpoint.to_string()),
            sample_rate: 0.1,
        };

        assert_eq!(config.endpoint, Some(endpoint.to_string()));
    }
}

#[test]
fn test_tracing_config_disabled_no_endpoint() {
    let config = TracingConfig {
        enabled: false,
        endpoint: None,
        sample_rate: 0.0,
    };

    assert!(!config.enabled);
    assert!(config.endpoint.is_none());
}

#[test]
fn test_tracing_config_clone() {
    let config = TracingConfig {
        enabled: true,
        endpoint: Some("http://jaeger:14268".to_string()),
        sample_rate: 0.5,
    };
    let cloned = config.clone();

    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.endpoint, cloned.endpoint);
    assert_eq!(config.sample_rate, cloned.sample_rate);
}

#[test]
fn test_tracing_config_serialization() {
    let config = TracingConfig {
        enabled: true,
        endpoint: Some("http://jaeger:14268".to_string()),
        sample_rate: 0.5,
    };

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: TracingConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.endpoint, deserialized.endpoint);
    assert_eq!(config.sample_rate, deserialized.sample_rate);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_unified_observability_full_config() {
    let config = UnifiedObservabilityConfig {
        dashboard: DashboardConfig {
            enabled: true,
            host: "0.0.0.0".to_string(),
            port: 3000,
            realtime_updates: true,
            update_interval_secs: 5,
        },
        logging: LoggingConfig {
            enabled: true,
            level: "debug".to_string(),
            format: "json".to_string(),
            rotation: LogRotationConfig {
                enabled: true,
                max_size_mb: 200,
                max_files: 20,
            },
        },
        tracing: TracingConfig {
            enabled: true,
            endpoint: Some("http://jaeger:14268".to_string()),
            sample_rate: 0.5,
        },
    };

    assert!(config.dashboard.enabled);
    assert_eq!(config.logging.level, "debug");
    assert!(config.tracing.enabled);
}

#[test]
fn test_unified_observability_minimal_config() {
    let config = UnifiedObservabilityConfig {
        dashboard: DashboardConfig {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 3000,
            realtime_updates: false,
            update_interval_secs: 30,
        },
        logging: LoggingConfig {
            enabled: true,
            level: "error".to_string(),
            format: "compact".to_string(),
            rotation: LogRotationConfig {
                enabled: false,
                max_size_mb: 0,
                max_files: 0,
            },
        },
        tracing: TracingConfig {
            enabled: false,
            endpoint: None,
            sample_rate: 0.0,
        },
    };

    assert!(!config.dashboard.enabled);
    assert_eq!(config.logging.level, "error");
    assert!(!config.tracing.enabled);
}

#[test]
fn test_unified_observability_serialization_round_trip() {
    let original = UnifiedObservabilityConfig::default();

    let json = serde_json::to_string(&original).expect("Should serialize");
    let deserialized: UnifiedObservabilityConfig =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(original.dashboard.enabled, deserialized.dashboard.enabled);
    assert_eq!(original.logging.level, deserialized.logging.level);
    assert_eq!(original.tracing.sample_rate, deserialized.tracing.sample_rate);
}
