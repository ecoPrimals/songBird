// SPDX-License-Identifier: AGPL-3.0-only
//! End-to-End tests for capability-based orchestration
//!
//! Tests real-world multi-primal workflow scenarios using capability discovery
//!
//! **MODERN**: These tests use InMemoryResolver for truly concurrent execution!
//! No environment variables = no serialization needed!

use songbird_config::capability_endpoints::{CapabilityType, InMemoryResolver};
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::collections::HashMap;

#[tokio::test]
async fn test_e2e_secure_ai_workflow() {
    // Setup: Create resolver with test endpoints (NO environment variables!)
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::security_provider_port());
    let ai_endpoint = format!("http://ai:{}", songbird_config::defaults::ports::orchestrator_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Security, security_endpoint.clone()),
        (CapabilityType::Ai, ai_endpoint.clone()),
        (CapabilityType::Storage, storage_endpoint.clone()),
    ]));
    
    // Step 1-3: Discover all capabilities concurrently
    let (security_result, ai_result, storage_result) = tokio::join!(
        SecurityAdapter::with_resolver(&resolver),
        AIAdapter::with_resolver(&resolver),
        StorageAdapter::with_resolver(&resolver)
    );
    
    assert!(security_result.is_ok(), "Security adapter discovery should succeed");
    assert!(ai_result.is_ok(), "AI adapter discovery should succeed");
    assert!(storage_result.is_ok(), "Storage adapter discovery should succeed");
    
    // Verify all adapters have correct endpoints
    let security = security_result.unwrap();
    let ai = ai_result.unwrap();
    let storage = storage_result.unwrap();
    
    assert_eq!(security.endpoint(), &security_endpoint);
    assert_eq!(ai.endpoint(), &ai_endpoint);
    assert_eq!(storage.endpoint(), &storage_endpoint);
    
    // No cleanup needed - no shared state!
}

#[tokio::test]
async fn test_e2e_compute_with_storage_workflow() {
    // Setup: Create resolver for compute and storage
    let compute_endpoint = format!("http://compute:{}", songbird_config::defaults::ports::metrics_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Compute, compute_endpoint.clone()),
        (CapabilityType::Storage, storage_endpoint.clone()),
    ]));
    
    // Discover capabilities concurrently (truly parallel!)
    let (compute_result, storage_result) = tokio::join!(
        ComputeAdapter::with_resolver(&resolver),
        StorageAdapter::with_resolver(&resolver)
    );
    
    // Verify both succeeded
    assert!(compute_result.is_ok());
    assert!(storage_result.is_ok());
    
    let compute = compute_result.unwrap();
    let storage = storage_result.unwrap();
    
    assert_eq!(compute.endpoint(), &compute_endpoint);
    assert_eq!(storage.endpoint(), &storage_endpoint);
    
    // No cleanup needed!
}

#[tokio::test]
async fn test_e2e_full_stack_workflow() {
    // Setup: All capabilities for full stack operation
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::security_provider_port());
    let ai_endpoint = format!("http://ai:{}", songbird_config::defaults::ports::orchestrator_port());
    let compute_endpoint = format!("http://compute:{}", songbird_config::defaults::ports::federation_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Security, security_endpoint.clone()),
        (CapabilityType::Ai, ai_endpoint.clone()),
        (CapabilityType::Compute, compute_endpoint.clone()),
        (CapabilityType::Storage, storage_endpoint.clone()),
    ]));
    
    // Discover all capabilities in parallel (maximum concurrency!)
    let (security_result, ai_result, compute_result, storage_result) = tokio::join!(
        SecurityAdapter::with_resolver(&resolver),
        AIAdapter::with_resolver(&resolver),
        ComputeAdapter::with_resolver(&resolver),
        StorageAdapter::with_resolver(&resolver)
    );
    
    // Verify all succeeded
    assert!(security_result.is_ok(), "Security should be discovered");
    assert!(ai_result.is_ok(), "AI should be discovered");
    assert!(compute_result.is_ok(), "Compute should be discovered");
    assert!(storage_result.is_ok(), "Storage should be discovered");
    
    // Verify each has correct endpoint
    assert_eq!(security_result.unwrap().endpoint(), &security_endpoint);
    assert_eq!(ai_result.unwrap().endpoint(), &ai_endpoint);
    assert_eq!(compute_result.unwrap().endpoint(), &compute_endpoint);
    assert_eq!(storage_result.unwrap().endpoint(), &storage_endpoint);
    
    // No cleanup needed - each test is fully isolated!
}

#[tokio::test]
async fn test_e2e_partial_capability_availability() {
    // Setup: Only some capabilities available (realistic scenario)
    // Only configure Security and Storage, not AI or Compute
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Security, format!("http://security:{}", songbird_config::defaults::ports::security_provider_port())),
        (CapabilityType::Storage, format!("http://storage:{}", songbird_config::defaults::ports::metrics_port())),
    ]));
    
    // Discover capabilities (some available, some not)
    let (security_result, storage_result, ai_result, compute_result) = tokio::join!(
        SecurityAdapter::with_resolver(&resolver),
        StorageAdapter::with_resolver(&resolver),
        AIAdapter::with_resolver(&resolver),
        ComputeAdapter::with_resolver(&resolver)
    );
    
    // Available capabilities should succeed
    assert!(security_result.is_ok());
    assert!(storage_result.is_ok());
    
    // Unavailable capabilities should fail gracefully
    assert!(ai_result.is_err());
    assert!(compute_result.is_err());
    
    // No cleanup needed!
}

