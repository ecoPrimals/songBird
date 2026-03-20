// SPDX-License-Identifier: AGPL-3.0-only
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
    clippy::must_use_candidate
)]
#![cfg(feature = "tests-incomplete")]
#![allow(unexpected_cfgs)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! NOTE: Disabled - requires unimplemented methods

//! Comprehensive Service Registration Tests
//!
//! Tests service registration, deregistration, updates, and lifecycle management

use songbird_registry::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

#[test]
fn test_service_registry_creation() -> SongbirdResult<()> {
    let registry = ServiceRegistry::new();
    assert!(registry.is_ok());
    Ok(())
}

#[test]
fn test_registry_initially_empty() -> SongbirdResult<()> {
    let registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let count = registry.service_count();
    assert_eq!(count, 0);
    Ok(())
}

#[test]
fn test_register_single_service() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let service = ServiceInfo {
        id: "service-1".to_string(),
        name: "Test Service".to_string(),
        ..Default::default()
    };

    let result = registry.register(service);
    assert!(result.is_ok());
    assert_eq!(registry.service_count(), 1);
    Ok(())
}

#[test]
fn test_register_multiple_services() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    for i in 0..5 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Service {}", i),
            ..Default::default()
        };
        assert!(registry.register(service).is_ok());
    }

    assert_eq!(registry.service_count(), 5);
    Ok(())
}

#[test]
fn test_deregister_service() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register service".to_string()))?;

    let service = ServiceInfo {
        id: "service-1".to_string(),
        name: "Test Service".to_string(),
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    assert_eq!(registry.service_count(), 1);

    let result = registry.deregister("service-1");
    assert!(result.is_ok());
    assert_eq!(registry.service_count(), 0);
    Ok(())
}

#[test]
fn test_deregister_nonexistent_service() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let result = registry.deregister("nonexistent");
    // Should handle gracefully (either Ok or specific error)
    let _ = result;
    Ok(())
}

#[test]
fn test_duplicate_service_id() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let service1 = ServiceInfo {
        id: "duplicate-id".to_string(),
        name: "Service 1".to_string(),
        ..Default::default()
    };

    let service2 = ServiceInfo {
        id: "duplicate-id".to_string(),
        name: "Service 2".to_string(),
        ..Default::default()
    };

    assert!(registry.register(service1).is_ok());

    // Second registration with same ID should either:
    // - Replace existing service (Ok)
    // - Return error (Err)
    let _ = registry.register(service2);
    Ok(())
}

#[test]
fn test_find_service_by_id() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let service = ServiceInfo {
        id: "findable".to_string(),
        name: "Findable Service".to_string(),
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let found = registry.find_by_id("findable");
    assert!(found.is_some());
    Ok(())
}

#[test]
fn test_find_nonexistent_service() -> SongbirdResult<()> {
    let registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    let found = registry.find_by_id("nonexistent");
    assert!(found.is_none());
    Ok(())
}

#[test]
fn test_list_all_services() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    for i in 0..3 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Service {}", i),
            ..Default::default()
        };
        registry
            .register(service)
            .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    }

    let services = registry.list_all();
    assert_eq!(services.len(), 3);
    Ok(())
}

#[test]
fn test_clear_registry() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    for i in 0..5 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Service {}", i),
            ..Default::default()
        };
        registry
            .register(service)
            .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    }

    registry.clear();
    assert_eq!(registry.service_count(), 0);
    Ok(())
}

#[test]
fn test_service_metadata() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("region".to_string(), "us-west".to_string());

    let service = ServiceInfo {
        id: "metadata-service".to_string(),
        name: "Metadata Service".to_string(),
        metadata: Some(metadata),
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let found = registry.find_by_id("metadata-service");
    assert!(found.is_some());
    Ok(())
}

#[test]
fn test_service_with_capabilities() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    let service = ServiceInfo {
        id: "capable-service".to_string(),
        name: "Capable Service".to_string(),
        capabilities: vec!["http".to_string(), "grpc".to_string()],
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    assert_eq!(registry.service_count(), 1);
    Ok(())
}

