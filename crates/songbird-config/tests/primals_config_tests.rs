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

//! Comprehensive tests for canonical primals configuration

use songbird_config::canonical::primals::*;
use std::time::Duration;

// ============================================================================
// PrimalType Tests
// ============================================================================

#[test]
fn test_primal_type_default() {
    let primal_type = PrimalType::default();
    assert!(matches!(primal_type, PrimalType::Unknown));
}

#[test]
fn test_primal_type_all_variants_display() {
    assert_eq!(PrimalType::Compute.to_string(), "compute");
    assert_eq!(PrimalType::Storage.to_string(), "storage");
    assert_eq!(PrimalType::Security.to_string(), "security");
    assert_eq!(PrimalType::AI.to_string(), "ai");
    assert_eq!(PrimalType::Orchestration.to_string(), "orchestration");
    assert_eq!(PrimalType::Gaming.to_string(), "gaming");
    assert_eq!(PrimalType::Communication.to_string(), "communication");
    assert_eq!(PrimalType::Media.to_string(), "media");
    assert_eq!(PrimalType::Database.to_string(), "database");
    assert_eq!(PrimalType::Analytics.to_string(), "analytics");
    assert_eq!(PrimalType::Development.to_string(), "development");
    assert_eq!(PrimalType::IoT.to_string(), "iot");
    assert_eq!(PrimalType::Blockchain.to_string(), "blockchain");
    assert_eq!(PrimalType::Financial.to_string(), "financial");
    assert_eq!(PrimalType::Identity.to_string(), "identity");
    assert_eq!(PrimalType::Cdn.to_string(), "cdn");
    assert_eq!(PrimalType::Email.to_string(), "email");
    assert_eq!(PrimalType::Search.to_string(), "search");
    assert_eq!(PrimalType::Backup.to_string(), "backup");
    assert_eq!(PrimalType::Compliance.to_string(), "compliance");
    assert_eq!(PrimalType::Unknown.to_string(), "unknown");
}

#[test]
fn test_primal_type_custom_display() {
    let custom = PrimalType::Custom("myprimal".to_string());
    assert_eq!(custom.to_string(), "custom-myprimal");
}

#[test]
fn test_primal_type_from_str_all_variants() {
    assert_eq!("compute".parse::<PrimalType>().unwrap(), PrimalType::Compute);
    assert_eq!("storage".parse::<PrimalType>().unwrap(), PrimalType::Storage);
    assert_eq!("security".parse::<PrimalType>().unwrap(), PrimalType::Security);
    assert_eq!("ai".parse::<PrimalType>().unwrap(), PrimalType::AI);
    assert_eq!("orchestration".parse::<PrimalType>().unwrap(), PrimalType::Orchestration);
    assert_eq!("gaming".parse::<PrimalType>().unwrap(), PrimalType::Gaming);
    assert_eq!("communication".parse::<PrimalType>().unwrap(), PrimalType::Communication);
    assert_eq!("media".parse::<PrimalType>().unwrap(), PrimalType::Media);
    assert_eq!("database".parse::<PrimalType>().unwrap(), PrimalType::Database);
    assert_eq!("analytics".parse::<PrimalType>().unwrap(), PrimalType::Analytics);
    assert_eq!("development".parse::<PrimalType>().unwrap(), PrimalType::Development);
    assert_eq!("iot".parse::<PrimalType>().unwrap(), PrimalType::IoT);
    assert_eq!("blockchain".parse::<PrimalType>().unwrap(), PrimalType::Blockchain);
    assert_eq!("financial".parse::<PrimalType>().unwrap(), PrimalType::Financial);
    assert_eq!("identity".parse::<PrimalType>().unwrap(), PrimalType::Identity);
    assert_eq!("cdn".parse::<PrimalType>().unwrap(), PrimalType::Cdn);
    assert_eq!("email".parse::<PrimalType>().unwrap(), PrimalType::Email);
    assert_eq!("search".parse::<PrimalType>().unwrap(), PrimalType::Search);
    assert_eq!("backup".parse::<PrimalType>().unwrap(), PrimalType::Backup);
    assert_eq!("compliance".parse::<PrimalType>().unwrap(), PrimalType::Compliance);
    assert_eq!("unknown".parse::<PrimalType>().unwrap(), PrimalType::Unknown);
}

