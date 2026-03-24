// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird Types Library
//!
//! **CANONICAL**: Core types and configurations for the Songbird ecosystem
#![forbid(unsafe_code)]
#![warn(missing_docs)]

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

// Re-export genetic lineage types
pub use lineage::{
    CurrentLineage, LineageError, LineageId, LineageProof, LineageSignature, LineageVerification,
};

// Re-export progressive trust types
pub use trust::{
    ElevationEvidence, ElevationPath, TrustEvaluation, TrustLevel, is_operation_allowed,
};

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
pub use zero_copy::{Shareable, Shared, arc, share, smart_cow};

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
pub mod defaults;
pub mod error_helpers;
pub mod errors;
pub mod health;
pub mod lineage; // Genetic lineage types
pub mod memory_optimized;
pub mod primal;
pub mod response;
pub mod service;
pub mod traits;
pub mod trust;
pub mod trust_tests; // Progressive trust model types
pub mod types;
pub mod zero_copy;
pub mod zero_copy_request;
pub mod zero_copy_service;

// Centralized primal name constants
pub mod primal_names;

// Pure Rust system metrics (replaces sysinfo crate for ecoBin v3.0)
pub mod sys_metrics;

// Modern safe buffer - 100% safe Rust (RECOMMENDED)
//
// Use `modern_safe_buffer::ModernSafeBuffer` for zero-copy operations.
// - ✅ 0 unsafe blocks
// - ✅ <1% performance difference vs unsafe
// - ✅ Fully compiler-verified safety
pub mod modern_safe_buffer;

// Note: The legacy `safe_zero_copy` module has been removed.
// It contained 7 unsafe blocks and has been superseded by `modern_safe_buffer`
// which achieves the same performance with 100% safe Rust.

// Backward compatibility alias for unified_constants
// This allows old code using `songbird_types::unified_constants` to continue working
pub use constants as unified_constants;
