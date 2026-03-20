// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Primal Discovery System
//!
//! **SMART REFACTORING**: This file was 1023 lines, now modularized into:
//! - `types.rs` - Core types and structures
//! - `environment.rs` - Environment-based discovery
//! - `health.rs` - Health checking logic
//! - `mod.rs` (this file) - Main API and orchestration
//!
//! This module provides name-agnostic primal discovery that works with any primal
//! without hardcoding specific primal names. The system uses capability-based
//! discovery and environment-based configuration.

#![allow(
    clippy::unused_self,
    clippy::match_same_arms,
    clippy::zero_sized_map_values,
    clippy::unused_async
)]

mod types;
mod environment;
mod health;

// Re-export main types
pub use types::{
    DiscoveredPrimal,
    DiscoveryConfig,
    DiscoveryMechanisms,
    DiscoveryMethod,
    DiscoveryError,
    PrimalHealth,
};
pub use environment::EnvironmentDiscovery;
pub use health::HealthChecker;

use std::collections::HashMap;
use tokio::time::Duration;
use tracing::info;

/// Universal primal discovery engine that works with any primal
///
/// # Architecture
/// 
/// This discovery engine follows zero-hardcoding principles:
/// - No primal names are hardcoded
/// - Discovery happens through capabilities
/// - Extensible to new discovery methods
/// - Caching for performance
#[derive(Debug, Clone)]
pub struct UniversalPrimalDiscovery {
    /// Discovery configuration
    config: DiscoveryConfig,
    /// Cache of discovered primals
    discovered_cache: HashMap<String, DiscoveredPrimal>,
}

impl UniversalPrimalDiscovery {
    /// Create a new universal primal discovery engine
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            discovered_cache: HashMap::new(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DiscoveryConfig::default())
    }

    /// Discover all available primals using configured mechanisms
    pub async fn discover_all(&mut self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        let mut all_discovered = Vec::new();

        // 1. Environment-based discovery
        if self.config.mechanisms.enable_environment_scan {
            let env_primals = EnvironmentDiscovery::discover();
            all_discovered.extend(env_primals);
        }

        // 2. Network scanning
        if self.config.mechanisms.enable_network_scanning {
            match NetworkScanner::scan(&self.config).await {
                Ok(mut network_primals) => {
                    debug!("Found {} primals via network scan", network_primals.len());
                    all_discovered.extend(network_primals);
                }
                Err(e) => {
                    debug!("Network scanning failed: {}", e);
                }
            }
        }

        // 3. Container orchestration discovery
        if self.config.mechanisms.enable_container_discovery {
            match ContainerDiscovery::discover(&self.config).await {
                Ok(mut container_primals) => {
                    debug!("Found {} primals via container discovery", container_primals.len());
                    all_discovered.extend(container_primals);
                }
                Err(e) => {
                    debug!("Container discovery failed: {}", e);
                }
            }
        }

        if all_discovered.is_empty() {
            return Err(DiscoveryError::NoPrimalsFound);
        }

        // Perform health checks
        let health_checker = HealthChecker::new(self.config.timeout);
        health_checker.check_all(&mut all_discovered).await;

        // Update cache
        for primal in &all_discovered {
            self.discovered_cache.insert(primal.endpoint.clone(), primal.clone());
        }

        info!("✅ Discovered {} primals total", all_discovered.len());
        Ok(all_discovered)
    }

    /// Discover primals by capability
    pub async fn discover_by_capability(&mut self, capability: &str) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        let all_primals = self.discover_all().await?;
        
        // Filter by capability (case-insensitive matching)
        let matching: Vec<_> = all_primals
            .into_iter()
            .filter(|primal| {
                primal.capabilities.iter().any(|cap| {
                    format!("{:?}", cap).to_lowercase().contains(&capability.to_lowercase())
                })
            })
            .collect();

        if matching.is_empty() {
            Err(DiscoveryError::NoPrimalsFound)
        } else {
            Ok(matching)
        }
    }

    /// Get cached discovered primals
    pub fn get_cached(&self) -> Vec<DiscoveredPrimal> {
        self.discovered_cache.values().cloned().collect()
    }

    /// Clear the discovery cache
    pub fn clear_cache(&mut self) {
        self.discovered_cache.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &DiscoveryConfig {
        &self.config
    }
}

impl Default for UniversalPrimalDiscovery {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_engine_creation() {
        let engine = UniversalPrimalDiscovery::with_defaults();
        assert!(engine.get_cached().is_empty());
    }

    #[tokio::test]
    async fn test_discovery_with_environment() {
        songbird_process_env::set_var("SECURITY_ENDPOINT", "http://localhost:8443");
        
        let mut engine = UniversalPrimalDiscovery::with_defaults();
        let result = engine.discover_all().await;
        
        assert!(result.is_ok());
        
        songbird_process_env::remove_var("SECURITY_ENDPOINT");
    }

    #[test]
    fn test_cache_operations() {
        let mut engine = UniversalPrimalDiscovery::with_defaults();
        assert!(engine.get_cached().is_empty());
        
        engine.clear_cache();
        assert!(engine.get_cached().is_empty());
    }
}