#[test]
fn test_primal_type_from_str_custom() {
    let result = "custom-myservice".parse::<PrimalType>().unwrap();
    assert_eq!(result, PrimalType::Custom("myservice".to_string()));
}

#[test]
fn test_primal_type_from_str_unknown_becomes_custom() {
    let result = "someunknowntype".parse::<PrimalType>().unwrap();
    assert_eq!(result, PrimalType::Custom("someunknowntype".to_string()));
}

#[test]
fn test_primal_type_case_insensitive() {
    assert_eq!("COMPUTE".parse::<PrimalType>().unwrap(), PrimalType::Compute);
    assert_eq!("Compute".parse::<PrimalType>().unwrap(), PrimalType::Compute);
    assert_eq!("cOmPuTe".parse::<PrimalType>().unwrap(), PrimalType::Compute);
}

// ============================================================================
// ServiceCategory Tests
// ============================================================================

#[test]
fn test_service_category_default() {
    let category = ServiceCategory::default();
    assert!(matches!(category, ServiceCategory::Application));
}

#[test]
fn test_service_category_all_variants_display() {
    assert_eq!(ServiceCategory::Infrastructure.to_string(), "infrastructure");
    assert_eq!(ServiceCategory::Application.to_string(), "application");
    assert_eq!(ServiceCategory::Data.to_string(), "data");
    assert_eq!(ServiceCategory::UI.to_string(), "ui");
    assert_eq!(ServiceCategory::Integration.to_string(), "integration");
    assert_eq!(ServiceCategory::Monitoring.to_string(), "monitoring");
    assert_eq!(ServiceCategory::Security.to_string(), "security");
    assert_eq!(ServiceCategory::Development.to_string(), "development");
    assert_eq!(ServiceCategory::Analytics.to_string(), "analytics");
    assert_eq!(ServiceCategory::Communication.to_string(), "communication");
}

#[test]
fn test_service_category_custom_display() {
    let custom = ServiceCategory::Custom("myservice".to_string());
    assert_eq!(custom.to_string(), "custom-myservice");
}

// ============================================================================
// QosMetrics Tests
// ============================================================================

#[test]
fn test_qos_metrics_default() {
    let metrics = QosMetrics::default();
    assert!(metrics.latency_ms.is_none());
    assert!(metrics.throughput_ops_sec.is_none());
    assert!(metrics.availability.is_none());
    assert!(metrics.reliability.is_none());
}

#[test]
fn test_qos_metrics_with_values() {
    let metrics = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.99),
        reliability: Some(0.999),
    };
    assert_eq!(metrics.latency_ms, Some(100.0));
    assert_eq!(metrics.throughput_ops_sec, Some(1000.0));
    assert_eq!(metrics.availability, Some(0.99));
    assert_eq!(metrics.reliability, Some(0.999));
}

#[test]
fn test_qos_metrics_clone() {
    let metrics = QosMetrics {
        latency_ms: Some(50.0),
        throughput_ops_sec: Some(500.0),
        availability: Some(0.95),
        reliability: Some(0.99),
    };
    let cloned = metrics.clone();
    assert_eq!(metrics.latency_ms, cloned.latency_ms);
    assert_eq!(metrics.availability, cloned.availability);
}

// ============================================================================
// ConnectionSettings Tests
// ============================================================================

#[test]
fn test_connection_settings_default() {
    let settings = ConnectionSettings::default();
    assert_eq!(settings.connection_timeout, Duration::from_secs(30));
    assert_eq!(settings.request_timeout, Duration::from_secs(60));
    assert_eq!(settings.max_retries, 3);
    assert!(settings.keep_alive);
}

