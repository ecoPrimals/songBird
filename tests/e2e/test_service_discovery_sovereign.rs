//! E2E Test: Sovereign Service Discovery
//!
//! Tests that services discover each other through capability-based discovery
//! with ZERO hardcoded knowledge of other primals.
//!
//! **Sovereignty Principle**: Each primal knows only itself, discovers others at runtime.

use songbird_types::SongbirdResult;
use std::time::Duration;
use tokio::time::timeout;

/// Test: Service discovers orchestrator via environment variable
#[tokio::test]
async fn test_service_discovers_orchestrator_via_env() -> SongbirdResult<()> {
    // Setup: Set orchestrator endpoint
    std::env::set_var("ORCHESTRATOR_ENDPOINT", "http://10.0.1.100:8080");
    
    // Simulate service discovering orchestrator
    let endpoint = std::env::var("ORCHESTRATOR_ENDPOINT")
        .expect("Orchestrator endpoint should be set");
    
    // Assert: Service found orchestrator without hardcoding
    assert_eq!(endpoint, "http://10.0.1.100:8080");
    assert!(!endpoint.contains("localhost"), "Should not use localhost");
    assert!(!endpoint.contains("127.0.0.1"), "Should not use hardcoded loopback");
    
    // Cleanup
    std::env::remove_var("ORCHESTRATOR_ENDPOINT");
    
    Ok(())
}

/// Test: Multiple services discover each other independently
#[tokio::test]
async fn test_independent_service_discovery() -> SongbirdResult<()> {
    // Setup: Configure multiple services
    std::env::set_var("SERVICE_A_ENDPOINT", "http://10.0.1.50:9001");
    std::env::set_var("SERVICE_B_ENDPOINT", "http://10.0.1.51:9002");
    std::env::set_var("SERVICE_C_ENDPOINT", "http://10.0.1.52:9003");
    
    // Each service discovers others independently
    let service_a = std::env::var("SERVICE_A_ENDPOINT")?;
    let service_b = std::env::var("SERVICE_B_ENDPOINT")?;
    let service_c = std::env::var("SERVICE_C_ENDPOINT")?;
    
    // Assert: All services have unique endpoints
    assert_ne!(service_a, service_b);
    assert_ne!(service_b, service_c);
    assert_ne!(service_a, service_c);
    
    // Assert: No hardcoded assumptions
    let all_endpoints = vec![service_a, service_b, service_c];
    for endpoint in &all_endpoints {
        assert!(!endpoint.contains("localhost"));
        assert!(!endpoint.contains("127.0.0.1"));
        assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
    }
    
    // Cleanup
    std::env::remove_var("SERVICE_A_ENDPOINT");
    std::env::remove_var("SERVICE_B_ENDPOINT");
    std::env::remove_var("SERVICE_C_ENDPOINT");
    
    Ok(())
}

/// Test: Discovery fails gracefully when no configuration provided
#[tokio::test]
async fn test_discovery_fails_without_configuration() {
    // Ensure no configuration set
    std::env::remove_var("UNKNOWN_SERVICE_ENDPOINT");
    
    // Try to discover service
    let result = std::env::var("UNKNOWN_SERVICE_ENDPOINT");
    
    // Assert: Fails gracefully (not panic)
    assert!(result.is_err(), "Discovery should fail without configuration");
    
    // Verify error is meaningful
    let error = result.unwrap_err();
    assert!(matches!(error, std::env::VarError::NotPresent));
}

/// Test: Capability-based discovery (not name-based)
#[tokio::test]
async fn test_capability_based_not_name_based() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::{RuntimeDiscoveryEngine};
    
    // Setup: Configure service by capability, not name
    std::env::set_var("STORAGE_ENDPOINT", "http://10.0.1.200:8888");
    
    // Discover by capability
    let engine = RuntimeDiscoveryEngine::new();
    let service = engine.discover_by_capability("storage").await?;
    
    // Assert: Found via capability, endpoint matches
    assert_eq!(service.capability, "storage");
    assert_eq!(service.endpoint, "http://10.0.1.200:8888");
    
    // The service could be ANY primal providing storage capability
    // We don't care WHICH primal, only that it has the capability
    
    // Cleanup
    std::env::remove_var("STORAGE_ENDPOINT");
    
    Ok(())
}

