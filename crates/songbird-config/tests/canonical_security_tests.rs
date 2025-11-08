//! Tests for canonical security types

use songbird_config::canonical::security::SecurityLevel;
use songbird_types::SongbirdResult;
use songbird_types::{SongbirdError, SongbirdResult};
use std::str::FromStr;

#[test]
fn test_security_level_default() {
    let level = SecurityLevel::default();
    assert_eq!(level, SecurityLevel::Public);
}

#[test]
fn test_security_level_variants() {
    assert_eq!(SecurityLevel::None, SecurityLevel::None);
    assert_eq!(SecurityLevel::Minimal, SecurityLevel::Minimal);
    assert_eq!(SecurityLevel::Basic, SecurityLevel::Basic);
    assert_eq!(SecurityLevel::Low, SecurityLevel::Low);
    assert_eq!(SecurityLevel::Medium, SecurityLevel::Medium);
    assert_eq!(SecurityLevel::High, SecurityLevel::High);
    assert_eq!(SecurityLevel::Maximum, SecurityLevel::Maximum);
}

#[test]
fn test_security_level_ordering() -> SongbirdResult<()> {
    // Test that security levels can be compared
    assert_ne!(SecurityLevel::None, SecurityLevel::Maximum);
    assert_ne!(SecurityLevel::Low, SecurityLevel::High);
    Ok(())
}

#[test]
fn test_security_level_display() -> SongbirdResult<()> {
    assert_eq!(SecurityLevel::None.to_string(), "none");
    assert_eq!(SecurityLevel::Minimal.to_string(), "minimal");
    assert_eq!(SecurityLevel::Basic.to_string(), "basic");
    assert_eq!(SecurityLevel::Low.to_string(), "low");
    assert_eq!(SecurityLevel::Medium.to_string(), "medium");
    assert_eq!(SecurityLevel::Standard.to_string(), "standard");
    assert_eq!(SecurityLevel::Public.to_string(), "public");
    assert_eq!(SecurityLevel::High.to_string(), "high");
    assert_eq!(SecurityLevel::Private.to_string(), "private");
    assert_eq!(SecurityLevel::Critical.to_string(), "critical");
    assert_eq!(SecurityLevel::Maximum.to_string(), "maximum");
    Ok(())
}

#[test]
fn test_security_level_from_str() -> SongbirdResult<()> {
    assert_eq!(
        SecurityLevel::from_str("none").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::None
    );
    assert_eq!(
        SecurityLevel::from_str("minimal").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::Minimal
    );
    assert_eq!(
        SecurityLevel::from_str("basic").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::Basic
    );
    assert_eq!(
        SecurityLevel::from_str("low").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::Low
    );
    assert_eq!(
        SecurityLevel::from_str("medium").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::Medium
    );
    assert_eq!(
        SecurityLevel::from_str("high").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::High
    );
    assert_eq!(
        SecurityLevel::from_str("maximum").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::Maximum
    );
    Ok(())
}

#[test]
fn test_security_level_from_str_case_insensitive() -> SongbirdResult<()> {
    assert_eq!(
        SecurityLevel::from_str("NONE").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::None
    );
    assert_eq!(
        SecurityLevel::from_str("Medium").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::Medium
    );
    assert_eq!(
        SecurityLevel::from_str("HIGH").or_else(|_| SongbirdError::configuration(format!(
            "TODO: Replace with proper error handling: {}",
            e
        )))?,
        SecurityLevel::High
    );
    Ok(())
}

#[test]
fn test_security_level_from_str_invalid() -> SongbirdResult<()> {
    assert!(SecurityLevel::from_str("invalid").is_err());
    assert!(SecurityLevel::from_str("").is_err());
    assert!(SecurityLevel::from_str("ultra").is_err());
    Ok(())
}

#[test]
fn test_security_level_serialization() -> SongbirdResult<()> {
    let level = SecurityLevel::High;
    let json = serde_json::to_string(&level).map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    let deserialized: SecurityLevel =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(level, deserialized);
    Ok(())
}

#[test]
fn test_security_level_clone() -> SongbirdResult<()> {
    let level = SecurityLevel::High;
    let cloned = level;
    assert_eq!(level, cloned);
    Ok(())
}

#[test]
fn test_security_level_debug() -> SongbirdResult<()> {
    let level = SecurityLevel::High;
    let debug_str = format!("{level:?}");
    assert!(debug_str.contains("High"));
    Ok(())
}

#[test]
fn test_security_level_hash() {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(SecurityLevel::Low);
    set.insert(SecurityLevel::High);
    set.insert(SecurityLevel::Low); // Duplicate

    assert_eq!(set.len(), 2);
    assert!(set.contains(&SecurityLevel::Low));
    assert!(set.contains(&SecurityLevel::High));
}

#[test]
fn test_all_security_level_variants() {
    let variants = [
        SecurityLevel::None,
        SecurityLevel::Minimal,
        SecurityLevel::Basic,
        SecurityLevel::Low,
        SecurityLevel::Medium,
        SecurityLevel::Standard,
        SecurityLevel::Public,
        SecurityLevel::High,
        SecurityLevel::Private,
        SecurityLevel::Critical,
        SecurityLevel::Confidential,
        SecurityLevel::Enhanced,
        SecurityLevel::Maximum,
        SecurityLevel::Classified,
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

#[test]
fn test_security_level_round_trip() -> SongbirdResult<()> {
    let levels = vec![
        SecurityLevel::None,
        SecurityLevel::Low,
        SecurityLevel::Medium,
        SecurityLevel::High,
        SecurityLevel::Maximum,
    ];

    for level in levels {
        let s = level.to_string();
        let parsed = SecurityLevel::from_str(&s).or_else(|_| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
        assert_eq!(level, parsed);
    }
    Ok(())
}