#[test]
fn test_connection_settings_clone() {
    let settings = ConnectionSettings::default();
    let cloned = settings.clone();
    assert_eq!(settings.connection_timeout, cloned.connection_timeout);
    assert_eq!(settings.max_retries, cloned.max_retries);
}

// ============================================================================
// HealthCheckConfig Tests
// ============================================================================

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();
    assert!(config.enabled);
    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.endpoint_path, "/health");
    assert_eq!(config.expected_status_codes, vec![200]);
    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.failure_threshold, 3);
}

#[test]
fn test_health_check_config_clone() {
    let config = HealthCheckConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.endpoint_path, cloned.endpoint_path);
}

// ============================================================================
// PrimalRegistry Tests
// ============================================================================

#[test]
fn test_primal_registry_new() {
    let registry = PrimalRegistry::new();
    assert!(registry.primals.is_empty());
}

#[test]
fn test_primal_registry_default() {
    let registry = PrimalRegistry::default();
    assert!(registry.primals.is_empty());
}

#[test]
fn test_primal_registry_register_and_get() {
    let mut registry = PrimalRegistry::new();
    let config = PrimalConfiguration::new_template("security", "Security Provider");
    registry.register_primal(config);

    let retrieved = registry.get_primal("security");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().display_name, "Security Provider");
}

#[test]
fn test_primal_registry_is_registered() {
    let mut registry = PrimalRegistry::new();
    let config = PrimalConfiguration::new_template("storage", "Storage Provider");
    registry.register_primal(config);

    assert!(registry.is_registered("storage"));
    assert!(!registry.is_registered("nonexistent"));
}

#[test]
fn test_primal_registry_get_enabled_primals() {
    let mut registry = PrimalRegistry::new();

    let mut enabled_config = PrimalConfiguration::new_template("compute", "Compute Provider");
    enabled_config.enabled = true;

    let disabled_config = PrimalConfiguration::new_template("storage", "Storage Provider");

    registry.register_primal(enabled_config);
    registry.register_primal(disabled_config);

    let enabled = registry.get_enabled_primals();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].primal_type, "compute");
}

// ============================================================================
// PrimalConfiguration Tests
// ============================================================================

#[test]
fn test_primal_configuration_new_template() {
    let config = PrimalConfiguration::new_template("ai", "AI Provider");
    assert_eq!(config.primal_type, "ai");
    assert_eq!(config.display_name, "AI Provider");
    assert!(!config.enabled);
    assert!(config.capabilities.is_empty());
}

// ============================================================================
// PrimalEndpoint Tests
// ============================================================================

#[test]
fn test_primal_endpoint_default() {
    let endpoint = PrimalEndpoint::default();
    assert!(endpoint.primary_url.is_empty());
    assert!(endpoint.use_tls);
}

#[test]
fn test_primal_endpoint_clone() {
    let endpoint = PrimalEndpoint {
        primary_url: "https://example.com".to_string(),
        use_tls: false,
    };
    let cloned = endpoint.clone();
    assert_eq!(endpoint.primary_url, cloned.primary_url);
    assert_eq!(endpoint.use_tls, cloned.use_tls);
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_primal_type_serialization() {
    let primal_type = PrimalType::Security;
    let json = serde_json::to_string(&primal_type).expect("Serialization should succeed");
    assert!(json.contains("Security"));
}

#[test]
fn test_qos_metrics_serialization() {
    let metrics = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: None,
        availability: Some(0.99),
        reliability: None,
    };
    let json = serde_json::to_string(&metrics).expect("Serialization should succeed");
    assert!(json.contains("latency_ms"));
    assert!(json.contains("100.0"));
}

#[test]
fn test_primal_registry_serialization() {
    let mut registry = PrimalRegistry::new();
    let config = PrimalConfiguration::new_template("test", "Test Provider");
    registry.register_primal(config);

    let json = serde_json::to_string(&registry).expect("Serialization should succeed");
    assert!(json.contains("test"));
    assert!(json.contains("Test Provider"));
}
