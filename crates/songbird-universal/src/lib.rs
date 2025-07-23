//! Songbird Universal Primal Integration
//!
//! This crate provides universal, name-agnostic primal integration capabilities
//! that work with any primal without hardcoded assumptions.

pub mod capabilities;
pub mod communication;
pub mod discovery;
pub mod traits;
pub mod types;

// Re-export main types for convenience
pub use capabilities::{DiscoveryConfig, UniversalCapabilityAdapter};
pub use discovery::{DiscoveryConfig as PrimalDiscoveryConfig, UniversalPrimalDiscovery};
pub use types::{
    Capability,
    // Additional types
    CapabilityRequirement,
    ConfigError,
    // Error types
    DiscoveryError,
    EventError,
    HealthStatus,
    LoadBalancingConfig,
    LoadBalancingError,
    MetricsError,
    PrimalCapability,
    PrimalType,
    ProtocolCharacteristics,
    ProtocolError,
    QosMetrics,
    RegisteredService,
    RegistryError,
    SecurityConfig,
    SecurityContext,
    SecurityError,
    SecurityLevel,
    ServiceCapability,
    ServiceError,
    ServiceHealth,
    ServiceInfo,
    UniversalEvent,
    UniversalRequest,
    UniversalResponse,
};

// Re-export everything from types for easy access
pub use types::*;