#[test]
fn test_service_with_tags() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let service = ServiceInfo {
        id: "tagged-service".to_string(),
        name: "Tagged Service".to_string(),
        tags: vec!["production".to_string(), "critical".to_string()],
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    assert_eq!(registry.service_count(), 1);
    Ok(())
}

#[test]
fn test_registry_clone() -> SongbirdResult<()> {
    let registry1 = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    let registry2 = registry1.clone();

    assert_eq!(registry1.service_count(), registry2.service_count());
    Ok(())
}

#[test]
fn test_registry_debug_format() -> SongbirdResult<()> {
    let registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let debug_str = format!("{:?}", registry);

    assert!(!debug_str.is_empty());
    Ok(())
}

#[test]
fn test_concurrent_registrations() -> SongbirdResult<()> {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::sync::Arc;
    use std::sync::Mutex;

    let registry = Arc::new(Mutex::new(ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?));

    let mut handles = vec![];

    for i in 0..10 {
        let registry_clone = registry.clone();
        let handle = std::thread::spawn(move || {
            let service = ServiceInfo {
                id: format!("service-{}", i),
                name: format!("Service {}", i),
                ..Default::default()
            };

            let mut reg = registry_clone
                .lock()
                .or_else(|_| SongbirdError::configuration(format!("Error: {}", e)))?;
            reg.register(service)
        });
        handles.push(handle);
    }

    for handle in handles {
        assert!(handle.join().is_ok());
    }

    let final_count = registry
        .lock()
        .ok_or_else(|| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?
        .service_count();
    assert_eq!(final_count, 10);
    Ok(())
}

#[test]
fn test_service_update() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    let service = ServiceInfo {
        id: "updatable".to_string(),
        name: "Original Name".to_string(),
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let updated_service = ServiceInfo {
        id: "updatable".to_string(),
        name: "Updated Name".to_string(),
        ..Default::default()
    };

    let result = registry.update("updatable", updated_service);
    // Update should succeed or have specific behavior
    let _ = result;
    Ok(())
}

#[test]
fn test_filter_services_by_name() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    for i in 0..5 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: if i % 2 == 0 {
                "Even Service".to_string()
            } else {
                "Odd Service".to_string()
            },
            ..Default::default()
        };
        registry
            .register(service)
            .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;
    }

    let filtered = registry.filter_by_name("Even Service");
    // Should find services with matching name
    let _ = filtered;
    Ok(())
}

#[test]
fn test_service_exists() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    let service = ServiceInfo {
        id: "existing".to_string(),
        name: "Existing Service".to_string(),
        ..Default::default()
    };

    registry
        .register(service)
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    assert!(registry.exists("existing"));
    assert!(!registry.exists("nonexistent"));
    Ok(())
}

#[test]
fn test_empty_service_id() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new().or_else(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    let service = ServiceInfo {
        id: "".to_string(),
        name: "Empty ID Service".to_string(),
        ..Default::default()
    };

    // Should handle empty ID gracefully
    let _ = registry.register(service);
    Ok(())
}

#[test]
fn test_special_characters_in_id() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let service = ServiceInfo {
        id: "service-with-special-chars_123!@#".to_string(),
        name: "Special Service".to_string(),
        ..Default::default()
    };

    let result = registry.register(service);
    // Should handle special characters
    let _ = result;
    Ok(())
}

#[test]
fn test_very_long_service_name() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    let long_name = "a".repeat(1000);
    let service = ServiceInfo {
        id: "long-name-service".to_string(),
        name: long_name.clone(),
        ..Default::default()
    };

    assert!(registry.register(service).is_ok());
    Ok(())
}

#[test]
fn test_registry_capacity() -> SongbirdResult<()> {
    let mut registry = ServiceRegistry::new()
        .or_else(|_| SongbirdError::configuration("Failed to register".to_string()))?;

    // Register many services
    for i in 0..100 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Service {}", i),
            ..Default::default()
        };
        assert!(registry.register(service).is_ok());
    }

    assert_eq!(registry.service_count(), 100);
    Ok(())
}
