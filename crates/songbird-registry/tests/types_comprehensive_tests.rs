// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
#![allow(clippy::all, reason = "test assertions and harness ergonomics")]
#![allow(unused, reason = "test assertions and harness ergonomics")]

//! Comprehensive Type Tests for Registry
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Tests for plugin, capability, health, and event types.

use serde_json::json;
use songbird_registry::types::*;
use songbird_types::SongbirdError;
use std::collections::HashMap;

// ========== PluginId Tests ==========

#[test]
fn test_plugin_id_creation() {
    let id = PluginId::new("test-plugin");
    assert_eq!(id.as_str(), "test-plugin");
}

#[test]
fn test_plugin_id_from_string() {
    let id: PluginId = "my-plugin".into();
    assert_eq!(id.as_str(), "my-plugin");

    let id2: PluginId = String::from("owned-plugin").into();
    assert_eq!(id2.as_str(), "owned-plugin");
}

#[test]
fn test_plugin_id_equality() {
    let id1 = PluginId::new("plugin-a");
    let id2 = PluginId::new("plugin-a");
    let id3 = PluginId::new("plugin-b");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_plugin_id_clone() {
    let id = PluginId::new("cloneable");
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

#[test]
fn test_plugin_id_display() {
    let id = PluginId::new("display-test");
    let display = format!("{id}");
    assert_eq!(display, "display-test");
}

#[test]
fn test_plugin_id_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let id = PluginId::new("serialize-me");
    let json = serde_json::to_string(&id)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;
    assert!(json.contains("serialize-me"));
    Ok(())
}

#[test]
fn test_plugin_id_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#""deserialized-plugin""#;
    let id: PluginId = serde_json::from_str(json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {e}")))?;
    assert_eq!(id.as_str(), "deserialized-plugin");
    Ok(())
}

// ========== Plugin Tests ==========

#[test]
fn test_plugin_basic_creation() {
    let plugin = Plugin::new("test", "Test Plugin", "1.0.0");
    assert_eq!(plugin.id.as_str(), "test");
    assert_eq!(plugin.name, "Test Plugin");
    assert_eq!(plugin.version, "1.0.0");
    assert!(plugin.capabilities.is_empty());
    assert!(plugin.dependencies.is_empty());
}

#[test]
fn test_plugin_with_capability() {
    let cap = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes256".to_string()],
        key_sizes: vec![256],
    });

    let plugin = Plugin::new("crypto", "Crypto Plugin", "1.0.0").with_capability(cap);

    assert_eq!(plugin.capabilities.len(), 1);
}

#[test]
fn test_plugin_with_dependency() {
    let plugin =
        Plugin::new("dependent", "Dependent Plugin", "1.0.0").with_dependency("base-plugin");

    assert_eq!(plugin.dependencies.len(), 1);
    assert_eq!(plugin.dependencies[0].as_str(), "base-plugin");
}

#[test]
fn test_plugin_with_metadata() {
    let metadata = PluginMetadata {
        author: "Test Author".to_string(),
        description: "Test Description".to_string(),
        tags: vec!["test".to_string(), "plugin".to_string()],
        health_endpoint: Some("/health".to_string()),
        extra: serde_json::Map::new(),
    };

    let plugin = Plugin::new("meta", "Meta Plugin", "1.0.0").with_metadata(metadata);

    assert_eq!(plugin.metadata.author, "Test Author");
    assert_eq!(plugin.metadata.tags.len(), 2);
}

#[test]
fn test_plugin_builder_chain() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    let metadata = PluginMetadata::default();

    let plugin = Plugin::new("complex", "Complex Plugin", "2.0.0")
        .with_capability(cap)
        .with_dependency("dep1")
        .with_dependency("dep2")
        .with_metadata(metadata);

    assert_eq!(plugin.capabilities.len(), 1);
    assert_eq!(plugin.dependencies.len(), 2);
}

#[test]
fn test_plugin_clone() {
    let plugin = Plugin::new("clone-test", "Clone Test", "1.0.0");
    let cloned = plugin.clone();

    assert_eq!(plugin.id, cloned.id);
    assert_eq!(plugin.name, cloned.name);
}

#[test]
fn test_plugin_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let plugin = Plugin::new("serialize", "Serialize Plugin", "1.0.0");
    let json = serde_json::to_string(&plugin)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;

    assert!(json.contains("serialize"));
    assert!(json.contains("Serialize Plugin"));
    assert!(json.contains("1.0.0"));
    Ok(())
}

// ========== PluginMetadata Tests ==========

#[test]
fn test_plugin_metadata_default() {
    let metadata = PluginMetadata::default();
    assert_eq!(metadata.author, "unknown");
    assert!(metadata.description.is_empty());
    assert!(metadata.tags.is_empty());
    assert!(metadata.health_endpoint.is_none());
}

