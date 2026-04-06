// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for Universal Adapter Configuration
//!
//! Comprehensive test coverage for adapter configuration structures.

use super::*;
use songbird_types::config::consolidated_canonical::CanonicalHealthCheckConfig as HealthCheckConfig;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

// ============================================================================
// UniversalAdapterConfig Tests
// ============================================================================

#[test]
fn test_universal_adapter_config_default() -> SongbirdResult<()> {
    let config = UniversalAdapterConfig::default();

    assert!(config.security_adapters.enabled);
    assert!(config.compute_adapters.enabled);
    assert!(config.storage_adapters.enabled);
    assert_eq!(config.settings.default_timeout_ms, 30000);
    Ok(())
}

#[test]
fn test_universal_adapter_config_serialization() -> SongbirdResult<()> {
    let config = UniversalAdapterConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: UniversalAdapterConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(config.security_adapters.enabled, deserialized.security_adapters.enabled);
    assert_eq!(config.settings.default_timeout_ms, deserialized.settings.default_timeout_ms);
    Ok(())
}

#[test]
fn test_universal_adapter_config_clone() {
    let config = UniversalAdapterConfig::default();
    let cloned = config.clone();

    assert_eq!(config.settings.default_timeout_ms, cloned.settings.default_timeout_ms);
}

// ============================================================================
// SecurityAdapterConfig Tests
// ============================================================================

#[test]
fn test_security_adapter_config_default() -> SongbirdResult<()> {
    let config = SecurityAdapterConfig::default();

    assert!(config.enabled);
    assert_eq!(config.discovery_mode, "auto");
    assert!(config.endpoint.is_none());
    assert_eq!(config.timeout_ms, 30000);
    assert_eq!(config.retry_count, 3);
    Ok(())
}

#[test]
fn test_security_adapter_config_with_endpoint() -> SongbirdResult<()> {
    let config = SecurityAdapterConfig {
        endpoint: Some("http://security-service:8080".to_string()),
        ..Default::default()
    };

    assert!(config.endpoint.is_some());
    assert_eq!(config.endpoint.unwrap(), "http://security-service:8080");
    Ok(())
}

#[test]
fn test_security_adapter_config_custom_timeout() {
    let config = SecurityAdapterConfig {
        timeout_ms: 60000,
        ..Default::default()
    };

    assert_eq!(config.timeout_ms, 60000);
}

#[test]
fn test_security_adapter_config_retry_count() {
    let config = SecurityAdapterConfig {
        retry_count: 5,
        ..Default::default()
    };

    assert_eq!(config.retry_count, 5);
}

// ============================================================================
// ComputeAdapterConfig Tests
// ============================================================================

#[test]
fn test_compute_adapter_config_default() {
    let config = ComputeAdapterConfig::default();

    assert!(config.enabled);
    assert_eq!(config.discovery_mode, "auto");
    assert!(config.endpoint.is_none());
    assert_eq!(config.timeout_ms, 30000);
    assert_eq!(config.retry_count, 3);
}

#[test]
fn test_compute_adapter_config_disabled() {
    let mut config = ComputeAdapterConfig::default();
    config.enabled = false;

    assert!(!config.enabled);
}

#[test]
fn test_compute_adapter_config_discovery_modes() -> SongbirdResult<()> {
    let mut config = ComputeAdapterConfig::default();

    config.discovery_mode = "manual".to_string();
    assert_eq!(config.discovery_mode, "manual");

    config.discovery_mode = "dns".to_string();
    assert_eq!(config.discovery_mode, "dns");
    Ok(())
}

// ============================================================================
// StorageAdapterConfig Tests
// ============================================================================

#[test]
fn test_storage_adapter_config_default() -> SongbirdResult<()> {
    let config = StorageAdapterConfig::default();

    assert!(config.enabled);
    assert_eq!(config.discovery_mode, "auto");
    assert_eq!(config.timeout_ms, 30000);
    Ok(())
}

#[test]
fn test_storage_adapter_config_serialization() -> SongbirdResult<()> {
    let config = StorageAdapterConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;

    assert!(json.contains("enabled"));
    assert!(json.contains("discovery_mode"));
    Ok(())
}

// ============================================================================
// SecurityProviderConfigSecurityConfig Tests
// ============================================================================

#[test]
fn test_security_provider_config_default() {
    let config = SecurityProviderConfigSecurityConfig::default();

    assert!(config.enabled);
    assert!(config.endpoint.is_none());
    assert_eq!(config.priority, 100);
}

