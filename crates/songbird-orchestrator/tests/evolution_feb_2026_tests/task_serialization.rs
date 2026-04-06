// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task lifecycle JSON serialization (Sled/JSON evolution).

use chrono::Utc;
use songbird_orchestrator::task_lifecycle::{
    Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId,
};
use std::sync::Arc;

#[test]
fn test_task_status_json_serialization() {
    // Test all TaskStatus variants serialize correctly with JSON
    let statuses = vec![
        TaskStatus::Queued,
        TaskStatus::Running {
            started_at: Utc::now(),
        },
        TaskStatus::Completed {
            completed_at: Utc::now(),
        },
        TaskStatus::Failed {
            failed_at: Utc::now(),
            error: Arc::from("test error"),
            retry_count: 0,
        },
        TaskStatus::Cancelled {
            cancelled_at: Utc::now(),
            reason: Some(Arc::from("user request")),
        },
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).expect("Should serialize to JSON");
        let deserialized: TaskStatus =
            serde_json::from_str(&json).expect("Should deserialize from JSON");

        // Verify round-trip (comparing discriminant since times may differ)
        match (&status, &deserialized) {
            (TaskStatus::Queued, TaskStatus::Queued) => {}
            (
                TaskStatus::Running {
                    ..
                },
                TaskStatus::Running {
                    ..
                },
            ) => {}
            (
                TaskStatus::Completed {
                    ..
                },
                TaskStatus::Completed {
                    ..
                },
            ) => {}
            (
                TaskStatus::Failed {
                    ..
                },
                TaskStatus::Failed {
                    ..
                },
            ) => {}
            (
                TaskStatus::Cancelled {
                    ..
                },
                TaskStatus::Cancelled {
                    ..
                },
            ) => {}
            _ => panic!("Status mismatch after round-trip"),
        }
    }
}

#[test]
fn test_task_status_externally_tagged_format() {
    // Verify we're using externally tagged format (bincode compatible)
    let status = TaskStatus::Queued;
    let json = serde_json::to_string(&status).unwrap();

    // Externally tagged format should be "Queued" or {"Queued": null}
    assert!(
        json == "\"Queued\"" || json.contains("Queued"),
        "Should use externally tagged format, got: {json}"
    );
}

#[test]
fn test_task_lifecycle_with_json_value_config() {
    // This was the root cause of bincode failures - serde_json::Value in config
    let spec = TaskSpec {
        task_type: Arc::from("compute"),
        config: serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            },
            "number": 42,
            "boolean": true,
            "null_value": null
        }),
        required_capabilities: vec![Arc::from("compute")],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let task = TaskLifecycle::new(UserId::new("test-user"), spec);

    // Serialize to JSON (our new format)
    let json = serde_json::to_vec(&task).expect("Should serialize TaskLifecycle to JSON");

    // Deserialize back
    let deserialized: TaskLifecycle =
        serde_json::from_slice(&json).expect("Should deserialize TaskLifecycle from JSON");

    assert_eq!(task.id, deserialized.id);
    assert_eq!(task.spec.task_type, deserialized.spec.task_type);
    assert_eq!(task.spec.config, deserialized.spec.config);
}

#[test]
fn test_task_lifecycle_complex_config_patterns() {
    // Test various JSON patterns that would break bincode
    let configs = vec![
        serde_json::json!(null),
        serde_json::json!([]),
        serde_json::json!({}),
        serde_json::json!([1, "mixed", true, null]),
        serde_json::json!({"deeply": {"nested": {"value": 123}}}),
        serde_json::json!({"unicode": "日本語テスト 🎵"}),
    ];

    for config in configs {
        let spec = TaskSpec {
            task_type: Arc::from("test"),
            config: config.clone(),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task =
            TaskLifecycle::new(UserId::new(format!("config-test-{}", uuid::Uuid::new_v4())), spec);
        let json = serde_json::to_vec(&task).expect("Should serialize");
        let deserialized: TaskLifecycle =
            serde_json::from_slice(&json).expect("Should deserialize");

        assert_eq!(task.spec.config, deserialized.spec.config);
    }
}

#[test]
fn test_priority_serialization() {
    // Test Priority enum serialization
    let priorities = vec![Priority::Low, Priority::Standard, Priority::High, Priority::Critical];

    for priority in priorities {
        let json = serde_json::to_string(&priority).expect("Should serialize priority");
        let deserialized: Priority =
            serde_json::from_str(&json).expect("Should deserialize priority");
        assert_eq!(priority, deserialized);
    }
}
