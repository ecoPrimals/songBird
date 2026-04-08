// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// RESPONSE STATUS TESTS
// ============================================================================

#[test]
fn test_response_status_all_variants() -> SongbirdResult<()> {
    let success = ResponseStatus::Success;
    let error = ResponseStatus::Error;
    let pending = ResponseStatus::Pending;
    let partial = ResponseStatus::PartialSuccess;

    assert_eq!(success, ResponseStatus::Success);
    assert_eq!(error, ResponseStatus::Error);
    assert_eq!(pending, ResponseStatus::Pending);
    assert_eq!(partial, ResponseStatus::PartialSuccess);
    Ok(())
}

#[test]
fn test_response_status_default() -> SongbirdResult<()> {
    let default = ResponseStatus::default();
    assert_eq!(default, ResponseStatus::Success);
    Ok(())
}

#[test]
fn test_response_status_equality() -> SongbirdResult<()> {
    assert_eq!(ResponseStatus::Success, ResponseStatus::Success);
    assert_ne!(ResponseStatus::Success, ResponseStatus::Error);
    Ok(())
}

#[test]
fn test_response_status_serialization() -> SongbirdResult<()> {
    let status = ResponseStatus::Pending;
    let json = serde_json::to_string(&status)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: ResponseStatus = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized, status);
    Ok(())
}
