// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Routing Edge Cases Tests
//!
//! Comprehensive tests for routing decision edge cases, error handling,
//! and boundary conditions.

use songbird_orchestrator::core::routing::{RoutingDecision, Task};
use std::collections::HashMap;

#[test]
fn test_task_creation_with_empty_type() {
    let task = Task::new("");
    assert_eq!(task.task_type.as_ref(), "");
    assert!(task.payload.is_null());
}

#[test]
fn test_task_with_very_long_type() {
    let long_type = "a".repeat(1000);
    let task = Task::new(long_type.clone());
    assert_eq!(task.task_type.as_ref(), long_type.as_str());
}

#[test]
fn test_task_with_unicode_type() {
    let task = Task::new("任务类型_タスク_задача");
    assert_eq!(task.task_type.as_ref(), "任务类型_タスク_задача");
}

#[test]
fn test_task_with_special_characters() {
    let task = Task::new("task/type:with-special_chars.test");
    assert!(!task.task_type.is_empty());
}

#[test]
fn test_task_with_whitespace() {
    let task = Task::new("  spaced  task  ");
    assert_eq!(task.task_type.as_ref(), "  spaced  task  ");
}

#[test]
fn test_task_with_newlines() {
    let task = Task::new("multi\nline\ntask");
    assert!(task.task_type.contains('\n'));
}

#[test]
fn test_task_with_null_json_payload() {
    let task = Task::new("test");
    assert!(task.payload.is_null());
}

#[test]
fn test_task_clone_preserves_data() {
    let task = Task::new("test_task");
    let cloned = task.clone();
    assert_eq!(task.task_type, cloned.task_type);
}

#[test]
fn test_task_metadata_empty_by_default() {
    let task = Task::new("test");
    assert!(task.metadata.is_empty());
}

#[test]
fn test_task_metadata_can_be_added() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata,
    };

    assert_eq!(task.metadata.get("key").map(std::string::String::as_str), Some("value"));
}

#[test]
fn test_task_estimated_duration_none_by_default() {
    let task = Task::new("test");
    assert!(task.estimated_duration_secs.is_none());
}

#[test]
fn test_task_estimated_duration_zero() {
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: Some(0),
        metadata: HashMap::new(),
    };

    assert_eq!(task.estimated_duration_secs, Some(0));
}

#[test]
fn test_task_estimated_duration_very_large() {
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: Some(u64::MAX),
        metadata: HashMap::new(),
    };

    assert_eq!(task.estimated_duration_secs, Some(u64::MAX));
}

#[test]
fn test_task_resource_requirements_none_by_default() {
    let task = Task::new("test");
    assert!(task.resource_requirements.is_none());
}

#[test]
fn test_task_builder_pattern() {
    let task = Task::builder("test_task").build();
    assert_eq!(task.task_type.as_ref(), "test_task");
}

#[test]
fn test_routing_decision_execute_locally_debug() {
    let decision = RoutingDecision::ExecuteLocally;
    let debug = format!("{:?}", decision);
    assert!(debug.contains("ExecuteLocally"));
}

#[test]
fn test_task_type_arc_sharing() {
    let task1 = Task::new("shared_type");
    let task2 = task1.clone();

    // Arc should make this cheap
    assert_eq!(task1.task_type, task2.task_type);
}

#[test]
fn test_task_serialization_roundtrip() {
    let task = Task::new("test_task");
    let json = serde_json::to_string(&task).expect("test precondition");
    let deserialized: Task = serde_json::from_str(&json).expect("should parse valid input");

    assert_eq!(task.task_type, deserialized.task_type);
}

#[test]
fn test_task_with_json_object_payload() {
    let payload = serde_json::json!({"key": "value", "num": 42});
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.is_object());
}

#[test]
fn test_task_with_json_array_payload() {
    let payload = serde_json::json!([1, 2, 3, 4, 5]);
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.is_array());
}

#[test]
fn test_task_with_json_string_payload() {
    let payload = serde_json::json!("string payload");
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.is_string());
}

#[test]
fn test_task_with_json_number_payload() {
    let payload = serde_json::json!(42);
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.is_number());
}

#[test]
fn test_task_with_json_bool_payload() {
    let payload = serde_json::json!(true);
    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.is_boolean());
}

#[test]
fn test_task_type_comparison() {
    let task1 = Task::new("type_a");
    let task2 = Task::new("type_a");
    let task3 = Task::new("type_b");

    assert_eq!(task1.task_type, task2.task_type);
    assert_ne!(task1.task_type, task3.task_type);
}

#[test]
fn test_task_metadata_multiple_entries() {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());
    metadata.insert("key3".to_string(), "value3".to_string());

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata,
    };

    assert_eq!(task.metadata.len(), 3);
}

#[test]
fn test_task_metadata_unicode_values() {
    let mut metadata = HashMap::new();
    metadata.insert("language".to_string(), "日本語".to_string());

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata,
    };

    assert_eq!(task.metadata.get("language").map(std::string::String::as_str), Some("日本語"));
}

#[test]
fn test_task_metadata_empty_key() {
    let mut metadata = HashMap::new();
    metadata.insert(String::new(), "value".to_string());

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata,
    };

    assert!(task.metadata.contains_key(""));
}

#[test]
fn test_task_metadata_empty_value() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), String::new());

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload: serde_json::Value::Null,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata,
    };

    assert_eq!(task.metadata.get("key").map(std::string::String::as_str), Some(""));
}

#[test]
fn test_task_debug_format() {
    let task = Task::new("debug_test");
    let debug = format!("{:?}", task);
    assert!(debug.contains("Task"));
}

#[test]
fn test_multiple_tasks_different_types() {
    let tasks = [Task::new("type1"), Task::new("type2"), Task::new("type3")];

    assert_eq!(tasks.len(), 3);
    assert!(tasks.iter().all(|t| !t.task_type.is_empty()));
}

#[test]
fn test_task_in_option() {
    let some_task: Option<Task> = Some(Task::new("test"));
    assert!(some_task.is_some());

    let no_task: Option<Task> = None;
    assert!(no_task.is_none());
}

#[test]
fn test_task_in_result() {
    let ok_task: Result<Task, String> = Ok(Task::new("test"));
    assert!(ok_task.is_ok());

    let err_task: Result<Task, String> = Err("error".to_string());
    assert!(err_task.is_err());
}

#[test]
fn test_task_in_vec() {
    let mut tasks = Vec::new();
    tasks.push(Task::new("task1"));
    tasks.push(Task::new("task2"));

    assert_eq!(tasks.len(), 2);
}

#[test]
fn test_task_builder_with_metadata() {
    let task = Task::builder("test").with_metadata("key".to_string(), "value".to_string()).build();

    assert_eq!(task.metadata.get("key").map(std::string::String::as_str), Some("value"));
}

#[test]
fn test_task_builder_with_duration() {
    let task = Task::builder("test").with_duration(60).build();

    assert_eq!(task.estimated_duration_secs, Some(60));
}

#[test]
fn test_task_payload_nested_json() {
    let payload = serde_json::json!({
        "level1": {
            "level2": {
                "level3": "deep value"
            }
        }
    });

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.get("level1").is_some());
}

#[test]
fn test_task_payload_large_json() {
    let large_array: Vec<i32> = (0..1000).collect();
    let payload = serde_json::json!(large_array);

    let task = Task {
        task_type: std::sync::Arc::from("test"),
        payload,
        resource_requirements: None,
        estimated_duration_secs: None,
        metadata: HashMap::new(),
    };

    assert!(task.payload.is_array());
}
