// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for canonical configuration factory
#![allow(clippy::expect_used, reason = "test assertions and harness ergonomics")]

use songbird_types::config::consolidated_canonical::factory::CanonicalConfigFactory;

#[test]
fn test_factory_new() {
    let factory = CanonicalConfigFactory::new();
    // Factory is a unit struct, just verify it can be created
    let _ = factory;
}

#[test]
fn test_factory_default() {
    let factory = CanonicalConfigFactory;
    let _ = factory;
}

#[test]
fn test_factory_create_default() {
    let config = CanonicalConfigFactory::create_default();
    // Default config should have reasonable values
    assert!(!config.system.environment.is_empty());
}

#[test]
fn test_factory_create_for_environment_production() {
    let config = CanonicalConfigFactory::create_for_environment("production");
    assert_eq!(config.system.environment, "production");
}

#[test]
fn test_factory_create_for_environment_development() {
    let config = CanonicalConfigFactory::create_for_environment("development");
    assert_eq!(config.system.environment, "development");
}

#[test]
fn test_factory_create_for_environment_test() {
    let config = CanonicalConfigFactory::create_for_environment("test");
    assert_eq!(config.system.environment, "test");
}

#[test]
fn test_factory_clone() {
    let factory = CanonicalConfigFactory::new();
    let cloned = factory.clone();
    // Both should serialize to the same value
    let json1 = serde_json::to_string(&factory).expect("Serialization should succeed");
    let json2 = serde_json::to_string(&cloned).expect("Serialization should succeed");
    assert_eq!(json1, json2);
}

#[test]
fn test_factory_debug() {
    let factory = CanonicalConfigFactory::new();
    let debug_str = format!("{:?}", factory);
    assert!(debug_str.contains("CanonicalConfigFactory"));
}

#[test]
fn test_factory_serialization() {
    let factory = CanonicalConfigFactory::new();
    let json = serde_json::to_string(&factory).expect("Serialization should succeed");
    // Unit struct serializes as null
    assert_eq!(json, "null");
}

#[test]
fn test_factory_deserialization() {
    let json = "null";
    let _factory: CanonicalConfigFactory =
        serde_json::from_str(json).expect("Deserialization should succeed");
}
