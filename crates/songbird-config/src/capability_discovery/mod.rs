// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-Based Service Discovery
//!
//! Modern replacement for hardcoded primal endpoints. Each primal discovers others
//! through runtime capability-based discovery, respecting sovereignty principles.
//!
//! ## Sovereignty Principles
//!
//! 1. **Self-Knowledge Only**: Each primal knows only about itself
//! 2. **Runtime Discovery**: All inter-primal communication discovered at runtime
//! 3. **No Hardcoding**: Zero compile-time dependencies on other primals
//! 4. **Capability-Based**: Route by what you need, not who provides it

#![allow(missing_docs, reason = "discovery client structs mirror `songbird-discovery` traits")]

use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

mod discover_impl;
mod types;

pub use types::{DiscoveryMethod, ServiceEndpoint};

type CapEnvReader = Arc<dyn Fn(&str) -> Result<String, std::env::VarError> + Send + Sync>;

/// Capability-based service discovery engine
pub struct CapabilityDiscovery {
    /// Discovered services cache
    services: Arc<RwLock<HashMap<String, Vec<ServiceEndpoint>>>>,

    /// Enabled discovery methods
    methods: Vec<DiscoveryMethod>,

    /// Optional env reader (tests inject; default reads process environment)
    env_reader: Option<CapEnvReader>,
}

impl CapabilityDiscovery {
    /// Create new discovery engine with default methods
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            methods: vec![
                DiscoveryMethod::Environment,
                DiscoveryMethod::DnsSD,
                DiscoveryMethod::MDNS,
            ],
            env_reader: None,
        }
    }

    /// Create with specific discovery methods
    #[must_use]
    pub fn with_methods(methods: Vec<DiscoveryMethod>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            methods,
            env_reader: None,
        }
    }

    /// Same as [`with_methods`](Self::with_methods) with an injectable env reader (concurrent-safe tests).
    #[must_use]
    pub fn with_methods_env_reader<F>(methods: Vec<DiscoveryMethod>, env_reader: F) -> Self
    where
        F: Fn(&str) -> Result<String, std::env::VarError> + Send + Sync + 'static,
    {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            methods,
            env_reader: Some(Arc::new(env_reader)),
        }
    }

    fn read_env(&self, key: &str) -> Result<String, std::env::VarError> {
        match &self.env_reader {
            Some(f) => f(key),
            None => std::env::var(key),
        }
    }

    /// Discover services providing a specific capability
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use songbird_config::capability_discovery::CapabilityDiscovery;
    ///
    /// #[tokio::main]
    /// async fn main() -> songbird_types::SongbirdResult<()> {
    ///     let discovery = CapabilityDiscovery::new();
    ///     
    ///     // Discover ANY provider offering "compute" capability
    ///     // Could be ToadStool, or ANY other compute provider
    ///     let compute_providers = discovery
    ///         .find_providers_by_capability("compute")
    ///         .await?;
    ///     
    ///     for provider in compute_providers {
    ///         println!("Found compute provider: {} at {}", provider.id, provider.url);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Discovery methods fail to find providers
    /// - Network errors occur during discovery
    /// - Cache operations fail
    pub async fn find_providers_by_capability(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        // Check cache first
        {
            let services = self.services.read().await;
            if let Some(cached) = services.get(capability)
                && !cached.is_empty()
            {
                return Ok(cached.clone());
            }
        }

        // Discover through enabled methods
        let mut discovered = Vec::new();

        for method in &self.methods {
            match self.discover_via_method(capability, method).await {
                Ok(mut endpoints) => {
                    debug!(
                        "Discovered {} providers for '{}' via {:?}",
                        endpoints.len(),
                        capability,
                        method
                    );
                    discovered.append(&mut endpoints);
                }
                Err(e) => {
                    debug!("Discovery failed for '{}' via {:?}: {}", capability, method, e);
                }
            }
        }

        if discovered.is_empty() {
            return Err(SongbirdError::Discovery {
                message: format!(
                    "No providers found for capability '{}'. Enable discovery or set {}_ENDPOINT environment variable.",
                    capability,
                    capability.to_uppercase()
                ),
                backend: Some("all_methods".to_string()),
                retry_strategy: Some(
                    "Set environment variable or enable discovery methods".to_string(),
                ),
            });
        }

        // Cache results
        {
            let mut services = self.services.write().await;
            services.insert(capability.to_string(), discovered.clone());
        }

        info!("✅ Discovered {} providers for capability '{}'", discovered.len(), capability);

        Ok(discovered)
    }

    /// Discover via specific method
    async fn discover_via_method(
        &self,
        capability: &str,
        method: &DiscoveryMethod,
    ) -> SongbirdResult<Vec<ServiceEndpoint>> {
        match method {
            DiscoveryMethod::Environment => self.discover_via_environment(capability).await,
            DiscoveryMethod::DnsSD => self.discover_via_dnssd(capability).await,
            DiscoveryMethod::MDNS => self.discover_via_mdns(capability).await,
            DiscoveryMethod::Registry {
                endpoint,
            } => self.discover_via_registry(capability, endpoint).await,
            DiscoveryMethod::ConfigFile {
                path,
            } => self.discover_via_config_file(capability, path).await,
        }
    }

    /// Clear cache for a specific capability
    pub async fn clear_cache(&self, capability: &str) {
        let mut services = self.services.write().await;
        services.remove(capability);
    }

    /// Clear all cached discoveries
    pub async fn clear_all_caches(&self) {
        let mut services = self.services.write().await;
        services.clear();
    }
}

impl Default for CapabilityDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common capabilities
impl CapabilityDiscovery {
    /// Discover compute providers (replaces hardcoded `ToadStool` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no compute providers are found via any discovery method.
    pub async fn discover_compute(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("compute").await
    }

    /// Discover storage providers (replaces hardcoded `NestGate` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no storage providers are found via any discovery method.
    pub async fn discover_storage(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("storage").await
    }

    /// Discover security providers (replaces hardcoded `BearDog` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no security providers are found via any discovery method.
    pub async fn discover_security(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("security").await
    }

    /// Discover AI providers (replaces hardcoded `Squirrel` endpoint)
    ///
    /// # Errors
    ///
    /// Returns an error if no AI providers are found via any discovery method.
    pub async fn discover_ai(&self) -> SongbirdResult<Vec<ServiceEndpoint>> {
        self.find_providers_by_capability("ai").await
    }
}

#[cfg(test)]
#[path = "capability_discovery_tests.rs"]
mod tests;
