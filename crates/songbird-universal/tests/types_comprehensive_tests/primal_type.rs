// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// PRIMAL TYPE TESTS
// ============================================================================

#[test]
fn test_primal_type_new() {
    let primal = PrimalType::new("compute");
    assert_eq!(primal.category, "compute");
    assert_eq!(primal.subcategory, None);
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_primal_type_from_string() {
    let primal = PrimalType::from_string("storage");
    assert_eq!(primal.category, "storage");
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_primal_type_as_str() {
    let primal = PrimalType::new("ai");
    assert_eq!(primal.as_str(), "ai");
}

#[test]
fn test_primal_type_display() -> SongbirdResult<()> {
    let primal = PrimalType::new("security");
    assert_eq!(format!("{}", primal), "security");
    Ok(())
}

#[test]
fn test_primal_type_equality() -> SongbirdResult<()> {
    let p1 = PrimalType::new("compute");
    let p2 = PrimalType::new("compute");
    let p3 = PrimalType::new("storage");

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
    Ok(())
}

#[test]
fn test_primal_type_clone() -> SongbirdResult<()> {
    let p1 = PrimalType::new("network");
    let p2 = p1.clone();
    assert_eq!(p1, p2);
    Ok(())
}

#[test]
fn test_primal_type_serialization() -> SongbirdResult<()> {
    let primal = PrimalType::new("compute");
    let json = serde_json::to_string(&primal)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: PrimalType = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized, primal);
    Ok(())
}
