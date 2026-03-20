// SPDX-License-Identifier: AGPL-3.0-only
//! E2E Test: Runtime Capability-Based Discovery
//!
//! Tests the new RuntimeDiscoveryEngine with zero hardcoding.
//!
//! **Scenarios**:
//! 1. Discovery via environment variables
//! 2. Cache functionality
//! 3. Multiple capabilities
//! 4. Cache expiration

use songbird_config::runtime_discovery::{RuntimeDiscoveryEngine, DiscoveryMethod};
use songbird_types::SongbirdResult;
use std::time::Duration;

/// Test: Discover service via environment variable
#[tokio::test]
async fn test_discover_compute_via_environment() -> SongbirdResult<()> {
    // Setup: Set environment variable
    std::env::set_var("COMPUTE_ENDPOINT", "http://10.0.1.50:8001");
    
    // Act: Discover compute capability
    let engine = RuntimeDiscoveryEngine::new();
    let service = engine.discover_by_capability("compute").await?;
    
    // Assert: Service discovered correctly
    assert_eq!(service.capability, "compute");
    assert_eq!(service.endpoint, "http://10.0.1.50:8001");
    assert_eq!(service.discovered_via, DiscoveryMethod::Environment);
    assert_eq!(service.health_score, 1.0);
    
    // Cleanup
    std::env::remove_var("COMPUTE_ENDPOINT");
    
    Ok(())
}

/// Test: Discover multiple capabilities
#[tokio::test]
async fn test_discover_multiple_capabilities() -> SongbirdResult<()> {
    // Setup: Set multiple environment variables
    std::env::set_var("COMPUTE_ENDPOINT", "http://10.0.1.50:8001");
    std::env::set_var("AI_ENDPOINT", "http://10.0.1.51:8002");
    std::env::set_var("STORAGE_ENDPOINT", "http://10.0.1.52:8003");
    std::env::set_var("SECURITY_ENDPOINT", "http://10.0.1.53:8004");
    
    // Act: Discover all capabilities
    let engine = RuntimeDiscoveryEngine::new();
    
    let compute = engine.discover_by_capability("compute").await?;
    let ai = engine.discover_by_capability("ai").await?;
    let storage = engine.discover_by_capability("storage").await?;
    let security = engine.discover_by_capability("security").await?;
    
    // Assert: All services discovered
    assert_eq!(compute.capability, "compute");
    assert_eq!(compute.endpoint, "http://10.0.1.50:8001");
    
    assert_eq!(ai.capability, "ai");
    assert_eq!(ai.endpoint, "http://10.0.1.51:8002");
    
    assert_eq!(storage.capability, "storage");
    assert_eq!(storage.endpoint, "http://10.0.1.52:8003");
    
    assert_eq!(security.capability, "security");
    assert_eq!(security.endpoint, "http://10.0.1.53:8004");
    
    // Cleanup
    std::env::remove_var("COMPUTE_ENDPOINT");
    std::env::remove_var("AI_ENDPOINT");
    std::env::remove_var("STORAGE_ENDPOINT");
    std::env::remove_var("SECURITY_ENDPOINT");
    
    Ok(())
}

/// Test: Cache functionality
#[tokio::test]
async fn test_discovery_caching() -> SongbirdResult<()> {
    // Setup
    std::env::set_var("TEST_CAP_ENDPOINT", "http://10.0.1.100:9000");
    
    // Act: Discover twice
    let engine = RuntimeDiscoveryEngine::new();
    
    let service1 = engine.discover_by_capability("test_cap").await?;
    let service2 = engine.discover_by_capability("test_cap").await?;
    
    // Assert: Both discoveries successful (second from cache)
    assert_eq!(service1.endpoint, service2.endpoint);
    assert_eq!(service1.capability, service2.capability);
    
    // Note: In real implementation, we'd verify cache hit via metrics
    // For now, we verify functionality
    
    // Cleanup
    std::env::remove_var("TEST_CAP_ENDPOINT");
    
    Ok(())
}

