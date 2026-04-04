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
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// SPDX-License-Identifier: AGPL-3.0-only
//! Coverage tests for `songbird_discovery::traits::feature_flags`
//!
//! Tests struct construction, defaults, serialization, and the
//! `ProviderCapabilities` helper methods.

use songbird_discovery::traits::feature_flags::{
    CacheConfig, DeviceInfo, EvaluationConfig, EvaluationContext, EvaluationRule, FeatureFlag,
    FeatureFlagConfig, FeatureFlagProviderInfo, FlagEvaluation, FlagMonitoringConfig, FlagStats,
    FlagType, GeoLocation, ManagerStatus, MonitoringOptions, ProviderCapabilities,
    ProviderCapability, ProviderConfig, RuleCondition, RuleOperator,
};
use std::collections::HashMap;

// ───────────────────────────────────────────────────────────────────────
// ProviderCapabilities tests
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_provider_capabilities_new_empty() {
    let caps = ProviderCapabilities::new(vec![]);
    assert!(!caps.supports_updates());
    assert!(!caps.supports_history());
    assert!(!caps.supports_targeting());
    assert!(!caps.supports_percentage_rollout());
}

#[test]
fn test_provider_capabilities_all() {
    let caps = ProviderCapabilities::new(vec![
        ProviderCapability::Updates,
        ProviderCapability::History,
        ProviderCapability::Targeting,
        ProviderCapability::PercentageRollout,
    ]);
    assert!(caps.supports_updates());
    assert!(caps.supports_history());
    assert!(caps.supports_targeting());
    assert!(caps.supports_percentage_rollout());
}

#[test]
fn test_provider_capabilities_partial() {
    let caps =
        ProviderCapabilities::new(vec![ProviderCapability::Updates, ProviderCapability::Targeting]);
    assert!(caps.supports_updates());
    assert!(!caps.supports_history());
    assert!(caps.supports_targeting());
    assert!(!caps.supports_percentage_rollout());
}

#[test]
fn test_provider_capabilities_equality() {
    let a = ProviderCapabilities::new(vec![ProviderCapability::Updates]);
    let b = ProviderCapabilities::new(vec![ProviderCapability::Updates]);
    assert_eq!(a, b);

    let c = ProviderCapabilities::new(vec![ProviderCapability::History]);
    assert_ne!(a, c);
}

#[test]
fn test_provider_capability_clone_debug() {
    let cap = ProviderCapability::Updates;
    let cloned = cap.clone();
    assert_eq!(cap, cloned);
    let debug = format!("{cap:?}");
    assert!(debug.contains("Updates"));
}

// ───────────────────────────────────────────────────────────────────────
// FeatureFlagConfig defaults
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_feature_flag_config_default() {
    let config = FeatureFlagConfig::default();

    // Default provider
    assert_eq!(config.default_user.provider_type, "memory");
    assert!(config.default_user.endpoint.is_none());
    assert!(config.default_user.api_key.is_none());
    assert_eq!(config.default_user.refresh_interval, Some(300));
    assert_eq!(config.default_user.timeout_ms, 5000);
    assert!(config.default_user.settings.is_empty());

    // Providers
    assert!(config.providers.is_empty());

    // Cache
    assert!(config.cache.enabled);
    assert_eq!(config.cache.ttl_seconds, 300);
    assert_eq!(config.cache.max_entries, 10000);
    assert!(config.cache.cache_evaluations);
    assert!(config.cache.cache_flags);

    // Evaluation
    assert_eq!(config.evaluation.default_timeout_ms, 1000);
    assert!(config.evaluation.enable_analytics);
    assert!(!config.evaluation.enable_debugging);
    assert_eq!(config.evaluation.max_rule_depth, 10);
    assert!(config.evaluation.enable_context_enrichment);

    // Monitoring
    assert!(config.monitoring.enabled);
    assert_eq!(config.monitoring.metrics_interval, 60);
    assert!(config.monitoring.monitoring_options.alert_on_errors);
    assert!(config.monitoring.monitoring_options.track_performance);
    assert!(!config.monitoring.monitoring_options.export_evaluations);
}

