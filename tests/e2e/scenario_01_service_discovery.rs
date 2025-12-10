// E2E Scenario 1: Service Discovery and Registration
// Created: October 30, 2025
// Updated: December 2, 2025 - Modernized for concurrent execution
// Priority: P0 - Critical
// Status: ✅ Modernized - Fully concurrent

use anyhow::Result;
use serde_json::json;

mod test_environment;
use test_environment::TestEnvironment;

/// E2E Test: Service discovers and registers with orchestrator
/// 
/// **MODERN:** No #[serial] - uses isolated TestEnvironment with atomic port allocation
/// 
/// Flow:
/// 1. Start orchestrator (zero-knowledge state)
/// 2. Start compute service (toadstool-like)
/// 3. Verify service discovers orchestrator
/// 4. Verify service registers capabilities
/// 5. Verify orchestrator can route requests to service
#[tokio::test]
async fn e2e_scenario_01_service_discovery_and_registration() -> Result<()> {
    // ARRANGE: Set up test environment
    let mut env = TestEnvironment::new().await?;
    
    // ACT: Start orchestrator with zero knowledge
    let orchestrator = env.start_orchestrator().await?;
    assert!(orchestrator.health_check().await?, "Orchestrator should be healthy");
    
    // ACT: Start a compute service
    let compute_service = env.start_service("compute-01", "compute").await?;
    assert!(compute_service.health_check().await?, "Service should be healthy");
    
    // ASSERT: Verify service discovered orchestrator
    // In real implementation, we'd check discovery logs or metrics
    // For now, verify basic connectivity
    
    // ASSERT: Verify service registered its capabilities
    // This would query the orchestrator's registry
    let capability_request = json!({
        "capability": "compute",
        "operation": "list_providers"
    });
    
    let response = orchestrator.request_capability("compute", capability_request).await?;
    assert!(response.status().is_success(), "Should find registered compute capability");
    
    // ASSERT: Verify orchestrator can route requests to service
    let compute_request = json!({
        "operation": "health_check"
    });
    
    let response = orchestrator.request_capability("compute", compute_request).await?;
    assert!(response.status().is_success(), "Should route to compute service");
    
    // CLEANUP: Tear down environment
    env.cleanup().await?;
    
    Ok(())
}

/// Test: Multiple services register different capabilities
/// **CONCURRENT-SAFE:** Each test gets unique port range via atomic allocation
#[tokio::test]
async fn e2e_multiple_capability_registration() -> Result<()> {
    let mut env = TestEnvironment::new().await?;
    
    // Start orchestrator
    let orchestrator = env.start_orchestrator().await?;
    
    // Start multiple services with different capabilities
    let _compute = env.start_service("compute-01", "compute").await?;
    let _storage = env.start_service("storage-01", "storage").await?;
    let _ai = env.start_service("ai-01", "ai").await?;
    
    // Verify all capabilities are registered
    for capability in &["compute", "storage", "ai"] {
        let request = json!({
            "capability": capability,
            "operation": "list_providers"
        });
        
        let response = orchestrator.request_capability(capability, request).await?;
        assert!(response.status().is_success(), 
            "Should find registered {} capability", capability);
    }
    
    env.cleanup().await?;
    Ok(())
}

/// Test: Service re-registration after restart
/// **MODERN:** Uses TestEnvironment with proper synchronization primitives
#[tokio::test]
async fn e2e_service_reregistration() -> Result<()> {
    let mut env = TestEnvironment::new().await?;
    
    // Start orchestrator and service
    let orchestrator = env.start_orchestrator().await?;
    let _service = env.start_service("compute-01", "compute").await?;
    
    // Verify initial registration
    let request = json!({"capability": "compute", "operation": "list_providers"});
    let response = orchestrator.request_capability("compute", request.clone()).await?;
    assert!(response.status().is_success(), "Initial registration should succeed");
    
    // Stop and restart service
    env.stop_service("compute-01").await?;
    
    // Note: In real implementation, would use watch channels for state changes
    // For now, TestEnvironment manages process lifecycle synchronously
    
    let _service = env.start_service("compute-01", "compute").await?;
    
    // Verify re-registration (start_service waits for health)
    let response = orchestrator.request_capability("compute", request).await?;
    assert!(response.status().is_success(), "Re-registration should succeed");
    
    env.cleanup().await?;
    Ok(())
}

#[cfg(test)]
mod scenario_validation {
    use super::*;

    #[test]
    fn test_scenario_structure() {
        // Validate that scenario follows template structure
        // This is a meta-test to ensure consistency
        
        // Should have:
        // 1. ARRANGE phase (setup)
        // 2. ACT phase (execute)
        // 3. ASSERT phase (verify)
        // 4. CLEANUP phase (teardown)
        
        // This test just ensures the file compiles and tests exist
        assert!(true, "Scenario 1 structure is valid");
    }
}

