// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Stress and chaos-style serialization / env tests.

use chrono::Utc;
use songbird_orchestrator::task_lifecycle::{
    Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId,
};
use std::sync::Arc;

#[test]
fn test_chaos_rapid_serialization_cycles() {
    // Rapidly serialize/deserialize to test for race conditions or memory issues
    for i in 0..1000 {
        let spec = TaskSpec {
            task_type: Arc::from("chaos"),
            config: serde_json::json!({"chaos": true, "iteration": i}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task = TaskLifecycle::new(UserId::new(format!("chaos-{i}")), spec);
        let json = serde_json::to_vec(&task).expect("Serialize should not fail");
        let _: TaskLifecycle = serde_json::from_slice(&json).expect("Deserialize should not fail");
    }
}

#[test]
fn test_chaos_concurrent_family_id_reads() {
    use std::sync::Arc as StdArc;
    use std::thread;

    let family_id = StdArc::new("chaos-family".to_string());

    let handles: Vec<_> = (0..100)
        .map(|i| {
            let fid = StdArc::clone(&family_id);
            thread::spawn(move || {
                // Simulate concurrent reads
                for _ in 0..100 {
                    let _ = fid.as_str();
                }
                i
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

#[test]
fn test_chaos_large_config_serialization() {
    // Test with increasingly large configs
    for size in [100, 1000, 10000] {
        let large_array: Vec<i32> = (0..size).collect();
        let spec = TaskSpec {
            task_type: Arc::from("large-config"),
            config: serde_json::json!({
                "large_array": large_array,
                "size": size
            }),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task = TaskLifecycle::new(UserId::new(format!("large-{size}")), spec);
        let json = serde_json::to_vec(&task).expect("Should handle large configs");
        let deserialized: TaskLifecycle = serde_json::from_slice(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
    }
}

#[test]
fn test_chaos_environment_variable_race() {
    // Rapidly set/unset environment variables to test for races
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let var_name = format!("CHAOS_VAR_{i}");
            thread::spawn(move || {
                for j in 0..100 {
                    songbird_process_env::set_var(&var_name, format!("value-{j}"));
                    let _ = songbird_process_env::var(&var_name);
                    songbird_process_env::remove_var(&var_name);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

#[test]
fn test_chaos_status_transitions() {
    // Rapid status transitions
    let spec = TaskSpec {
        task_type: Arc::from("transitions"),
        config: serde_json::json!({}),
        required_capabilities: vec![],
        resources: ResourceRequirements::default(),
        priority: Priority::Standard,
    };

    let mut task = TaskLifecycle::new(UserId::new("transition-test"), spec);

    for _ in 0..100 {
        task.status = TaskStatus::Running {
            started_at: Utc::now(),
        };
        let json = serde_json::to_vec(&task).unwrap();
        let _: TaskLifecycle = serde_json::from_slice(&json).unwrap();

        task.status = TaskStatus::Completed {
            completed_at: Utc::now(),
        };
        let json = serde_json::to_vec(&task).unwrap();
        let _: TaskLifecycle = serde_json::from_slice(&json).unwrap();
    }
}
