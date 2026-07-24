// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical configuration patterns and types

/// Shared constants and accessor helpers for defaults and ports.
pub mod constants;
/// Discovery-related configuration fragments.
pub mod discovery;
/// Environment profile and deployment metadata.
pub mod environment;
/// Helpers that replace one-off literals with named accessors.
pub mod hardcoded_elimination;
/// Load balancer and traffic steering settings.
pub mod load_balancing;
/// Service port configuration (env-driven, validated, capability-bridged).
pub mod port_config;
// Network configuration - refactored into modular structure (Nov 10, 2025)
// Previously: single 1,261-line file
// Now: organized into domain modules with full backward compatibility
/// Network topology, ports, timeouts, and protocol tuning.
pub mod network;
/// Metrics, tracing, and logging hooks.
pub mod observability;
/// Throughput and latency tuning knobs.
pub mod performance;
/// Primal registry and capability wiring.
pub mod primals;
/// Retry, circuit breaking, and backoff policies.
pub mod resilience;
/// TLS, auth, and hardening options.
pub mod security;
/// First-class service endpoint metadata.
pub mod service;

// Re-export canonical constants for easy access (Phase 4: Enhanced Nov 8, 2025)
pub use constants::{
    // Structured exports (note: submodules accessible via constants::network::, etc.)
    CanonicalNetworkDefaults,
    // DEFAULT_BIND_ADDRESS, // Removed: Use get_bind_address() function instead
    DEFAULT_CACHE_TTL,
    // Constants
    DEFAULT_CONFIG_PATH,
    DEFAULT_EVALUATION_TIMEOUT,
    // DEFAULT_LOCALHOST, // Removed: Use get_bind_address() function instead
    DEFAULT_METRICS_INTERVAL,
    FALLBACK_CANONICAL_DISCOVERY_PORT,
    FALLBACK_CANONICAL_GAMING_PORT,
    FALLBACK_CANONICAL_ORCHESTRATOR_PORT,
    FALLBACK_CANONICAL_SECURITY_PORT,
    FALLBACK_PRODUCTION_HTTPS_PORT,
    FALLBACK_PROTOCOL_SECURE_WEBSOCKET_PORT,
    FALLBACK_PROTOCOL_TCP_PORT,
    FALLBACK_PROTOCOL_UDP_PORT,
    FALLBACK_PROTOCOL_WEBSOCKET_PORT,
    FALLBACK_STAGING_HTTP_PORT,
    // LOCALHOST_IPV4, // Removed: Use get_bind_address() function instead
    default_bind_address,
    default_discovery_port,
    default_subnet,
    enable_zero_copy,
    external_address,
    find_primals_with_capability,
    find_primals_with_capability_in_env,
    get_batch_size,
    // Network configuration
    get_bind_address,
    get_buffer_pool_size,
    get_cache_dir,
    get_canonical_bind_address,
    // CORS configuration
    get_canonical_cors_origins,
    get_canonical_discovery_endpoint,
    // Endpoint configuration
    get_canonical_endpoint,
    get_canonical_endpoint_with,
    get_canonical_gaming_endpoint,
    get_canonical_orchestrator_endpoint,
    get_canonical_security_endpoint,
    get_common_primal_ports,
    get_config_dir,
    get_configured_primal_names,
    // Timeout configuration
    get_connection_timeout_ms,
    get_dashboard_port,
    get_data_dir,
    get_default_bind_address,
    // Directory configuration
    get_log_dir,
    // Logging configuration
    get_log_level,
    // Resource management
    get_max_connections,
    get_port_range_end,
    // Port configuration
    get_port_range_start,
    // Primal configuration
    get_temp_dir,
    get_worker_threads,
    // Environment checks
    is_development_environment,
    is_production_environment,
    node_id,
    // Protocol configuration
    protocol_port_mappings,
};

#[allow(deprecated, reason = "re-exporting deprecated shim for backward compatibility")]
pub use constants::get_primal_endpoint;

// Re-export all canonical types
pub use discovery::*;
pub use environment::*;
pub use load_balancing::*;

// Network exports
pub use network::{GamingScale, NetworkConfig};

pub use observability::*;
pub use performance::*;
pub use primals::*;

// Resilience exports
pub use resilience::{CircuitBreakerConfig, RateLimitingConfig, RetryConfig};

pub use security::*;

// Service exports
pub use service::ServiceConfig;

// Type aliases for backward compatibility with proper definitions
/// Coarse health rollup used by orchestration and dashboards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceHealth {
    /// Instance is meeting its SLO.
    Healthy,
    /// Instance is impaired but still serving limited traffic.
    Degraded,
    /// Instance should stop receiving new sessions.
    Unhealthy,
    /// Health probe has not succeeded yet or data is stale.
    Unknown,
}

/// Alias kept for older call sites that referred to `HealthStatus`.
pub type HealthStatus = ServiceHealth;
/// Alias for federation code that prefixed health types with `Universal`.
pub type UniversalHealthStatus = ServiceHealth;
