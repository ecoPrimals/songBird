//! Health monitoring configuration
//!
//! **MIGRATION COMPLETE**: This module now uses the unified configuration system.
//! All configuration types have been migrated to `songbird_config::unified`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// UNIFIED CONFIGURATION RE-EXPORTS
// ============================================================================

/// Universal health monitoring configuration - **MIGRATED TO UNIFIED**
/// 
/// This re-exports the unified health configuration from songbird_config.
/// The migration to songbird_config::unified is complete.
pub use songbird_config::unified::observability::UnifiedObservabilityConfig as UniversalHealthConfig;

/// Health check configuration - **MIGRATED TO UNIFIED**
/// 
/// This re-exports the unified health check configuration.
/// The migration to songbird_config::unified is complete.
pub use songbird_config::unified::core::HealthCheckConfig;

// Legacy compatibility types have been removed.
// Use songbird_config::unified::observability types instead.
