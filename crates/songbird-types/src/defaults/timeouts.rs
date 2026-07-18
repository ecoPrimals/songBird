// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default timeout values

use std::time::Duration;

/// Default upper bound for completing a single request.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// Default time to wait when establishing a connection.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default idle period before closing or recycling a connection.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
/// Default interval for health-check polling loops.
pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
/// Default timeout for long-running compute / execution tasks.
pub const DEFAULT_COMPUTE_TIMEOUT: Duration = Duration::from_secs(300);
/// Default cache TTL for capability/session caches.
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
/// Default SSO / auth token validation timeout.
pub const DEFAULT_AUTH_VALIDATION_TIMEOUT: Duration = Duration::from_secs(10);
/// Default peer/socket peek timeout during protocol auto-detection.
pub const DEFAULT_PEEK_TIMEOUT: Duration = Duration::from_secs(5);
/// Default discovery bridge poll interval.
pub const DEFAULT_DISCOVERY_POLL_INTERVAL: Duration = Duration::from_secs(10);
/// Default graceful shutdown timeout.
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Default startup timeout for orchestrated services.
pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

/// Default socket I/O timeout for IPC probes and auto-discovery.
pub const DEFAULT_SOCKET_IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Default quick connectivity check timeout.
pub const DEFAULT_CONNECTIVITY_CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// Default circuit breaker timeout before half-open retry.
pub const DEFAULT_CIRCUIT_BREAKER_TIMEOUT: Duration = Duration::from_secs(60);

/// Default rate limiter window.
pub const DEFAULT_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Default task/resource cleanup interval (1 hour).
pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(3600);

/// Default retry initial backoff.
pub const DEFAULT_RETRY_INITIAL_BACKOFF: Duration = Duration::from_millis(100);

/// Default retry maximum backoff.
pub const DEFAULT_RETRY_MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Polling interval for accept loops (non-blocking accept with timeout).
pub const DEFAULT_ACCEPT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Default BTSP NDJSON handshake read timeout.
/// Accounts for security provider relay latency (crypto operations) with margin.
pub const DEFAULT_BTSP_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);

/// Default federation heartbeat interval.
pub const DEFAULT_FEDERATION_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Default federation rendezvous refresh interval.
pub const DEFAULT_FEDERATION_RENDEZVOUS_INTERVAL: Duration = Duration::from_secs(60);

/// Default security adapter RPC timeout.
pub const DEFAULT_SECURITY_ADAPTER_TIMEOUT: Duration = Duration::from_secs(5);

/// Default TLS handshake record read timeout (`ServerHello`, etc.).
pub const DEFAULT_TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Default timeout for reading individual encrypted TLS records during handshake.
pub const DEFAULT_TLS_RECORD_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Window for reading optional post-handshake messages (`NewSessionTicket`).
pub const DEFAULT_POST_HANDSHAKE_READ_WINDOW: Duration = Duration::from_millis(200);

/// Default IPC JSON-RPC response read timeout.
pub const DEFAULT_IPC_JSON_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Default connection pool maximum idle time.
pub const DEFAULT_POOL_MAX_IDLE_TIME: Duration = Duration::from_secs(300);

/// Default connection pool acquire timeout.
pub const DEFAULT_POOL_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Default security provider RPC timeout.
pub const DEFAULT_SECURITY_RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// Default neural API call timeout.
pub const DEFAULT_NEURAL_API_TIMEOUT: Duration = Duration::from_secs(5);

/// Default mDNS discovery scan timeout.
pub const DEFAULT_MDNS_TIMEOUT: Duration = Duration::from_secs(3);

/// Default DNS-SD discovery timeout.
pub const DEFAULT_DNSSD_TIMEOUT: Duration = Duration::from_secs(5);

/// Default UDP hole-punch per-attempt timeout.
pub const DEFAULT_HOLE_PUNCH_ATTEMPT_TIMEOUT: Duration = Duration::from_millis(200);

/// Default delay between UDP hole-punch attempts.
pub const DEFAULT_HOLE_PUNCH_ATTEMPT_DELAY: Duration = Duration::from_millis(50);

/// Default relay service wait cycle cap.
pub const DEFAULT_RELAY_WAIT_CYCLE: Duration = Duration::from_secs(300);

/// Default timeout for mesh peer latency probes.
pub const DEFAULT_MESH_PROBE_TIMEOUT: Duration = Duration::from_millis(5000);

/// Default container runtime API probe timeout.
pub const DEFAULT_CONTAINER_API_TIMEOUT: Duration = Duration::from_secs(10);
