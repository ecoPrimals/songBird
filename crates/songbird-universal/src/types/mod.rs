// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Types for Capability-Based Primal Integration
//!
//! This module provides comprehensive type definitions for the universal
//! capability adapter system, organized by domain for clarity and maintainability.
//!
//! # Module Organization
//!
//! ## Core Modules
//!
//! - **`capability`**: Core capability and primal abstractions - defines WHAT services can do
//! - **`service`**: Service lifecycle and identification - defines WHO provides capabilities
//! - **`communication`**: Inter-service communication primitives - defines HOW services talk
//! - **`config`**: Configuration and reliability patterns - defines HOW WELL services behave
//! - **`errors`**: Error types for all subsystems - defines WHAT can go wrong
//!
//! # Usage
//!
//! Types can be imported directly from this module for backward compatibility:
//!
//! ```rust
//! use songbird_universal::types::{PrimalType, ServiceInfo, UniversalRequest};
//! ```
//!
//! Or from their specific submodules for semantic clarity:
//!
//! ```rust
//! use songbird_universal::types::capability::PrimalType;
//! use songbird_universal::types::service::ServiceInfo;
//! use songbird_universal::types::communication::UniversalRequest;
//! ```

// Submodule declarations
pub mod capability;
pub mod communication;
pub mod config;
// ✅ REMOVED: Deprecated error types migrated to songbird_types::SongbirdError
// pub mod errors;
// pub mod errors_migration; // ✅ REMOVED: Migration helpers no longer needed
pub mod service;

// Re-export all public types for backward compatibility
// This maintains the existing API while enabling semantic organization

// Capability types
// ✅ REMOVED: Deprecated Capability type alias (Nov 9, 2025) - use DiscoveredCapability instead
pub use capability::{
    CapabilityRequirement, DiscoveredCapability, DiscoveryFilters, HealthStatus, PrimalCapability,
    PrimalType, QosMetrics, SecurityLevel, ServiceCapability,
};

// Service types
pub use service::{
    RegisteredService, ResourceSpec, ServiceEndpoint, ServiceEvent, ServiceHealth,
    ServiceIdentification, ServiceInfo,
};

// Communication types
pub use communication::{
    ProtocolCharacteristics, ResponseStatus, SecurityContext, UniversalEvent, UniversalRequest,
    UniversalResponse,
};

// Configuration types
pub use config::{
    CircuitBreakerConfig, FeatureFlags, HealthCheckConfig, LoadBalancingConfig,
    LoadBalancingStrategy, RetryConfig, SecurityConfig,
};

// Error types
// ✅ REMOVED: All error types unified into songbird_types::SongbirdError
// Deprecated error types removed - use songbird_types::SongbirdError instead
// pub use errors::{
//     ConfigError, DiscoveryError, EventError, LoadBalancingError, MetricsError, ProtocolError,
//     RegistryError, SecurityError, ServiceError,
// };

#[cfg(test)]
mod config_tests;

#[cfg(test)]
mod service_tests;