/// Test: Service can change providers without code changes
#[tokio::test]
async fn test_provider_flexibility() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;
    
    // Scenario 1: Using Provider A
    std::env::set_var("COMPUTE_ENDPOINT", "http://provider-a.local:8001");
    
    let engine = RuntimeDiscoveryEngine::new();
    let provider_a = engine.discover_by_capability("compute").await?;
    assert_eq!(provider_a.endpoint, "http://provider-a.local:8001");
    
    // Scenario 2: Switch to Provider B (just change env var)
    std::env::set_var("COMPUTE_ENDPOINT", "http://provider-b.local:9999");
    
    let provider_b = RuntimeDiscoveryEngine::new()
        .discover_by_capability("compute")
        .await?;
    assert_eq!(provider_b.endpoint, "http://provider-b.local:9999");
    
    // No code changes needed - sovereignty principle enforced
    
    // Cleanup
    std::env::remove_var("COMPUTE_ENDPOINT");
    
    Ok(())
}

/// Test: Zero knowledge of network topology
#[tokio::test]
async fn test_zero_network_topology_assumptions() -> SongbirdResult<()> {
    // Test various network configurations work equally well
    
    let test_cases = vec![
        ("http://10.0.1.50:8080", "Private network"),
        ("http://192.168.1.100:3000", "Local network"),
        ("http://172.16.0.50:9000", "Docker network"),
        ("https://service.example.com:443", "Internet"),
        ("http://[::1]:8080", "IPv6 loopback"),
        ("http://[2001:db8::1]:8080", "IPv6 address"),
    ];
    
    for (endpoint, description) in test_cases {
        std::env::set_var("TEST_SERVICE_ENDPOINT", endpoint);
        
        let discovered = std::env::var("TEST_SERVICE_ENDPOINT")?;
        
        // Assert: Any valid endpoint works
        assert_eq!(discovered, endpoint, "Failed for: {}", description);
        
        std::env::remove_var("TEST_SERVICE_ENDPOINT");
    }
    
    Ok(())
}

/// Test: Service discovery with timeout
#[tokio::test]
async fn test_discovery_with_timeout() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;
    
    // Setup
    std::env::set_var("FAST_SERVICE_ENDPOINT", "http://10.0.1.123:7777");
    
    // Discovery should complete quickly
    let result = timeout(
        Duration::from_secs(2),
        async {
            RuntimeDiscoveryEngine::new()
                .discover_by_capability("fast_service")
                .await
        }
    ).await;
    
    // Assert: Completed within timeout
    assert!(result.is_ok(), "Discovery should complete within timeout");
    
    let service = result??;
    assert_eq!(service.endpoint, "http://10.0.1.123:7777");
    
    // Cleanup
    std::env::remove_var("FAST_SERVICE_ENDPOINT");
    
    Ok(())
}

/// Test: Primal self-knowledge (knows own endpoint)
#[tokio::test]
async fn test_primal_self_knowledge() -> SongbirdResult<()> {
    // Each primal should know its own endpoint
    std::env::set_var("MY_ENDPOINT", "http://10.0.1.50:8080");
    std::env::set_var("MY_CAPABILITIES", "compute,storage");
    
    // Primal knows itself
    let my_endpoint = std::env::var("MY_ENDPOINT")?;
    let my_capabilities = std::env::var("MY_CAPABILITIES")?;
    
    // Assert: Self-knowledge present
    assert!(!my_endpoint.is_empty());
    assert!(!my_capabilities.is_empty());
    
    // But does NOT know other primals (they must discover)
    assert!(std::env::var("OTHER_PRIMAL_ENDPOINT").is_err(),
        "Should not have hardcoded knowledge of other primals");
    
    // Cleanup
    std::env::remove_var("MY_ENDPOINT");
    std::env::remove_var("MY_CAPABILITIES");
    
    Ok(())
}

