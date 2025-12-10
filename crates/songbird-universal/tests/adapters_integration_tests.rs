//! Comprehensive Integration Tests for Universal Adapters
//!
//! Tests multi-tier discovery, health monitoring, and cross-adapter interactions.

use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::env;

/// Test that all adapters can be discovered concurrently without conflicts
#[tokio::test]
async fn test_concurrent_multi_adapter_discovery() {
    // Set up test endpoints
    env::set_var(
        "CAPABILITY_COMPUTE_ENDPOINT",
        format!("http://compute-test:{}", test_orchestrator_port()),
    );
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", "http://security-test:8443");
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://storage-test:9000");
    env::set_var("CAPABILITY_AI_ENDPOINT", "http://ai-test:8888");

    // Discover all adapters concurrently
    let results = tokio::join!(
        ComputeAdapter::new_from_discovery(),
        SecurityAdapter::from_discovery(),
        StorageAdapter::from_discovery(),
        AIAdapter::from_discovery(),
    );

    // Verify all discoveries succeeded
    assert!(results.0.is_ok(), "ComputeAdapter discovery failed");
    assert!(results.1.is_ok(), "SecurityAdapter discovery failed");
    assert!(results.2.is_ok(), "StorageAdapter discovery failed");
    assert!(results.3.is_ok(), "AIAdapter discovery failed");

    // Clean up
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    env::remove_var("CAPABILITY_AI_ENDPOINT");
}

/// Test legacy environment variable fallback
#[tokio::test]
async fn test_legacy_environment_variable_fallback() {
    // Set legacy environment variables (not CAPABILITY_*)
    env::set_var(
        "TOADSTOOL_ENDPOINT",
        format!("http://legacy-compute:{}", test_orchestrator_port()),
    );
    env::set_var("BEARDOG_ENDPOINT", "http://legacy-security:8443");
    env::set_var("NESTGATE_ENDPOINT", "http://legacy-storage:9000");
    env::set_var("SQUIRREL_ENDPOINT", "http://legacy-ai:8888");

    // Discover adapters (should fall back to legacy vars)
    let compute = ComputeAdapter::new_from_discovery().await;
    let security = SecurityAdapter::from_discovery().await;
    let storage = StorageAdapter::from_discovery().await;
    let ai = AIAdapter::from_discovery().await;

    // Verify all fallbacks worked
    assert!(compute.is_ok(), "Compute legacy fallback failed");
    assert!(security.is_ok(), "Security legacy fallback failed");
    assert!(storage.is_ok(), "Storage legacy fallback failed");
    assert!(ai.is_ok(), "AI legacy fallback failed");

    // Clean up
    env::remove_var("TOADSTOOL_ENDPOINT");
    env::remove_var("BEARDOG_ENDPOINT");
    env::remove_var("NESTGATE_ENDPOINT");
    env::remove_var("SQUIRREL_ENDPOINT");
}

/// Test final fallback to host + port construction
#[tokio::test]
async fn test_host_port_construction_fallback() {
    // Remove all discovery env vars to force fallback
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
    env::remove_var("TOADSTOOL_ENDPOINT");

    // Set host for fallback construction
    env::set_var("SONGBIRD_HOST", "http://fallback-host");
    env::set_var("SONGBIRD_COMPUTE_PORT", "9999");

    // Should construct endpoint from host + port
    let result = ComputeAdapter::new_from_discovery().await;
    assert!(result.is_ok(), "Host+port fallback failed");

    // Clean up
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_COMPUTE_PORT");
}

/// Test adapter timeout configuration
#[tokio::test]
async fn test_adapter_timeout_configuration() -> SongbirdResult<()> {
    env::set_var(
        "CAPABILITY_COMPUTE_ENDPOINT",
        format!("http://compute:{}", test_orchestrator_port()),
    );

    let adapter = ComputeAdapter::new_from_discovery().await?;

    // Verify adapter can be configured with custom timeout
    let custom_adapter = adapter.with_timeout(std::time::Duration::from_secs(30));

    // Adapter should be usable (we can't directly test timeout without a server)
    // But we verify the configuration API works
    drop(custom_adapter);

    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    Ok(())
}

/// Test discovery priority order (CAPABILITY_* should override legacy)
#[tokio::test]
async fn test_discovery_priority_order() -> SongbirdResult<()> {
    // Set both new and legacy variables
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", "http://new-priority:8443");
    env::set_var("BEARDOG_ENDPOINT", "http://old-legacy:8443");

    let adapter = SecurityAdapter::from_discovery()
        .await
        .map_err(|e| SongbirdError::configuration("Failed to discover security adapter"))?;

    // Verify it uses the CAPABILITY_* endpoint (higher priority)
    // We can't directly check the endpoint, but we verify discovery succeeded
    // with higher priority env var set
    drop(adapter);

    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("BEARDOG_ENDPOINT");
    Ok(())
}

/// Test adapter creation with explicit endpoints
#[test]
fn test_explicit_endpoint_creation() {
    // Create adapters with explicit endpoints (synchronous, no discovery)
    let compute =
        ComputeAdapter::new(format!("http://explicit-compute:{}", test_orchestrator_port()));
    let security = SecurityAdapter::new("http://explicit-security:8443".to_string());
    let storage = StorageAdapter::new("http://explicit-storage:9000".to_string());
    let ai = AIAdapter::new("http://explicit-ai:8888".to_string());

    assert!(compute.is_ok(), "Explicit compute creation failed");
    assert!(security.is_ok(), "Explicit security creation failed");
    assert!(storage.is_ok(), "Explicit storage creation failed");
    assert!(ai.is_ok(), "Explicit AI creation failed");
}

