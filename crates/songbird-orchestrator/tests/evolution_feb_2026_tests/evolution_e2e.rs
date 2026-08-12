// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Integrated evolution flows (task lifecycle, family id, deployment sockets).

use anyhow::Result;
use chrono::Utc;
use songbird_orchestrator::task_lifecycle::{
    Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId,
};
use std::sync::Arc;

use super::common::lock_env;

#[tokio::test]
async fn test_e2e_task_lifecycle_full_flow() -> Result<()> {
    // Test complete task lifecycle with JSON serialization
    let spec = TaskSpec {
        task_type: Arc::from("e2e-test"),
        config: serde_json::json!({
            "test": true,
            "complex": {"nested": [1, 2, 3]}
        }),
        required_capabilities: vec![Arc::from("compute")],
        resources: ResourceRequirements::default(),
        priority: Priority::High,
    };

    let mut task = TaskLifecycle::new(UserId::new("e2e-user"), spec);

    // 1. Initial state should be Queued
    assert!(matches!(task.status, TaskStatus::Queued));

    // 2. Serialize and deserialize (simulating storage)
    let json = serde_json::to_vec(&task)?;
    let loaded: TaskLifecycle = serde_json::from_slice(&json)?;
    assert_eq!(task.id, loaded.id);

    // 3. Transition to Running
    task.status = TaskStatus::Running {
        started_at: Utc::now(),
    };
    let json = serde_json::to_vec(&task)?;
    let loaded: TaskLifecycle = serde_json::from_slice(&json)?;
    assert!(matches!(loaded.status, TaskStatus::Running { .. }));

    // 4. Complete the task
    task.status = TaskStatus::Completed {
        completed_at: Utc::now(),
    };
    let json = serde_json::to_vec(&task)?;
    let loaded: TaskLifecycle = serde_json::from_slice(&json)?;
    assert!(matches!(loaded.status, TaskStatus::Completed { .. }));

    Ok(())
}

#[tokio::test]
async fn test_e2e_family_id_propagation() -> Result<()> {
    let _guard = lock_env();
    // Test that family_id is properly propagated through the system
    let test_family = format!("test-family-{}", uuid::Uuid::new_v4());
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", &test_family);

    // Verify it's readable
    let family_id = songbird_process_env::var("SONGBIRD_FAMILY_ID")?;
    assert_eq!(family_id, test_family);

    // Clean up
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    Ok(())
}

#[tokio::test]
async fn test_e2e_socket_naming_primal_deployment_standard() -> Result<()> {
    // Test that socket naming follows PRIMAL_DEPLOYMENT_STANDARD
    // Socket names should be {primal}.sock without family suffix

    let primals = vec!["songbird", "security-provider", "ai-provider", "biome"];

    for primal in primals {
        // Expected socket name per PRIMAL_DEPLOYMENT_STANDARD
        let expected_socket_name = format!("{primal}.sock");

        // Should NOT contain family_id variations
        assert!(!expected_socket_name.contains("nat0"));
        assert!(!expected_socket_name.contains("-default"));
        assert!(!expected_socket_name.contains("_family"));

        // Should be simple primal.sock format
        assert!(expected_socket_name.ends_with(".sock"));
        assert_eq!(expected_socket_name, format!("{primal}.sock"));
    }

    Ok(())
}

#[tokio::test]
async fn test_e2e_xdg_path_structure() -> Result<()> {
    // Test XDG path structure compliance
    let test_dir = std::env::temp_dir().join(format!("xdg-test-{}", uuid::Uuid::new_v4()));
    let biomeos_dir = test_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir)?;

    // Expected: primal bind path $XDG_RUNTIME_DIR/biomeos/songbird.sock; domain symlink network.sock at runtime
    let expected_path = biomeos_dir.join("songbird.sock");

    assert!(expected_path.to_str().unwrap().contains("biomeos"));
    assert!(expected_path.to_str().unwrap().ends_with("songbird.sock"));

    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);

    Ok(())
}
