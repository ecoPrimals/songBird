// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
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

//! Comprehensive Capability Types Tests
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![expect(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![expect(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![expect(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Tests for capability type definitions in songbird-registry.

use songbird_registry::types::capability::*;
use std::collections::HashMap;

// ============================================================================
// CAPABILITY TYPE TESTS
// ============================================================================

#[test]
fn test_capability_type_encryption() {
    let cap_type = CapabilityType::Encryption {
        algorithms: vec!["aes256".to_string()],
        key_sizes: vec![256],
    };

    if let CapabilityType::Encryption {
        algorithms,
        key_sizes,
    } = cap_type
    {
        assert_eq!(algorithms.len(), 1);
        assert_eq!(key_sizes.len(), 1);
    } else {
        panic!("Expected Encryption capability type");
    }
}

#[test]
fn test_capability_type_service_discovery() {
    let cap_type = CapabilityType::ServiceDiscovery {
        protocols: vec!["mdns".to_string(), "dns-sd".to_string()],
    };

    if let CapabilityType::ServiceDiscovery {
        protocols,
    } = cap_type
    {
        assert_eq!(protocols.len(), 2);
    } else {
        panic!("Expected ServiceDiscovery capability type");
    }
}

#[test]
fn test_capability_type_compute() {
    let cap_type = CapabilityType::Compute {
        cpu_cores: 8,
        memory_gb: 16,
    };

    if let CapabilityType::Compute {
        cpu_cores,
        memory_gb,
    } = cap_type
    {
        assert_eq!(cpu_cores, 8);
        assert_eq!(memory_gb, 16);
    } else {
        panic!("Expected Compute capability type");
    }
}

#[test]
fn test_capability_type_network() {
    let cap_type = CapabilityType::Network {
        bandwidth_mbps: 1000,
        latency_ms: 10,
    };

    if let CapabilityType::Network {
        bandwidth_mbps,
        latency_ms,
    } = cap_type
    {
        assert_eq!(bandwidth_mbps, 1000);
        assert_eq!(latency_ms, 10);
    } else {
        panic!("Expected Network capability type");
    }
}

#[test]
fn test_capability_type_storage() {
    let cap_type = CapabilityType::Storage {
        size_gb: 500,
        storage_type: "nvme".to_string(),
    };

    if let CapabilityType::Storage {
        size_gb,
        storage_type,
    } = cap_type
    {
        assert_eq!(size_gb, 500);
        assert_eq!(storage_type, "nvme");
    } else {
        panic!("Expected Storage capability type");
    }
}

#[test]
fn test_capability_type_custom() {
    let mut attributes = HashMap::new();
    attributes.insert("key1".to_string(), "value1".to_string());

    let cap_type = CapabilityType::Custom {
        name: "custom-capability".to_string(),
        attributes,
    };

    if let CapabilityType::Custom {
        name,
        attributes,
    } = cap_type
    {
        assert_eq!(name, "custom-capability");
        assert_eq!(attributes.len(), 1);
    } else {
        panic!("Expected Custom capability type");
    }
}

#[test]
fn test_capability_type_equality() {
    let cap1 = CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    };

    let cap2 = CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    };

    assert_eq!(cap1, cap2);
}

#[test]
fn test_capability_type_clone() {
    let cap1 = CapabilityType::Network {
        bandwidth_mbps: 100,
        latency_ms: 5,
    };

    let cap2 = cap1.clone();
    assert_eq!(cap1, cap2);
}

#[test]
fn test_capability_type_debug() {
    let cap_type = CapabilityType::Storage {
        size_gb: 100,
        storage_type: "ssd".to_string(),
    };

    let debug_str = format!("{cap_type:?}");
    assert!(debug_str.contains("Storage"));
}

#[test]
fn test_capability_type_serialization() {
    let cap_type = CapabilityType::Encryption {
        algorithms: vec!["aes128".to_string()],
        key_sizes: vec![128],
    };

    let json = serde_json::to_string(&cap_type).expect("Failed to serialize");
    let deserialized: CapabilityType = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, cap_type);
}

// ============================================================================
// CAPABILITY TESTS
// ============================================================================

#[test]
fn test_capability_creation() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    assert_eq!(cap.version, "1.0.0");
    assert!(cap.active);
}

#[test]
fn test_capability_with_version() {
    let cap = Capability::new(CapabilityType::Storage {
        size_gb: 100,
        storage_type: "ssd".to_string(),
    })
    .with_version("2.0.0");

    assert_eq!(cap.version, "2.0.0");
}

#[test]
fn test_capability_with_active_flag() {
    let cap = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 100,
        latency_ms: 10,
    })
    .with_active(false);

    assert!(!cap.active);
}

