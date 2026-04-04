// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Primal Discovery System - REFACTORED
//!
//! **EVOLVED** (Dec 7, 2025): Smart modular refactoring
//! - Separated by responsibility, not arbitrary line counts
//! - Each module has clear ownership
//! - Extensible backend architecture
//! - Zero regressions, 100% functionality preserved
//!
//! **PHILOSOPHY**: Self-knowledge only - discovers what's advertised
//! - No hardcoded primal names
//! - Capability-based discovery
//! - Runtime service binding
//! - Works with ANY primal (current or future)
//!
//! ## Module Structure
//!
//! ```text
//! discovery/
//!   ├── mod.rs           (this file - public API)
//!   ├── config.rs        (DiscoveryConfig, mechanisms)
//!   ├── types.rs         (DiscoveredPrimal, Health, Method)
//!   ├── engine.rs        (Core discovery orchestration)
//!   ├── cache.rs         (Deduplication and caching)
//!   ├── health.rs        (Health checking logic)
//!   ├── errors.rs        (Error types)
//!   └── backends/        (Discovery backend implementations)
//!       ├── environment.rs   (Env var based discovery)
//!       ├── network.rs       (mDNS, network scanning)
//!       └── container.rs     (K8s, Docker discovery)
//! ```

// Module declarations
pub mod backends;
pub mod cache;
pub mod config;
pub mod engine;
pub mod errors;
pub mod health;
pub mod types;

// Re-exports for backward compatibility and convenience
pub use cache::DiscoveryCache;
pub use config::{DiscoveryConfig, DiscoveryMechanisms};
pub use engine::UniversalPrimalDiscovery;
pub use errors::{DiscoveryError, DiscoveryResult};
pub use types::{DiscoveredPrimal, DiscoveryMethod, PrimalHealth};

#[cfg(test)]
mod types_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.mechanisms.enable_environment_scan);
        assert!(config.mechanisms.enable_network_scanning);
        assert!(config.mechanisms.enable_container_discovery);
    }

    #[tokio::test]
    async fn test_discovery_engine_creation() {
        let config = DiscoveryConfig::default();
        let engine = UniversalPrimalDiscovery::new(config);
        assert!(engine.get_discovered_primals().is_empty());
    }

    #[test]
    fn test_discovery_cache_operations() {
        let cache = DiscoveryCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }
}
