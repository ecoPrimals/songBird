// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Crate-level helpers for common capability names (`compute`, `ai`, `storage`, `security`).

use songbird_types::SongbirdResult;
use std::time::Duration;

use super::engine::RuntimeDiscoveryEngine;
use super::types::DiscoveredService;

/// Discover compute service (`COMPUTE_ENDPOINT`, …).
pub async fn discover_compute() -> SongbirdResult<DiscoveredService> {
    discover_by_capability_timed("compute", Duration::from_secs(5)).await
}

/// Like [`discover_compute`] but with an explicit timeout (tests use a short duration).
pub async fn discover_by_capability_timed(
    capability: &str,
    timeout: Duration,
) -> SongbirdResult<DiscoveredService> {
    RuntimeDiscoveryEngine::with_timeout(timeout).discover_by_capability(capability).await
}

/// Discover AI service (`AI_ENDPOINT`, …).
pub async fn discover_ai() -> SongbirdResult<DiscoveredService> {
    discover_by_capability_timed("ai", Duration::from_secs(5)).await
}

/// Discover storage service (`STORAGE_ENDPOINT`, …).
pub async fn discover_storage() -> SongbirdResult<DiscoveredService> {
    discover_by_capability_timed("storage", Duration::from_secs(5)).await
}

/// Discover security service (`SECURITY_ENDPOINT`, …).
pub async fn discover_security() -> SongbirdResult<DiscoveredService> {
    discover_by_capability_timed("security", Duration::from_secs(5)).await
}
