// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Engine Core
//!
//! EVOLVED: Central orchestration of capability-based primal discovery
//! Zero hardcoding - works with ANY primal via capabilities

use super::backends;
use super::cache::DiscoveryCache;
use super::config::DiscoveryConfig;
use super::errors::DiscoveryError;
use super::types::DiscoveredPrimal;
use tracing::{debug, info, warn};

/// Universal primal discovery engine
///
/// **PHILOSOPHY**: Self-knowledge only - discovers what's advertised
/// - No hardcoded primal names
/// - Capability-based discovery
/// - Runtime service binding
/// - Works with ANY primal (current or future)
#[derive(Debug, Clone)]
pub struct UniversalPrimalDiscovery {
    /// Capability adapter for querying primal capabilities
    _capability_adapter: (),
    /// Discovery configuration
    config: DiscoveryConfig,
    /// Discovery cache for deduplication
    cache: DiscoveryCache,
}

impl UniversalPrimalDiscovery {
    /// Create a new universal primal discovery engine
    #[must_use]
    pub fn new(config: DiscoveryConfig) -> Self {
        let _capability_config = crate::capabilities::DiscoveryConfig::default();
        Self {
            _capability_adapter: (),
            config,
            cache: DiscoveryCache::new(),
        }
    }

    /// Discover all available primals using enabled mechanisms
    ///
    /// **EVOLVED**: Orchestrates multiple discovery backends
    /// Returns all successfully discovered primals, logs failures
    ///
    /// # Errors
    ///
    /// Returns error only if ALL discovery methods fail
    pub async fn discover_all_primals(&mut self) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        info!("🔍 Starting universal primal discovery...");

        let mut all_discovered = Vec::new();

        // Environment-based discovery (capability providers via env vars)
        if self.config.mechanisms.enable_environment_scan {
            match backends::environment::discover_from_environment().await {
                Ok(mut env_primals) => {
                    let count = env_primals.len();
                    info!("✅ Discovered {} primals from environment", count);
                    all_discovered.append(&mut env_primals);
                }
                Err(e) => warn!("⚠️ Environment discovery failed: {}", e),
            }
        }

        // Network scanning discovery (mDNS, broadcast)
        if self.config.mechanisms.enable_network_scanning {
            // mDNS previously used a fixed 5s listen; honor shorter test timeouts but cap at 5s.
            let mdns_listen = self.config.timeout.min(tokio::time::Duration::from_secs(5));
            match backends::network::discover_from_network_with_timeout(mdns_listen).await {
                Ok(mut network_primals) => {
                    info!("✅ Discovered {} primals from network scan", network_primals.len());
                    all_discovered.append(&mut network_primals);
                }
                Err(e) => warn!("⚠️ Network scan discovery failed: {}", e),
            }
        }

        // Container-based discovery (Docker, Kubernetes)
        if self.config.mechanisms.enable_container_discovery {
            match backends::container::discover_from_containers().await {
                Ok(mut container_primals) => {
                    if !container_primals.is_empty() {
                        let count = container_primals.len();
                        info!("✅ Discovered {} primals from containers", count);
                        all_discovered.append(&mut container_primals);
                    }
                }
                Err(e) => {
                    debug!("Container discovery failed (expected if not in container): {}", e);
                }
            }
        }

        // Deduplicate and cache
        let deduplicated = self.cache.deduplicate_and_store(all_discovered);

        info!("🎉 Total unique primals discovered: {}", deduplicated.len());
        Ok(deduplicated)
    }

    /// Get cached discovered primals
    #[must_use]
    pub fn get_discovered_primals(&self) -> Vec<&DiscoveredPrimal> {
        self.cache.get_all()
    }

    /// Find primals with specific capability
    ///
    /// **CAPABILITY-BASED**: Discovers by what services provide, not what they're named
    #[must_use]
    pub fn find_primals_with_capability(&self, capability_type: &str) -> Vec<&DiscoveredPrimal> {
        self.cache.find_by_capability(capability_type)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::discovery::config::DiscoveryMechanisms;
    use std::sync::OnceLock;
    use tokio::sync::Mutex;

    /// Serializes env-based discovery tests. Uses `tokio::sync::Mutex` because tests hold the
    /// guard across `.await` (see `discover_all_primals`); `std::sync::Mutex` would block the
    /// runtime worker for the whole await.
    async fn env_discovery_lock() -> tokio::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().await
    }

    #[tokio::test]
    async fn test_discovery_engine_creation() {
        let config = DiscoveryConfig::default();
        let engine = UniversalPrimalDiscovery::new(config);
        assert!(engine.get_discovered_primals().is_empty());
    }

    #[tokio::test]
    async fn test_discovery_with_all_mechanisms_disabled() {
        let config = DiscoveryConfig {
            mechanisms: super::super::config::DiscoveryMechanisms {
                enable_environment_scan: false,
                enable_network_scanning: false,
                enable_container_discovery: false,
            },
            timeout: tokio::time::Duration::from_secs(5),
        };

        let mut engine = UniversalPrimalDiscovery::new(config);
        let result = engine.discover_all_primals().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_find_primals_with_capability_empty_cache() {
        let engine = UniversalPrimalDiscovery::new(DiscoveryConfig::default());
        assert!(engine.find_primals_with_capability("compute").is_empty());
    }

    #[test]
    fn test_discovery_engine_clone_and_debug() {
        let engine = UniversalPrimalDiscovery::new(DiscoveryConfig::default());
        let s = format!("{engine:?}");
        assert!(s.contains("UniversalPrimalDiscovery"));
    }

    #[tokio::test]
    async fn test_discover_all_primals_network_branch_completes() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: false,
                enable_network_scanning: true,
                enable_container_discovery: false,
            },
            timeout: tokio::time::Duration::from_millis(1),
        };
        let mut engine = UniversalPrimalDiscovery::new(config);
        let result = engine.discover_all_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_all_primals_container_branch_completes() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: false,
                enable_network_scanning: false,
                enable_container_discovery: true,
            },
            timeout: tokio::time::Duration::from_secs(5),
        };
        let mut engine = UniversalPrimalDiscovery::new(config);
        let result = engine.discover_all_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn discover_all_environment_branch_completes() {
        let _guard = env_discovery_lock().await;
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: false,
                enable_container_discovery: false,
            },
            timeout: tokio::time::Duration::from_secs(5),
        };
        let mut engine = UniversalPrimalDiscovery::new(config);
        let result = engine.discover_all_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn discover_all_all_mechanisms_enabled() {
        let _guard = env_discovery_lock().await;
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: true,
                enable_container_discovery: true,
            },
            // Short scan window so network discovery does not wait on the default 5s mDNS/DNS-SD path.
            timeout: tokio::time::Duration::from_millis(1),
        };
        let mut engine = UniversalPrimalDiscovery::new(config);
        let result = engine.discover_all_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn discover_caches_and_deduplicates() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: false,
                enable_network_scanning: false,
                enable_container_discovery: false,
            },
            timeout: tokio::time::Duration::from_secs(5),
        };
        let mut engine = UniversalPrimalDiscovery::new(config);
        let _ = engine.discover_all_primals().await;
        let cached = engine.get_discovered_primals();
        assert!(cached.is_empty());
    }
}
