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

//! Comprehensive Plugin Types Tests
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
//! Tests for plugin type definitions in songbird-registry.

use songbird_registry::types::capability::*;
use songbird_registry::types::plugin::*;

// ============================================================================
// PLUGIN ID TESTS
// ============================================================================

#[test]
fn test_plugin_id_creation() {
    let id = PluginId::new("test-plugin");
    assert_eq!(id.as_str(), "test-plugin");
}

#[test]
fn test_plugin_id_from_string() {
    let id: PluginId = "my-plugin".to_string().into();
    assert_eq!(id.as_str(), "my-plugin");
}

#[test]
fn test_plugin_id_from_str() {
    let id: PluginId = "str-plugin".into();
    assert_eq!(id.as_str(), "str-plugin");
}

#[test]
fn test_plugin_id_equality() {
    let id1 = PluginId::new("plugin-1");
    let id2 = PluginId::new("plugin-1");
    let id3 = PluginId::new("plugin-2");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_plugin_id_clone() {
    let id1 = PluginId::new("clone-test");
    let id2 = id1.clone();
    assert_eq!(id1, id2);
}

#[test]
fn test_plugin_id_display() {
    let id = PluginId::new("display-test");
    assert_eq!(format!("{id}"), "display-test");
}

#[test]
fn test_plugin_id_debug() {
    let id = PluginId::new("debug-test");
    let debug_str = format!("{:?}", id);
    assert!(debug_str.contains("PluginId"));
}

#[test]
fn test_plugin_id_serialization() {
    let id = PluginId::new("serialize-test");
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: PluginId = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, id);
}

#[test]
fn test_plugin_id_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let id = PluginId::new("hash-test");
    map.insert(id.clone(), "value");

    assert_eq!(map.get(&id), Some(&"value"));
}

// ============================================================================
// PLUGIN METADATA TESTS
// ============================================================================

#[test]
fn test_plugin_metadata_default() {
    let metadata = PluginMetadata::default();
    assert_eq!(metadata.author, "unknown");
    assert_eq!(metadata.description, "");
    assert_eq!(metadata.tags.len(), 0);
    assert!(metadata.health_endpoint.is_none());
    assert_eq!(metadata.extra.len(), 0);
}

#[test]
fn test_plugin_metadata_creation() {
    let metadata = PluginMetadata {
        author: "Alice".to_string(),
        description: "Test plugin".to_string(),
        tags: vec!["test".to_string(), "example".to_string()],
        health_endpoint: Some("/health".to_string()),
        extra: serde_json::Map::new(),
    };

    assert_eq!(metadata.author, "Alice");
    assert_eq!(metadata.description, "Test plugin");
    assert_eq!(metadata.tags.len(), 2);
    assert_eq!(metadata.health_endpoint, Some("/health".to_string()));
}

#[test]
fn test_plugin_metadata_with_extra() {
    let mut extra = serde_json::Map::new();
    extra.insert("key1".to_string(), serde_json::json!("value1"));
    extra.insert("key2".to_string(), serde_json::json!(42));

    let metadata = PluginMetadata {
        author: "Bob".to_string(),
        description: "Plugin with extras".to_string(),
        tags: vec![],
        health_endpoint: None,
        extra,
    };

    assert_eq!(metadata.extra.len(), 2);
}

#[test]
fn test_plugin_metadata_clone() {
    let metadata1 = PluginMetadata::default();
    let metadata2 = metadata1.clone();
    assert_eq!(metadata1.author, metadata2.author);
}

#[test]
fn test_plugin_metadata_debug() {
    let metadata = PluginMetadata::default();
    let debug_str = format!("{:?}", metadata);
    assert!(debug_str.contains("PluginMetadata"));
}

#[test]
fn test_plugin_metadata_serialization() {
    let metadata = PluginMetadata {
        author: "Charlie".to_string(),
        description: "Serialize test".to_string(),
        tags: vec!["tag1".to_string()],
        health_endpoint: Some("/api/health".to_string()),
        extra: serde_json::Map::new(),
    };

    let json = serde_json::to_string(&metadata).expect("Failed to serialize");
    let deserialized: PluginMetadata = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.author, metadata.author);
}

// ============================================================================
// PLUGIN TESTS
// ============================================================================

#[test]
fn test_plugin_creation() {
    let plugin = Plugin::new("test-id", "Test Plugin", "1.0.0");
    assert_eq!(plugin.id.as_str(), "test-id");
    assert_eq!(plugin.name, "Test Plugin");
    assert_eq!(plugin.version, "1.0.0");
    assert_eq!(plugin.capabilities.len(), 0);
    assert_eq!(plugin.dependencies.len(), 0);
}

#[test]
fn test_plugin_with_capability() {
    let capability = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    let plugin = Plugin::new("compute-plugin", "Compute", "1.0.0").with_capability(capability);

    assert_eq!(plugin.capabilities.len(), 1);
}

#[test]
fn test_plugin_with_multiple_capabilities() {
    let cap1 = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    let cap2 = Capability::new(CapabilityType::Storage {
        size_gb: 100,
        storage_type: "ssd".to_string(),
    });

    let plugin = Plugin::new("multi-cap", "Multi Capability", "1.0.0")
        .with_capability(cap1)
        .with_capability(cap2);

    assert_eq!(plugin.capabilities.len(), 2);
}

#[test]
fn test_plugin_with_dependency() {
    let plugin =
        Plugin::new("dependent", "Dependent Plugin", "1.0.0").with_dependency("base-plugin");

    assert_eq!(plugin.dependencies.len(), 1);
}

#[test]
fn test_plugin_with_multiple_dependencies() {
    let plugin = Plugin::new("complex", "Complex Plugin", "2.0.0")
        .with_dependency("dep1")
        .with_dependency("dep2")
        .with_dependency("dep3");

    assert_eq!(plugin.dependencies.len(), 3);
}

