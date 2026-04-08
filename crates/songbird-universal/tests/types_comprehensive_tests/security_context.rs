// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// SECURITY CONTEXT TESTS
// ============================================================================

#[test]
fn test_security_context_creation() {
    let context = SecurityContext {
        user_id: Some("user-123".to_string()),
        session_id: "session-abc".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        security_level: SecurityLevel::High,
    };

    assert_eq!(context.user_id, Some("user-123".to_string()));
    assert_eq!(context.permissions.len(), 2);
}

#[test]
fn test_security_context_anonymous() -> SongbirdResult<()> {
    let context = SecurityContext {
        user_id: None,
        session_id: "anon-session".to_string(),
        permissions: vec!["read".to_string()],
        security_level: SecurityLevel::Basic,
    };

    assert!(context.user_id.is_none());
    assert_eq!(context.permissions.len(), 1);
    Ok(())
}

#[test]
fn test_security_context_serialization() -> SongbirdResult<()> {
    let context = SecurityContext {
        user_id: Some("user-456".to_string()),
        session_id: "session-xyz".to_string(),
        permissions: vec!["admin".to_string()],
        security_level: SecurityLevel::Maximum,
    };

    let json = serde_json::to_string(&context)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: SecurityContext = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.user_id, context.user_id);
    Ok(())
}
