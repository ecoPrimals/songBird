// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::TaskLifecycleManager;
use super::events::TaskEvent;
use crate::task_lifecycle::types::{Priority, ResourceRequirements};
use crate::task_lifecycle::{TaskFilter, TaskSpec, TowerId, UserId};
use anyhow::Result;

async fn create_test_manager() -> Result<TaskLifecycleManager> {
    let temp_dir = std::env::temp_dir().join(format!("songbird-test-{}", uuid::Uuid::new_v4()));
    TaskLifecycleManager::new(temp_dir.to_str().unwrap()).await
}

#[tokio::test]
async fn test_task_creation() -> Result<()> {
    let manager = create_test_manager().await?;

    let owner = UserId::from("alice");
    let spec = TaskSpec {
        task_type: "test-task".into(),
        config: serde_json::json!({}),
        required_capabilities: vec!["compute".into()],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task_id = manager.create_task(owner.clone(), spec).await?;

    let task = manager.get_task(task_id).await?;
    assert!(task.is_some());

    let task = task.unwrap();
    assert_eq!(task.owner, owner);

    Ok(())
}

#[tokio::test]
async fn test_task_lifecycle() -> Result<()> {
    let manager = create_test_manager().await?;

    let owner = UserId::from("alice");
    let spec = TaskSpec {
        task_type: "test-task".into(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task_id = manager.create_task(owner, spec).await?;

    let tower = TowerId::from("tower-1");
    manager.start_task(task_id, tower.clone()).await?;

    manager.update_progress(task_id, 0.5).await?;

    manager.pause_task(task_id).await?;

    manager.resume_task(task_id, tower).await?;

    manager.complete_task(task_id).await?;

    let task = manager.get_task(task_id).await?.unwrap();
    assert!(matches!(task.status, crate::task_lifecycle::types::TaskStatus::Completed { .. }));
    assert_eq!(task.progress, 1.0);

    Ok(())
}

#[tokio::test]
async fn test_checkpoint_and_resume() -> Result<()> {
    let manager = create_test_manager().await?;

    let owner = UserId::from("alice");
    let spec = TaskSpec {
        task_type: "test-task".into(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task_id = manager.create_task(owner, spec).await?;
    let tower = TowerId::from("tower-1");
    manager.start_task(task_id, tower).await?;

    let state = vec![1, 2, 3, 4, 5];
    let checkpoint_id = manager.create_checkpoint(task_id, state.clone()).await?;

    let (resumed_task_id, resumed_state) = manager.resume_from_checkpoint(&checkpoint_id).await?;

    assert_eq!(resumed_task_id, task_id);
    assert_eq!(resumed_state, state);

    Ok(())
}

#[tokio::test]
async fn test_event_emission() -> Result<()> {
    let manager = create_test_manager().await?;
    let mut rx = manager.subscribe_events();

    let owner = UserId::from("alice");
    let spec = TaskSpec {
        task_type: "test-task".into(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let _task_id = manager.create_task(owner.clone(), spec).await?;

    let event = rx.recv().await?;
    assert!(matches!(event, TaskEvent::Created { .. }));

    Ok(())
}

#[tokio::test]
async fn test_list_tasks() -> Result<()> {
    let manager = create_test_manager().await?;

    let owner = UserId::from("alice");

    for i in 0..5 {
        let spec = TaskSpec {
            task_type: format!("test-{i}").into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        manager.create_task(owner.clone(), spec).await?;
    }

    let filter = TaskFilter {
        owner: Some(owner),
        ..Default::default()
    };

    let tasks = manager.list_tasks(&filter).await?;
    assert_eq!(tasks.len(), 5);

    Ok(())
}
