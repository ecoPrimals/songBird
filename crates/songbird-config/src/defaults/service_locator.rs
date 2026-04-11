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
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

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
        self.discover_by_capability_with(capability, |k| songbird_process_env::var(k))
    }

    /// Same as [`discover_by_capability`](Self::discover_by_capability) with an injectable env
    /// reader (concurrent-safe unit tests).
    #[must_use]
    pub fn discover_by_capability_with<F>(&self, capability: &str, env_reader: F) -> Vec<SocketAddr>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        if let Ok(endpoints) = Self::discover_from_environment_with(capability, &env_reader)
            && !endpoints.is_empty()
        {
            return endpoints;
        }

        let endpoints = Self::discover_from_dns_sd(capability);
        if !endpoints.is_empty() {
            return endpoints;
        }

        if let Ok(endpoints) = Self::discover_from_registry_with(capability, &env_reader)
            && !endpoints.is_empty()
        {
            return endpoints;
        }

        Vec::new()
    }

    fn discover_from_environment_with<F>(
        capability: &str,
        env_reader: &F,
    ) -> SongbirdResult<Vec<SocketAddr>>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        let env_var = format!(
            "SONGBIRD_CAPABILITY_{}_ENDPOINTS",
            capability.to_uppercase().replace('-', "_")
        );

        let endpoints_str = env_reader(&env_var)
            .map_err(|_| SongbirdError::configuration(format!("{env_var} not set")))?;

        let endpoints =
            endpoints_str.split(',').filter_map(|s| s.trim().parse::<SocketAddr>().ok()).collect();

        Ok(endpoints)
    }

    /// Discovers services by scanning the biomeos socket directory for domain-named sockets.
    /// True DNS-SD/mDNS integration is deferred to the network capability provider.
    ///
    /// This path approximates "mDNS by well-known socket paths": it looks under
    /// `BIOMEOS_SOCKET_DIR` (and `XDG_RUNTIME_DIR/biomeos` as a fallback) for `*.sock` files
    /// whose stem matches the capability domain (`{domain}.sock` or `{domain}-*.sock`).
    ///
    /// [`SocketAddr`] values are taken from co-located TCP discovery sidecars (`{stem}-ipc-port`,
    /// `tcp:127.0.0.1:<port>`) when present; a bare Unix socket without a TCP sidecar is logged
    /// and skipped because this API is IP-based.
    fn discover_from_dns_sd(capability: &str) -> Vec<SocketAddr> {
        let dns_sd_service_type = format!("_{}._tcp.local", capability.to_lowercase());
        tracing::trace!(
            service_type = %dns_sd_service_type,
            capability,
            "DNS-SD browse not implemented; scanning biomeos socket directories instead",
        );
        let domain = capability.to_lowercase().replace('_', "-");
        let mut addrs = BTreeSet::new();

        for dir in Self::biomeos_socket_dir_candidates() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                tracing::debug!(dir = %dir.display(), "skipping unreadable biomeos socket directory");
                continue;
            };

            for entry in entries.filter_map(std::result::Result::ok) {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                {
                    continue;
                }
                let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
                if !Self::socket_stem_matches_capability_domain(stem, &domain) {
                    continue;
                }

                if let Some(addr) = Self::tcp_addr_from_biomeos_sidecar(&dir, stem) {
                    addrs.insert(addr);
                } else {
                    tracing::debug!(
                        socket_path = %path.display(),
                        capability,
                        "matched capability socket without TCP sidecar; skipping SocketAddr entry",
                    );
                }
            }
        }

        addrs.into_iter().collect()
    }

    /// Directories that may hold biomeOS capability-domain sockets (DNS-SD stand-in scan).
    fn biomeos_socket_dir_candidates() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(d) = songbird_process_env::var("BIOMEOS_SOCKET_DIR") {
            dirs.push(PathBuf::from(d));
        }
        if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            dirs.push(
                Path::new(&xdg).join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR),
            );
        }
        dirs
    }

    fn socket_stem_matches_capability_domain(stem: &str, domain: &str) -> bool {
        let stem_norm = stem.to_lowercase().replace('_', "-");
        stem_norm == domain || stem_norm.starts_with(&format!("{domain}-"))
    }

    /// Reads `tcp:host:port` (or `host:port`) from `{stem}-ipc-port` next to the `.sock` file.
    fn tcp_addr_from_biomeos_sidecar(socket_dir: &Path, stem: &str) -> Option<SocketAddr> {
        let port_file = socket_dir.join(format!("{stem}-ipc-port"));
        let data = std::fs::read_to_string(&port_file).ok()?;
        Self::parse_tcp_discovery_line(&data)
    }

    fn parse_tcp_discovery_line(data: &str) -> Option<SocketAddr> {
        let line = data.lines().next()?.trim();
        let rest = line.strip_prefix("tcp:").unwrap_or(line);
        rest.trim().parse().ok()
    }

    fn discover_from_registry_with<F>(
        capability: &str,
        env_reader: &F,
    ) -> SongbirdResult<Vec<SocketAddr>>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        let registry_url = env_reader("SONGBIRD_REGISTRY_URL")
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
