// SPDX-License-Identifier: AGPL-3.0-only
//! Integration tests for task lifecycle management
//!
//! **Evolution**: Each test uses an isolated temp directory for its sled database,
//! enabling fully concurrent execution without file lock contention.
//! No `sleep()`, no serial — just isolated state.
#![allow(clippy::unwrap_used)] // Tests may use unwrap for cleaner code
#![allow(clippy::expect_used)] // Tests may use expect for cleaner code
#![allow(clippy::float_cmp)] // Tests may compare floats for exact values

use songbird_orchestrator::task_lifecycle::types::{Priority, ResourceRequirements, TaskSpec};
use songbird_orchestrator::task_lifecycle::{TaskLifecycleManager, TowerId, UserId};
use tempfile::TempDir;

/// Create an isolated manager with its own temp database directory.
/// This is the concurrent-safe pattern: each test gets its own sled instance.
async fn isolated_manager() -> (TaskLifecycleManager, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("tasks.db");
    let manager = TaskLifecycleManager::new(db_path.to_str().unwrap())
        .await
        .expect("Failed to create manager");
    (manager, dir) // dir must be kept alive to prevent cleanup
}

#[tokio::test]
async fn test_full_task_lifecycle() {
    let (manager, _dir) = isolated_manager().await;

    // Create task
    let owner = UserId::from("test-user");
    let spec = TaskSpec {
        task_type: "integration-test".into(),
        config: serde_json::json!({"test": true}),
        required_capabilities: vec!["compute".into()],
        resources: ResourceRequirements::default(),
        priority: Priority::High,
    };

    let task_id = manager.create_task(owner.clone(), spec).await.expect("Failed to create task");

    // Verify task created
    let task =
        manager.get_task(task_id).await.expect("Failed to get task").expect("Task not found");

    assert_eq!(task.owner, owner);
    assert_eq!(task.progress, 0.0);

    // Start task
    let tower = TowerId::from("test-tower");
    manager.start_task(task_id, tower.clone()).await.expect("Failed to start task");

    // Update progress
    manager.update_progress(task_id, 0.25).await.expect("Failed to update progress");
    manager.update_progress(task_id, 0.50).await.expect("Failed to update progress");

    // Create checkpoint
    let state = vec![1, 2, 3, 4, 5];
    let checkpoint_id = manager
        .create_checkpoint(task_id, state.clone())
        .await
        .expect("Failed to create checkpoint");

    // Verify checkpoint
    let (resumed_task_id, resumed_state) = manager
        .resume_from_checkpoint(&checkpoint_id)
        .await
        .expect("Failed to resume from checkpoint");

    assert_eq!(resumed_task_id, task_id);
    assert_eq!(resumed_state, state);

    // Continue task
    manager.update_progress(task_id, 0.75).await.expect("Failed to update progress");
    manager.update_progress(task_id, 1.0).await.expect("Failed to update progress");

    // Complete task
    manager.complete_task(task_id).await.expect("Failed to complete task");

    // Verify final state
    let final_task =
        manager.get_task(task_id).await.expect("Failed to get final task").expect("Task not found");

    assert_eq!(final_task.progress, 1.0);
    assert!(matches!(
        final_task.status,
        songbird_orchestrator::task_lifecycle::types::TaskStatus::Completed { .. }
    ));
}

#[tokio::test]
async fn test_task_pause_and_resume() {
    let (manager, _dir) = isolated_manager().await;

    let owner = UserId::from("test-user");
    let spec = TaskSpec {
        task_type: "pauseable-task".into(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task_id = manager.create_task(owner, spec).await.unwrap();

    // Start task
    let tower = TowerId::from("test-tower");
    manager.start_task(task_id, tower.clone()).await.unwrap();

    // Update some progress
    manager.update_progress(task_id, 0.3).await.unwrap();

    // Pause task
    manager.pause_task(task_id).await.unwrap();

    let paused_task = manager.get_task(task_id).await.unwrap().unwrap();
    assert!(matches!(
        paused_task.status,
        songbird_orchestrator::task_lifecycle::types::TaskStatus::Paused { .. }
    ));

    // Resume task
    manager.resume_task(task_id, tower).await.expect("Failed to resume task");

    let resumed_task = manager.get_task(task_id).await.unwrap().unwrap();
    assert!(matches!(
        resumed_task.status,
        songbird_orchestrator::task_lifecycle::types::TaskStatus::Running { .. }
    ));
    assert_eq!(resumed_task.progress, 0.3); // Progress preserved
}

#[tokio::test]
async fn test_task_cancellation() {
    let (manager, _dir) = isolated_manager().await;

    let owner = UserId::from("test-user");
    let spec = TaskSpec {
        task_type: "cancellable-task".into(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task_id = manager.create_task(owner, spec).await.unwrap();

    // Start task
    let tower = TowerId::from("test-tower");
    manager.start_task(task_id, tower).await.unwrap();

    // Cancel with reason
    let reason = "User requested cancellation";
    manager.cancel_task(task_id, Some(reason.into())).await.unwrap();

    let cancelled_task = manager.get_task(task_id).await.unwrap().unwrap();
    assert!(matches!(
        cancelled_task.status,
        songbird_orchestrator::task_lifecycle::types::TaskStatus::Cancelled { .. }
    ));
}

#[tokio::test]
async fn test_multiple_checkpoints() {
    let (manager, _dir) = isolated_manager().await;

    let owner = UserId::from("test-user");
    let spec = TaskSpec {
        task_type: "checkpoint-test".into(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task_id = manager.create_task(owner, spec).await.unwrap();
    let tower = TowerId::from("test-tower");
    manager.start_task(task_id, tower).await.unwrap();

    // Create multiple checkpoints
    let checkpoint1 = manager.create_checkpoint(task_id, vec![1, 2, 3]).await.unwrap();

    manager.update_progress(task_id, 0.5).await.unwrap();

    let checkpoint2 = manager.create_checkpoint(task_id, vec![4, 5, 6]).await.unwrap();

    manager.update_progress(task_id, 0.75).await.unwrap();

    let checkpoint3 = manager.create_checkpoint(task_id, vec![7, 8, 9]).await.unwrap();

    // Verify all checkpoints are different
    assert_ne!(checkpoint1, checkpoint2);
    assert_ne!(checkpoint2, checkpoint3);
    assert_ne!(checkpoint1, checkpoint3);

    // Verify we can resume from each
    let (id1, state1) = manager.resume_from_checkpoint(&checkpoint1).await.unwrap();
    let (id2, state2) = manager.resume_from_checkpoint(&checkpoint2).await.unwrap();
    let (id3, state3) = manager.resume_from_checkpoint(&checkpoint3).await.unwrap();

    assert_eq!(id1, task_id);
    assert_eq!(id2, task_id);
    assert_eq!(id3, task_id);

    assert_eq!(state1, vec![1, 2, 3]);
    assert_eq!(state2, vec![4, 5, 6]);
    assert_eq!(state3, vec![7, 8, 9]);
}
