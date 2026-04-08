// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;
use std::collections::HashMap;

// ============================================================================
// UNIVERSAL RESPONSE TESTS
// ============================================================================

#[test]
fn test_universal_response_success() {
    let response = UniversalResponse {
        request_id: "req-001".to_string(),
        status: ResponseStatus::Success,
        data: Some(serde_json::json!({"result": "ok"})),
        error: None,
        metadata: HashMap::new(),
    };

    assert_eq!(response.status, ResponseStatus::Success);
    assert!(response.data.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_universal_response_error() -> SongbirdResult<()> {
    let response = UniversalResponse {
        request_id: "req-002".to_string(),
        status: ResponseStatus::Error,
        data: None,
        error: Some("Operation failed".to_string()),
        metadata: HashMap::new(),
    };

    assert_eq!(response.status, ResponseStatus::Error);
    assert!(response.error.is_some());
    Ok(())
}

#[test]
fn test_universal_response_serialization() -> SongbirdResult<()> {
    let response = UniversalResponse {
        request_id: "req-003".to_string(),
        status: ResponseStatus::Success,
        data: None,
        error: None,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&response)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: UniversalResponse = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized.request_id, response.request_id);
    Ok(())
}
