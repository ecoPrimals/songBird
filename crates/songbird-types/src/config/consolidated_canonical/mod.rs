//! # 🔧 Consolidated Canonical Configuration System
//!
//! **SINGLE SOURCE OF TRUTH FOR ALL CONFIGURATIONS** ✅
//!
//! This module consolidates ALL configuration structures from across the Songbird ecosystem
//! into a single, unified, canonical configuration system. This replaces:
//! 
//! - `songbird-config` crate configurations
//! - `songbird-types` fragmented config modules (25+ config types,
//! - All deprecated configuration aliases and compatibility layers
//!
//! ## Consolidation Summary
//! - **25+ configuration types** → Single `CanonicalSongbirdConfig`
//! - **614 config structs** → Organized hierarchical system
//! - **Multiple config crates** → Single canonical source
//! - **Legacy compatibility** → Clean migration path

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import all sub-modules
pub mod system;
pub mod network;
pub mod security;
pub mod performance;
pub mod discovery;
pub mod observability;
pub mod gaming;
pub mod primals;
pub mod federation;
pub mod environment;
pub mod factory;

// Re-export all types from sub-modules
pub use system::*;
pub use network::*;
pub use security::*;
pub use performance::*;
pub use discovery::*;
pub use observability::*;
pub use gaming::*;
pub use primals::*;
pub use federation::*;
pub use environment::*;
pub use factory::*;

// ============================================================================
// CANONICAL CONFIGURATION - Single Source of Truth
// ============================================================================

/// **CANONICAL**: Main Songbird configuration - replaces ALL fragmented configs
/// 
/// This single configuration structure replaces:
/// - `songbird-config::SongbirdConfig`
/// - `songbird-types::config::UnifiedSongbirdConfig` (multiple versions,
/// - All 25+ fragmented config types across modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalSongbirdConfig {


    /// System-wide configuration
    pub system: CanonicalSystemConfig,
    
    /// Network and communication configuration
    pub network: CanonicalNetworkConfig,
    
    /// Security and authentication configuration
    pub security: CanonicalSecurityConfig,
    
    /// Performance and optimization configuration
    pub performance: CanonicalPerformanceConfig,
    
    /// Service discovery and registration configuration
    pub discovery: CanonicalDiscoveryConfig,
    
    /// Observability, monitoring, and metrics configuration
    pub observability: CanonicalObservabilityConfig,
    
    /// Gaming protocol and bridge configuration
    pub gaming: CanonicalGamingConfig,
    
    /// Universal primal provider configuration
    pub primals: CanonicalPrimalConfig,
    
    /// Federation and clustering configuration
    pub federation: CanonicalFederationConfig,
    
    /// Environment and deployment configuration
    pub environment: CanonicalEnvironmentConfig,
    
    /// Extensibility - custom configuration fields
    pub custom: HashMap<String, serde_json::Value>,


}

impl Default for CanonicalSongbirdConfig {


    fn default() -> Self {
        Self {
            system: CanonicalSystemConfig::default(),
            network: CanonicalNetworkConfig::default(),
            security: CanonicalSecurityConfig::default(),
            performance: CanonicalPerformanceConfig::default(),
            discovery: CanonicalDiscoveryConfig::default(),
            observability: CanonicalObservabilityConfig::default(),
            gaming: CanonicalGamingConfig::default(),
            primals: CanonicalPrimalConfig::default(),
            federation: CanonicalFederationConfig::default(),
            environment: CanonicalEnvironmentConfig::default(),
            custom: HashMap::new()),
        

}
    }
} 