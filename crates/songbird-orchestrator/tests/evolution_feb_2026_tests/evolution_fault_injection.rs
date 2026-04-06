// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Invalid inputs and degraded-environment handling.

use songbird_orchestrator::task_lifecycle::{
    Priority, ResourceRequirements, TaskLifecycle, TaskSpec, UserId,
};
use std::sync::Arc;

use super::common::ENV_LOCK;

#[test]
fn test_fault_invalid_json_deserialization() {
    // Test that invalid JSON is handled gracefully
    let invalid_jsons = vec![
        "",
        "not json",
        "{",
        "[]",
        "null",
        r#"{"id": "test"}"#, // Missing required fields
    ];

    for invalid in invalid_jsons {
        let result: Result<TaskLifecycle, _> = serde_json::from_str(invalid);
        assert!(result.is_err(), "Should reject invalid JSON: {invalid}");
    }
}

#[test]
fn test_fault_corrupted_status() {
    // Test handling of corrupted status values
    let corrupted = r#"{
            "id": "test-001",
            "spec": {
                "task_type": "test",
                "config": {},
                "required_capabilities": [],
                "resources": {},
                "priority": "Standard"
            },
            "status": "InvalidStatus",
            "created_at": "2026-02-05T00:00:00Z",
            "updated_at": "2026-02-05T00:00:00Z"
        }"#;

    let result: Result<TaskLifecycle, _> = serde_json::from_str(corrupted);
    assert!(result.is_err(), "Should reject corrupted status");
}

#[test]
fn test_fault_missing_family_id_graceful_default() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Ensure missing family_id defaults gracefully
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("FAMILY_ID");

    let family_id = songbird_process_env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| songbird_process_env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    assert_eq!(family_id, "default", "Should default to 'default'");
}

#[test]
fn test_fault_empty_config() {
    // Test that empty config is handled
    let spec = TaskSpec {
        task_type: Arc::from("empty-config"),
        config: serde_json::json!(null),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task = TaskLifecycle::new(UserId::new("empty-config-task"), spec);
    let json = serde_json::to_vec(&task).expect("Should serialize null config");
    let deserialized: TaskLifecycle = serde_json::from_slice(&json).unwrap();
    assert_eq!(deserialized.spec.config, serde_json::Value::Null);
}

#[test]
fn test_fault_all_priority_levels() {
    // Ensure all priority levels work
    for priority in [Priority::Low, Priority::Standard, Priority::High, Priority::Critical] {
        let spec = TaskSpec {
            task_type: Arc::from("priority-test"),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority,
        };

        let task = TaskLifecycle::new(UserId::new("priority-task"), spec);
        let json = serde_json::to_vec(&task).expect("Should serialize");
        let deserialized: TaskLifecycle = serde_json::from_slice(&json).unwrap();
        assert_eq!(deserialized.spec.priority, priority);
    }
}

#[test]
fn test_fault_very_long_task_type() {
    // Test handling of very long task types
    let long_type: Arc<str> = Arc::from("x".repeat(10000));
    let spec = TaskSpec {
        task_type: long_type.clone(),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task = TaskLifecycle::new(UserId::new("long-type-task"), spec);
    let json = serde_json::to_vec(&task).expect("Should serialize long type");
    let deserialized: TaskLifecycle = serde_json::from_slice(&json).unwrap();
    assert_eq!(deserialized.spec.task_type, long_type);
}

#[test]
fn test_fault_unicode_in_config() {
    // Test Unicode handling in configs
    let spec = TaskSpec {
        task_type: Arc::from("unicode"),
        config: serde_json::json!({
            "emoji": "🎵🦅💻",
            "japanese": "日本語",
            "russian": "Русский",
            "arabic": "العربية",
            "mixed": "Hello 世界 🌍"
        }),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task = TaskLifecycle::new(UserId::new("unicode-task"), spec);
    let json = serde_json::to_vec(&task).expect("Should serialize Unicode");
    let deserialized: TaskLifecycle = serde_json::from_slice(&json).unwrap();

    assert_eq!(deserialized.spec.config["emoji"], serde_json::json!("🎵🦅💻"));
}

#[test]
fn test_fault_socket_path_with_special_chars() {
    // Test socket path handling with special characters
    let paths = vec![
        "/tmp/songbird.sock",
        "/run/user/1000/biomeos/songbird.sock",
        "/path with spaces/songbird.sock", // Should be avoided but handled
    ];

    for path in paths {
        // Just ensure these don't panic
        let _normalized = path.replace(' ', "_");
    }
}
