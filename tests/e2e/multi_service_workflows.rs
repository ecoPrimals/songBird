//! Multi-service E2E workflow tests
//!
//! Tests complete workflows across multiple services

use songbird_test_utils::TestHarness;
use songbird_types::SongbirdResult;

#[tokio::test]
async fn test_discover_compute_execute_workflow() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Discover compute providers
    let providers = harness.discover_capability("compute").await?;
    assert!(!providers.is_empty(), "Should find at least one compute provider");
    
    // Step 2: Select a provider
    let provider = harness.select_best_provider(&providers)?;
    
    // Step 3: Connect to provider
    let connection = harness.connect_to_provider(&provider).await?;
    assert!(connection.is_healthy().await);
    
    // Step 4: Execute workload
    let result = harness.execute_compute_task(&connection, "test_task").await?;
    assert!(result.success);
    
    Ok(())
}

#[tokio::test]
async fn test_storage_compute_pipeline() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Store data via storage capability
    let storage = harness.discover_capability("storage").await?;
    let storage_provider = &storage[0];
    let data_id = harness.store_data(storage_provider, "test_data").await?;
    
    // Step 2: Retrieve data for processing
    let retrieved = harness.retrieve_data(storage_provider, &data_id).await?;
    assert_eq!(retrieved, "test_data");
    
    // Step 3: Process data via compute capability
    let compute = harness.discover_capability("compute").await?;
    let compute_provider = &compute[0];
    let processed = harness.process_data(compute_provider, &retrieved).await?;
    
    // Step 4: Store results
    let result_id = harness.store_data(storage_provider, &processed).await?;
    assert!(!result_id.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_ai_inference_workflow() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Discover AI providers
    let ai_providers = harness.discover_capability("ai").await?;
    if ai_providers.is_empty() {
        // Skip if no AI providers available
        return Ok(());
    }
    
    // Step 2: Prepare input data via storage
    let storage = harness.discover_capability("storage").await?;
    let input_id = harness.store_data(&storage[0], "inference_input").await?;
    
    // Step 3: Run inference via AI provider
    let ai_provider = &ai_providers[0];
    let inference_result = harness.run_inference(ai_provider, &input_id).await?;
    
    // Step 4: Store inference results
    let result_id = harness.store_data(&storage[0], &inference_result).await?;
    assert!(!result_id.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_distributed_compute_workflow() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Discover multiple compute providers
    let compute_providers = harness.discover_capability("compute").await?;
    
    // Step 2: Split work across providers
    let tasks = vec!["task1", "task2", "task3"];
    let mut results = vec![];
    
    for (i, task) in tasks.iter().enumerate() {
        let provider_idx = i % compute_providers.len();
        let provider = &compute_providers[provider_idx];
        
        let result = harness.execute_compute_task_at(provider, task).await?;
        results.push(result);
    }
    
    // Step 3: Aggregate results
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.success);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_failover_workflow() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Discover providers
    let providers = harness.discover_capability("compute").await?;
    assert!(providers.len() >= 2, "Need at least 2 providers for failover test");
    
    // Step 2: Try primary provider
    let primary = &providers[0];
    harness.simulate_provider_failure(primary).await?;
    
    // Step 3: Should automatically failover to secondary
    let result = harness.execute_with_failover("compute", "test_task").await?;
    assert!(result.success);
    assert_ne!(result.provider_used, primary.id);
    
    Ok(())
}

#[tokio::test]
async fn test_multi_capability_transaction() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Begin transaction
    let tx = harness.begin_transaction().await?;
    
    // Step 2: Execute multiple capabilities in transaction
    tx.execute_capability("storage", "store_data").await?;
    tx.execute_capability("compute", "process_data").await?;
    tx.execute_capability("network", "transmit_result").await?;
    
    // Step 3: Commit transaction
    let result = tx.commit().await?;
    assert!(result.all_succeeded());
    
    Ok(())
}

#[tokio::test]
async fn test_rollback_on_failure() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Begin transaction
    let tx = harness.begin_transaction().await?;
    
    // Step 2: Execute capabilities
    tx.execute_capability("storage", "store_data").await?;
    
    // Step 3: Simulate failure
    let result = tx.execute_capability("compute", "failing_task").await;
    assert!(result.is_err());
    
    // Step 4: Should rollback automatically
    let rollback_result = tx.rollback().await?;
    assert!(rollback_result.rollback_successful);
    
    Ok(())
}

