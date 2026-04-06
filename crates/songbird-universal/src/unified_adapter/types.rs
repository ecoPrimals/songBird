// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::capabilities::Capability;
use crate::types::{HealthStatus, ServiceInfo};
use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;

/// **UNIFIED**: Capability registry for discovered services
///
/// **FUTURE OPTIMIZATION**: Consider `Arc<str>` for service IDs and capability names
/// when profiling shows clone overhead. Current design prioritizes simplicity.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// Map of service ID to their capabilities
    pub service_capabilities: HashMap<String, Vec<Capability>>,
    /// Map of capability type to services that provide it
    pub capability_providers: HashMap<String, Vec<String>>,
    /// Service metadata and health information
    pub service_info: HashMap<String, ServiceInfo>,
    /// Last update timestamp for each service
    pub last_updated: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

/// **UNIFIED**: Service connection information
#[derive(Debug, Clone)]
pub struct ServiceConnection {
    /// Service endpoint
    pub endpoint: String,
    /// Connection health status
    pub health: HealthStatus,
    /// Performance metrics
    pub metrics: std::collections::HashMap<String, f64>, // Simplified metrics for now
    /// Last successful communication
    pub last_contact: chrono::DateTime<chrono::Utc>,
}

/// **UNIFIED**: Adapter configuration
#[derive(Debug, Clone)]
pub struct UnifiedAdapterConfig {
    /// Discovery timeout
    pub discovery_timeout: std::time::Duration,
    /// Health check interval
    pub health_check_interval: std::time::Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable automatic service discovery
    pub auto_discovery: bool,
    /// Service discovery endpoints
    pub discovery_endpoints: Vec<String>,
}

impl Default for UnifiedAdapterConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: std::time::Duration::from_secs(30),
            health_check_interval: std::time::Duration::from_secs(60),
            max_concurrent_requests: 100,
            auto_discovery: true,
            discovery_endpoints: {
                let host = SafeEnv::get_or_default(
                    "ADAPTER_DISCOVERY_HOST",
                    songbird_config::canonical::constants::get_bind_address(),
                );
                let capabilities_port = SafeEnv::get_port(
                    "ADAPTER_CAPABILITIES_PORT",
                    songbird_config::canonical::constants::network::default_orchestrator_port(),
                )
                .to_string();
                let services_port = SafeEnv::get_port(
                    "ADAPTER_SERVICES_PORT",
                    songbird_config::defaults::ports::discovery_port(),
                )
                .to_string();
                vec![
                    format!("http://{}:{}/capabilities", host, capabilities_port),
                    format!("http://{}:{}/services", host, services_port),
                ]
            },
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of registered services
    pub total_services: usize,
    /// Total number of available capabilities
    pub total_capabilities: usize,
    /// Number of healthy services
    pub healthy_services: usize,
}