#[test]
fn test_capability_inactive() {
    let cap = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["deprecated".to_string()],
        key_sizes: vec![64],
    })
    .with_active(false);

    assert!(!cap.active);
}

#[test]
fn test_capability_compatibility_same_type() {
    let cap1 = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    let cap2 = Capability::new(CapabilityType::Compute {
        cpu_cores: 8,
        memory_gb: 16,
    });

    assert!(cap1.compatible_with(&cap2));
}

#[test]
fn test_capability_compatibility_different_types() {
    let cap1 = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    let cap2 = Capability::new(CapabilityType::Storage {
        size_gb: 100,
        storage_type: "ssd".to_string(),
    });

    assert!(!cap1.compatible_with(&cap2));
}

#[test]
fn test_capability_compatibility_encryption() {
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
fn test_capability_equality() {
    let cap1 = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 100,
        latency_ms: 10,
    });

    let cap2 = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 100,
        latency_ms: 10,
    });

    assert_eq!(cap1, cap2);
}

#[test]
fn test_capability_clone() {
    let cap1 = Capability::new(CapabilityType::ServiceDiscovery {
        protocols: vec!["mdns".to_string()],
    });

    let cap2 = cap1.clone();
    assert_eq!(cap1, cap2);
}

#[test]
fn test_capability_debug() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 2,
        memory_gb: 4,
    });

    let debug_str = format!("{cap:?}");
    assert!(debug_str.contains("Capability"));
}

#[test]
fn test_capability_serialization() {
    let cap = Capability::new(CapabilityType::Storage {
        size_gb: 200,
        storage_type: "hdd".to_string(),
    })
    .with_version("1.5.0");

    let json = serde_json::to_string(&cap).expect("Failed to serialize");
    let deserialized: Capability = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.version, cap.version);
}

// ============================================================================
// BUILDER PATTERN TESTS
// ============================================================================

#[test]
fn test_capability_builder_chaining() {
    let cap = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 1000,
        latency_ms: 5,
    })
    .with_version("2.1.0")
    .with_active(true);

    assert_eq!(cap.version, "2.1.0");
    assert!(cap.active);
}

#[test]
fn test_capability_default_values() {
    let cap = Capability::new(CapabilityType::Custom {
        name: "test".to_string(),
        attributes: HashMap::new(),
    });

    assert_eq!(cap.version, "1.0.0");
    assert!(cap.active);
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_encryption_multiple_algorithms() {
    let cap = Capability::new(CapabilityType::Encryption {
        algorithms: vec![
            "aes128".to_string(),
            "aes256".to_string(),
            "rsa2048".to_string(),
            "rsa4096".to_string(),
        ],
        key_sizes: vec![128, 256, 2048, 4096],
    });

    if let CapabilityType::Encryption {
        algorithms,
        key_sizes,
    } = cap.capability_type
    {
        assert_eq!(algorithms.len(), 4);
        assert_eq!(key_sizes.len(), 4);
    }
}

#[test]
fn test_service_discovery_single_protocol() {
    let cap = Capability::new(CapabilityType::ServiceDiscovery {
        protocols: vec!["dns-sd".to_string()],
    });

    if let CapabilityType::ServiceDiscovery {
        protocols,
    } = cap.capability_type
    {
        assert_eq!(protocols.len(), 1);
    }
}

#[test]
fn test_compute_minimal_resources() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 1,
        memory_gb: 1,
    });

    if let CapabilityType::Compute {
        cpu_cores,
        memory_gb,
    } = cap.capability_type
    {
        assert_eq!(cpu_cores, 1);
        assert_eq!(memory_gb, 1);
    }
}

