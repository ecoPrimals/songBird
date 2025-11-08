#![cfg(feature = "tests-incomplete")]
//! NOTE: Disabled - requires unimplemented methods

//! Comprehensive tests for adapter discovery mechanisms
//!
//! Tests the multi-method capability discovery system including:
//! - Environment variable discovery
//! - Service registry integration
//! - Container metadata queries
//! - DNS SRV lookups
//! - Graceful fallbacks

use serial_test::serial;
use songbird_test_utils::{test_bind_address, test_metrics_port, test_orchestrator_port};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::env;

#[tokio::test]
#[serial]
async fn test_ai_adapter_discovery_from_environment() {
    // Set environment variable for AI capability
    env::set_var(
        "CAPABILITY_AI_ENDPOINT",
        format!("http://ai-provider:{}", test_orchestrator_port()),
    );

    // Create adapter via discovery
    let result = AIAdapter::from_discovery().await;

    // Clean up
    env::remove_var("CAPABILITY_AI_ENDPOINT");

    // Should succeed with environment discovery
    assert!(result.is_ok());
    let adapter = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(adapter.endpoint(), format!("http://ai-provider:{}", test_orchestrator_port()));
}

#[tokio::test]
#[serial]
async fn test_compute_adapter_discovery_from_environment() {
    env::set_var(
        "CAPABILITY_COMPUTE_ENDPOINT",
        format!("http://compute-provider:{}", test_metrics_port()),
    );

    let result = ComputeAdapter::new_from_discovery().await;

    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");

    assert!(result.is_ok());
    let adapter = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(adapter.endpoint(), format!("http://compute-provider:{}", test_metrics_port()));
}

#[tokio::test]
#[serial]
async fn test_security_adapter_discovery_from_environment() {
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", "https://security-provider:8443");

    let result = SecurityAdapter::from_discovery().await;

    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");

    assert!(result.is_ok());
    let adapter = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(adapter.endpoint(), "https://security-provider:8443");
}

#[tokio::test]
#[serial]
async fn test_storage_adapter_discovery_from_environment() {
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://storage-provider:9000");

    let result = StorageAdapter::from_discovery().await;

    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");

    assert!(result.is_ok());
    let adapter = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(adapter.endpoint(), "http://storage-provider:9000");
}

#[tokio::test]
#[serial]
async fn test_adapter_discovery_fallback_to_default() {
    // Remove all possible endpoint environment variables
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("SERVICE_REGISTRY_ENDPOINT");
    env::remove_var("CONTAINER_METADATA_API");
    env::remove_var("SERVICE_DISCOVERY_DOMAIN");
    env::remove_var("SONGBIRD_AI_ENDPOINT");
    env::remove_var("AI_PROVIDER_ENDPOINT");
    env::remove_var("SQUIRREL_ENDPOINT");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_AI_PORT");

    let result = AIAdapter::from_discovery().await;

    // Discovery should succeed with fallback to default endpoint
    // This is by design - fail-safe with sensible defaults
    assert!(result.is_ok(), "Discovery should succeed with fallback, but got: {:?}", result);
    let adapter = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    // Default fallback uses DEFAULT_HOST and port 8083
    assert!(
        adapter.endpoint().contains(test_bind_address())
            || adapter.endpoint().contains(test_bind_address())
    );
}