#[test]
fn test_security_provider_config_priority() {
    let mut config = SecurityProviderConfigSecurityConfig::default();
    config.priority = 50;

    assert_eq!(config.priority, 50);
}

// ============================================================================
// ComputeProviderConfigComputeConfig Tests
// ============================================================================

#[test]
fn test_compute_provider_config_default() {
    let config = ComputeProviderConfigComputeConfig::default();

    assert!(config.enabled);
    assert_eq!(config.priority, 100);
}

#[test]
fn test_compute_provider_config_with_endpoint() {
    let mut config = ComputeProviderConfigComputeConfig::default();
    config.endpoint = Some("http://compute:9090".to_string());

    assert!(config.endpoint.is_some());
}

// ============================================================================
// StorageProviderConfigStorageConfig Tests
// ============================================================================

#[test]
fn test_storage_provider_config_default() {
    let config = StorageProviderConfigStorageConfig::default();

    assert!(config.enabled);
    assert_eq!(config.priority, 100);
}

// ============================================================================
// AdapterSettings Tests
// ============================================================================

#[test]
fn test_adapter_settings_default() {
    let settings = AdapterSettings::default();

    assert_eq!(settings.default_timeout_ms, 30000);
    assert_eq!(settings.max_concurrent_requests, 100);
    assert!(settings.enable_standalone_failover);
}

#[test]
fn test_adapter_settings_custom_timeout() {
    let mut settings = AdapterSettings::default();
    settings.default_timeout_ms = 45000;

    assert_eq!(settings.default_timeout_ms, 45000);
}

#[test]
fn test_adapter_settings_max_concurrent() {
    let mut settings = AdapterSettings::default();
    settings.max_concurrent_requests = 200;

    assert_eq!(settings.max_concurrent_requests, 200);
}

#[test]
fn test_adapter_settings_failover_disabled() {
    let mut settings = AdapterSettings::default();
    settings.enable_standalone_failover = false;

    assert!(!settings.enable_standalone_failover);
}

// ============================================================================
// CircuitBreakerConfig Tests
// ============================================================================

#[test]
fn test_circuit_breaker_default() {
    let config = CircuitBreakerConfig::default();

    assert!(config.enabled);
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.success_threshold, 3);
}

#[test]
fn test_circuit_breaker_custom_thresholds() {
    let mut config = CircuitBreakerConfig::default();
    config.failure_threshold = 10;
    config.success_threshold = 5;

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.success_threshold, 5);
}

#[test]
fn test_circuit_breaker_timeout() -> SongbirdResult<()> {
    let mut config = CircuitBreakerConfig::default();
    config.timeout = Duration::from_secs(120);

    assert_eq!(config.timeout, Duration::from_secs(120));
    Ok(())
}

#[test]
fn test_circuit_breaker_disabled() -> SongbirdResult<()> {
    let mut config = CircuitBreakerConfig::default();
    config.enabled = false;

    assert!(!config.enabled);
    Ok(())
}

#[test]
fn test_circuit_breaker_serialization() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: CircuitBreakerConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(config.failure_threshold, deserialized.failure_threshold);
    assert_eq!(config.timeout, deserialized.timeout);
    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_adapter_config_integration() {
    let config = UniversalAdapterConfig {
        security_adapters: SecurityAdapterConfig {
            enabled: true,
            discovery_mode: "manual".to_string(),
            endpoint: Some("http://security:8080".to_string()),
            health_check: HealthCheckConfig::default(),
            timeout_ms: 45000,
            retry_count: 5,
            security_provider_config: SecurityProviderConfigSecurityConfig::default(),
        },
        compute_adapters: ComputeAdapterConfig::default(),
        storage_adapters: StorageAdapterConfig::default(),
        settings: AdapterSettings {
            default_timeout_ms: 40000,
            max_concurrent_requests: 150,
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                failure_threshold: 8,
                timeout: Duration::from_secs(90),
                success_threshold: 4,
                half_open_max_requests: 10,
            },
            enable_standalone_failover: true,
        },
    };

    assert_eq!(config.security_adapters.timeout_ms, 45000);
    assert_eq!(config.settings.default_timeout_ms, 40000);
    assert_eq!(config.settings.circuit_breaker.failure_threshold, 8);
}

#[test]
fn test_adapter_config_all_disabled() {
    let mut config = UniversalAdapterConfig::default();
    config.security_adapters.enabled = false;
    config.compute_adapters.enabled = false;
    config.storage_adapters.enabled = false;

    assert!(!config.security_adapters.enabled);
    assert!(!config.compute_adapters.enabled);
    assert!(!config.storage_adapters.enabled);
}
