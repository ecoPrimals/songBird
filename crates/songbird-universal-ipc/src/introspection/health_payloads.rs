// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health probe JSON payloads.
//!
//! Callers provide real subsystem state; this module formats the wire response.

use serde_json::Value;
use songbird_types::primal_names;

/// Minimal liveness probe result (`health.liveness`).
///
/// Returns `{status: "alive"}` per Capability Wire Standard v1.0 L1 checklist.
/// Liveness is unconditional — if the process responds, it's alive.
#[must_use]
pub fn health_liveness() -> Value {
    serde_json::json!({ "status": "alive" })
}

/// Subsystem readiness status.
#[allow(clippy::struct_excessive_bools, reason = "each bool represents a distinct subsystem")]
pub struct SubsystemStatus {
    pub ipc: bool,
    pub discovery: bool,
    pub federation: bool,
    pub tls: bool,
    pub relay: bool,
    pub mesh: bool,
}

impl Default for SubsystemStatus {
    fn default() -> Self {
        Self {
            ipc: true,
            discovery: false,
            federation: false,
            tls: false,
            relay: false,
            mesh: false,
        }
    }
}

impl SubsystemStatus {
    fn status_str(ready: bool) -> &'static str {
        if ready {
            "up"
        } else {
            "degraded"
        }
    }

    fn is_ready(&self) -> bool {
        self.ipc && self.discovery
    }
}

/// Readiness probe result (`health.readiness`).
///
/// Reports actual subsystem state. A node is "ready" only when IPC and discovery
/// are up. Other subsystems may be degraded without blocking readiness.
#[must_use]
pub fn health_readiness(status: &SubsystemStatus) -> Value {
    let overall = if status.is_ready() {
        "ready"
    } else {
        "not_ready"
    };
    serde_json::json!({
        "status": overall,
        "subsystems": {
            "ipc": SubsystemStatus::status_str(status.ipc),
            "discovery": SubsystemStatus::status_str(status.discovery),
            "federation": SubsystemStatus::status_str(status.federation),
            "tls": SubsystemStatus::status_str(status.tls)
        }
    })
}

/// Full health check result (`health.check`).
///
/// Reports detailed subsystem health including relay and mesh.
#[must_use]
pub fn health_check(status: &SubsystemStatus, uptime_secs: Option<u64>) -> Value {
    let overall = if status.is_ready() {
        "healthy"
    } else {
        "degraded"
    };
    serde_json::json!({
        "status": overall,
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_s": uptime_secs,
        "subsystems": {
            "ipc": SubsystemStatus::status_str(status.ipc),
            "discovery": SubsystemStatus::status_str(status.discovery),
            "federation": SubsystemStatus::status_str(status.federation),
            "tls": SubsystemStatus::status_str(status.tls),
            "relay": SubsystemStatus::status_str(status.relay),
            "mesh": SubsystemStatus::status_str(status.mesh)
        }
    })
}

/// Generate health response with runtime metrics (HEALTH-01 compliant).
///
/// Returns `{status, primal, version, uptime_s}` per ecosystem health contract.
#[must_use]
pub fn health(uptime_secs: u64, service_count: usize) -> Value {
    serde_json::json!({
        "status": "healthy",
        "primal": primal_names::SELF_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_s": uptime_secs,
        "services": service_count,
    })
}
