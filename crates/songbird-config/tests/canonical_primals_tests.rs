//! Tests for canonical primal types

use songbird_universal::PrimalType;
use std::str::FromStr;

#[test]
fn test_primal_type_default() {
    let primal = PrimalType::default();
    assert_eq!(primal, PrimalType::Unknown);
}

#[test]
fn test_primal_type_variants() {
    assert_eq!(PrimalType::Compute, PrimalType::Compute);
    assert_eq!(PrimalType::Storage, PrimalType::Storage);
    assert_eq!(PrimalType::Security, PrimalType::Security);
    assert_eq!(PrimalType::AI, PrimalType::AI);
    assert_eq!(PrimalType::Orchestration, PrimalType::Orchestration);
    assert_eq!(PrimalType::Gaming, PrimalType::Gaming);
    assert_eq!(PrimalType::Unknown, PrimalType::Unknown);
}

#[test]
fn test_primal_type_custom() {
    let custom1 = PrimalType::Custom("MyService".to_string());
    let custom2 = PrimalType::Custom("MyService".to_string());
    let custom3 = PrimalType::Custom("Other".to_string());
    
    assert_eq!(custom1, custom2);
    assert_ne!(custom1, custom3);
}

#[test]
fn test_primal_type_display() {
    assert_eq!(PrimalType::Compute.to_string(), "compute");
    assert_eq!(PrimalType::Storage.to_string(), "storage");
    assert_eq!(PrimalType::Security.to_string(), "security");
    assert_eq!(PrimalType::AI.to_string(), "ai");
    assert_eq!(PrimalType::Orchestration.to_string(), "orchestration");
    assert_eq!(PrimalType::Gaming.to_string(), "gaming");
    assert_eq!(PrimalType::Communication.to_string(), "communication");
    assert_eq!(PrimalType::Unknown.to_string(), "unknown");
    assert_eq!(PrimalType::Custom("test".to_string()).to_string(), "custom-test");
}

#[test]
fn test_primal_type_from_str() {
    assert_eq!(PrimalType::from_str("compute").unwrap(), PrimalType::Compute);
    assert_eq!(PrimalType::from_str("storage").unwrap(), PrimalType::Storage);
    assert_eq!(PrimalType::from_str("security").unwrap(), PrimalType::Security);
    assert_eq!(PrimalType::from_str("ai").unwrap(), PrimalType::AI);
    assert_eq!(PrimalType::from_str("orchestration").unwrap(), PrimalType::Orchestration);
    assert_eq!(PrimalType::from_str("gaming").unwrap(), PrimalType::Gaming);
    assert_eq!(PrimalType::from_str("unknown").unwrap(), PrimalType::Unknown);
}

#[test]
fn test_primal_type_from_str_case_insensitive() {
    assert_eq!(PrimalType::from_str("COMPUTE").unwrap(), PrimalType::Compute);
    assert_eq!(PrimalType::from_str("Storage").unwrap(), PrimalType::Storage);
    assert_eq!(PrimalType::from_str("SECURITY").unwrap(), PrimalType::Security);
}

#[test]
fn test_primal_type_from_str_custom() {
    match PrimalType::from_str("custom-myservice").unwrap() {
        PrimalType::Custom(name) => assert_eq!(name, "myservice"),
        _ => panic!("Expected Custom variant"),
    }
}

#[test]
fn test_primal_type_from_str_invalid() {
    assert!(PrimalType::from_str("invalid-primal-type").is_err());
    assert!(PrimalType::from_str("").is_err());
}

#[test]
fn test_primal_type_serialization() {
    let primal = PrimalType::Compute;
    let json = serde_json::to_string(&primal).unwrap();
    let deserialized: PrimalType = serde_json::from_str(&json).unwrap();
    assert_eq!(primal, deserialized);
}

#[test]
fn test_primal_type_custom_serialization() {
    let primal = PrimalType::Custom("test".to_string());
    let json = serde_json::to_string(&primal).unwrap();
    let deserialized: PrimalType = serde_json::from_str(&json).unwrap();
    assert_eq!(primal, deserialized);
}

#[test]
fn test_primal_type_clone() {
    let primal = PrimalType::Compute;
    let cloned = primal.clone();
    assert_eq!(primal, cloned);
}

#[test]
fn test_primal_type_debug() {
    let primal = PrimalType::Compute;
    let debug_str = format!("{:?}", primal);
    assert!(debug_str.contains("Compute"));
}

#[test]
fn test_primal_type_hash() {
    use std::collections::HashSet;
    
    let mut set = HashSet::new();
    set.insert(PrimalType::Compute);
    set.insert(PrimalType::Storage);
    set.insert(PrimalType::Compute); // Duplicate
    
    assert_eq!(set.len(), 2);
    assert!(set.contains(&PrimalType::Compute));
    assert!(set.contains(&PrimalType::Storage));
}

#[test]
fn test_all_primal_type_variants() {
    let variants = vec![
        PrimalType::Compute,
        PrimalType::Storage,
        PrimalType::Security,
        PrimalType::AI,
        PrimalType::Orchestration,
        PrimalType::Gaming,
        PrimalType::Communication,
        PrimalType::Media,
        PrimalType::Database,
        PrimalType::Analytics,
        PrimalType::Development,
        PrimalType::IoT,
        PrimalType::Blockchain,
        PrimalType::Financial,
        PrimalType::Identity,
        PrimalType::Cdn,
        PrimalType::Email,
        PrimalType::Search,
        PrimalType::Backup,
        PrimalType::Compliance,
        PrimalType::Unknown,
    ];
    
    // Ensure all variants are unique
    for (i, v1) in variants.iter().enumerate() {
        for (j, v2) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(v1, v2);
            } else {
                assert_ne!(v1, v2);
            }
        }
    }
}

