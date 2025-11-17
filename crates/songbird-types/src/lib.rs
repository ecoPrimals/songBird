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

// NOTE: types module declared later in the file (line ~73)
// Re-export canonical types
pub use types::{ErrorSeverity, HookErrorHandling, WarningSeverity};

// Re-export error types
pub use errors::{AutomationHint, SecurityError, SongbirdError, SongbirdResult, Urgency};
// Re-export error helper traits for unwrap elimination
pub use error_helpers::{OptionElimination, SafeEnv, SafeParse, UnwrapElimination};

// Re-export response types
pub use response::{
    AIFirstResponse,
    BoolResponse,
    JsonResponse,
    PaginatedResponse,
    ResponseError,
    StringResponse, // ✅ REMOVED: SongbirdResult (duplicate import from errors module)
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

// Re-export all canonical constants
pub use constants::*;

// Module declarations
pub mod adapters;
pub mod config;
pub mod constants;
pub mod error_helpers;
pub mod errors;
pub mod health;
pub mod memory_optimized;
// pub mod performance;  // Temporarily disabled - syntax errors need fixing
pub mod primal;
pub mod response;
pub mod service;
pub mod traits;
pub mod types;
pub mod zero_copy;

// Backward compatibility alias for unified_constants
// This allows old code using `songbird_types::unified_constants` to continue working
pub use constants as unified_constants;