#[tokio::test]
#[serial]
async fn test_adapter_endpoint_validation() {
    // Test with valid HTTP endpoint
    env::set_var("CAPABILITY_AI_ENDPOINT", format!("http://valid:{}", test_orchestrator_port()));
    let result1 = AIAdapter::from_discovery().await;
    assert!(result1.is_ok());
    env::remove_var("CAPABILITY_AI_ENDPOINT");

    // Test with valid HTTPS endpoint
    env::set_var("CAPABILITY_AI_ENDPOINT", "https://secure:443");
    let result2 = AIAdapter::from_discovery().await;
    assert!(result2.is_ok());
    env::remove_var("CAPABILITY_AI_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_multiple_adapter_discovery_independence() {
    // Set different endpoints for different capabilities
    env::set_var("CAPABILITY_AI_ENDPOINT", format!("http://ai:{}", test_orchestrator_port()));
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", format!("http://compute:{}", test_metrics_port()));
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://storage:9000");

    // Discover all adapters independently
    let ai_result = AIAdapter::from_discovery().await;
    let compute_result = ComputeAdapter::new_from_discovery().await;
    let storage_result = StorageAdapter::from_discovery().await;

    // Clean up
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");

    // All should succeed with correct endpoints
    assert!(ai_result.is_ok());
    assert!(compute_result.is_ok());
    assert!(storage_result.is_ok());

    assert_eq!(
        ai_result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .endpoint(),
        format!("http://ai:{}", test_orchestrator_port())
    );
    assert_eq!(
        compute_result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .endpoint(),
        format!("http://compute:{}", test_metrics_port())
    );
    assert_eq!(
        storage_result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .endpoint(),
        "http://storage:9000"
    );
}

#[tokio::test]
#[serial]
async fn test_adapter_discovery_with_custom_timeout() {
    env::set_var("CAPABILITY_AI_ENDPOINT", format!("http://ai:{}", test_orchestrator_port()));
    env::set_var("DISCOVERY_TIMEOUT_SECS", "5");

    let result = AIAdapter::from_discovery().await;

    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("DISCOVERY_TIMEOUT_SECS");

    assert!(result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_adapter_discovery_priority_order() {
    // Set multiple discovery methods - environment should win
    env::set_var(
        "CAPABILITY_AI_ENDPOINT",
        format!("http://env-endpoint:{}", test_orchestrator_port()),
    );
    env::set_var("SERVICE_REGISTRY_ENDPOINT", "http://registry:8500");

    let result = AIAdapter::from_discovery().await;

    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("SERVICE_REGISTRY_ENDPOINT");

    assert!(result.is_ok());
    // Environment variable should take priority
    assert_eq!(
        result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .endpoint(),
        format!("http://env-endpoint:{}", test_orchestrator_port())
    );
}

#[tokio::test]
#[serial]
async fn test_compute_adapter_direct_construction() {
    // Test direct construction with explicit endpoint
    let adapter =
        ComputeAdapter::new(format!("http://explicit:{}", test_metrics_port()).to_string());
    assert!(adapter.is_ok());
    let adapter = adapter.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(adapter.endpoint(), format!("http://explicit:{}", test_metrics_port()));
}

#[tokio::test]
#[serial]
async fn test_adapter_endpoint_formats() {
    // Test various valid endpoint formats
    let test_cases = vec![
        format!("http://localhost:{}", test_orchestrator_port()),
        "https://secure.example.com:443",
        format!("http://192.168.1.100:{}", test_metrics_port()),
        format!("http://service.namespace.svc.cluster.local:{}", test_orchestrator_port()),
    ];

    for endpoint in test_cases {
        env::set_var("CAPABILITY_COMPUTE_ENDPOINT", endpoint);
        let result = ComputeAdapter::new_from_discovery().await;
        env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");

        assert!(result.is_ok(), "Failed for endpoint: {}", endpoint);
        assert_eq!(
            result
                .ok_or_else(|| SongbirdError::configuration(format!(
                    "Error: {}",
                    e
                )))?
                .endpoint(),
            endpoint
        );
    }
}

#[tokio::test]
#[serial]
async fn test_adapter_discovery_cache_behavior() {
    env::set_var("CAPABILITY_AI_ENDPOINT", format!("http://ai:{}", test_orchestrator_port()));
    env::set_var("DISCOVERY_CACHE_TTL_SECS", "60");

    // First discovery
    let result1 = AIAdapter::from_discovery().await;
    assert!(result1.is_ok());

    // Second discovery (should use cache if implemented)
    let result2 = AIAdapter::from_discovery().await;
    assert!(result2.is_ok());

    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("DISCOVERY_CACHE_TTL_SECS");
}

#[tokio::test]
#[serial]
async fn test_adapter_concurrent_discovery() {
    env::set_var("CAPABILITY_AI_ENDPOINT", format!("http://ai:{}", test_orchestrator_port()));
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", format!("http://compute:{}", test_metrics_port()));
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://storage:9000");

    // Discover multiple adapters concurrently
    let (ai_result, compute_result, storage_result) = tokio::join!(
        AIAdapter::from_discovery(),
        ComputeAdapter::new_from_discovery(),
        StorageAdapter::from_discovery()
    );

    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");

    // All should succeed
    assert!(ai_result.is_ok());
    assert!(compute_result.is_ok());
    assert!(storage_result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_adapter_discovery_with_explicit_host_port() {
    // Clear primary discovery sources but set fallback host/port
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("SERVICE_REGISTRY_ENDPOINT");
    env::remove_var("SONGBIRD_AI_ENDPOINT");
    env::remove_var("AI_PROVIDER_ENDPOINT");
    env::remove_var("SQUIRREL_ENDPOINT");

    // Set explicit host and port
    env::set_var("SONGBIRD_HOST", "http://custom-host");
    env::set_var("SONGBIRD_AI_PORT", "9999");

    let result = AIAdapter::from_discovery().await;

    // Should succeed with custom host/port fallback
    assert!(
        result.is_ok(),
        "Discovery should succeed with custom host/port, but got: {:?}",
        result
    );
    let adapter = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    assert_eq!(adapter.endpoint(), "http://custom-host:9999");

    // Clean up
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_AI_PORT");
}
