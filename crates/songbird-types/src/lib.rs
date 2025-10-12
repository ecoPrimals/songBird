//! Songbird Types Library
//!
//! **CANONICAL**: Core types and configurations for the Songbird ecosystem

// Re-export all configuration modules
pub use config::*;

// Re-export core types
pub use types::{
    CanonicalAddress, CanonicalEndpoint, CanonicalNodeType, CanonicalRequest, CanonicalResponse,
};

// Re-export service types
pub use service::{
    AllowedValues, CanonicalServiceConfig, CanonicalServiceConfigParameter, CanonicalServiceInfo,
    CanonicalServiceStatus, CanonicalServiceType, ServiceMetrics,
};

// Re-export primal types
pub use primal::{
    CanonicalPrimalConfig, CanonicalPrimalId, CanonicalPrimalResponse, CanonicalPrimalType,
};

// Re-export error types
pub use errors::{SecurityError, SongbirdError, SongbirdResult};

// Re-export response types
pub use response::{
    AIFirstResponse, BoolResponse, JsonResponse, PaginatedResponse, ResponseError,
    SongbirdResponse, StringResponse,
};

// Re-export health types
pub use health::{CanonicalHealthCheck, CanonicalHealthConfig, CanonicalHealthStatus};

// Re-export memory optimized types
pub use memory_optimized::{
    EndpointProtocol, OptimizedCapabilities, OptimizedEndpoint, OptimizedHost, OptimizedPrimalId,
};

// Re-export zero-copy utilities
pub use zero_copy::{arc, share, smart_cow, Shareable, Shared};

// Re-export traits
pub use traits::{
    CanonicalConfigProvider, CanonicalHealthCheck as HealthCheckTrait, CanonicalLoadBalancer,
    CanonicalObservabilityProvider, CanonicalServiceDiscovery, DetailedHealthInfo, HealthStatus,
    ServiceInstanceStatus,
};

// Re-export constants that actually exist
pub use constants::{
    CanonicalDiscoveryDefaults, CanonicalEnvironmentConstants, CanonicalNetworkAddresses,
    CanonicalNetworkLimits, CanonicalPerformanceDefaults, CanonicalResourceDefaults,
};

// Module declarations
pub mod adapters;
pub mod config;
pub mod constants;
pub mod errors;
pub mod health;
pub mod memory_optimized;
pub mod primal;
pub mod response;
pub mod service;
pub mod traits;
pub mod types;
pub mod zero_copy;

// Backward compatibility alias for unified_constants
// This allows old code using `songbird_types::unified_constants` to continue working
pub use constants as unified_constants;
