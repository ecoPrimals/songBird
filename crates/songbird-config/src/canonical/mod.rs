//! Canonical configuration patterns and types

pub mod constants;
pub mod discovery;
pub mod environment;
pub mod load_balancing;
pub mod network;
pub mod observability;
pub mod performance;
pub mod primals;
pub mod resilience;
pub mod security;
pub mod service;

// Re-export canonical constants for easy access (Phase 4: Enhanced Nov 8, 2025)
pub use constants::{
    // Network configuration
    get_bind_address, get_canonical_bind_address, get_default_bind_address, default_bind_address,
    default_discovery_port, external_address, default_subnet, node_id,
    // Port configuration
    get_port_range_start, get_port_range_end, get_dashboard_port,
    // Endpoint configuration
    get_canonical_endpoint, get_canonical_discovery_endpoint, get_canonical_security_endpoint,
    get_canonical_orchestrator_endpoint, get_canonical_gaming_endpoint,
    // Primal configuration
    get_primal_endpoint, get_configured_primal_names, get_common_primal_ports,
    find_primals_with_capability,
    // Timeout configuration
    get_connection_timeout_ms,
    // Resource management
    get_max_connections, get_worker_threads, get_buffer_pool_size, get_batch_size,
    enable_zero_copy,
    // Directory configuration
    get_log_dir, get_cache_dir, get_data_dir, get_config_dir, get_temp_dir,
    // Logging configuration
    get_log_level,
    // Environment checks
    is_development_environment, is_production_environment,
    // CORS configuration
    get_canonical_cors_origins,
    // Protocol configuration
    protocol_port_mappings,
    // Constants
    DEFAULT_CONFIG_PATH, LOCALHOST_IPV4, DEFAULT_BIND_ADDRESS, DEFAULT_LOCALHOST,
    DEFAULT_CACHE_TTL, DEFAULT_EVALUATION_TIMEOUT, DEFAULT_METRICS_INTERVAL,
    // Structured exports (note: submodules accessible via constants::network::, etc.)
    CanonicalNetworkDefaults,
};

// Re-export all canonical types
pub use discovery::*;
pub use environment::*;
pub use load_balancing::*;

// Network exports  
pub use network::{
    NetworkConfig, GamingScale,
};

pub use observability::*;
pub use performance::*;
pub use primals::*;

// Resilience exports
pub use resilience::{
    RetryConfig, CircuitBreakerConfig, RateLimitingConfig,
};

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
