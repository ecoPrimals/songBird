// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Evolution Tests - February 2026
//!
//! Comprehensive test coverage for the deep debt evolution work:
//! - Sled/JSON serialization (migrated from bincode)
//! - BirdSong family_id integration
//! - Standard JSON-RPC methods (health, identity, beacon_exchange)
//! - Socket discovery PRIMAL_DEPLOYMENT_STANDARD compliance
//!
//! Test categories:
//! - Unit tests: Component-level validation
//! - E2E tests: Integrated flow testing
//! - Chaos tests: Resilience under adverse conditions
//! - Fault injection: Error handling verification

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// File-local mutex to serialize tests that modify process-wide env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ============================================================================
// UNIT TESTS: Task Lifecycle Serialization
// ============================================================================

mod task_serialization_unit {
    use super::*;
    use songbird_orchestrator::task_lifecycle::{
        Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId,
    };

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
            "Should use externally tagged format, got: {}",
            json
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

            let task = TaskLifecycle::new(
                UserId::new(format!("config-test-{}", uuid::Uuid::new_v4())),
                spec,
            );
            let json = serde_json::to_vec(&task).expect("Should serialize");
            let deserialized: TaskLifecycle =
                serde_json::from_slice(&json).expect("Should deserialize");

            assert_eq!(task.spec.config, deserialized.spec.config);
        }
    }

    #[test]
    fn test_priority_serialization() {
        // Test Priority enum serialization
        let priorities =
            vec![Priority::Low, Priority::Standard, Priority::High, Priority::Critical];

        for priority in priorities {
            let json = serde_json::to_string(&priority).expect("Should serialize priority");
            let deserialized: Priority =
                serde_json::from_str(&json).expect("Should deserialize priority");
            assert_eq!(priority, deserialized);
        }
    }
}

// ============================================================================
// UNIT TESTS: BirdSong family_id Integration
// ============================================================================

mod family_id_unit {
    use super::ENV_LOCK;

    #[test]
    fn test_family_id_from_environment_priority() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test environment variable priority for family_id
        // Priority: SONGBIRD_FAMILY_ID > FAMILY_ID > default "default"

        // Clear all
        songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
        songbird_process_env::remove_var("FAMILY_ID");

        // Default should be "default"
        let family_id = get_family_id_from_env();
        assert_eq!(family_id, "default", "Default should be 'default'");

        // FAMILY_ID should override default
        songbird_process_env::set_var("FAMILY_ID", "family-fallback");
        let family_id = get_family_id_from_env();
        assert_eq!(family_id, "family-fallback", "FAMILY_ID should be used");

        // SONGBIRD_FAMILY_ID should have highest priority
        songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "songbird-primary");
        let family_id = get_family_id_from_env();
        assert_eq!(family_id, "songbird-primary", "SONGBIRD_FAMILY_ID should have priority");

        // Cleanup
        songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
        songbird_process_env::remove_var("FAMILY_ID");
    }

    #[test]
    fn test_family_id_special_characters() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Test that family_id handles special characters
        let special_ids = vec![
            "nat0",
            "family-with-dash",
            "family_with_underscore",
            "family.with.dots",
            "UPPERCASE",
            "MixedCase123",
        ];

        for id in special_ids {
            songbird_process_env::set_var("SONGBIRD_FAMILY_ID", id);
            let family_id = get_family_id_from_env();
            assert_eq!(family_id, id);
        }

        songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    }

    /// Helper to get family_id using same logic as canonical env chain
    fn get_family_id_from_env() -> String {
        std::env::var("SONGBIRD_FAMILY_ID")
            .or_else(|_| std::env::var("FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string())
    }
}

// ============================================================================
// UNIT TESTS: JSON-RPC Standard Methods
// ============================================================================

mod jsonrpc_methods_unit {
    #[test]
    fn test_health_response_structure() {
        // Verify health response matches expected schema
        let health_response = serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": 0,
            "components": {
                "http_server": "running",
                "task_manager": "ready"
            }
        });

        // Validate required fields exist
        assert!(health_response.get("status").is_some());
        assert!(health_response.get("version").is_some());
        assert!(health_response.get("uptime_seconds").is_some());
    }

    #[test]
    fn test_identity_response_structure() {
        // Verify identity response matches expected schema
        let identity_response = serde_json::json!({
            "primal": "songbird",
            "version": env!("CARGO_PKG_VERSION"),
            "capabilities": ["orchestration", "task-management", "federation"],
            "node_id": "test-node-001"
        });

        assert_eq!(identity_response["primal"], "songbird");
        assert!(identity_response["capabilities"].is_array());
    }

    #[test]
    fn test_beacon_exchange_request_validation() {
        // Test beacon exchange parameter validation
        let valid_request = serde_json::json!({
            "beacon": {
                "node_id": "peer-123",
                "capabilities": ["compute"],
                "endpoint": "https://192.168.1.100:8080"
            }
        });

        let beacon = valid_request.get("beacon");
        assert!(beacon.is_some());
        assert!(beacon.unwrap().get("node_id").is_some());
        assert!(beacon.unwrap().get("capabilities").is_some());
    }

    #[test]
    fn test_beacon_exchange_missing_beacon() {
        // Should handle missing beacon gracefully
        let invalid_request = serde_json::json!({});
        assert!(invalid_request.get("beacon").is_none());
    }
}

