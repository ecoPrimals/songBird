// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Basic types tests for songbird-types
//!
//! NOTE: Updated to use modern capability-based PrimalType (Nov 2025)
//! Old hardcoded primal names (BearDog, Squirrel, etc.) have been replaced
//! with capability-based types (Security, Storage, Compute, AI)

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::useless_vec,
    reason = "test assertions and harness ergonomics"
)]
use songbird_types::traits::canonical::PrimalType;

#[test]
fn test_primal_type_security() {
    // Modern: BearDog → Security capability
    let primal = PrimalType::Security;
    assert_eq!(format!("{:?}", primal), "Security");
}

#[test]
fn test_primal_type_storage() {
    // Modern: Squirrel → Storage capability
    let primal = PrimalType::Storage;
    assert_eq!(format!("{:?}", primal), "Storage");
}

#[test]
fn test_primal_type_compute() {
    // Modern: ToadStool → Compute capability
    let primal = PrimalType::Compute;
    assert_eq!(format!("{:?}", primal), "Compute");
}

#[test]
fn test_primal_type_ai() {
    // Modern: NestGate → AI capability
    let primal = PrimalType::AI;
    assert_eq!(format!("{:?}", primal), "AI");
}

#[test]
fn test_primal_type_network() {
    // Modern: Songbird → Network capability
    let primal = PrimalType::Network;
    assert_eq!(format!("{:?}", primal), "Network");
}

#[test]
fn test_all_primal_types() {
    let primals = [
        PrimalType::Security,
        PrimalType::Storage,
        PrimalType::Compute,
        PrimalType::AI,
        PrimalType::Network,
    ];
    assert_eq!(primals.len(), 5);
}

#[test]
fn test_primal_type_custom() {
    // Custom primal types supported via Custom variant
    let primal = PrimalType::Custom("BearDog".to_string());
    assert!(matches!(primal, PrimalType::Custom(_)));
}

#[test]
fn test_primal_type_clone() {
    let primal1 = PrimalType::Security;
    let primal2 = primal1.clone();
    assert_eq!(format!("{:?}", primal1), format!("{:?}", primal2));
}

#[test]
fn test_primal_type_copy() {
    let primal1 = PrimalType::AI;
    let primal2 = primal1.clone(); // Clone because PrimalType has Custom(String) variant
    let primal3 = primal1; // Move occurs here
    assert_eq!(format!("{:?}", primal2), "AI");
    assert_eq!(format!("{:?}", primal3), "AI");
}

#[test]
fn test_primal_type_in_vec() {
    let primals = vec![
        PrimalType::Security, // BearDog -> Security capability
        PrimalType::AI,       // Squirrel -> AI capability
        PrimalType::Storage,  // NestGate -> Storage capability
    ];
    assert_eq!(primals.len(), 3);
}

#[test]
fn test_primal_type_match() {
    let primal = PrimalType::Security;
    let result = match primal {
        PrimalType::Security => "security",
        PrimalType::AI => "ai",
        PrimalType::Storage => "storage",
        PrimalType::Compute => "compute",
        PrimalType::Network => "network",
        PrimalType::Custom(ref name) => name.as_str(),
    };
    assert_eq!(result, "security");
}

#[test]
fn test_primal_type_serialization() {
    let primal = PrimalType::Security;
    let json = serde_json::to_string(&primal).expect("test precondition");
    assert!(!json.is_empty());
}

#[test]
fn test_primal_type_deserialization() {
    // Modern: Use capability-based types, not hardcoded names
    let json = "\"Storage\""; // Squirrel → Storage capability
    let primal: PrimalType = serde_json::from_str(json).expect("should parse valid input");
    assert_eq!(format!("{:?}", primal), "Storage");
}

#[test]
fn test_primal_types_roundtrip() {
    let primals = vec![
        PrimalType::Security,
        PrimalType::Storage,
        PrimalType::Compute,
        PrimalType::AI,
        PrimalType::Network,
    ];
    for primal in primals {
        let json = serde_json::to_string(&primal).expect("test precondition");
        let deserialized: PrimalType =
            serde_json::from_str(&json).expect("should parse valid input");
        assert_eq!(format!("{:?}", primal), format!("{:?}", deserialized));
    }
}

#[test]
fn test_primal_type_option() {
    let maybe_primal: Option<PrimalType> = Some(PrimalType::Security);
    assert!(maybe_primal.is_some());
    let no_primal: Option<PrimalType> = None;
    assert!(no_primal.is_none());
}

#[test]
fn test_primal_type_result() {
    let success: Result<PrimalType, String> = Ok(PrimalType::AI);
    assert!(success.is_ok());
    let failure: Result<PrimalType, String> = Err("error".to_string());
    assert!(failure.is_err());
}

#[test]
fn test_primal_type_hash_map() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(PrimalType::Security, "security");
    map.insert(PrimalType::AI, "ai");
    map.insert(PrimalType::Storage, "storage");
    assert_eq!(map.len(), 3);
    assert_eq!(map.get(&PrimalType::Security), Some(&"security"));
}

#[test]
fn test_primal_type_btree_map() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert(PrimalType::Security, 1);
    map.insert(PrimalType::AI, 2);
    assert_eq!(map.len(), 2);
}

#[test]
fn test_primal_type_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(PrimalType::Security);
    set.insert(PrimalType::AI);
    set.insert(PrimalType::Security); // Duplicate
    assert_eq!(set.len(), 2); // Should still be 2
}

#[test]
fn test_primal_type_display_match_all() {
    let primals = [
        (PrimalType::Security, "Security"),
        (PrimalType::AI, "AI"),
        (PrimalType::Storage, "Storage"),
        (PrimalType::Compute, "Compute"),
        (PrimalType::Network, "Network"),
    ];
    for (primal, expected) in primals {
        let debug = format!("{:?}", primal);
        assert_eq!(debug, expected);
    }
}
