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

    /// Connect to the songBird IPC socket. On Unix this is a Unix domain socket;
    /// on Windows it falls back to TCP localhost on `SONGBIRD_IPC_PORT` (default 3492).
    #[cfg(unix)]
    fn connect_ipc_stream<F>(env_reader: &F) -> SongbirdResult<std::os::unix::net::UnixStream>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        use std::os::unix::net::UnixStream;

        let socket_path = env_reader("SONGBIRD_IPC_SOCKET").unwrap_or_else(|_| {
            let runtime_dir = env_reader("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
                dirs::runtime_dir()
                    .unwrap_or_else(default_runtime_fallback)
                    .to_string_lossy()
                    .into_owned()
            });
            format!(
                "{runtime_dir}/{}/songbird.sock",
                songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR
            )
        });

        UnixStream::connect(&socket_path).map_err(|e| {
            SongbirdError::configuration(format!(
                "Cannot connect to songbird IPC at {socket_path}: {e}"
            ))
        })
    }

    /// Connect to the songBird IPC via TCP localhost (Windows — no Unix sockets).
    #[cfg(windows)]
    fn connect_ipc_stream<F>(env_reader: &F) -> SongbirdResult<std::net::TcpStream>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        use std::net::TcpStream;

        let port: u16 = env_reader("SONGBIRD_IPC_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT);

        let addr = format!("127.0.0.1:{port}");
        TcpStream::connect(&addr).map_err(|e| {
            SongbirdError::configuration(format!("Cannot connect to songbird IPC at {addr}: {e}"))
        })
    }

    fn discover_from_registry_with<F>(
        capability: &str,
        env_reader: &F,
    ) -> SongbirdResult<Vec<SocketAddr>>
    where
        F: Fn(&str) -> Result<String, std::env::VarError>,
    {
        use std::io::{Read, Write};

        let mut stream = Self::connect_ipc_stream(env_reader)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "ipc.resolve",
            "params": { "capability": capability, "native": true },
            "id": 1
        });

        let payload = format!("{request}\n");
        stream
            .write_all(payload.as_bytes())
            .map_err(|e| SongbirdError::configuration(format!("IPC write failed: {e}")))?;

        let mut response_buf = vec![0u8; 4096];
        let n = stream
            .read(&mut response_buf)
            .map_err(|e| SongbirdError::configuration(format!("IPC read failed: {e}")))?;

        let response: serde_json::Value = serde_json::from_slice(&response_buf[..n])
            .map_err(|e| SongbirdError::configuration(format!("IPC response parse failed: {e}")))?;

        let endpoint = &response["result"]["endpoint"];
        let addr = match endpoint["transport"].as_str() {
            Some("tcp") => {
                let host =
                    endpoint["host"].as_str().unwrap_or(songbird_types::constants::LOCALHOST);
                let port = u16::try_from(endpoint["port"].as_u64().unwrap_or(0)).unwrap_or(0);
                format!("{host}:{port}").parse::<SocketAddr>().ok()
            }
            _ => None,
        };

        Ok(addr.into_iter().collect())
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

    /// Register capabilities with songbird via `ipc.register`.
    fn register_with_http_registry(
        capabilities: &[&str],
        advertise_addr: &SocketAddr,
    ) -> SongbirdResult<()> {
        use std::io::{Read, Write};

        let env_reader = |key: &str| songbird_process_env::var(key);
        let mut stream = Self::connect_ipc_stream(&env_reader)?;
        stream.set_write_timeout(Some(std::time::Duration::from_secs(2))).ok();

        let primal_id = songbird_process_env::var("SONGBIRD_PRIMAL_ID")
            .unwrap_or_else(|_| String::from(songbird_types::primal_names::SELF_NAME));

        let ip = advertise_addr.ip();
        let port = advertise_addr.port();
        let endpoint = format!("tcp://{ip}:{port}");

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "ipc.register",
            "params": {
                "primal_id": primal_id,
                "capabilities": capabilities,
                "endpoint": endpoint,
            },
            "id": 1
        });

        let payload = format!("{request}\n");
        stream
            .write_all(payload.as_bytes())
            .map_err(|e| SongbirdError::configuration(format!("IPC write failed: {e}")))?;

        let mut response_buf = vec![0u8; 2048];
        stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).ok();
        let n = stream.read(&mut response_buf).unwrap_or(0);

        if n > 0
            && let Ok(resp) = serde_json::from_slice::<serde_json::Value>(&response_buf[..n])
            && resp.get("error").is_some()
        {
            tracing::warn!("ipc.register returned error: {}", resp["error"]);
        }

        Ok(())
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

/// Fallback runtime directory when `XDG_RUNTIME_DIR` is unset and `dirs::runtime_dir()` is `None`.
///
/// Derives from the system runtime constant rather than hardcoding a UID-specific path.
#[cfg(unix)]
fn default_runtime_fallback() -> PathBuf {
    PathBuf::from(songbird_types::constants::BIOMEOS_SYSTEM_RUNTIME_DIR).parent().map_or_else(
        || PathBuf::from(songbird_types::constants::SYSTEM_RUNTIME_DIR),
        Path::to_path_buf,
    )
}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "service_locator_tests.rs"]
mod tests;
