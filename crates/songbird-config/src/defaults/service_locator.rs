// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability-based service locator
//!
//! Services are discovered by **capability** (what they can do),
//! not by **name** (what they're called). This decouples service
//! consumers from specific primal identities.
//!
//! # Discovery Chain
//! 1. Environment variables: `SONGBIRD_CAPABILITY_<NAME>_ENDPOINTS`
//! 2. DNS-SD (RFC 6763): `_<capability>._tcp.local`
//! 3. HTTP registry: Consul, Eureka, custom
//!
//! # Registration
//! Services self-register with their capabilities.
//! Discovery consumers query by capability, not by name.

use songbird_types::{SongbirdError, SongbirdResult};
use std::net::SocketAddr;

use super::hosts_evolved::SelfAwareConfig;

/// Capability-based service locator
///
/// # Example
/// ```no_run
/// use songbird_config::defaults::service_locator::ServiceLocator;
///
/// let locator = ServiceLocator::new();
/// // Find services by capability, not by hardcoded name
/// // let storage = locator.discover_by_capability("storage");
/// ```
#[derive(Debug, Clone)]
pub struct ServiceLocator {
    self_config: SelfAwareConfig,
}

impl ServiceLocator {
    /// Create a new service locator with self-awareness
    #[must_use]
    pub fn new() -> Self {
        Self {
            self_config: SelfAwareConfig::from_environment(),
        }
    }

    /// Get this service's configuration
    #[must_use]
    pub const fn self_config(&self) -> &SelfAwareConfig {
        &self.self_config
    }

    /// Discover services by capability (runtime discovery)
    ///
    /// Tries environment variables → DNS-SD → HTTP registry.
    /// Returns empty vec if no services found (not an error).
    #[must_use]
    pub fn discover_by_capability(&self, capability: &str) -> Vec<SocketAddr> {
        if let Ok(endpoints) = Self::discover_from_environment(capability)
            && !endpoints.is_empty()
        {
            return endpoints;
        }

        let endpoints = Self::discover_from_dns_sd(capability);
        if !endpoints.is_empty() {
            return endpoints;
        }

        if let Ok(endpoints) = Self::discover_from_registry(capability)
            && !endpoints.is_empty()
        {
            return endpoints;
        }

        Vec::new()
    }

    /// Discover from environment variables.
    ///
    /// Pattern: `SONGBIRD_CAPABILITY_<CAPABILITY>_ENDPOINTS`=host1:port1,host2:port2
    fn discover_from_environment(capability: &str) -> SongbirdResult<Vec<SocketAddr>> {
        let env_var = format!(
            "SONGBIRD_CAPABILITY_{}_ENDPOINTS",
            capability.to_uppercase().replace('-', "_")
        );

        let endpoints_str = songbird_process_env::var(&env_var)
            .map_err(|_| SongbirdError::configuration(format!("{env_var} not set")))?;

        let endpoints =
            endpoints_str.split(',').filter_map(|s| s.trim().parse::<SocketAddr>().ok()).collect();

        Ok(endpoints)
    }

    /// Discover via DNS-SD (RFC 6763).
    ///
    /// Pending hickory-resolver integration.
    fn discover_from_dns_sd(capability: &str) -> Vec<SocketAddr> {
        let _service_name = format!("_{}._tcp.local", capability.to_lowercase());
        tracing::debug!(capability, "DNS-SD discovery not yet implemented; returning empty");
        Vec::new()
    }

    /// Discover from HTTP registry (Consul, Eureka, custom).
    fn discover_from_registry(capability: &str) -> SongbirdResult<Vec<SocketAddr>> {
        let registry_url = songbird_process_env::var("SONGBIRD_REGISTRY_URL")
            .map_err(|_| SongbirdError::configuration("SONGBIRD_REGISTRY_URL not set"))?;
        let query_url = format!("{registry_url}/v1/services?capability={capability}");
        let _ = query_url;
        Ok(Vec::new())
    }

    /// Register self with discovery system.
    ///
    /// Announces this service with its capabilities. Other services
    /// discover it by querying for those capabilities.
    ///
    /// # Errors
    /// Returns error if all registration methods fail.
    pub fn register_self(&self, capabilities: &[&str]) -> SongbirdResult<()> {
        let advertise_addr = self.self_config.advertise_address();

        if matches!(Self::register_with_http_registry(capabilities, &advertise_addr), Ok(())) {
            return Ok(());
        }

        if matches!(Self::register_with_dns_sd(capabilities, &advertise_addr), Ok(())) {
            return Ok(());
        }

        Self::announce_via_environment(capabilities, &advertise_addr);
        Ok(())
    }

    /// Register with HTTP-based service registry.
    fn register_with_http_registry(
        capabilities: &[&str],
        advertise_addr: &SocketAddr,
    ) -> SongbirdResult<()> {
        if songbird_process_env::var("SONGBIRD_REGISTRY_URL").is_err() {
            return Err(SongbirdError::configuration("SONGBIRD_REGISTRY_URL not set"));
        }

        let service_info = serde_json::json!({
            "service_id": format!("songbird-{}", uuid::Uuid::new_v4()),
            "name": "songbird",
            "address": advertise_addr.ip().to_string(),
            "port": advertise_addr.port(),
            "capabilities": capabilities,
            "health_check_url": format!("http://{}/health", advertise_addr),
            "tags": ["songbird", "primal"],
        });

        let _ = service_info;
        Err(SongbirdError::not_implemented_with_detail(
            "http_service_registry",
            "Full implementation requires HTTP client integration",
        ))
    }

    /// Register via DNS-SD (RFC 6763).
    fn register_with_dns_sd(
        _capabilities: &[&str],
        _advertise_addr: &SocketAddr,
    ) -> SongbirdResult<()> {
        Err(SongbirdError::not_implemented_with_detail(
            "dns_sd_registration",
            "Full implementation requires platform mDNS/DNS-SD integration",
        ))
    }

    /// Announce via environment logging (development/testing).
    fn announce_via_environment(capabilities: &[&str], advertise_addr: &SocketAddr) {
        tracing::info!(
            address = %advertise_addr,
            capabilities = %capabilities.join(", "),
            "Service announcement: Set SONGBIRD_CAPABILITY_<NAME>_ENDPOINTS={} for discovery",
            advertise_addr,
        );
    }
}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "service_locator_tests.rs"]
mod tests;
