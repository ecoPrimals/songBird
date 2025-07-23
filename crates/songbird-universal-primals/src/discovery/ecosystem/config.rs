//! Ecosystem Discovery Configuration

/// Ecosystem service discovery configuration
#[derive(Debug, Clone)]
pub struct EcosystemDiscoveryConfig {
    /// Base ecosystem directory (typically ../)
    pub ecosystem_base_path: String,
    /// Timeout for health checks in milliseconds
    pub health_check_timeout_ms: u64,
    /// Maximum concurrent discovery operations
    pub max_concurrent_discoveries: usize,
    /// Enable local filesystem primal detection
    pub enable_filesystem_discovery: bool,
    /// Enable network-based primal discovery
    pub enable_network_discovery: bool,
    /// Enable capability inference when direct discovery fails
    pub enable_capability_inference: bool,
}

impl Default for EcosystemDiscoveryConfig {
    fn default() -> Self {
        Self {
            ecosystem_base_path: "../".to_string(),
            health_check_timeout_ms: 5000,
            max_concurrent_discoveries: 10,
            enable_filesystem_discovery: true,
            enable_network_discovery: true,
            enable_capability_inference: true,
        }
    }
}
