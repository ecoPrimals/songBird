// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;
use std::collections::HashMap;

// ============================================================================
// UNIVERSAL REQUEST TESTS
// ============================================================================

#[test]
fn test_universal_request_creation() {
    let request = UniversalRequest {
        request_id: "req-001".to_string(),
        source: "client-1".to_string(),
        target: "service-1".to_string(),
        action: "query".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    assert_eq!(request.request_id, "req-001");
    assert_eq!(request.action, "query");
}

#[test]
fn test_universal_request_with_parameters() -> SongbirdResult<()> {
    let mut params = HashMap::new();
    params.insert("key".to_string(), serde_json::json!("value"));

    let request = UniversalRequest {
        request_id: "req-002".to_string(),
        source: "client-2".to_string(),
        target: "service-2".to_string(),
        action: "execute".to_string(),
        parameters: params,
        security_context: None,
    };

    assert_eq!(request.parameters.len(), 1);
    Ok(())
}

#[test]
fn test_universal_request_serialization() -> SongbirdResult<()> {
    let request = UniversalRequest {
        request_id: "req-003".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test-action".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let json = serde_json::to_string(&request)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: UniversalRequest = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.request_id, request.request_id);
    Ok(())
}