#[test]
fn test_compute_maximum_resources() {
    let cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 128,
        memory_gb: 1024,
    });

    if let CapabilityType::Compute {
        cpu_cores,
        memory_gb,
    } = cap.capability_type
    {
        assert_eq!(cpu_cores, 128);
        assert_eq!(memory_gb, 1024);
    }
}

#[test]
fn test_network_high_bandwidth() {
    let cap = Capability::new(CapabilityType::Network {
        bandwidth_mbps: 100_000, // 100 Gbps
        latency_ms: 1,
    });

    if let CapabilityType::Network {
        bandwidth_mbps,
        latency_ms,
    } = cap.capability_type
    {
        assert_eq!(bandwidth_mbps, 100_000);
        assert_eq!(latency_ms, 1);
    }
}

#[test]
fn test_storage_types() {
    let storage_types = vec!["ssd", "hdd", "nvme", "ram", "tape"];

    for storage_type in storage_types {
        let cap = Capability::new(CapabilityType::Storage {
            size_gb: 100,
            storage_type: storage_type.to_string(),
        });

        if let CapabilityType::Storage {
            storage_type: st,
            ..
        } = cap.capability_type
        {
            assert_eq!(st, storage_type);
        }
    }
}

#[test]
fn test_custom_capability_empty_attributes() {
    let cap = Capability::new(CapabilityType::Custom {
        name: "empty".to_string(),
        attributes: HashMap::new(),
    });

    if let CapabilityType::Custom {
        attributes,
        ..
    } = cap.capability_type
    {
        assert_eq!(attributes.len(), 0);
    }
}

#[test]
fn test_custom_capability_many_attributes() {
    let mut attributes = HashMap::new();
    for i in 0..100 {
        attributes.insert(format!("key{i}"), format!("value{i}"));
    }

    let cap = Capability::new(CapabilityType::Custom {
        name: "many-attrs".to_string(),
        attributes: attributes.clone(),
    });

    if let CapabilityType::Custom {
        attributes: attrs,
        ..
    } = cap.capability_type
    {
        assert_eq!(attrs.len(), 100);
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_capability_complete_workflow() {
    let cap = Capability::new(CapabilityType::Encryption {
        algorithms: vec!["aes256-gcm".to_string(), "chacha20-poly1305".to_string()],
        key_sizes: vec![256],
    })
    .with_version("3.0.0")
    .with_active(true);

    assert_eq!(cap.version, "3.0.0");
    assert!(cap.active);

    if let CapabilityType::Encryption {
        algorithms,
        ..
    } = &cap.capability_type
    {
        assert_eq!(algorithms.len(), 2);
    }
}

#[test]
fn test_multiple_capability_types() {
    let capabilities = vec![
        Capability::new(CapabilityType::Compute {
            cpu_cores: 4,
            memory_gb: 8,
        }),
        Capability::new(CapabilityType::Storage {
            size_gb: 100,
            storage_type: "ssd".to_string(),
        }),
        Capability::new(CapabilityType::Network {
            bandwidth_mbps: 1000,
            latency_ms: 10,
        }),
    ];

    assert_eq!(capabilities.len(), 3);
}

#[test]
fn test_capability_version_evolution() {
    let v1 = Capability::new(CapabilityType::ServiceDiscovery {
        protocols: vec!["mdns".to_string()],
    })
    .with_version("1.0.0");

    let v2 = Capability::new(CapabilityType::ServiceDiscovery {
        protocols: vec!["mdns".to_string(), "dns-sd".to_string()],
    })
    .with_version("2.0.0");

    assert!(v1.compatible_with(&v2));
    assert_ne!(v1.version, v2.version);
}

#[test]
fn test_capability_activation_lifecycle() {
    let mut cap = Capability::new(CapabilityType::Compute {
        cpu_cores: 4,
        memory_gb: 8,
    });

    assert!(cap.active);

    cap = cap.with_active(false);
    assert!(!cap.active);

    cap = cap.with_active(true);
    assert!(cap.active);
}
