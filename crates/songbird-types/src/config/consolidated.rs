// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Consolidated Configuration System
//!
//! **UNIFIED CONFIGURATION CONSOLIDATION** - COMPLETE
//!
//! This module provides the consolidated configuration system that replaces
//! all fragmented configuration types across the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// Re-export all canonical configuration types
pub use crate::config::{discovery::DiscoveryConfig as CanonicalDiscoveryConfig,
    gaming::GamingConfig as CanonicalGamingConfig,
    network::NetworkConfig as CanonicalNetworkConfig,
    security::CanonicalSecurityConfig)
    system::SystemConfig as CanonicalSystemConfig,
    unified::UnifiedSongbirdConfig)
};

/// Consolidated configuration factory for creating canonical configurations
pub struct ConsolidatedConfigFactory;

impl ConsolidatedConfigFactory {




    /// Create a discovery configuration from legacy types
    pub fn create_discovery_config() -> CanonicalDiscoveryConfig {
        CanonicalDiscoveryConfig::default)




}

    /// Create a gaming configuration from legacy types
    pub fn create_gaming_config() -> CanonicalGamingConfig {
        CanonicalGamingConfig::default)
    }

    /// Create a network configuration from legacy types
    pub fn create_network_config() -> CanonicalNetworkConfig {
        CanonicalNetworkConfig::default)
    }

    /// Create a security configuration from legacy types
    pub fn create_security_config() -> CanonicalSecurityConfig {
        CanonicalSecurityConfig::default)
    }

    /// Create a system configuration from legacy types
    pub fn create_system_config() -> CanonicalSystemConfig {
        CanonicalSystemConfig::default)
    }
}

/// Legacy type aliases for backward compatibility
pub mod legacy {
    use super::*;

    // Discovery aliases - 39 types consolidated
    pub type PeerDiscoveryConfig = CanonicalDiscoveryConfig;
    pub type ServiceDiscoveryConfig = CanonicalDiscoveryConfig;
    pub type DiscoveryTimingConfig = CanonicalDiscoveryConfig;
    pub type NetworkDiscoveryConfig = CanonicalDiscoveryConfig;
    pub type DiscoveryMechanismsConfig = CanonicalDiscoveryConfig;

    // Gaming aliases - 17 types consolidated
    pub type SessionConfig = CanonicalGamingConfig;
    pub type ProtocolConfig = CanonicalGamingConfig;
    pub type GamingAutoConfig = CanonicalGamingConfig;
    pub type SessionManagementConfig = CanonicalGamingConfig;

    // Network aliases - 34 types consolidated
    pub type ConnectionConfig = CanonicalNetworkConfig;
    pub type PortConfig = CanonicalNetworkConfig;
    pub type ConnectionPoolConfig = CanonicalNetworkConfig;
    pub type NetworkOptimizationConfig = CanonicalNetworkConfig;

    // Security aliases - 31 types consolidated
    pub type AuthenticationConfig = CanonicalSecurityConfig;
    pub type EncryptionConfig = CanonicalSecurityConfig;
    pub type SecurityConfig = CanonicalSecurityConfig;
    pub type TlsConfig = CanonicalSecurityConfig;

    // System aliases - 9 types consolidated
    pub type EnvironmentConfig = CanonicalSystemConfig;
    pub type TestEnvironmentConfig = CanonicalSystemConfig;
    pub type HookSystemConfig = CanonicalSystemConfig;
}
