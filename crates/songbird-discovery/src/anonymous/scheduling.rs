// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Broadcast timing: periodic interval and session rotation for anonymous discovery.

use std::time::{SystemTime, UNIX_EPOCH};

/// Tokio interval ticking every `interval_secs` seconds.
#[must_use]
pub fn broadcast_interval(interval_secs: u64) -> tokio::time::Interval {
    tokio::time::interval(tokio::time::Duration::from_secs(interval_secs))
}

/// Session ID for beacon rotation (timestamp-based, rotates every ~hour).
///
/// Prevents long-term tracking by changing session identifiers.
#[must_use]
pub fn rotating_session_id() -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    // Session ID rotates every hour (3600 seconds)
    // Production: Change to 86400 for daily rotation
    let session_slot = timestamp / 3600;

    format!("session-{session_slot}")
}