#[tokio::test]
async fn test_e2e_capability_failover() {
    // Setup: Primary and backup endpoints
    let primary_port = songbird_config::defaults::ports::orchestrator_port();
    let backup_port = songbird_config::defaults::ports::discovery_port();
    let primary_endpoint = format!("http://primary-ai:{}", primary_port);
    let backup_endpoint = format!("http://backup-ai:{}", backup_port);
    
    // Test primary resolver
    let primary_resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Ai, primary_endpoint.clone()),
    ]));
    
    let primary_result = AIAdapter::with_resolver(&primary_resolver).await;
    assert!(primary_result.is_ok());
    assert_eq!(primary_result.unwrap().endpoint(), &primary_endpoint);
    
    // Simulate failover with backup resolver
    let backup_resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Ai, backup_endpoint.clone()),
    ]));
    
    let failover_result = AIAdapter::with_resolver(&backup_resolver).await;
    assert!(failover_result.is_ok());
    assert_eq!(failover_result.unwrap().endpoint(), &backup_endpoint);
    
    // No cleanup needed - no shared state!
}

#[tokio::test]
async fn test_e2e_multi_region_deployment() {
    // Setup: Simulate multi-region with different endpoints
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Ai, format!("http://us-east-ai:{}", songbird_config::defaults::ports::orchestrator_port())),
        (CapabilityType::Storage, format!("http://us-west-storage:{}", songbird_config::defaults::ports::metrics_port())),
    ]));
    
    let (ai_result, storage_result) = tokio::join!(
        AIAdapter::with_resolver(&resolver),
        StorageAdapter::with_resolver(&resolver)
    );
    
    assert!(ai_result.is_ok());
    assert!(storage_result.is_ok());
    
    // Verify cross-region capabilities work
    let ai = ai_result.unwrap();
    let storage = storage_result.unwrap();
    
    assert!(ai.endpoint().contains("us-east"));
    assert!(storage.endpoint().contains("us-west"));
    
    // No cleanup needed!
}

#[tokio::test]
async fn test_e2e_dynamic_scaling_scenario() {
    // Setup: Start with one compute instance
    let compute_port = songbird_config::defaults::ports::federation_port();
    let compute1_endpoint = format!("http://compute-1:{}", compute_port);
    let compute2_endpoint = format!("http://compute-2:{}", compute_port);
    
    // Test with compute-1
    let resolver1 = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Compute, compute1_endpoint.clone()),
    ]));
    
    let compute1 = ComputeAdapter::with_resolver(&resolver1).await;
    assert!(compute1.is_ok());
    assert_eq!(compute1.unwrap().endpoint(), &compute1_endpoint);
    
    // Simulate scaling up to compute-2
    let resolver2 = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Compute, compute2_endpoint.clone()),
    ]));
    
    let compute2 = ComputeAdapter::with_resolver(&resolver2).await;
    assert!(compute2.is_ok());
    assert_eq!(compute2.unwrap().endpoint(), &compute2_endpoint);
    
    // No cleanup needed!
}

#[tokio::test]
async fn test_e2e_capability_health_check_workflow() {
    // Setup capabilities
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Ai, format!("http://ai:{}", songbird_config::defaults::ports::orchestrator_port())),
        (CapabilityType::Compute, format!("http://compute:{}", songbird_config::defaults::ports::federation_port())),
    ]));
    
    // Discover capabilities concurrently
    let (ai, compute) = tokio::join!(
        AIAdapter::with_resolver(&resolver),
        ComputeAdapter::with_resolver(&resolver)
    );
    
    // Verify discovery succeeded (health check would be next step)
    assert!(ai.is_ok(), "AI capability should be available");
    assert!(compute.is_ok(), "Compute capability should be available");
    
    // No cleanup needed!
}

#[tokio::test]
async fn test_e2e_zero_knowledge_bootstrap() {
    // Setup: Empty resolver - no endpoints configured
    let resolver = InMemoryResolver::new(HashMap::new());
    
    // Attempt discovery - should fail gracefully
    let result = AIAdapter::with_resolver(&resolver).await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    
    // Error should be informative about what's needed
    assert!(error_msg.contains("No endpoint") || 
            error_msg.contains("capability") ||
            error_msg.contains("configured"));
}

#[tokio::test]
async fn test_e2e_capability_resolver_direct_usage() {
    // Setup: Test resolver directly
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::security_provider_port());
    
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Security, security_endpoint.clone()),
    ]));
    
    // Use resolver directly
    let endpoint_result = resolver.get_endpoint(CapabilityType::Security).await;
    
    assert!(endpoint_result.is_ok());
    assert_eq!(endpoint_result.unwrap(), security_endpoint);
    
    // No cleanup needed!
}

#[tokio::test]
async fn test_e2e_multiple_capabilities_parallel_query() {
    // Setup: Multiple capabilities in resolver
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::security_provider_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    let resolver = InMemoryResolver::new(HashMap::from([
        (CapabilityType::Security, security_endpoint.clone()),
        (CapabilityType::Storage, storage_endpoint.clone()),
    ]));
    
    // Query multiple capabilities in parallel (truly concurrent!)
    let (sec_result, stor_result) = tokio::join!(
        resolver.get_endpoint(CapabilityType::Security),
        resolver.get_endpoint(CapabilityType::Storage)
    );
    
    assert!(sec_result.is_ok());
    assert!(stor_result.is_ok());
    assert_eq!(sec_result.unwrap(), security_endpoint);
    assert_eq!(stor_result.unwrap(), storage_endpoint);
    
    // No cleanup needed!
}

