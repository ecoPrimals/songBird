//! End-to-End tests for capability-based orchestration
//!
//! Tests real-world multi-primal workflow scenarios using capability discovery

use songbird_config::capability_endpoints;
use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use serial_test::serial;
use std::env;

#[tokio::test]
#[serial]
async fn test_e2e_secure_ai_workflow() {
    // Setup: Configure capabilities for a secure AI workflow
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::beardog_port());
    let ai_endpoint = format!("http://ai:{}", songbird_config::defaults::ports::orchestrator_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", &security_endpoint);
    env::set_var("CAPABILITY_AI_ENDPOINT", &ai_endpoint);
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", &storage_endpoint);
    
    // Step 1: Discover security capability
    let security_result = SecurityAdapter::from_discovery().await;
    assert!(security_result.is_ok(), "Security adapter discovery should succeed");
    
    // Step 2: Discover AI capability
    let ai_result = AIAdapter::from_discovery().await;
    assert!(ai_result.is_ok(), "AI adapter discovery should succeed");
    
    // Step 3: Discover storage capability
    let storage_result = StorageAdapter::from_discovery().await;
    assert!(storage_result.is_ok(), "Storage adapter discovery should succeed");
    
    // Verify all adapters have correct endpoints
    let security = security_result.unwrap();
    let ai = ai_result.unwrap();
    let storage = storage_result.unwrap();
    
    assert_eq!(security.endpoint(), &security_endpoint);
    assert_eq!(ai.endpoint(), &ai_endpoint);
    assert_eq!(storage.endpoint(), &storage_endpoint);
    
    // Cleanup
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_compute_with_storage_workflow() {
    // Setup: Configure compute and storage workflow
    let compute_endpoint = format!("http://compute:{}", songbird_config::defaults::ports::metrics_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", &compute_endpoint);
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", &storage_endpoint);
    
    // Discover capabilities concurrently
    let (compute_result, storage_result) = tokio::join!(
        ComputeAdapter::new_from_discovery(),
        StorageAdapter::from_discovery()
    );
    
    // Verify both succeeded
    assert!(compute_result.is_ok());
    assert!(storage_result.is_ok());
    
    let compute = compute_result.unwrap();
    let storage = storage_result.unwrap();
    
    assert_eq!(compute.endpoint(), &compute_endpoint);
    assert_eq!(storage.endpoint(), &storage_endpoint);
    
    // Cleanup
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_full_stack_workflow() {
    // Setup: All capabilities for full stack operation
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::beardog_port());
    let ai_endpoint = format!("http://ai:{}", songbird_config::defaults::ports::orchestrator_port());
    let compute_endpoint = format!("http://compute:{}", songbird_config::defaults::ports::federation_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", &security_endpoint);
    env::set_var("CAPABILITY_AI_ENDPOINT", &ai_endpoint);
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", &compute_endpoint);
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", &storage_endpoint);
    
    // Discover all capabilities in parallel
    let (security_result, ai_result, compute_result, storage_result) = tokio::join!(
        SecurityAdapter::from_discovery(),
        AIAdapter::from_discovery(),
        ComputeAdapter::new_from_discovery(),
        StorageAdapter::from_discovery()
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
    
    // Cleanup
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_partial_capability_availability() {
    // Setup: Only some capabilities available (realistic scenario)
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", &format!("http://security:{}", songbird_config::defaults::ports::beardog_port()));
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", &format!("http://storage:{}", songbird_config::defaults::ports::metrics_port()));
    // AI and Compute intentionally not configured
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
    env::remove_var("SERVICE_REGISTRY_ENDPOINT");
    
    // Discover available capabilities
    let security_result = SecurityAdapter::from_discovery().await;
    let storage_result = StorageAdapter::from_discovery().await;
    let ai_result = AIAdapter::from_discovery().await;
    let compute_result = ComputeAdapter::new_from_discovery().await;
    
    // Available capabilities should succeed
    assert!(security_result.is_ok());
    assert!(storage_result.is_ok());
    
    // Unavailable capabilities should fail gracefully
    assert!(ai_result.is_err());
    assert!(compute_result.is_err());
    
    // Cleanup
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_capability_failover() {
    // Setup: Primary endpoint, then simulate failover
    let primary_port = songbird_config::defaults::ports::orchestrator_port();
    let backup_port = songbird_config::defaults::ports::discovery_port();
    let primary_endpoint = format!("http://primary-ai:{}", primary_port);
    let backup_endpoint = format!("http://backup-ai:{}", backup_port);
    
    env::set_var("CAPABILITY_AI_ENDPOINT", &primary_endpoint);
    
    let primary_result = AIAdapter::from_discovery().await;
    assert!(primary_result.is_ok());
    assert_eq!(primary_result.unwrap().endpoint(), &primary_endpoint);
    
    // Simulate failover by changing endpoint
    env::set_var("CAPABILITY_AI_ENDPOINT", &backup_endpoint);
    
    let failover_result = AIAdapter::from_discovery().await;
    assert!(failover_result.is_ok());
    assert_eq!(failover_result.unwrap().endpoint(), &backup_endpoint);
    
    // Cleanup
    env::remove_var("CAPABILITY_AI_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_multi_region_deployment() {
    // Setup: Simulate multi-region with different endpoints
    env::set_var("CAPABILITY_AI_ENDPOINT", &format!("http://us-east-ai:{}", songbird_config::defaults::ports::orchestrator_port()));
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", &format!("http://us-west-storage:{}", songbird_config::defaults::ports::metrics_port()));
    
    let ai_result = AIAdapter::from_discovery().await;
    let storage_result = StorageAdapter::from_discovery().await;
    
    assert!(ai_result.is_ok());
    assert!(storage_result.is_ok());
    
    // Verify cross-region capabilities work
    let ai = ai_result.unwrap();
    let storage = storage_result.unwrap();
    
    assert!(ai.endpoint().contains("us-east"));
    assert!(storage.endpoint().contains("us-west"));
    
    // Cleanup
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_dynamic_scaling_scenario() {
    // Setup: Start with one compute instance
    let compute_port = songbird_config::defaults::ports::federation_port();
    let compute1_endpoint = format!("http://compute-1:{}", compute_port);
    let compute2_endpoint = format!("http://compute-2:{}", compute_port);
    
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", &compute1_endpoint);
    
    let compute1 = ComputeAdapter::new_from_discovery().await;
    assert!(compute1.is_ok());
    
    // Simulate scaling up - new instance
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", &compute2_endpoint);
    
    let compute2 = ComputeAdapter::new_from_discovery().await;
    assert!(compute2.is_ok());
    assert_eq!(compute2.unwrap().endpoint(), &compute2_endpoint);
    
    // Cleanup
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_capability_health_check_workflow() {
    // Setup capabilities
    env::set_var("CAPABILITY_AI_ENDPOINT", &format!("http://ai:{}", songbird_config::defaults::ports::orchestrator_port()));
    env::set_var("CAPABILITY_COMPUTE_ENDPOINT", &format!("http://compute:{}", songbird_config::defaults::ports::federation_port()));
    
    // Discover capabilities
    let ai = AIAdapter::from_discovery().await;
    let compute = ComputeAdapter::new_from_discovery().await;
    
    // Verify discovery succeeded (health check would be next step)
    assert!(ai.is_ok(), "AI capability should be available");
    assert!(compute.is_ok(), "Compute capability should be available");
    
    // Cleanup
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("CAPABILITY_COMPUTE_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_zero_knowledge_bootstrap() {
    // Setup: No environment variables, no registry - pure failure case
    env::remove_var("CAPABILITY_AI_ENDPOINT");
    env::remove_var("SERVICE_REGISTRY_ENDPOINT");
    env::remove_var("CONTAINER_METADATA_API");
    env::remove_var("SERVICE_DISCOVERY_DOMAIN");
    
    // Attempt discovery - should fail gracefully
    let result = AIAdapter::from_discovery().await;
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    
    // Error should be informative about what's needed
    assert!(error_msg.contains("No endpoint found") || 
            error_msg.contains("Set CAPABILITY_AI_ENDPOINT") ||
            error_msg.contains("enable discovery"));
}

#[tokio::test]
#[serial]
async fn test_e2e_capability_endpoint_function() {
    // Setup
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::beardog_port());
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", &security_endpoint);
    
    // Use convenience function
    let endpoint_result = capability_endpoints::get_capability_endpoint("security").await;
    
    assert!(endpoint_result.is_ok());
    assert_eq!(endpoint_result.unwrap(), security_endpoint);
    
    // Cleanup
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
}

#[tokio::test]
#[serial]
async fn test_e2e_multiple_capabilities_parallel_query() {
    // Setup
    let security_endpoint = format!("http://security:{}", songbird_config::defaults::ports::beardog_port());
    let storage_endpoint = format!("http://storage:{}", songbird_config::defaults::ports::metrics_port());
    
    env::set_var("CAPABILITY_SECURITY_ENDPOINT", &security_endpoint);
    env::set_var("CAPABILITY_STORAGE_ENDPOINT", &storage_endpoint);
    
    // Query multiple capabilities in parallel
    let capabilities = vec!["security", "storage"];
    let results = capability_endpoints::get_multiple_endpoints(&capabilities).await;
    
    assert!(results.is_ok());
    let endpoints = results.unwrap();
    assert_eq!(endpoints.len(), 2);
    assert_eq!(endpoints[0], security_endpoint);
    assert_eq!(endpoints[1], storage_endpoint);
    
    // Cleanup
    env::remove_var("CAPABILITY_SECURITY_ENDPOINT");
    env::remove_var("CAPABILITY_STORAGE_ENDPOINT");
}

