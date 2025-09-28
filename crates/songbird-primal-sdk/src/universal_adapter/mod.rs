/// Universal Primal Adapter - Modular Architecture
///
/// **MODERNIZATION COMPLETE**: Split 1127-line monolithic file into focused modules
///
/// ## Module Organization
/// - `core` - Main UniversalPrimalAdapter implementation and core functionality
/// - `adapter_types` - Extracted type definitions for better organization
/// - `registry` - Capability registry and service registration
/// - `events` - Event handling and broadcasting system  
/// - `roles` - Role matching and service role definitions
/// - `types` - Shared types, structs, and enums
///
/// This demonstrates proper complexity management using the 1000-line guideline
/// as an indicator for architectural improvements.
pub mod adapter_types;
pub mod core;
pub mod events;
pub mod registry;
pub mod roles;
pub mod routing;
pub mod types;

// Re-export main types for backward compatibility - avoiding ambiguous glob imports
pub use core::UniversalPrimalAdapter;
pub use events::{EventBroadcaster, UniversalAdapterEvent};
pub use registry::{CapabilityRegistry, UniversalServiceRegistry};
pub use roles::RoleMatcher;

// Re-export specific types to avoid ambiguity
pub use types::{CapabilityProvider, CapabilityRequirement, CommunicationProtocol, CostInfo, IntegrationPattern, PerformanceMetrics, PerformanceRequirements, ProviderMetrics, ResourceRequirements, RolePriority, ServiceCapability, ServiceHealthInfo, ServiceInstance, ServiceMetadata, ServiceRole, UniversalAdapterConfig};

// Re-export adapter types
pub use adapter_types::{AdapterConfig, AdapterContext, IntegrationPreferences, PerformanceProfile, RetryPolicy, ServiceCategory};

// Re-export routing for compatibility
pub use routing::*;