#[test]
fn test_plugin_with_metadata() {
    let metadata = PluginMetadata {
        author: "Developer".to_string(),
        description: "A test plugin".to_string(),
        tags: vec!["testing".to_string()],
        health_endpoint: Some("/health".to_string()),
        extra: serde_json::Map::new(),
    };

    let plugin = Plugin::new("meta-test", "Meta Test", "1.0.0").with_metadata(metadata);

    assert_eq!(plugin.metadata.author, "Developer");
}

#[test]
fn test_plugin_clone() {
    let plugin1 = Plugin::new("clone-test", "Clone Test", "1.0.0");
    let plugin2 = plugin1.clone();

    assert_eq!(plugin1.id, plugin2.id);
    assert_eq!(plugin1.name, plugin2.name);
}

#[test]
fn test_plugin_debug() {
    let plugin = Plugin::new("debug-test", "Debug Test", "1.0.0");
    let debug_str = format!("{:?}", plugin);
    assert!(debug_str.contains("Plugin"));
}

#[test]
fn test_plugin_serialization() {
    let plugin = Plugin::new("serialize-test", "Serialize", "1.0.0");
    let json = serde_json::to_string(&plugin).expect("Failed to serialize");
    let deserialized: Plugin = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.id, plugin.id);
    assert_eq!(deserialized.name, plugin.name);
}

#[test]
fn test_plugin_complete_workflow() {
    // Create a complete plugin with all features
    let mut extra = serde_json::Map::new();
    extra.insert("license".to_string(), serde_json::json!("MIT"));

    let metadata = PluginMetadata {
        author: "Team".to_string(),
        description: "Full-featured plugin".to_string(),
        tags: vec!["production".to_string(), "stable".to_string()],
        health_endpoint: Some("/api/v1/health".to_string()),
        extra,
    };

    let cap1 = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 1000,
        latency_ms: 5,
    });

    let cap2 = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes256".to_string(), "rsa4096".to_string()],
        key_sizes: vec![256, 4096],
    });

    let plugin = Plugin::new("production-plugin", "Production Plugin", "2.0.0")
        .with_capability(cap1)
        .with_capability(cap2)
        .with_dependency("crypto-lib")
        .with_dependency("network-lib")
        .with_metadata(metadata);

    // Verify all fields
    assert_eq!(plugin.id.as_str(), "production-plugin");
    assert_eq!(plugin.name, "Production Plugin");
    assert_eq!(plugin.version, "2.0.0");
    assert_eq!(plugin.capabilities.len(), 2);
    assert_eq!(plugin.dependencies.len(), 2);
    assert_eq!(plugin.metadata.author, "Team");
    assert_eq!(plugin.metadata.tags.len(), 2);
}

// ============================================================================
// BUILDER PATTERN TESTS
// ============================================================================

#[test]
fn test_plugin_builder_chaining() {
    let plugin = Plugin::new("builder", "Builder Test", "1.0.0")
        .with_capability(Capability::new(CapabilityType::Compute {
            cpu_cores: 8,
            memory_gb: 16,
        }))
        .with_dependency("dep1");

    assert_eq!(plugin.capabilities.len(), 1);
    assert_eq!(plugin.dependencies.len(), 1);
}

#[test]
fn test_plugin_builder_empty_plugin() {
    let plugin = Plugin::new("minimal", "Minimal", "0.1.0");

    assert!(plugin.capabilities.is_empty());
    assert!(plugin.dependencies.is_empty());
    assert_eq!(plugin.metadata.author, "unknown");
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_plugin_id_empty_string() {
    let id = PluginId::new("");
    assert_eq!(id.as_str(), "");
}

#[test]
fn test_plugin_id_special_characters() {
    let id = PluginId::new("plugin-with-dashes_and_underscores.123");
    assert_eq!(id.as_str(), "plugin-with-dashes_and_underscores.123");
}

#[test]
fn test_plugin_version_formats() {
    let v1 = Plugin::new("p1", "P1", "1.0.0");
    let v2 = Plugin::new("p2", "P2", "2.0.0-beta");
    let v3 = Plugin::new("p3", "P3", "0.1.0-alpha.1");

    assert_eq!(v1.version, "1.0.0");
    assert_eq!(v2.version, "2.0.0-beta");
    assert_eq!(v3.version, "0.1.0-alpha.1");
}

#[test]
fn test_plugin_long_description() {
    let long_desc = "A".repeat(1000);
    let metadata = PluginMetadata {
        author: "Test".to_string(),
        description: long_desc,
        tags: vec![],
        health_endpoint: None,
        extra: serde_json::Map::new(),
    };

    let plugin = Plugin::new("long", "Long", "1.0.0").with_metadata(metadata);

    assert_eq!(plugin.metadata.description.len(), 1000);
}

#[test]
fn test_plugin_many_tags() {
    let tags: Vec<String> = (0..100).map(|i| format!("tag{}", i)).collect();

    let metadata = PluginMetadata {
        author: "Test".to_string(),
        description: "Many tags".to_string(),
        tags,
        health_endpoint: None,
        extra: serde_json::Map::new(),
    };

    let plugin = Plugin::new("tags", "Tags", "1.0.0").with_metadata(metadata);

    assert_eq!(plugin.metadata.tags.len(), 100);
}

#[test]
fn test_plugin_circular_dependency_prevention() {
    // Note: Actual circular dependency detection would be in the registry
    // This just tests that we can add dependencies
    let plugin = Plugin::new("self", "Self", "1.0.0").with_dependency("self"); // Registry should prevent this

    assert_eq!(plugin.dependencies.len(), 1);
}
