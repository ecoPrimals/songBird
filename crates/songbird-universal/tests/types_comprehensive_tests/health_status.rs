// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::*;

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_all_variants() {
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded;
    let unhealthy = HealthStatus::Unhealthy;
    let unknown = HealthStatus::Unknown;

    assert_eq!(healthy, HealthStatus::Healthy);
    assert_eq!(degraded, HealthStatus::Degraded);
    assert_eq!(unhealthy, HealthStatus::Unhealthy);
    assert_eq!(unknown, HealthStatus::Unknown);
}

#[test]
fn test_health_status_default() -> SongbirdResult<()> {
    let default = HealthStatus::default();
    assert_eq!(default, HealthStatus::Unknown);
    Ok(())
}

#[test]
fn test_health_status_equality() -> SongbirdResult<()> {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    Ok(())
}

#[test]
fn test_health_status_clone() -> SongbirdResult<()> {
    let health1 = HealthStatus::Healthy;
    let health2 = health1.clone();
    assert_eq!(health1, health2);
    Ok(())
}

#[test]
fn test_health_status_serialization() -> SongbirdResult<()> {
    let health = HealthStatus::Degraded;
    let json = serde_json::to_string(&health)
        .map_err(|_e| SongbirdError::configuration("Failed to serialize"))?;
    let deserialized: HealthStatus = serde_json::from_str(&json)
        .map_err(|_e| SongbirdError::configuration("Failed to deserialize"))?;

    assert_eq!(deserialized, health);
    Ok(())
}