#[test]
fn test_feature_flag_config_serialization() {
    let config = FeatureFlagConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: FeatureFlagConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.default_user.provider_type, "memory");
    assert_eq!(deserialized.cache.ttl_seconds, 300);
}

// ───────────────────────────────────────────────────────────────────────
// EvaluationContext defaults
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_evaluation_context_default() {
    let ctx = EvaluationContext::default();
    assert!(ctx.user_id.is_none());
    assert!(ctx.service_id.is_none());
    assert!(ctx.request_id.is_none());
    assert!(ctx.environment.is_none());
    assert!(ctx.version.is_none());
    assert!(ctx.geo_location.is_none());
    assert!(ctx.device_info.is_none());
    assert!(ctx.attributes.is_empty());
    // Timestamp should be recent
    let now = chrono::Utc::now();
    let diff = (now - ctx.timestamp).num_seconds().abs();
    assert!(diff < 5, "Timestamp should be within 5 seconds of now");
}

#[test]
fn test_evaluation_context_serialization() {
    let ctx = EvaluationContext {
        user_id: Some("user-123".to_string()),
        service_id: Some("svc-abc".to_string()),
        request_id: Some("req-456".to_string()),
        environment: Some("production".to_string()),
        version: Some("1.0.0".to_string()),
        geo_location: Some(GeoLocation {
            country: Some("US".to_string()),
            region: Some("CA".to_string()),
            city: Some("San Francisco".to_string()),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
        }),
        device_info: Some(DeviceInfo {
            platform: Some("linux".to_string()),
            os_version: Some("6.17".to_string()),
            app_version: Some("2.0".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("Songbird/0.2.1".to_string()),
        }),
        attributes: HashMap::new(),
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&ctx).unwrap();
    let deserialized: EvaluationContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id.as_deref(), Some("user-123"));
    assert_eq!(deserialized.environment.as_deref(), Some("production"));

    let geo = deserialized.geo_location.unwrap();
    assert_eq!(geo.country.as_deref(), Some("US"));
    assert_eq!(geo.latitude, Some(37.7749));

    let device = deserialized.device_info.unwrap();
    assert_eq!(device.platform.as_deref(), Some("linux"));
}

// ───────────────────────────────────────────────────────────────────────
// FeatureFlag struct
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_feature_flag_construction_and_serialization() {
    let flag = FeatureFlag {
        name: "dark_mode".to_string(),
        description: "Enable dark mode UI".to_string(),
        category: "ui".to_string(),
        default_value: serde_json::json!(true),
        flag_type: FlagType::Boolean,
        rules: vec![],
        metadata: HashMap::new(),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        enabled: true,
        tags: vec!["ui".to_string(), "visual".to_string()],
    };

    let json = serde_json::to_string(&flag).unwrap();
    let deserialized: FeatureFlag = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "dark_mode");
    assert!(deserialized.enabled);
    assert_eq!(deserialized.tags.len(), 2);
}

#[test]
fn test_flag_type_variants() {
    let types = vec![
        FlagType::Boolean,
        FlagType::String,
        FlagType::Number,
        FlagType::Json,
        FlagType::Variant {
            variants: vec!["a".into(), "b".into()],
        },
        FlagType::Percentage,
    ];

    for ft in &types {
        let json = serde_json::to_string(ft).unwrap();
        let deserialized: FlagType = serde_json::from_str(&json).unwrap();
        let debug = format!("{deserialized:?}");
        assert!(!debug.is_empty());
    }
}