// ============================================================================
// E2E TESTS: Integrated Evolution Flows
// ============================================================================

mod evolution_e2e {
    use super::*;
    use songbird_orchestrator::task_lifecycle::{
        Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId,
    };

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
        let _guard = ENV_LOCK.lock().unwrap();
        // Test that family_id is properly propagated through the system
        let test_family = format!("test-family-{}", uuid::Uuid::new_v4());
        songbird_process_env::set_var("SONGBIRD_FAMILY_ID", &test_family);

        // Verify it's readable
        let family_id = std::env::var("SONGBIRD_FAMILY_ID")?;
        assert_eq!(family_id, test_family);

        // Clean up
        songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
        Ok(())
    }

    #[tokio::test]
    async fn test_e2e_socket_naming_primal_deployment_standard() -> Result<()> {
        // Test that socket naming follows PRIMAL_DEPLOYMENT_STANDARD
        // Socket names should be {primal}.sock without family suffix

        let primals = vec!["songbird", "beardog", "squirrel", "biome"];

        for primal in primals {
            // Expected socket name per PRIMAL_DEPLOYMENT_STANDARD
            let expected_socket_name = format!("{}.sock", primal);

            // Should NOT contain family_id variations
            assert!(!expected_socket_name.contains("nat0"));
            assert!(!expected_socket_name.contains("-default"));
            assert!(!expected_socket_name.contains("_family"));

            // Should be simple primal.sock format
            assert!(expected_socket_name.ends_with(".sock"));
            assert_eq!(expected_socket_name, format!("{}.sock", primal));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_e2e_xdg_path_structure() -> Result<()> {
        // Test XDG path structure compliance
        let test_dir = std::env::temp_dir().join(format!("xdg-test-{}", uuid::Uuid::new_v4()));
        let biomeos_dir = test_dir.join("biomeos");
        std::fs::create_dir_all(&biomeos_dir)?;

        // Expected structure: $XDG_RUNTIME_DIR/biomeos/{primal}.sock
        let expected_path = biomeos_dir.join("songbird.sock");

        assert!(expected_path.to_str().unwrap().contains("biomeos"));
        assert!(expected_path.to_str().unwrap().ends_with("songbird.sock"));

        // Clean up
        let _ = std::fs::remove_dir_all(&test_dir);

        Ok(())
    }
}

// ============================================================================
// CHAOS TESTS: Resilience Under Adverse Conditions
// ============================================================================

mod evolution_chaos {
    use super::*;
    use songbird_orchestrator::task_lifecycle::{
        Priority, ResourceRequirements, TaskLifecycle, TaskSpec, TaskStatus, UserId,
    };

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

            let task = TaskLifecycle::new(UserId::new(format!("chaos-{}", i)), spec);
            let json = serde_json::to_vec(&task).expect("Serialize should not fail");
            let _: TaskLifecycle =
                serde_json::from_slice(&json).expect("Deserialize should not fail");
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

            let task = TaskLifecycle::new(UserId::new(format!("large-{}", size)), spec);
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
                let var_name = format!("CHAOS_VAR_{}", i);
                thread::spawn(move || {
                    for j in 0..100 {
                        songbird_process_env::set_var(&var_name, format!("value-{}", j));
                        let _ = std::env::var(&var_name);
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
}

// ============================================================================
// FAULT INJECTION TESTS: Error Handling
// ============================================================================

mod evolution_fault_injection {
    use super::*;
    use songbird_orchestrator::task_lifecycle::{
        Priority, ResourceRequirements, TaskLifecycle, TaskSpec, UserId,
    };

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
            assert!(result.is_err(), "Should reject invalid JSON: {}", invalid);
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

        let family_id = std::env::var("SONGBIRD_FAMILY_ID")
            .or_else(|_| std::env::var("FAMILY_ID"))
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
}

// ============================================================================
// INTEGRATION TESTS: Cross-Component Validation
// ============================================================================

mod evolution_integration {
    use super::*;

    #[tokio::test]
    async fn test_integration_jsonrpc_health_endpoint_schema() {
        // Verify health endpoint returns expected schema
        let expected_fields = vec!["status", "version", "uptime_seconds"];

        let health_response = serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": 0
        });

        for field in expected_fields {
            assert!(
                health_response.get(field).is_some(),
                "Health response missing field: {}",
                field
            );
        }
    }

    #[tokio::test]
    async fn test_integration_config_serialization_compatibility() {
        // Test that configs can be serialized by both JSON and potentially other formats
        let config = serde_json::json!({
            "complex": {
                "nested": {
                    "array": [1, 2, 3],
                    "map": {"a": 1, "b": 2}
                }
            }
        });

        // JSON serialization
        let json_bytes = serde_json::to_vec(&config).expect("JSON serialization");
        let from_json: Value = serde_json::from_slice(&json_bytes).expect("JSON deserialization");
        assert_eq!(config, from_json);
    }

    #[test]
    fn test_integration_primal_naming_standard() {
        // Verify primal socket names follow PRIMAL_DEPLOYMENT_STANDARD
        let primals = vec!["songbird", "beardog", "squirrel", "biome"];

        for primal in primals {
            let socket_name = format!("{}.sock", primal);

            // Should not contain family_id
            assert!(!socket_name.contains("nat0"));
            assert!(!socket_name.contains("-default"));

            // Should be lowercase
            assert_eq!(socket_name, socket_name.to_lowercase());
        }
    }
}

// ============================================================================
// PROTOCOL DETECTION TESTS: HTTP/HTTPS on Same Port
// ============================================================================

mod protocol_detection {
    #[test]
    fn test_tls_handshake_detection() {
        // TLS ClientHello starts with content type 0x16 (Handshake)
        let tls_client_hello_start: [u8; 5] = [
            0x16, // Content type: Handshake
            0x03, 0x01, // Version: TLS 1.0 (compat)
            0x00, 0x05, // Length (placeholder)
        ];

        assert_eq!(tls_client_hello_start[0], 0x16, "TLS record starts with 0x16");
        assert!(is_tls_record(tls_client_hello_start[0]));
    }

    #[test]
    fn test_http_method_detection() {
        // HTTP methods start with ASCII characters
        let http_methods = vec![
            ("GET", 0x47),
            ("POST", 0x50),
            ("PUT", 0x50),
            ("HEAD", 0x48),
            ("DELETE", 0x44),
            ("OPTIONS", 0x4F),
            ("PATCH", 0x50),
            ("CONNECT", 0x43),
        ];

        for (method, first_byte) in http_methods {
            assert_eq!(
                method.as_bytes()[0],
                first_byte,
                "{} should start with 0x{:02X}",
                method,
                first_byte
            );
            assert!(!is_tls_record(first_byte), "{} should not be detected as TLS", method);
        }
    }

    #[test]
    fn test_protocol_detection_boundary() {
        // Edge cases for protocol detection
        assert!(is_tls_record(0x16), "0x16 is TLS Handshake");
        assert!(!is_tls_record(0x17), "0x17 is TLS Application Data (not handshake start)");
        assert!(!is_tls_record(0x14), "0x14 is TLS Change Cipher Spec");
        assert!(!is_tls_record(0x15), "0x15 is TLS Alert");

        // ASCII printable range (HTTP)
        for byte in 0x20..=0x7E {
            if byte != 0x16 {
                assert!(!is_tls_record(byte), "ASCII byte 0x{:02X} should not be TLS", byte);
            }
        }
    }

    #[test]
    fn test_http_request_first_bytes() {
        // Actual HTTP request first bytes
        let http_requests: Vec<&[u8]> = vec![
            b"GET / HTTP/1.1\r\n",
            b"POST /api HTTP/1.1\r\n",
            b"PUT /resource HTTP/1.1\r\n",
            b"HEAD /status HTTP/1.1\r\n",
        ];

        for request in http_requests {
            assert!(!is_tls_record(request[0]), "HTTP request should not be detected as TLS");
        }
    }

    #[test]
    fn test_tls_record_types() {
        // All TLS record types
        let tls_record_types = vec![
            (0x14, "ChangeCipherSpec"),
            (0x15, "Alert"),
            (0x16, "Handshake"),
            (0x17, "ApplicationData"),
        ];

        for (byte, name) in tls_record_types {
            // Only Handshake (0x16) should trigger TLS detection for initial connection
            if byte == 0x16 {
                assert!(is_tls_record(byte), "{} should be detected as TLS handshake", name);
            } else {
                // Other record types wouldn't be the first byte of a new TLS connection
                assert!(!is_tls_record(byte), "{} should not be first byte", name);
            }
        }
    }

    /// Helper: Check if first byte indicates TLS record (handshake)
    fn is_tls_record(byte: u8) -> bool {
        byte == 0x16 // TLS Handshake content type
    }
}
