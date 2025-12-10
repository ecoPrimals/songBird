//! Canonical configuration patterns and types

pub mod constants;
pub mod discovery;
pub mod environment;
pub mod hardcoded_elimination;
pub mod load_balancing;
// Network configuration - refactored into modular structure (Nov 10, 2025)
// Previously: single 1,261-line file
// Now: organized into domain modules with full backward compatibility
pub mod network;
pub mod observability;
pub mod performance;
pub mod primals;
pub mod resilience;
pub mod security;
pub mod service;

// Test fixtures for canonical types
// TEMPORARILY DISABLED (Nov 10, 2025): Needs updating for consolidated APIs (~77 errors)
// TODO: Update testing.rs to match current canonical struct definitions
// #[cfg(any(test, feature = "test-fixtures"))]
// pub mod testing;

// Re-export canonical constants for easy access (Phase 4: Enhanced Nov 8, 2025)
pub use constants::{
    default_bind_address,
    default_discovery_port,
    default_subnet,
    enable_zero_copy,
    external_address,
    find_primals_with_capability,
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
    get_primal_endpoint,
    get_temp_dir,
    get_worker_threads,
    // Environment checks
    is_development_environment,
    is_production_environment,
    node_id,
    // Protocol configuration
    protocol_port_mappings,
    // Structured exports (note: submodules accessible via constants::network::, etc.)
    CanonicalNetworkDefaults,
    DEFAULT_BIND_ADDRESS,
    DEFAULT_CACHE_TTL,
    // Constants
    DEFAULT_CONFIG_PATH,
    DEFAULT_EVALUATION_TIMEOUT,
    DEFAULT_LOCALHOST,
    DEFAULT_METRICS_INTERVAL,
    LOCALHOST_IPV4,
};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

pub type HealthStatus = ServiceHealth;
pub type UniversalHealthStatus = ServiceHealth;