#[test]
fn test_plugin_metadata_with_values() {
    let mut extra = serde_json::Map::new();
    extra.insert("custom_key".to_string(), json!("custom_value"));

    let metadata = PluginMetadata {
        author: "Author Name".to_string(),
        description: "A test plugin".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        health_endpoint: Some("/api/health".to_string()),
        extra,
    };

    assert_eq!(metadata.author, "Author Name");
    assert_eq!(metadata.tags.len(), 2);
    assert_eq!(metadata.extra.len(), 1);
}

// ========== Capability Tests ==========

#[test]
fn test_capability_encryption() {
    let cap = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes256".to_string(), "rsa2048".to_string()],
        key_sizes: vec![256, 2048],
    });

    assert!(cap.active);
    assert_eq!(cap.version, "1.0.0");
}

#[test]
fn test_capability_service_discovery() {
    let cap = Capability::new(CapabilityType::ServiceDiscovery {
        protocols: vec!["mdns".to_string(), "dns-sd".to_string()],
    });

    assert!(cap.active);
}

#[test]
fn test_capability_compute() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 8,
        memory_gb: 16,
    });

    assert!(cap.active);
}

#[test]
fn test_capability_network() {
    let cap = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 1000,
        latency_ms: 10,
    });

    assert!(cap.active);
}

#[test]
fn test_capability_storage() {
    let cap = Capability::new(CapabilityType::Storage {
        size_gb: 500,
        storage_type: "nvme".to_string(),
    });

    assert!(cap.active);
}

#[test]
fn test_capability_custom() {
    let mut attributes = HashMap::new();
    attributes.insert("key1".to_string(), "value1".to_string());

    let cap = Capability::new(CapabilityType::Custom {
        name: "custom-cap".to_string(),
        attributes,
    });

    assert!(cap.active);
}

#[test]
fn test_capability_with_version() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    })
    .with_version("2.5.0");

    assert_eq!(cap.version, "2.5.0");
}

#[test]
fn test_capability_with_active() {
    let cap = Capability::new(CapabilityType::Storage {
        size_gb: 100,
        storage_type: "ssd".to_string(),
    })
    .with_active(false);

    assert!(!cap.active);
}

#[test]
fn test_capability_compatibility_same_type() {
    let cap1 = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes256".to_string()],
        key_sizes: vec![256],
    });

    let cap2 = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes128".to_string()],
        key_sizes: vec![128],
    });

    assert!(cap1.compatible_with(&cap2));
}

#[test]
fn test_capability_compatibility_different_type() {
    let cap1 = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes256".to_string()],
        key_sizes: vec![256],
    });

    let cap2 = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    assert!(!cap1.compatible_with(&cap2));
}

#[test]
fn test_capability_clone() {
    let cap = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 100,
        latency_ms: 5,
    });

    let cloned = cap.clone();
    assert_eq!(cap.version, cloned.version);
    assert_eq!(cap.active, cloned.active);
}

#[test]
fn test_capability_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let cap = Capability::new(CapabilityType::ServiceDiscovery {
        protocols: vec!["mdns".to_string()],
    });

    let json = serde_json::to_string(&cap)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;
    assert!(json.contains("service_discovery"));
    assert!(json.contains("mdns"));
    Ok(())
}

// ========== HealthStatus Tests ==========

#[test]
fn test_health_status_healthy() {
    let status = HealthStatus::healthy();
    assert!(status.healthy);
    assert_eq!(status.score, 1.0);
}

#[test]
fn test_health_status_unhealthy() {
    let status = HealthStatus::unhealthy("Service down");
    assert!(!status.healthy);
    assert_eq!(status.score, 0.0);
    assert!(status.message.is_some());
}

#[test]
fn test_health_status_degraded() {
    let status = HealthStatus::degraded(0.7, "High load");
    assert!(status.healthy); // 0.7 > 0.5
    assert_eq!(status.score, 0.7);
}

#[test]
fn test_health_status_with_response_time() {
    use std::time::Duration;
    let status = HealthStatus::healthy().with_response_time(Duration::from_millis(50));
    assert_eq!(status.response_time.as_millis(), 50);
}

#[test]
fn test_health_status_with_metadata() {
    let status = HealthStatus::healthy().with_metadata("cpu", "25%".to_string());
    assert!(status.message.is_some());
    assert!(status.message.expect("test precondition").contains("cpu=25%"));
}

#[test]
fn test_health_status_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let status = HealthStatus::healthy();
    let json = serde_json::to_string(&status)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;
    assert!(json.contains("healthy") || json.contains("score"));
    Ok(())
}

// ========== HealthCheckType Tests ==========

#[test]
fn test_health_check_type_http_endpoint() {
    let check = HealthCheckType::HttpEndpoint {
        url: "http://localhost:8080/health".to_string(),
        expected_status: 200,
    };
    assert!(matches!(check, HealthCheckType::HttpEndpoint { .. }));
}

