// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// SECURITY CONFIG TESTS
// ============================================================================

#[test]
fn test_security_config_creation() {
    let config = SecurityConfig {
        enabled: true,
        level: SecurityLevel::High,
        authentication_required: true,
        tls_enabled: true,
        certificate_path: Some("/path/to/cert.pem".to_string()),
    };

    assert!(config.enabled);
    assert_eq!(config.level, SecurityLevel::High);
    assert!(config.authentication_required);
}

#[test]
fn test_security_config_disabled() -> SongbirdResult<()> {
    let config = SecurityConfig {
        enabled: false,
        level: SecurityLevel::None,
        authentication_required: false,
        tls_enabled: false,
        certificate_path: None,
    };

    assert!(!config.enabled);
    assert_eq!(config.level, SecurityLevel::None);
    Ok(())
}

#[test]
fn test_security_config_serialization() -> SongbirdResult<()> {
    let config = SecurityConfig {
        enabled: true,
        level: SecurityLevel::Maximum,
        authentication_required: true,
        tls_enabled: true,
        certificate_path: Some("/cert.pem".to_string()),
    };

    let json = serde_json::to_string(&config)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: SecurityConfig = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.enabled, config.enabled);
    Ok(())
}
