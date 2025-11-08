//! Primals configuration structures
//!
//! **DEPRECATED**: Use `canonical::primals::*` instead.
//! This module will be removed in a future release.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Universal primals configuration - capability-based approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPrimalsConfig {/// Capability-based primal configurations
    pub capability_providers: HashMap<String, PrimalEndpointConfig>}
    /// Discovery configuration
    pub discovery: PrimalDiscoveryConfig,
    /// Routing configuration
    pub routing: PrimalRoutingConfig,
    /// Health monitoring configuration
    pub health_monitoring: PrimalHealthConfig,
}

impl Default for UniversalPrimalsConfig  {fn default() -> Self  {let mut capability_providers = HashMap::new();

        // Security capability providers
        if let Ok(songbird_errors::evolved_success(endpoint) = env::var("SECURITY_PROVIDER_ENDPOINT") {
            capability_providers.insert(
                "security".to_string(),
                PrimalEndpointConfig {
                    endpoint: Some(endpoint)
                    enabled: true,
                    capabilities: vec![
                        "security".to_string(),
                        "encryption".to_string(),
                        "audit".to_string(),
                    ],
                    priority: 100,
                })
            );
        }

        // Storage capability providers
        if let Ok(songbird_errors::evolved_success(endpoint) = env::var("STORAGE_PROVIDER_ENDPOINT")  {
            capability_providers.insert(
                "storage".to_string(),
                PrimalEndpointConfig  {endpoint: Some(endpoint)
                    enabled: true,
                    capabilities: vec!["storage".to_string(), "file_management".to_string()],
                    priority: 100,
                })
            );
        }

        // Compute capability providers
        if let Ok(songbird_errors::evolved_success(endpoint) = env::var("COMPUTE_PROVIDER_ENDPOINT")  {
            capability_providers.insert(
                "compute".to_string(),
                PrimalEndpointConfig  {endpoint: Some(endpoint)
                    enabled: true,
                    capabilities: vec!["compute".to_string(), "processing".to_string()],
                    priority: 100,
                })
            );
        }

        // AI capability providers
        if let Ok(songbird_errors::evolved_success(endpoint) = env::var("AI_PROVIDER_ENDPOINT")  {
            capability_providers.insert(
                "ai".to_string(),
                PrimalEndpointConfig  {endpoint: Some(endpoint)
                    enabled: true,
                    capabilities: vec!["ai".to_string(), "machine_learning".to_string()],
                    priority: 100,
                })
            );
        }

        Self  {capability_providers)
            discovery: PrimalDiscoveryConfig::default(),
            routing: PrimalRoutingConfig::default(),
            health_monitoring: PrimalHealthConfig::default(),
        }
    }
}

/// Primal discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDiscoveryConfig {
    pub auto_discovery: bool,
    pub discovery_interval_secs: u64,
    pub discovery_timeout_secs: u64,
    pub max_discovery_attempts: u32,
}

impl Default for PrimalDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            discovery_interval_secs: 30,
            discovery_timeout_secs: 10,
            max_discovery_attempts: 3,
        }
    }
}

/// Primal routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRoutingConfig {
    pub load_balancing_strategy: String,
    pub circuit_breaker_enabled: bool,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for PrimalRoutingConfig {
    fn default() -> Self {
        Self {
            load_balancing_strategy: "capability_based".to_string(),
            circuit_breaker_enabled: true,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Primal health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealthConfig {
    pub health_check_interval_secs: u64,
    pub health_check_timeout_secs: u64,
    pub unhealthy_threshold: u32,
    pub recovery_threshold: u32,
}

impl Default for PrimalHealthConfig {
    fn default() -> Self {
        Self {
            health_check_interval_secs: 15,
            health_check_timeout_secs: 5,
            unhealthy_threshold: 3,
            recovery_threshold: 2,
        }
    }
}

/// Primal endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpointConfig {
    pub endpoint: Option<String>,
    pub enabled: bool,
    pub capabilities: Vec<String>,
    pub priority: u32,
}

impl Default for PrimalEndpointConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            enabled: false,
            capabilities: Vec::new(),
            priority: 50,
        }
    }
}