// ───────────────────────────────────────────────────────────────────────
// EvaluationRule and RuleCondition
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_evaluation_rule_serialization() {
    let rule = EvaluationRule {
        id: "rule-1".to_string(),
        description: "Beta users get new feature".to_string(),
        conditions: vec![RuleCondition {
            attribute: "user_group".to_string(),
            operator: RuleOperator::Equals,
            value: serde_json::json!("beta"),
            negate: false,
        }],
        value: serde_json::json!(true),
        priority: 10,
        enabled: true,
        traffic_percentage: Some(50.0),
    };

    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: EvaluationRule = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "rule-1");
    assert_eq!(deserialized.priority, 10);
    assert_eq!(deserialized.traffic_percentage, Some(50.0));
    assert_eq!(deserialized.conditions.len(), 1);
}

#[test]
fn test_rule_operator_variants() {
    let operators = vec![
        RuleOperator::Equals,
        RuleOperator::NotEquals,
        RuleOperator::Contains,
        RuleOperator::StartsWith,
        RuleOperator::EndsWith,
        RuleOperator::GreaterThan,
        RuleOperator::LessThan,
        RuleOperator::GreaterThanOrEqual,
        RuleOperator::LessThanOrEqual,
        RuleOperator::In,
        RuleOperator::NotIn,
        RuleOperator::Regex,
        RuleOperator::VersionMatch,
        RuleOperator::DateBefore,
        RuleOperator::DateAfter,
        RuleOperator::Custom {
            function_name: "my_func".to_string(),
        },
    ];

    for op in &operators {
        let json = serde_json::to_string(op).unwrap();
        let deserialized: RuleOperator = serde_json::from_str(&json).unwrap();
        let debug = format!("{deserialized:?}");
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_rule_condition_negation() {
    let cond = RuleCondition {
        attribute: "country".to_string(),
        operator: RuleOperator::In,
        value: serde_json::json!(["US", "CA", "UK"]),
        negate: true,
    };

    let json = serde_json::to_string(&cond).unwrap();
    let deserialized: RuleCondition = serde_json::from_str(&json).unwrap();
    assert!(deserialized.negate);
    assert_eq!(deserialized.attribute, "country");
}

// ───────────────────────────────────────────────────────────────────────
// FlagEvaluation
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_flag_evaluation_serialization() {
    let eval = FlagEvaluation {
        feature_name: "new_ui".to_string(),
        value: serde_json::json!(true),
        matched_rule: Some("rule-1".to_string()),
        context: EvaluationContext::default(),
        timestamp: chrono::Utc::now(),
        duration_ms: 5,
        used_default: false,
        errors: vec![],
    };

    let json = serde_json::to_string(&eval).unwrap();
    let deserialized: FlagEvaluation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.feature_name, "new_ui");
    assert!(!deserialized.used_default);
    assert_eq!(deserialized.duration_ms, 5);
}

#[test]
fn test_flag_evaluation_with_errors() {
    let eval = FlagEvaluation {
        feature_name: "broken_flag".to_string(),
        value: serde_json::json!(false),
        matched_rule: None,
        context: EvaluationContext::default(),
        timestamp: chrono::Utc::now(),
        duration_ms: 100,
        used_default: true,
        errors: vec!["Rule evaluation timeout".to_string(), "Provider unreachable".to_string()],
    };

    assert!(eval.used_default);
    assert_eq!(eval.errors.len(), 2);
}

// ───────────────────────────────────────────────────────────────────────
// FlagStats
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_flag_stats_serialization() {
    let stats = FlagStats {
        total_evaluations: 1000,
        true_evaluations: 750,
        false_evaluations: 250,
        default_usages: 10,
        avg_evaluation_time_ms: 2.5,
        unique_contexts: 100,
        last_evaluation: Some(chrono::Utc::now()),
        rule_distribution: {
            let mut map = HashMap::new();
            map.insert("rule-1".to_string(), 500);
            map.insert("rule-2".to_string(), 250);
            map
        },
    };

    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: FlagStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_evaluations, 1000);
    assert_eq!(deserialized.true_evaluations, 750);
    assert_eq!(deserialized.rule_distribution.len(), 2);
}

