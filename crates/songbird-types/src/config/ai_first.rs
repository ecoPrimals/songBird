//! AI-First Citizen API /// Configuration capability // Configuration

use serde::{Deserialize, Serialize};

/// **CANONICAL**: AI-First Citizen API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct CanonicalAIFirstConfig {
    /// Enable AI-First API features
    /// Enabled field
    pub enabled: bool,
    /// Structured error context for automation
    pub structured_errors: bool,
    /// Enable capability discovery
    pub capability_discovery: bool,
    /// Comprehensive observability
    pub observability: bool,
}

impl Default for CanonicalAIFirstConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            structured_errors: true,
            capability_discovery: true,
            observability: true,
        }
    }
}
