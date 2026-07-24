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
pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;
/// Default health check port (env: `SONGBIRD_HEALTH_PORT`)
pub const DEFAULT_HEALTH_PORT: u16 = 8002;
/// Default crypto provider TCP transport port (security provider fallback)
pub const DEFAULT_CRYPTO_TRANSPORT_PORT: u16 = 9876;
/// Default federation bind port (env: `SONGBIRD_FEDERATION_BIND_PORT`)
pub const DEFAULT_FEDERATION_BIND_PORT: u16 = 7000;
/// Default dashboard port (env: `SONGBIRD_DASHBOARD_PORT`)
pub const DEFAULT_DASHBOARD_PORT: u16 = 8003;
/// Default relay port (env: `SONGBIRD_RELAY_PORT`)
pub const DEFAULT_RELAY_PORT: u16 = 3479;
/// Minimum port for dynamic hash-based allocation (avoids well-known ports)
pub const DYNAMIC_PORT_RANGE_MIN: u16 = 1024;
/// Size of the dynamic port allocation window
pub const DYNAMIC_PORT_RANGE_SIZE: u64 = 60_000;
/// Standard HTTPS port (RFC 2818)
pub const HTTPS_STANDARD_PORT: u16 = 443;
/// Default `BirdSong` relay protocol port (env: `SONGBIRD_BIRDSONG_PORT`)
pub const DEFAULT_BIRDSONG_PORT: u16 = 42_424;
/// Default QUIC transport port (env: `SONGBIRD_QUIC_PORT`)
pub const DEFAULT_QUIC_PORT: u16 = 4433;
/// Default request timeout in milliseconds
pub const DEFAULT_TIMEOUT_MS: u64 = 5000;
/// Discovery HTTP service port (env: `SONGBIRD_DISCOVERY_PORT`)
pub const DEFAULT_DISCOVERY_SERVICE_PORT: u16 = 8081;
/// Prometheus-standard observability / metrics scrape port (env: `SONGBIRD_METRICS_PORT`)
pub const DEFAULT_OBSERVABILITY_PORT: u16 = 9090;
/// Dashboard frontend dev-server port (env: `SONGBIRD_DASHBOARD_PORT`)
pub const DEFAULT_DASHBOARD_UI_PORT: u16 = 3000;
/// Federation coordination port (env: `SONGBIRD_FEDERATION_PORT`)
pub const DEFAULT_FEDERATION_COORDINATION_PORT: u16 = 8082;
/// tarpc binary RPC transport port (env: `SONGBIRD_TARPC_PORT`)
pub const DEFAULT_TARPC_RPC_PORT: u16 = 8091;
/// `StarCraft` IPX / gaming base port (env: `SONGBIRD_GAMING_PORT`)
pub const DEFAULT_GAMING_BASE_PORT: u16 = 6112;
/// Default mesh peer federation port for cross-gate communication (env: `SONGBIRD_FEDERATION_PORT`)
pub const DEFAULT_MESH_PEER_PORT: u16 = 7700;
/// Ephemeral bind address for port-0 allocation
pub const EPHEMERAL_BIND_ADDR: &str = "127.0.0.1:0";
/// Consul agent HTTP API default port
pub const CONSUL_DEFAULT_PORT: u16 = 8500;
/// Eureka server default port
pub const EUREKA_DEFAULT_PORT: u16 = 8761;
/// Broadcast/mDNS discovery port (env: `SONGBIRD_BROADCAST_DISCOVERY_PORT`)
pub const DEFAULT_BROADCAST_DISCOVERY_PORT: u16 = 2300;
/// Default AI provider port (env: `SONGBIRD_AI_PROVIDER_PORT`)
pub const DEFAULT_AI_PROVIDER_PORT: u16 = 8083;
/// Default federation broadcast port (secondary range)
pub const DEFAULT_FEDERATION_BROADCAST_PORT: u16 = 8090;
/// Default IPC listen port (env: `SONGBIRD_IPC_PORT`)
pub const DEFAULT_IPC_LISTEN_PORT: u16 = 9901;
/// Default drawbridge HTTP proxy port (env: `SONGBIRD_DRAWBRIDGE_ADDR`)
pub const DEFAULT_DRAWBRIDGE_PORT: u16 = 7780;
/// Default drawbridge bind address (env: `SONGBIRD_DRAWBRIDGE_ADDR`)
pub const DEFAULT_DRAWBRIDGE_ADDR: &str = "127.0.0.1:7780";
