//! Canonical configuration patterns and types

pub mod constants;
pub mod environment;
pub mod network;
pub mod service;

// Re-export canonical constants for easy access
pub use constants::{
    get_canonical_bind_address, get_canonical_cors_origins, get_canonical_discovery_endpoint,
    get_canonical_endpoint, get_canonical_gaming_endpoint, get_canonical_orchestrator_endpoint,
    get_canonical_security_endpoint, is_development_environment, is_production_environment,
    CanonicalNetworkDefaults,
};

// Re-export all canonical types
pub use environment::*;
pub use network::*;
pub use service::*;

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
