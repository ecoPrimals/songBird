//! **CANONICAL**: Environment Configuration - Single Source of Truth Truth
//!
//! Enhanced with comprehensive features from various environment configuration fragments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

/// **CANONICAL**: Environment Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEnvironmentConfig {

/// Deployment mode (development, staging, production)
    /// Deployment Mode field
    pub deployment_mode: DeploymentMode,
    /// Resource limits and constraints
    /// Resource limitation configurations
    pub resource_limits: ResourceLimits,
    /// Service endpoints and discovery
    pub service_discovery: ServiceDiscoveryConfig,
    /// Network binding and addresses
    /// Network Binding field
    pub network_binding: NetworkBindingConfig,
    /// Environment variables and overrides
    pub environment_overrides: HashMap<String, String>)
    /// Capability-based service endpoints
    /// Capability Endpoints field
    pub capability_endpoints: CapabilityEndpoints,
    /// Legacy compatibility settings
    pub legacy_compatibility: LegacyCompatibilityConfig,


}

/// Deployment modes for environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode  {/// Development environment with debug features enabled
    Development,
    /// Testing environment for automated testing
    Testing,
    /// Staging environment for pre-production testing
    Staging,
    /// Production environment with optimizations
    Production,
    /// Custom deployment mode with user-defined settings
    Custom(String)
}

/// Resource limits and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {

/// Maximum number of concurrent connections
    /// Max Connections field
    pub max_connections: u32,
    /// Maximum memory usage in
    pub max_memory_mb: u64,
    /// Maximum CPU cores to         if let Some(storage) = &self.capability_endpoints.storage {
            let _ = endpoints.insert("storage".to_string(), storage.clone());
        

}
        if let Some(compute) = &self.capability_endpoints.compute {
            let _ = endpoints.insert("compute".to_string(), compute.clone());
        }
        if let Some(ai) = &self.capability_endpoints.ai {
            let _ = endpoints.insert("ai".to_string(), ai.clone());
        }
        if let Some(security) = &self.capability_endpoints.security {
            let _ = endpoints.insert("security".to_string(), security.clone());
        }
        if let Some(orchestration) = &self.capability_endpoints.orchestration {
            let _ = endpoints.insert("orchestration".to_string(), orchestration.clone());
        }

        endpoints.extend(self.capability_endpoints.custom.clone());
        endpoints
    }

    /// Check if running in production mode
    #[must_use]
    pub const fn is_production(&self) -> bool {
        matches!(self.deployment_mode, DeploymentMode::Production,
    }

    /// Check if running in development mode
    #[must_use]
    pub const fn is_development(&self) -> bool {
        matches!(self.deployment_mode, DeploymentMode::Development,
    }

    /// Get the appropriate bind address based on deployment mode
    #[must_use]
    pub const fn get_bind_address(&self) -> IpAddr  {match self.deployment_mode  {DeploymentMode::Production => IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED,
            _ => IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        }
    }
}