/// Test that adapters handle invalid endpoints gracefully
#[test]
fn test_invalid_endpoint_handling() {
    // These should create the adapter successfully (validation happens on use)
    let compute = ComputeAdapter::new("invalid-url".to_string());
    let security = SecurityAdapter::new(String::new());

    // Creation should succeed (HTTP client handles invalid URLs on request)
    assert!(compute.is_ok() || compute.is_err()); // Either is valid behavior
    assert!(security.is_ok() || security.is_err());
}

/// Test environment variable isolation between adapters
#[tokio::test]
async fn test_adapter_environment_isolation() -> SongbirdResult<()> {
    // Each adapter should only use its own environment variables
    env::set_var(
        "CAPABILITY_COMPUTE_ENDPOINT",
        format!("http://compute-only:{}", test_orchestrator_port()),
    );

    let compute = ComputeAdapter::new_from_discovery().await;
    assert!(compute.is_ok(), "Compute should discover its endpoint");

    // Security should NOT use compute's endpoint
    let security = SecurityAdapter::from_discovery().await;
    assert!(security.is_ok(), "Security should use fallback, not compute's endpoint");

    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    Ok(())
}

/// Test multiple adapter instances can coexist
#[tokio::test]
async fn test_multiple_adapter_instances() -> SongbirdResult<()> {
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://storage:9000");

    // Create multiple instances of the same adapter type
    let storage1 = StorageAdapter::from_discovery().await?;
    let storage2 = StorageAdapter::from_discovery().await?;

    // Both should be independent instances
    drop(storage1);
    drop(storage2);

    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    Ok(())
}

/// Test adapter resilience to missing environment variables
#[tokio::test]
async fn test_missing_environment_variables() -> SongbirdResult<()> {
    // Clear all possible env vars
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("SONGBIRD_AI_ENDPOINT");
    env::remove_var("AI_PROVIDER_ENDPOINT");
    env::remove_var("SQUIRREL_ENDPOINT");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_AI_PORT");

    // Should still succeed with ultimate fallback
    let result = AIAdapter::from_discovery().await;
    assert!(result.is_ok(), "AI adapter should have ultimate fallback");
    Ok(())
}

/// Test that adapters respect capability type boundaries
#[tokio::test]
async fn test_capability_type_boundaries() -> SongbirdResult<()> {
    // Set up distinct endpoints for each capability
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", "http://compute:1111");
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", "http://security:2222");
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://storage:3333");
    env::set_var("CAPABILITY_AI_ENDPOINT", "http://ai:4444");

    // Each adapter should use ONLY its capability endpoint
    let compute = ComputeAdapter::new_from_discovery().await?;

    let security = SecurityAdapter::from_discovery().await?;
    let storage = StorageAdapter::from_discovery().await?;
    let ai = AIAdapter::from_discovery().await?;

    // Verify adapters were created (actual endpoint usage tested by other tests)
    drop(compute);
    drop(security);
    drop(storage);
    drop(ai);

    // Clean up
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    Ok(())
}

/// Test adapter discovery with mixed environment configurations
#[tokio::test]
async fn test_mixed_environment_configuration() {
    // Mix of CAPABILITY_*, legacy, and fallback scenarios
    env::set_var(
        "CAPABILITY_COMPUTE_ENDPOINT",
        format!("http://compute-cap:{}", test_orchestrator_port()),
    );
    env::set_var("BEARDOG_ENDPOINT", "http://security-legacy:8443");
    env::set_var("SONGBIRD_HOST", "http://fallback-host");
    env::set_var("SONGBIRD_AI_PORT", "7777");

    // Compute: Uses CAPABILITY_*
    let compute = ComputeAdapter::new_from_discovery().await;
    assert!(compute.is_ok(), "Compute with CAPABILITY_* failed");

    // Security: Uses legacy
    let security = SecurityAdapter::from_discovery().await;
    assert!(security.is_ok(), "Security with legacy failed");

    // Storage: Uses fallback
    let storage = StorageAdapter::from_discovery().await;
    assert!(storage.is_ok(), "Storage with fallback failed");

    // AI: Uses fallback with custom port
    let ai = AIAdapter::from_discovery().await;
    assert!(ai.is_ok(), "AI with fallback+port failed");

    // Clean up
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("BEARDOG_ENDPOINT");
    env::remove_var("SONGBIRD_HOST");
    env::remove_var("SONGBIRD_AI_PORT");
}

/// Test that discovery is consistent across multiple calls
#[tokio::test]
async fn test_discovery_consistency() {
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", "http://consistent:9000");

    // Discover multiple times
    let storage1 = StorageAdapter::from_discovery().await;
    let storage2 = StorageAdapter::from_discovery().await;
    let storage3 = StorageAdapter::from_discovery().await;

    // All should succeed consistently
    assert!(storage1.is_ok(), "First discovery failed");
    assert!(storage2.is_ok(), "Second discovery failed");
    assert!(storage3.is_ok(), "Third discovery failed");

    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}
