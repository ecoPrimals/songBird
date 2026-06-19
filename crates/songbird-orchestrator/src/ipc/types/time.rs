// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Time helpers for IPC JSON payloads.

use std::time::SystemTime;

/// Helper to convert `SystemTime` to ISO 8601 string
#[must_use]
pub fn system_time_to_iso8601(time: SystemTime) -> String {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();

    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Simple ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ
    chrono::DateTime::from_timestamp(secs as i64, nanos).map_or_else(
        || String::from("1970-01-01T00:00:00Z"),
        |dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    )
}
