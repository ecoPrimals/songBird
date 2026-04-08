// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// SECURITY LEVEL TESTS
// ============================================================================

#[test]
fn test_security_level_all_variants() {
    let none = SecurityLevel::None;
    let basic = SecurityLevel::Basic;
    let standard = SecurityLevel::Standard;
    let high = SecurityLevel::High;
    let maximum = SecurityLevel::Maximum;

    assert_eq!(none, SecurityLevel::None);
    assert_eq!(basic, SecurityLevel::Basic);
    assert_eq!(standard, SecurityLevel::Standard);
    assert_eq!(high, SecurityLevel::High);
    assert_eq!(maximum, SecurityLevel::Maximum);
}

#[test]
fn test_security_level_default() -> SongbirdResult<()> {
    let default = SecurityLevel::default();
    assert_eq!(default, SecurityLevel::Standard);
    Ok(())
}

#[test]
fn test_security_level_equality() -> SongbirdResult<()> {
    assert_eq!(SecurityLevel::High, SecurityLevel::High);
    assert_ne!(SecurityLevel::High, SecurityLevel::Basic);
    Ok(())
}

#[test]
fn test_security_level_clone() -> SongbirdResult<()> {
    let level1 = SecurityLevel::Maximum;
    let level2 = level1.clone();
    assert_eq!(level1, level2);
    Ok(())
}

#[test]
fn test_security_level_serialization() -> SongbirdResult<()> {
    let level = SecurityLevel::High;
    let json = serde_json::to_string(&level)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: SecurityLevel = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized, level);
    Ok(())
}
