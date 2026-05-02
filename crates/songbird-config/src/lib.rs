// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird Configuration System
//!
//! This crate provides comprehensive configuration management for Songbird,
//! supporting environment-based configuration, validation, and zero-hardcoded systems.
//!
//! ## 🎯 **Configuration Consolidation Status** (November 2025)
//!
//! This crate is undergoing consolidation to establish `canonical/` as the single source of truth.
//!
//! ### **Recommended Usage** (Modern)
//! ```text
//! // PREFERRED: Use canonical exports
//! use songbird_config::canonical::{
//!     NetworkConfig,
//!     EnvironmentConfig,
//!     // Note: SecurityConfig not re-exported (use canonical::security directly)
//!     ServiceConfig,
//! };
//! ```
//!
//! ### **Legacy Usage** (Deprecated - use canonical instead)
//! ```text
//! // DEPRECATED: Old config module (use canonical:: instead)
//! use songbird_config::config::NetworkConfig;  // Migrate to canonical::NetworkConfig
//! ```
//!
//! ### **Migration Path**
//! - **Old**: `use songbird_config::config::*;`
//! - **New**: `use songbird_config::canonical::*;`
//! - **Timeline**: Old paths maintained for backward compatibility through Q1 2026
//!
//! ## Key Features
//! - **Canonical Configuration**: Single source of truth in `canonical/` module
//! - **Environment-Based Configuration**: Support for dev/staging/production
//! - **Zero-Hardcoded Values**: All values configurable through environment or files
//! - **Type-Safe Defaults**: Compile-time configuration validation
//! - **Performance Tuning**: Configurable performance parameters

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::unused_async,
    clippy::too_many_lines,
    clippy::implicit_hasher,
    reason = "config crate: broad surface; doc and style exceptions during consolidation"
)]
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        reason = "test code: relaxed lints for assertions, mock construction, and test ergonomics"
    )
)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CANONICAL MODULE - SINGLE SOURCE OF TRUTH
// ============================================================================
/// **PRIMARY**: Canonical configuration types - use these for all new code
pub mod canonical;

/// **UNIFIED**: Unified configuration types consolidating various config patterns
pub mod unified;

// ============================================================================
// STABLE MODULES - KEEP
// ============================================================================
/// **TEST INFRASTRUCTURE**: Thread-safe environment overrides for concurrent testing
pub mod env_override;

/// **MODERN ARCHITECTURE**: Environment provider trait for dependency injection
pub mod env_provider;

/// **MODERN**: Capability-based service discovery (replaces hardcoded endpoints)
pub mod capability_discovery;
/// Shared helpers used by discovery adapters and tests.
pub mod discovery_helpers;
/// Runtime wiring that binds configuration to live discovery clients.
pub mod runtime_discovery;
/// Resolves endpoints after environment overrides are applied.
pub mod runtime_endpoint_resolver;

/// **PRODUCTION-READY**: mDNS and advanced discovery implementations
pub mod discovery;

/// **MODERN**: Agnostic primal configuration (zero hardcoded primal names)
pub mod agnostic_primal_config;

/// **MODERN**: Primal discovery functions (replaces deprecated endpoint constants)
pub mod primal_discovery;

/// Capability-based endpoint configuration
pub mod capability_endpoints;

/// Default values for common configuration
pub mod defaults;

/// Discoverable endpoint types
pub mod discoverable_endpoint;

/// Zero-touch deployment configuration
pub mod zero_touch;

/// Capability-based runtime discovery (zero hardcoding)
pub mod capability_based_runtime_discovery;

/// Port discovery - Dynamic port allocation
pub mod port_discovery;

/// Capability-based port configuration (zero hardcoding)
pub mod capability_port_config;

/// **MODERN**: Timeout configuration (replaces hardcoded Duration::from_secs)
pub mod timeouts;

// ============================================================================
// LEGACY MODULES - BEING CONSOLIDATED
// ============================================================================
/// **DEPRECATED**: Use `canonical::` instead - being consolidated into canonical
///
/// This module contains fragmented configuration that is being migrated to `canonical/`.
/// For new code, use `songbird_config::canonical::*` instead.
#[deprecated(
    since = "0.2.0",
    note = "Use `canonical::` module instead. This module is being phased out. \
            Migration: `config::NetworkConfig` → `canonical::NetworkConfig`"
)]
pub mod config;

// ✅ REMOVED: Environment config helpers (Nov 9, 2025)
// Consolidated into canonical::environment

// ============================================================================
// PUBLIC API - PREFER CANONICAL
// ============================================================================

// ============================================================================
// PRIMARY EXPORT - The ONE TRUE Configuration
// ============================================================================

/// **PRIMARY**: Re-export the canonical configuration from songbird-types
///
/// All new code should use this configuration system.
pub use songbird_types::config::CanonicalSongbirdConfig;

/// Use this alias when you want a short name for [`CanonicalSongbirdConfig`].
pub type Config = CanonicalSongbirdConfig;

// ============================================================================
// CANONICAL MODULE EXPORTS
// ============================================================================

// Re-export canonical types for convenience (PREFERRED)
// Note: Only export types that exist in canonical module
pub use canonical::{CanonicalNetworkDefaults, HealthStatus, ServiceHealth, UniversalHealthStatus};

// Individual module re-exports for convenience
pub use canonical::environment::EnvironmentConfig as CanonicalEnvironmentConfig;
pub use canonical::network::NetworkConfig as CanonicalNetworkConfig;
pub use canonical::service::ServiceConfig as CanonicalServiceConfig;

// ============================================================================
// DEPRECATED - Backward Compatibility Only
// ============================================================================

/// **DEPRECATED**: Use `CanonicalSongbirdConfig` from songbird-types instead
///
/// # Migration
/// ```
/// // OLD
/// use songbird_config::SongbirdConfig;
///
/// // NEW  
/// use songbird_types::config::CanonicalSongbirdConfig;
/// // or use songbird_config::Config;
/// ```
#[allow(
    deprecated,
    reason = "backward-compat type alias; callers migrate to CanonicalSongbirdConfig"
)]
#[deprecated(since = "0.2.0", note = "Use songbird_types::config::CanonicalSongbirdConfig instead")]
pub type SongbirdConfig = songbird_types::config::CanonicalSongbirdConfig;

// Legacy re-exports (DEPRECATED - maintained for backward compatibility)
#[allow(deprecated, reason = "backward-compat glob re-export of legacy config module")]
pub use config::*;

// Legacy `EnvironmentConfig` / `EnvironmentConfigClean` re-exports removed (Nov 9, 2025);
// use `canonical::environment::EnvironmentConfig` instead.

/// Performance configuration for fine-tuning system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Buffer pool size for high-performance operations
    pub buffer_pool_size: Option<usize>,

    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,

    /// Number of worker threads (default: CPU cores)
    pub worker_threads: Option<usize>,

    /// Connection pool size for networking
    pub connection_pool_size: Option<usize>,

    /// Request timeout in milliseconds
    pub request_timeout_ms: Option<u64>,

    /// Enable zero-copy optimizations where possible
    pub enable_zero_copy: Option<bool>,

    /// Batch processing size for bulk operations
    pub batch_size: Option<usize>,

    /// Custom performance parameters
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            buffer_pool_size: None, // Calculated based on environment
            max_memory_mb: None,    // Detected from system
            worker_threads: None,   // Defaults to CPU cores
            connection_pool_size: Some(100),
            request_timeout_ms: Some(30000),
            enable_zero_copy: Some(true),
            batch_size: Some(1000),
            custom_params: None,
        }
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;

// Test helpers exposed for both unit and integration tests
pub mod test_helpers;