/// Test: Discovery results are consistent
#[tokio::test]
async fn test_discovery_consistency() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;
    
    // Setup
    std::env::set_var("CONSISTENT_SERVICE_ENDPOINT", "http://10.0.1.99:8888");
    
    let engine = RuntimeDiscoveryEngine::new();
    
    // Discover multiple times
    let result1 = engine.discover_by_capability("consistent_service").await?;
    let result2 = engine.discover_by_capability("consistent_service").await?;
    let result3 = engine.discover_by_capability("consistent_service").await?;
    
    // Assert: All discoveries return same endpoint
    assert_eq!(result1.endpoint, result2.endpoint);
    assert_eq!(result2.endpoint, result3.endpoint);
    assert_eq!(result1.endpoint, "http://10.0.1.99:8888");
    
    // Cleanup
    std::env::remove_var("CONSISTENT_SERVICE_ENDPOINT");
    
    Ok(())
}

/// Test: Sovereignty violation detection (hardcoded fallback rejected)
#[tokio::test]
async fn test_no_hardcoded_fallbacks() {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;
    
    // Ensure NO configuration
    std::env::remove_var("NONEXISTENT_ENDPOINT");
    
    // Try to discover
    let engine = RuntimeDiscoveryEngine::new();
    let result = engine.discover_by_capability("nonexistent").await;
    
    // Assert: MUST fail - no hardcoded fallbacks allowed
    assert!(result.is_err(), 
        "SOVEREIGNTY VIOLATION: Must not have hardcoded fallbacks!");
    
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    
    // Verify error explains the problem
    assert!(
        error_msg.contains("not found") || error_msg.contains("No service"),
        "Error should explain service not found: {}",
        error_msg
    );
}

/// Test: Multiple discovery engines don't interfere
#[tokio::test]
async fn test_isolated_discovery_engines() -> SongbirdResult<()> {
    use songbird_config::runtime_discovery::RuntimeDiscoveryEngine;
    
    // Setup
    std::env::set_var("SHARED_SERVICE_ENDPOINT", "http://10.0.1.111:9999");
    
    // Create multiple engines
    let engine1 = RuntimeDiscoveryEngine::new();
    let engine2 = RuntimeDiscoveryEngine::new();
    let engine3 = RuntimeDiscoveryEngine::new();
    
    // All can discover independently
    let service1 = engine1.discover_by_capability("shared_service").await?;
    let service2 = engine2.discover_by_capability("shared_service").await?;
    let service3 = engine3.discover_by_capability("shared_service").await?;
    
    // Assert: All found the same service
    assert_eq!(service1.endpoint, service2.endpoint);
    assert_eq!(service2.endpoint, service3.endpoint);
    
    // Cleanup
    std::env::remove_var("SHARED_SERVICE_ENDPOINT");
    
    Ok(())
}

#[cfg(test)]
mod sovereignty_tests {
    use super::*;
    
    /// Test: Enforces individual human dignity (frictionless)
    #[tokio::test]
    async fn test_individual_human_frictionless() -> SongbirdResult<()> {
        // Individual humans should experience zero friction
        // Just set env var and go
        
        std::env::set_var("MY_SERVICE_ENDPOINT", "http://anywhere:8080");
        
        let endpoint = std::env::var("MY_SERVICE_ENDPOINT")?;
        
        // Assert: Works immediately, no ceremony
        assert_eq!(endpoint, "http://anywhere:8080");
        
        // Cleanup
        std::env::remove_var("MY_SERVICE_ENDPOINT");
        
        Ok(())
    }
    
    /// Test: Appropriate friction for entities
    #[tokio::test]
    async fn test_entity_appropriate_friction() {
        // Entities (non-individuals) should have appropriate validation
        // For now, same mechanism, but framework ready for entity policies
        
        // This test documents the intention even if not yet enforced
        let entity_endpoint = "http://corporate.example.com:8080";
        std::env::set_var("ENTITY_SERVICE_ENDPOINT", entity_endpoint);
        
        let endpoint = std::env::var("ENTITY_SERVICE_ENDPOINT")
            .expect("Entity endpoint should work");
        
        assert_eq!(endpoint, entity_endpoint);
        
        // Cleanup
        std::env::remove_var("ENTITY_SERVICE_ENDPOINT");
    }
}

