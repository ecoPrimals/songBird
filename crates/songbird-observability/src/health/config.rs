//! Health monitoring configuration
//!
//! **MIGRATION COMPLETE**: This module now uses the canonical configuration system.
//! All configuration types have been migrated to `songbird_config::canonical`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// CANONICAL CONFIGURATION RE-EXPORTS
// ============================================================================

/// Universal health monitoring configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical health configuration from songbird_config.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::canonical::observability::ObservabilityConfig as UniversalHealthConfig;

/// Health check configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical health check configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::unified::core::HealthCheckConfig;

// Legacy compatibility types have been removed.
// Use songbird_config::canonical::observability types instead.
