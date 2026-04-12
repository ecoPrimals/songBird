// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default port allocations
//!
//! These are fallback defaults when runtime discovery is unavailable.
//! Primals should discover actual ports via capability-based discovery.

/// Default HTTP API port (env: `SONGBIRD_HTTP_PORT`)
pub const DEFAULT_HTTP_PORT: u16 = 8080;
/// Default HTTPS API port (env: `SONGBIRD_HTTPS_PORT`)
pub const DEFAULT_HTTPS_PORT: u16 = 8443;
/// Default metrics port (env: `SONGBIRD_METRICS_PORT`)
pub const DEFAULT_METRICS_PORT: u16 = 8081;
/// Default tarpc RPC port (env: `SONGBIRD_TARPC_PORT`)
pub const DEFAULT_TARPC_PORT: u16 = 8001;
/// Default federation port (env: `SONGBIRD_FEDERATION_PORT`)
pub const DEFAULT_FEDERATION_PORT: u16 = 8000;
/// Default Songbird service port (env: `SONGBIRD_PORT`)
///
/// The canonical well-known port for Songbird IPC, onion endpoints, and IGD
/// port mappings when not overridden by environment or capability discovery.
pub const DEFAULT_SONGBIRD_PORT: u16 = 3492;
/// Default STUN port (env: `SONGBIRD_STUN_PORT`)
pub const DEFAULT_STUN_PORT: u16 = 3478;
/// Default compute bridge port
pub const DEFAULT_COMPUTE_PORT: u16 = 9000;
/// Default port range start
pub const DEFAULT_PORT_RANGE_START: u16 = 8000;
/// Default port range end
pub const DEFAULT_PORT_RANGE_END: u16 = 9000;
/// Default execution agent port (env: `SONGBIRD_AGENT_PORT`)
pub const DEFAULT_EXECUTION_AGENT_PORT: u16 = 9020;
/// Default test runner port
pub const DEFAULT_TEST_RUNNER_PORT: u16 = 8080;
/// Heartbeat interval in milliseconds
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 5000;
/// Default orchestrator API port (env: `SONGBIRD_ORCHESTRATOR_PORT`)
pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8000;
/// Default health check port (env: `SONGBIRD_HEALTH_PORT`)
pub const DEFAULT_HEALTH_PORT: u16 = 8002;
/// Default crypto provider TCP transport port (security provider fallback)
pub const DEFAULT_CRYPTO_TRANSPORT_PORT: u16 = 9876;
/// Default federation bind port (env: `SONGBIRD_FEDERATION_BIND_PORT`)
pub const DEFAULT_FEDERATION_BIND_PORT: u16 = 7000;
/// Default request timeout in milliseconds
pub const DEFAULT_TIMEOUT_MS: u64 = 5000;