// ───────────────────────────────────────────────────────────────────────
// FeatureFlagProviderInfo and ManagerStatus
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_provider_info_serialization() {
    let info = FeatureFlagProviderInfo {
        name: "memory".to_string(),
        version: "1.0.0".to_string(),
        capabilities: ProviderCapabilities::new(vec![
            ProviderCapability::Updates,
            ProviderCapability::History,
        ]),
        backend_type: "in-memory".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: FeatureFlagProviderInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "memory");
    assert!(deserialized.capabilities.supports_updates());
    assert!(deserialized.capabilities.supports_history());
}

#[test]
fn test_manager_status_serialization() {
    let status = ManagerStatus {
        providers_count: 2,
        flags_count: 50,
        evaluations_per_second: 1000.0,
        cache_hit_rate: 0.95,
        last_refresh: Some(chrono::Utc::now()),
        healthy: true,
    };

    let json = serde_json::to_string(&status).unwrap();
    let deserialized: ManagerStatus = serde_json::from_str(&json).unwrap();
    assert!(deserialized.healthy);
    assert_eq!(deserialized.providers_count, 2);
    assert_eq!(deserialized.flags_count, 50);
}

// ───────────────────────────────────────────────────────────────────────
// Config sub-types
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_provider_config_serialization() {
    let config = ProviderConfig {
        provider_type: "consul".to_string(),
        endpoint: Some("http://consul:8500".to_string()),
        api_key: Some("secret-key".to_string()),
        refresh_interval: Some(60),
        timeout_ms: 3000,
        settings: {
            let mut s = HashMap::new();
            s.insert("dc".to_string(), serde_json::json!("dc1"));
            s
        },
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.provider_type, "consul");
    assert_eq!(deserialized.endpoint.as_deref(), Some("http://consul:8500"));
}

#[test]
fn test_cache_config_serialization() {
    let config = CacheConfig {
        enabled: false,
        ttl_seconds: 60,
        max_entries: 100,
        cache_evaluations: false,
        cache_flags: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: CacheConfig = serde_json::from_str(&json).unwrap();
    assert!(!deserialized.enabled);
    assert_eq!(deserialized.max_entries, 100);
}

#[test]
fn test_evaluation_config_serialization() {
    let config = EvaluationConfig {
        default_timeout_ms: 500,
        enable_analytics: false,
        enable_debugging: true,
        max_rule_depth: 5,
        enable_context_enrichment: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: EvaluationConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.enable_debugging);
    assert_eq!(deserialized.max_rule_depth, 5);
}

#[test]
fn test_monitoring_config_serialization() {
    let config = FlagMonitoringConfig {
        enabled: true,
        metrics_interval: 30,
        monitoring_options: MonitoringOptions {
            alert_on_errors: true,
            track_performance: false,
            export_evaluations: true,
        },
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: FlagMonitoringConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.monitoring_options.export_evaluations);
    assert!(!deserialized.monitoring_options.track_performance);
}

// ───────────────────────────────────────────────────────────────────────
// GeoLocation and DeviceInfo
// ───────────────────────────────────────────────────────────────────────

#[test]
fn test_geo_location_partial() {
    let geo = GeoLocation {
        country: Some("DE".to_string()),
        region: None,
        city: None,
        latitude: None,
        longitude: None,
    };

    let json = serde_json::to_string(&geo).unwrap();
    let deserialized: GeoLocation = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.country.as_deref(), Some("DE"));
    assert!(deserialized.city.is_none());
}

#[test]
fn test_device_info_partial() {
    let device = DeviceInfo {
        platform: Some("android".to_string()),
        os_version: None,
        app_version: Some("3.0".to_string()),
        device_type: None,
        user_agent: None,
    };

    let json = serde_json::to_string(&device).unwrap();
    let deserialized: DeviceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.platform.as_deref(), Some("android"));
    assert_eq!(deserialized.app_version.as_deref(), Some("3.0"));
}