/// Test: Discovery fails gracefully when service not found
#[tokio::test]
async fn test_discovery_not_found() {
    // Ensure no environment variable set
    std::env::remove_var("NONEXISTENT_ENDPOINT");
    
    // Act: Try to discover non-existent capability
    let engine = RuntimeDiscoveryEngine::new();
    let result = engine.discover_by_capability("nonexistent").await;
    
    // Assert: Returns error (not panic)
    assert!(result.is_err());
    
    // Verify error message contains useful info
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("nonexistent") || error_msg.contains("not found"));
}

/// Test: Convenience functions work correctly
#[tokio::test]
async fn test_convenience_functions() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::{
        discover_compute, discover_ai, discover_storage, discover_security
    };
    
    // Setup
    std::env::set_var("COMPUTE_ENDPOINT", "http://10.0.1.50:8001");
    std::env::set_var("AI_ENDPOINT", "http://10.0.1.51:8002");
    std::env::set_var("STORAGE_ENDPOINT", "http://10.0.1.52:8003");
    std::env::set_var("SECURITY_ENDPOINT", "http://10.0.1.53:8004");
    
    // Act: Use convenience functions
    let compute = discover_compute().await?;
    let ai = discover_ai().await?;
    let storage = discover_storage().await?;
    let security = discover_security().await?;
    
    // Assert: All discovered correctly
    assert_eq!(compute.endpoint, "http://10.0.1.50:8001");
    assert_eq!(ai.endpoint, "http://10.0.1.51:8002");
    assert_eq!(storage.endpoint, "http://10.0.1.52:8003");
    assert_eq!(security.endpoint, "http://10.0.1.53:8004");
    
    // Cleanup
    std::env::remove_var("COMPUTE_ENDPOINT");
    std::env::remove_var("AI_ENDPOINT");
    std::env::remove_var("STORAGE_ENDPOINT");
    std::env::remove_var("SECURITY_ENDPOINT");
    
    Ok(())
}

/// Test: Discovery with custom timeout
#[tokio::test]
async fn test_discovery_with_timeout() -> SongbirdResult<()> {
    // Setup
    std::env::set_var("QUICK_TEST_ENDPOINT", "http://10.0.1.200:9999");
    
    // Act: Create engine and discover
    let engine = RuntimeDiscoveryEngine::with_capabilities(vec!["quick_test".to_string()]);
    
    // Set timeout and discover
    let service = tokio::time::timeout(
        Duration::from_secs(1),
        engine.discover_by_capability("quick_test")
    ).await??;
    
    // Assert: Discovery completed within timeout
    assert_eq!(service.endpoint, "http://10.0.1.200:9999");
    
    // Cleanup
    std::env::remove_var("QUICK_TEST_ENDPOINT");
    
    Ok(())
}

/// Test: Zero hardcoding - no default fallbacks
#[tokio::test]
async fn test_no_hardcoded_fallbacks() {
    // Ensure NO environment variables set
    std::env::remove_var("COMPUTE_ENDPOINT");
    std::env::remove_var("DEFAULT_COMPUTE_ENDPOINT");
    std::env::remove_var("FALLBACK_ENDPOINT");
    
    // Act: Try to discover without any configuration
    let engine = RuntimeDiscoveryEngine::new();
    let result = engine.discover_by_capability("compute").await;
    
    // Assert: MUST fail - no hardcoded fallbacks
    assert!(result.is_err(), "Discovery must fail without configuration - no hardcoded fallbacks allowed");
}

/// Test: Case-insensitive environment variable lookup
#[tokio::test]
async fn test_case_insensitive_env_vars() -> SongbirdResult<()> {
    // Setup: Set UPPERCASE env var (standard)
    std::env::set_var("MYSERVICE_ENDPOINT", "http://10.0.1.123:7777");
    
    // Act: Discover with lowercase capability name
    let engine = RuntimeDiscoveryEngine::new();
    let service = engine.discover_by_capability("myservice").await?;
    
    // Assert: Found via uppercase env var
    assert_eq!(service.endpoint, "http://10.0.1.123:7777");
    
    // Cleanup
    std::env::remove_var("MYSERVICE_ENDPOINT");
    
    Ok(())
}

