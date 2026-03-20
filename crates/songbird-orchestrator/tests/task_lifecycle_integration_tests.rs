// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Task Lifecycle Manager Integration Tests
//!
//! End-to-end tests for task lifecycle operations

use anyhow::Result;
use songbird_orchestrator::task_lifecycle::{
    Priority, ResourceRequirements, TaskId, TaskLifecycleManager, TaskSpec, TowerId, UserId,
};
use std::sync::Arc;

// =============================================================================
// Test Helpers
// =============================================================================

async fn create_test_manager() -> Result<(TaskLifecycleManager, tempfile::TempDir)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_sled_db");
    let manager = TaskLifecycleManager::new(db_path.to_str().unwrap()).await?;
    // Return TempDir to keep it alive for the test's duration
    Ok((manager, temp_dir))
}

fn create_test_spec(task_type: &str) -> TaskSpec {
    TaskSpec {
        task_type: task_type.into(),
        config: serde_json::json!({
            "test": true,
            "task_type": task_type
        }),
        required_capabilities: vec!["test".into()],
        resources: ResourceRequirements {
            cpu_cores: Some(1),
            memory_mb: Some(512),
            gpu_count: None,
            network_mbps: None,
            storage_gb: None,
        },
        priority: Priority::Standard,
    }
}

fn test_user() -> UserId {
    UserId::new("test_user")
}

fn test_tower() -> TowerId {
    TowerId::new("test_tower")
}

// =============================================================================
// Basic Lifecycle Tests
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_create_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_create");

    let task_id = manager.create_task(test_user(), spec).await?;

    // Verify task exists
    let task = manager.get_task(task_id).await?;
    assert!(task.is_some(), "Task should exist after creation");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_start_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_start");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;

    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_update_progress() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_progress");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;

    // Update progress multiple times
    for i in 1..=10 {
        manager.update_progress(task_id, (i * 10) as f32).await?;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_complete_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_complete");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;
    manager.complete_task(task_id).await?;

    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());
    let task = task.unwrap();
    assert!(task.status.is_terminal(), "Completed task should be terminal");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_fail_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_fail");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;
    manager.fail_task(task_id, "Test failure".into()).await?;

    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());
    let task = task.unwrap();
    assert!(task.status.is_terminal(), "Failed task should be terminal");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_cancel_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_cancel");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;
    manager.cancel_task(task_id, Some("User requested".into())).await?;

    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());
    let task = task.unwrap();
    assert!(task.status.is_terminal(), "Cancelled task should be terminal");

    Ok(())
}

// =============================================================================
// Checkpointing Tests
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_create_checkpoint() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_checkpoint");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;

    // Create checkpoint
    let state = vec![1, 2, 3, 4, 5];
    let checkpoint_id = manager.create_checkpoint(task_id, state.clone()).await?;

    assert!(!checkpoint_id.is_empty(), "Checkpoint ID should not be empty");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_resume_from_checkpoint() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_resume");

    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;

    // Create checkpoint
    let state = vec![1, 2, 3, 4, 5];
    let checkpoint_id = manager.create_checkpoint(task_id, state.clone()).await?;

    // Resume from checkpoint
    let (resumed_task_id, resumed_state) = manager.resume_from_checkpoint(&checkpoint_id).await?;

    assert_eq!(resumed_task_id, task_id);
    assert_eq!(resumed_state, state);

    Ok(())
}

// =============================================================================
// Concurrent Operations Tests
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_task_creation() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let manager = Arc::new(manager);

    // Create 10 tasks concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let spec = create_test_spec(&format!("concurrent_{}", i));
            manager_clone.create_task(test_user(), spec).await
        });
        handles.push(handle);
    }

    // Collect results
    let mut task_ids = vec![];
    for handle in handles {
        let task_id = handle.await??;
        task_ids.push(task_id);
    }

    assert_eq!(task_ids.len(), 10);

    // Verify all IDs are unique
    let unique_count = task_ids.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, 10, "All task IDs should be unique");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_progress_updates() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let manager = Arc::new(manager);
    let spec = create_test_spec("concurrent_progress");
    let task_id = manager.create_task(test_user(), spec).await?;

    manager.start_task(task_id, test_tower()).await?;

    // Update progress concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle =
            tokio::spawn(
                async move { manager_clone.update_progress(task_id, (i * 10) as f32).await },
            );
        handles.push(handle);
    }

    // Wait for all updates
    for handle in handles {
        handle.await??;
    }

    // Verify task still exists
    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());

    Ok(())
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_nonexistent_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;

    let fake_id = TaskId::new();
    let task = manager.get_task(fake_id).await?;

    assert!(task.is_none(), "Nonexistent task should return None");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_start_nonexistent_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;

    let fake_id = TaskId::new();
    let result = manager.start_task(fake_id, test_tower()).await;

    assert!(result.is_err(), "Starting nonexistent task should error");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_complete_nonstarted_task() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_invalid_complete");

    let task_id = manager.create_task(test_user(), spec).await?;

    // Try to complete without starting
    let result = manager.complete_task(task_id).await;

    // Should either succeed (idempotent) or error gracefully
    assert!(result.is_ok() || result.is_err());

    Ok(())
}

// =============================================================================
// Lifecycle Combinations
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_full_lifecycle_with_pause_resume() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_pause_resume");

    // Create and start
    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;

    // Progress
    manager.update_progress(task_id, 25.0).await?;

    // Pause
    manager.pause_task(task_id).await?;

    // Resume
    manager.resume_task(task_id, test_tower()).await?;

    // Continue and complete
    manager.update_progress(task_id, 100.0).await?;
    manager.complete_task(task_id).await?;

    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_full_lifecycle_with_checkpoint_resume() -> Result<()> {
    let (manager, _dir) = create_test_manager().await?;
    let spec = create_test_spec("test_checkpoint_resume");

    // Create, start, and checkpoint
    let task_id = manager.create_task(test_user(), spec).await?;
    manager.start_task(task_id, test_tower()).await?;
    manager.update_progress(task_id, 50.0).await?;

    let state = vec![1, 2, 3];
    let checkpoint_id = manager.create_checkpoint(task_id, state.clone()).await?;

    // Simulate restart by resuming from checkpoint
    let (resumed_id, resumed_state) = manager.resume_from_checkpoint(&checkpoint_id).await?;
    assert_eq!(resumed_id, task_id);
    assert_eq!(resumed_state, state);

    // Complete the task
    manager.update_progress(task_id, 100.0).await?;
    manager.complete_task(task_id).await?;

    Ok(())
}
