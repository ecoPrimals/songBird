// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health probe JSON payloads.

use serde_json::Value;
use songbird_types::primal_names;

/// Minimal liveness probe result (`health.liveness`).
///
/// Returns `{status: "alive"}` per Capability Wire Standard v1.0 L1 checklist.
#[must_use]
pub fn health_liveness() -> Value {
    serde_json::json!({ "status": "alive" })
}

/// Readiness probe result (`health.readiness`).
#[must_use]
pub fn health_readiness() -> Value {
    serde_json::json!({
        "status": "ready",
        "subsystems": {
            "ipc": "up",
            "discovery": "up",
            "federation": "up",
            "tls": "up"
        }
    })
}

/// Full health check result (`health.check`).
#[must_use]
pub fn health_check() -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": null,
        "subsystems": {
            "ipc": "up",
            "discovery": "up",
            "federation": "up",
            "tls": "up",
            "relay": "up",
            "mesh": "up"
        }
    })
}

/// Generate health response with runtime metrics.
#[must_use]
pub fn health(uptime_secs: u64, service_count: usize) -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
        "services": service_count,
    })
}
