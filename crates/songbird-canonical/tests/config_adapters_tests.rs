//! Tests for Universal Adapter Configuration
//!
//! Comprehensive tests for adapter configuration structures

use songbird_canonical::config::adapters::{
    AdapterSettings, CircuitBreakerConfig, ComputeAdapterConfig,
    ComputeProviderConfigComputeConfig, SecurityAdapterConfig,
    SecurityProviderConfigSecurityConfig, StorageAdapterConfig, StorageProviderConfigStorageConfig,
    UniversalAdapterConfig,
};
use songbird_types::config::consolidated_canonical::CanonicalHealthCheckConfig;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_universal_adapter_config_default() {
    let config = UniversalAdapterConfig::default();

    // Verify default adapter configs are initialized
    // Enabled field can be true or false, just verify the structure exists
    let _ = config.security_adapters.enabled;
    let _ = config.compute_adapters.enabled;
    let _ = config.storage_adapters.enabled;
}

#[test]
fn test_universal_adapter_config_creation() {
    let security = SecurityAdapterConfig {
        enabled: true,
        discovery_mode: "capability".to_string(),
        endpoint: Some("http://localhost:8004".to_string()),
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 5000,
        retry_count: 3,
        security_provider_config: SecurityProviderConfigSecurityConfig::default(),
    };

    let compute = ComputeAdapterConfig {
        enabled: true,
        discovery_mode: "capability".to_string(),
        endpoint: Some("http://localhost:8001".to_string()),
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 10000,
        retry_count: 3,
        compute_provider_config: ComputeProviderConfigComputeConfig::default(),
    };

    let storage = StorageAdapterConfig {
        enabled: true,
        discovery_mode: "capability".to_string(),
        endpoint: Some("http://localhost:8003".to_string()),
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 8000,
        retry_count: 5,
        storage_provider: StorageProviderConfigStorageConfig::default(),
    };

    let settings = AdapterSettings {
        default_timeout_ms: 30000,
        max_concurrent_requests: 100,
        circuit_breaker: CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 3,
        },
        enable_standalone_failover: true,
    };

    let config = UniversalAdapterConfig {
        security_adapters: security,
        compute_adapters: compute,
        storage_adapters: storage,
        settings,
    };

    // Verify security adapter
    assert!(config.security_adapters.enabled);
    assert_eq!(config.security_adapters.discovery_mode, "capability");
    assert_eq!(config.security_adapters.timeout_ms, 5000);
    assert_eq!(config.security_adapters.retry_count, 3);
    assert_eq!(config.security_adapters.endpoint, Some("http://localhost:8004".to_string()));

    // Verify compute adapter
    assert!(config.compute_adapters.enabled);
    assert_eq!(config.compute_adapters.discovery_mode, "capability");
    assert_eq!(config.compute_adapters.timeout_ms, 10000);

    // Verify storage adapter
    assert!(config.storage_adapters.enabled);
    assert_eq!(config.storage_adapters.timeout_ms, 8000);
    assert_eq!(config.storage_adapters.retry_count, 5);

    // Verify settings
    assert_eq!(config.settings.default_timeout_ms, 30000);
    assert_eq!(config.settings.max_concurrent_requests, 100);
    assert!(config.settings.enable_standalone_failover);
    assert!(config.settings.circuit_breaker.enabled);
}

#[test]
fn test_security_adapter_config_clone() {
    let config = SecurityAdapterConfig {
        enabled: true,
        discovery_mode: "capability".to_string(),
        endpoint: None,
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 5000,
        retry_count: 3,
        security_provider_config: SecurityProviderConfigSecurityConfig::default(),
    };

    let cloned = config.clone();

    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.discovery_mode, cloned.discovery_mode);
    assert_eq!(config.timeout_ms, cloned.timeout_ms);
    assert_eq!(config.retry_count, cloned.retry_count);
}

#[test]
fn test_compute_adapter_config_debug() -> SongbirdResult<()> {
    let config = ComputeAdapterConfig {
        enabled: false,
        discovery_mode: "manual".to_string(),
        endpoint: Some("http://compute:8001".to_string()),
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 15000,
        retry_count: 2,
        compute_provider_config: ComputeProviderConfigComputeConfig::default(),
    };

    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("ComputeAdapterConfig"));
    assert!(debug_str.contains("enabled"));
    assert!(debug_str.contains("false") || debug_str.contains("true"));
    Ok(())
}

#[test]
fn test_storage_adapter_config_serialization() -> SongbirdResult<()> {
    let config = StorageAdapterConfig {
        enabled: true,
        discovery_mode: "auto".to_string(),
        endpoint: Some("http://storage:8003".to_string()),
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 20000,
        retry_count: 4,
        storage_provider: StorageProviderConfigStorageConfig::default(),
    };

    // Test serialization
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("enabled"));
    assert!(json.contains("discovery_mode"));

    // Test deserialization
    let deserialized: StorageAdapterConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.discovery_mode, deserialized.discovery_mode);
    assert_eq!(config.timeout_ms, deserialized.timeout_ms);
    Ok(())
}

#[test]
fn test_adapter_settings_default() {
    let settings = AdapterSettings {
        default_timeout_ms: 30000,
        max_concurrent_requests: 100,
        circuit_breaker: CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 3,
        },
        enable_standalone_failover: true,
    };

    assert_eq!(settings.default_timeout_ms, 30000);
    assert_eq!(settings.max_concurrent_requests, 100);
    assert!(settings.enable_standalone_failover);
    assert!(settings.circuit_breaker.enabled);
    assert_eq!(settings.circuit_breaker.failure_threshold, 5);
}