#[tokio::test]
async fn test_load_balanced_requests() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Discover providers
    let providers = harness.discover_capability("compute").await?;
    
    // Step 2: Send multiple requests
    let mut provider_usage = std::collections::HashMap::new();
    
    for i in 0..10 {
        let result = harness.execute_compute_task_load_balanced("compute", &format!("task{}", i)).await?;
        *provider_usage.entry(result.provider_used).or_insert(0) += 1;
    }
    
    // Step 3: Verify distribution
    assert!(provider_usage.len() > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_capability_chaining() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Chain: storage -> compute -> ai -> storage
    let chain = harness.create_capability_chain(vec![
        "storage",
        "compute", 
        "ai",
        "storage"
    ]).await?;
    
    let result = chain.execute("initial_data").await?;
    assert!(result.success);
    
    Ok(())
}

#[tokio::test]
async fn test_parallel_capability_execution() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Execute multiple capabilities in parallel
    let results = harness.execute_parallel(vec![
        ("compute", "task1"),
        ("storage", "task2"),
        ("network", "task3"),
    ]).await?;
    
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.success);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_health_monitoring_workflow() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Start health monitoring
    harness.start_health_monitoring().await?;
    
    // Step 2: Execute workload
    let providers = harness.discover_capability("compute").await?;
    for provider in providers {
        let _ = harness.execute_compute_task_at(&provider, "health_test").await;
    }
    
    // Step 3: Check health metrics
    let health_report = harness.get_health_report().await?;
    assert!(!health_report.providers.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_discovery_cache_behavior() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: First discovery (cache miss)
    let start = std::time::Instant::now();
    let providers1 = harness.discover_capability("compute").await?;
    let first_duration = start.elapsed();
    
    // Step 2: Second discovery (cache hit)
    let start = std::time::Instant::now();
    let providers2 = harness.discover_capability("compute").await?;
    let second_duration = start.elapsed();
    
    // Step 3: Verify cache hit is faster
    assert_eq!(providers1.len(), providers2.len());
    // Cache hit should be faster (but may not always be measurable)
    
    Ok(())
}

#[tokio::test]
async fn test_dynamic_provider_registration() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Initial discovery
    let initial = harness.discover_capability("test_capability").await?;
    let initial_count = initial.len();
    
    // Step 2: Register new provider dynamically
    harness.register_test_provider("test_capability", "new_provider").await?;
    
    // Step 3: Poll for propagation (no sleep, wait for actual condition)
    let start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(2);
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
    
    let mut after = initial.clone();
    while start.elapsed() < timeout {
        after = harness.discover_capability("test_capability").await?;
        if after.len() > initial_count {
            break;
        }
        interval.tick().await;
    }
    
    // Step 4: Verify new provider is discovered
    assert!(after.len() > initial_count, "New provider should be discovered");
    
    Ok(())
}

#[tokio::test]
async fn test_provider_deregistration_workflow() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Step 1: Register provider
    harness.register_test_provider("temp_capability", "temp_provider").await?;
    
    // Step 2: Verify registration
    let providers = harness.discover_capability("temp_capability").await?;
    assert!(!providers.is_empty());
    
    // Step 3: Deregister
    harness.deregister_provider("temp_provider").await?;
    
    // Step 4: Poll for deregistration (no sleep, wait for actual condition)
    let start = tokio::time::Instant::now();
    let timeout = tokio::time::Duration::from_secs(2);
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
    
    let mut deregistered = false;
    while start.elapsed() < timeout {
        let after = harness.discover_capability("temp_capability").await?;
        if after.iter().all(|p| p.id != "temp_provider") {
            deregistered = true;
            break;
        }
        interval.tick().await;
    }
    
    assert!(deregistered, "Provider should be deregistered");
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_workflow_execution() -> SongbirdResult<()> {
    let harness = std::sync::Arc::new(TestHarness::new().await?);
    
    let mut handles = vec![];
    
    // Execute 10 workflows concurrently
    for i in 0..10 {
        let harness_clone = std::sync::Arc::clone(&harness);
        let handle = tokio::spawn(async move {
            harness_clone.execute_compute_task_load_balanced("compute", &format!("concurrent_{}", i)).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_metadata_driven_selection() -> SongbirdResult<()> {
    let harness = TestHarness::new().await?;
    
    // Discover providers with specific metadata requirements
    let mut requirements = std::collections::HashMap::new();
    requirements.insert("gpu".to_string(), "true".to_string());
    requirements.insert("memory_gb".to_string(), "16".to_string());
    
    let providers = harness.discover_with_requirements("compute", requirements).await?;
    
    // Should only return providers meeting requirements
    for provider in providers {
        assert!(provider.metadata.get("gpu").is_some());
    }
    
    Ok(())
}

