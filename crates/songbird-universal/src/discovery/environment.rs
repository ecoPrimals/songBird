//! Environment-based Discovery
//!
//! Discovers primals from environment variables.
//! Part of the smart refactoring of discovery.rs

use super::types::{DiscoveredPrimal, DiscoveryMethod, PrimalHealth};
use crate::capabilities::Capability;
use crate::types::PrimalType;
use songbird_types::SafeEnv;
use std::collections::HashMap;
use tracing::{debug, info};

/// Environment-based primal discovery
pub struct EnvironmentDiscovery;

impl EnvironmentDiscovery {
    /// Discover primals from environment variables
    ///
    /// Scans for environment variables following patterns:
    /// - `{CAPABILITY}_ENDPOINT` - e.g., SECURITY_ENDPOINT, STORAGE_ENDPOINT
    /// - `{PRIMAL_NAME}_ENDPOINT` - Legacy support
    ///
    /// # Example
    /// ```bash
    /// export SECURITY_ENDPOINT="http://localhost:8443"
    /// export STORAGE_ENDPOINT="http://localhost:9000"
    /// export AI_ENDPOINT="http://localhost:9001"
    /// ```
    pub fn discover() -> Vec<DiscoveredPrimal> {
        let mut discovered = Vec::new();

        // Capability-based discovery (modern approach)
        let capability_patterns = vec![
            ("SECURITY", PrimalType::Security, vec![Capability::Security]),
            ("STORAGE", PrimalType::Storage, vec![Capability::Storage]),
            ("AI", PrimalType::AI, vec![Capability::AI]),
            ("COMPUTE", PrimalType::Compute, vec![Capability::Compute]),
            ("NETWORK", PrimalType::Network, vec![Capability::Networking]),
        ];

        for (capability_name, primal_type, capabilities) in capability_patterns {
            let endpoint_var = format!("{}_ENDPOINT", capability_name);
            
            if let Ok(endpoint) = SafeEnv::get(&endpoint_var) {
                debug!("Discovered {} primal at {}", capability_name, endpoint);
                
                let primal = DiscoveredPrimal {
                    name: format!("{} Provider", capability_name),
                    primal_type,
                    endpoint,
                    capabilities,
                    health: PrimalHealth::Unknown,
                    discovery_method: DiscoveryMethod::Environment,
                    metadata: [(
                        "source".to_string(),
                        endpoint_var,
                    )]
                    .into_iter()
                    .collect(),
                };
                
                discovered.push(primal);
            }
        }

        if !discovered.is_empty() {
            info!("✅ Discovered {} primals from environment", discovered.len());
        }

        discovered
    }

    /// Discover all environment variables that look like primal endpoints
    pub fn discover_all_endpoints() -> HashMap<String, String> {
        let mut endpoints = HashMap::new();

        // Scan for any *_ENDPOINT pattern
        for (key, value) in std::env::vars() {
            if key.ends_with("_ENDPOINT") {
                endpoints.insert(key, value);
            }
        }

        endpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_discovery() {
        // Set up test environment
        std::env::set_var("SECURITY_ENDPOINT", "http://localhost:8443");
        std::env::set_var("STORAGE_ENDPOINT", "http://localhost:9000");

        let discovered = EnvironmentDiscovery::discover();
        
        assert!(!discovered.is_empty(), "Should discover primals from environment");
        
        // Clean up
        std::env::remove_var("SECURITY_ENDPOINT");
        std::env::remove_var("STORAGE_ENDPOINT");
    }

    #[test]
    fn test_discover_all_endpoints() {
        std::env::set_var("TEST_ENDPOINT", "http://test:1234");
        std::env::set_var("ANOTHER_ENDPOINT", "http://another:5678");

        let endpoints = EnvironmentDiscovery::discover_all_endpoints();
        
        assert!(endpoints.contains_key("TEST_ENDPOINT"));
        assert!(endpoints.contains_key("ANOTHER_ENDPOINT"));

        std::env::remove_var("TEST_ENDPOINT");
        std::env::remove_var("ANOTHER_ENDPOINT");
    }
}