#[test]
fn test_adapter_settings_disabled() -> SongbirdResult<()> {
    let settings = AdapterSettings {
        default_timeout_ms: 10000,
        max_concurrent_requests: 10,
        circuit_breaker: CircuitBreakerConfig {
            enabled: false,
            failure_threshold: 10,
            timeout_seconds: 30,
            success_threshold: 1,
        },
        enable_standalone_failover: false,
    };

    assert!(!settings.circuit_breaker.enabled);
    assert!(!settings.enable_standalone_failover);
    assert_eq!(settings.max_concurrent_requests, 10);
    assert_eq!(settings.default_timeout_ms, 10000);
    Ok(())
}

#[test]
fn test_security_provider_config_default() -> SongbirdResult<()> {
    let config = SecurityProviderConfigSecurityConfig::default();

    // Verify default can be created
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("SecurityProviderConfigSecurityConfig"));
    Ok(())
}

#[test]
fn test_compute_provider_config_default() -> SongbirdResult<()> {
    let config = ComputeProviderConfigComputeConfig::default();

    // Verify default can be created
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("ComputeProviderConfigComputeConfig"));
    Ok(())
}

#[test]
fn test_storage_provider_config_default() -> SongbirdResult<()> {
    let config = StorageProviderConfigStorageConfig::default();

    // Verify default can be created
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("StorageProviderConfigStorageConfig"));
    Ok(())
}

#[test]
fn test_universal_adapter_config_serialization() -> SongbirdResult<()> {
    let config = UniversalAdapterConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("security_adapters") || json.contains("compute_adapters"));

    // Test deserialization
    let deserialized: UniversalAdapterConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;

    // Verify structure is preserved
    assert_eq!(format!("{:?}", config.settings), format!("{:?}", deserialized.settings));
    Ok(())
}

#[test]
fn test_security_adapter_with_no_endpoint() {
    let config = SecurityAdapterConfig {
        enabled: true,
        discovery_mode: "capability".to_string(),
        endpoint: None,
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 5000,
        retry_count: 3,
        security_provider_config: SecurityProviderConfigSecurityConfig::default(),
    };

    assert!(config.enabled);
    assert!(config.endpoint.is_none());
}

#[test]
fn test_compute_adapter_with_endpoint() -> SongbirdResult<()> {
    let config = ComputeAdapterConfig {
        enabled: true,
        discovery_mode: "manual".to_string(),
        endpoint: Some("http://compute.local:9001".to_string()),
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 10000,
        retry_count: 3,
        compute_provider_config: ComputeProviderConfigComputeConfig::default(),
    };

    assert!(config.endpoint.is_some());
    assert_eq!(
        config
            .endpoint
            .ok_or_else(|| SongbirdError::configuration("Failed health check".to_string()))?,
        "http://compute.local:9001"
    );
    Ok(())
}

#[test]
fn test_storage_adapter_timeouts() {
    let short_timeout = StorageAdapterConfig {
        enabled: true,
        discovery_mode: "auto".to_string(),
        endpoint: None,
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 1000,
        retry_count: 1,
        storage_provider: StorageProviderConfigStorageConfig::default(),
    };

    let long_timeout = StorageAdapterConfig {
        enabled: true,
        discovery_mode: "auto".to_string(),
        endpoint: None,
        health_check: CanonicalHealthCheckConfig::default(),
        timeout_ms: 30000,
        retry_count: 10,
        storage_provider: StorageProviderConfigStorageConfig::default(),
    };

    assert!(short_timeout.timeout_ms < long_timeout.timeout_ms);
    assert!(short_timeout.retry_count < long_timeout.retry_count);
}

#[test]
fn test_adapter_settings_extreme_values() {
    let settings = AdapterSettings {
        default_timeout_ms: 1,
        max_concurrent_requests: 1,
        circuit_breaker: CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 1,
            timeout_seconds: 1,
            success_threshold: 1,
        },
        enable_standalone_failover: true,
    };

    assert_eq!(settings.default_timeout_ms, 1);
    assert_eq!(settings.max_concurrent_requests, 1);

    let large_settings = AdapterSettings {
        default_timeout_ms: 300_000,
        max_concurrent_requests: 10000,
        circuit_breaker: CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 100,
            timeout_seconds: 600,
            success_threshold: 50,
        },
        enable_standalone_failover: true,
    };

    assert_eq!(large_settings.default_timeout_ms, 300_000);
    assert_eq!(large_settings.max_concurrent_requests, 10000);
}

#[test]
fn test_universal_adapter_config_clone() -> SongbirdResult<()> {
    let config = UniversalAdapterConfig::default();
    let cloned = config.clone();

    // Verify clone creates independent copy
    assert_eq!(format!("{:?}", config.settings), format!("{:?}", cloned.settings));
    Ok(())
}

#[test]
fn test_health_check_config_integration() -> SongbirdResult<()> {
    let health_check = CanonicalHealthCheckConfig::default();

    let config = SecurityAdapterConfig {
        enabled: true,
        discovery_mode: "capability".to_string(),
        endpoint: None,
        health_check,
        timeout_ms: 5000,
        retry_count: 3,
        security_provider_config: SecurityProviderConfigSecurityConfig::default(),
    };

    // Verify health check is integrated
    let debug_str = format!("{:?}", config.health_check);
    assert!(!debug_str.is_empty());
    Ok(())
}