#[test]
fn test_health_check_type_process() {
    let check = HealthCheckType::ProcessCheck {
        process_name: "my-service".to_string(),
    };
    assert!(matches!(check, HealthCheckType::ProcessCheck { .. }));
}

#[test]
fn test_health_check_type_memory() {
    let check = HealthCheckType::MemoryUsage {
        max_percentage: 80.0,
    };
    assert!(matches!(check, HealthCheckType::MemoryUsage { .. }));
}

#[test]
fn test_health_check_type_cpu() {
    let check = HealthCheckType::CpuUsage {
        max_percentage: 90.0,
    };
    assert!(matches!(check, HealthCheckType::CpuUsage { .. }));
}

#[test]
fn test_health_check_type_custom_script() {
    let check = HealthCheckType::CustomScript {
        script_path: "/usr/local/bin/health-check.sh".to_string(),
    };
    assert!(matches!(check, HealthCheckType::CustomScript { .. }));
}

// ========== HealthCheckConfig Tests ==========

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();
    assert!(config.interval.as_secs() > 0);
    assert!(config.timeout.as_secs() > 0);
    assert!(config.failure_threshold > 0);
}

#[test]
fn test_health_check_config_with_type() {
    use std::time::Duration;

    let check_type = HealthCheckType::HttpEndpoint {
        url: "http://localhost:8080/api/health".to_string(),
        expected_status: 200,
    };

    let config = HealthCheckConfig {
        check_type,
        interval: Duration::from_secs(30),
        timeout: Duration::from_secs(10),
        failure_threshold: 3,
        success_threshold: 1,
    };

    assert_eq!(config.interval.as_secs(), 30);
    assert_eq!(config.failure_threshold, 3);
}

// ========== EventType Tests ==========

#[test]
fn test_event_type_plugin_registered() {
    let event = EventType::PluginRegistered {
        plugin_id: PluginId::new("test-plugin"),
    };

    assert!(matches!(event, EventType::PluginRegistered { .. }));
}

#[test]
fn test_event_type_plugin_unregistered() {
    let event = EventType::PluginUnregistered {
        plugin_id: PluginId::new("test-plugin"),
    };

    assert!(matches!(event, EventType::PluginUnregistered { .. }));
}

#[test]
fn test_event_type_plugin_scaled() {
    let event = EventType::PluginScaled {
        plugin_id: PluginId::new("test-plugin"),
        from: 1,
        to: 3,
    };

    assert!(matches!(event, EventType::PluginScaled { .. }));
}

#[test]
fn test_event_type_health_changed() {
    let event = EventType::HealthChanged {
        plugin_id: PluginId::new("test-plugin"),
        status: HealthStatus::healthy(),
    };

    assert!(matches!(event, EventType::HealthChanged { .. }));
}

#[test]
fn test_event_type_error() {
    let event = EventType::Error {
        message: "Test error".to_string(),
        plugin_id: Some(PluginId::new("test-plugin")),
    };

    assert!(matches!(event, EventType::Error { .. }));
}

// ========== RegistryEvent Tests ==========

#[test]
fn test_registry_event_creation() {
    use std::time::SystemTime;

    let before = SystemTime::now();
    let event = RegistryEvent::new(EventType::PluginRegistered {
        plugin_id: PluginId::new("test"),
    });
    let after = SystemTime::now();

    assert!(event.timestamp >= before && event.timestamp <= after);
}

#[test]
fn test_registry_event_plugin_id() {
    let event = RegistryEvent::new(EventType::PluginRegistered {
        plugin_id: PluginId::new("test"),
    });

    assert!(event.plugin_id().is_some());
    assert_eq!(event.plugin_id().expect("test precondition").as_str(), "test");
}

#[test]
fn test_registry_event_with_context() {
    let event = RegistryEvent::new(EventType::Error {
        message: "Test error".to_string(),
        plugin_id: None,
    })
    .with_context(serde_json::json!({"severity": "high"}));

    assert!(event.context.is_some());
}

#[test]
fn test_registry_event_clone() {
    let event = RegistryEvent::new(EventType::PluginRegistered {
        plugin_id: PluginId::new("test"),
    });

    let cloned = event.clone();
    assert_eq!(event.timestamp, cloned.timestamp);
}

#[test]
fn test_registry_event_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let event = RegistryEvent::new(EventType::PluginRegistered {
        plugin_id: PluginId::new("serialize-test"),
    });

    let json = serde_json::to_string(&event)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;
    assert!(!json.is_empty());
    Ok(())
}

// ========== Thread Safety Tests ==========

#[test]
fn test_all_types_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<PluginId>();
    assert_send::<Plugin>();
    assert_send::<PluginMetadata>();
    assert_send::<Capability>();
    assert_send::<CapabilityType>();
    assert_send::<HealthStatus>();
    assert_send::<HealthCheckType>();
    assert_send::<HealthCheckConfig>();
    assert_send::<EventType>();
    assert_send::<RegistryEvent>();
}
