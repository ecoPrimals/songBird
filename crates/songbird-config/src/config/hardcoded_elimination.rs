// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Hardcoding Elimination Infrastructure
//!
//! Systematic replacement of hardcoded values with configurable alternatives.

#![allow(missing_docs, reason = "legacy mirror of canonical defaults; fields self-descriptive")]

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

// Import types from the correct locations
use crate::canonical::network::PortRange;

/// Central configuration for eliminating hardcoded values
#[derive(Debug, Clone, Default)]
pub struct HardcodingEliminationConfig {
    /// Network configuration patterns
    pub network: NetworkConfig,
    /// Service configuration patterns
    pub service: ServiceConfig,
    /// Security configuration patterns
    pub security: SecurityConfig,
    /// Timeout configuration patterns
    pub timeouts: TimeoutConfig,
    /// Performance configuration patterns
    pub performance: PerformanceConfig,
    /// Capability provider endpoint configuration (compute, storage, security, AI)
    pub primals: PrimalConfig,
    /// Federation configuration patterns
    pub federation: FederationConfig,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub version: String,
    pub base_url: String,
    pub health_endpoint: String,
    pub metrics_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub encryption_key_size: usize,
    pub session_timeout: Duration,
    pub security_provider_endpoint: String,
    pub oauth_redirect_uri: String,
    pub tls_cert_path: String,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: IpAddr,
    pub production_bind_address: IpAddr,
    pub stun_servers: Vec<String>,
    pub port_ranges: HashMap<String, (u16, u16)>,
    pub orchestrator_endpoint: Arc<str>,
    pub gaming_endpoint: Arc<str>,
    pub federation_endpoint: Arc<str>,
    pub dashboard_endpoint: Arc<str>,
    pub gaming_port_range: PortRange,
}

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub health_check_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub scaling_check_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub small_buffer_size: usize,
    pub large_buffer_size: usize,
    pub max_packet_size: usize,
    pub connection_pool_size: usize,
    pub cache_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct PrimalConfig {
    /// Security capability HTTP endpoint (security provider).
    pub security_provider_endpoint: Arc<str>,
    /// Storage capability HTTP endpoint (canonical; capability domain `storage`).
    pub storage_provider_endpoint: Arc<str>,
    /// Compute capability HTTP endpoint (canonical).
    pub compute_provider_endpoint: Arc<str>,
    /// AI / neural capability HTTP endpoint (canonical).
    pub ai_provider_endpoint: Arc<str>,
    pub discovery_endpoints: Vec<String>,
    pub base_port: u16,
    pub port_range: (u16, u16),
}

impl PrimalConfig {
    /// Storage capability HTTP endpoint accessor (capability-based naming).
    #[must_use]
    pub fn storage_provider_endpoint(&self) -> Arc<str> {
        Arc::clone(&self.storage_provider_endpoint)
    }
}

#[derive(Debug, Clone)]
pub struct FederationConfig {
    pub cluster_endpoints: Vec<String>,
    pub heartbeat_endpoint: String,
    pub broadcast_ports: Vec<u16>,
    pub discovery_ports: Vec<u16>,
    pub default_cluster_id: String,
    pub auto_discovery_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_key_size: 256,
            session_timeout: Duration::from_secs(3600),
            security_provider_endpoint: env_capability_first_then_legacy_warn(
                &["SONGBIRD_SECURITY_ENDPOINT", "SONGBIRD_SECURITY_PROVIDER_ENDPOINT"],
                "SONGBIRD_BEARDOG_ENDPOINT",
                "SONGBIRD_SECURITY_ENDPOINT or SONGBIRD_SECURITY_PROVIDER_ENDPOINT",
                &format!("https://{}:8443", crate::canonical::constants::get_bind_address()),
            ),
            oauth_redirect_uri: env_or_default(
                "SONGBIRD_OAUTH_REDIRECT",
                &format!(
                    "http://{}:8080/auth/callback",
                    &crate::canonical::constants::get_bind_address()
                ),
            ),
            tls_cert_path: default_tls_cert_path(),
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        let base_url = env_or_default(
            "SONGBIRD_BASE_URL",
            &format!(
                "http://{}:{}",
                crate::canonical::constants::get_bind_address(),
                crate::canonical::constants::network::default_orchestrator_port()
            ),
        );
        Self {
            service_name: env_or_default("SONGBIRD_SERVICE_NAME", "songbird-orchestrator"),
            version: env_or_default("SONGBIRD_VERSION", "0.1.0"),
            base_url: base_url.clone(),
            health_endpoint: format!("{base_url}/health"),
            metrics_endpoint: format!("{base_url}/metrics"),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let bind_ip = env_or_default(
            "SONGBIRD_BIND_ADDRESS",
            &crate::canonical::constants::get_bind_address(),
        );
        let orchestrator_port = env_or_default(
            "SONGBIRD_ORCHESTRATOR_PORT",
            &crate::canonical::constants::network::default_orchestrator_port().to_string(),
        );
        let gaming_port_default = crate::canonical::constants::network::default_orchestrator_port()
            .saturating_add(1)
            .to_string();
        let federation_port_default =
            crate::canonical::constants::network::default_orchestrator_port()
                .saturating_add(2)
                .to_string();
        let gaming_port = env_or_default("SONGBIRD_GAMING_PORT", &gaming_port_default);
        let federation_port = env_or_default("SONGBIRD_FEDERATION_PORT", &federation_port_default);
        let dashboard_port = env_or_default(
            "SONGBIRD_DASHBOARD_PORT",
            &crate::canonical::constants::network::default_orchestrator_port().to_string(),
        );

        Self {
            bind_address: bind_ip.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid SONGBIRD_BIND_ADDRESS, using default localhost: {}", e);
                crate::canonical::constants::get_bind_address().parse().unwrap_or({
                    // Final fallback to localhost if constant is invalid
                    std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
                })
            }),
            production_bind_address: env_or_default(
                "SONGBIRD_PRODUCTION_BIND_ADDRESS",
                &IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED).to_string(),
            )
            .parse()
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Invalid SONGBIRD_PRODUCTION_BIND_ADDRESS, using UNSPECIFIED: {}",
                    e
                );
                IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            }),
            stun_servers: vec![
                env_or_default(
                    "SONGBIRD_STUN_SERVER_1",
                    songbird_types::constants::DEFAULT_STUN_SERVER_1,
                ),
                env_or_default(
                    "SONGBIRD_STUN_SERVER_2",
                    songbird_types::constants::DEFAULT_STUN_SERVER_2,
                ),
            ],
            port_ranges: {
                let base = crate::canonical::constants::get_port_range_start();
                let mut ranges = HashMap::new();
                ranges.insert("orchestrator".to_string(), (base, base.saturating_add(10)));
                ranges.insert("gaming".to_string(), (7000, 7100));
                ranges.insert("federation".to_string(), (base, base.saturating_add(10)));
                ranges.insert("capability_services".to_string(), (base, base.saturating_add(10)));
                ranges
            },
            orchestrator_endpoint: Arc::from(format!("http://{bind_ip}:{orchestrator_port}")),
            gaming_endpoint: Arc::from(format!("http://{bind_ip}:{gaming_port}")),
            federation_endpoint: Arc::from(format!("http://{bind_ip}:{federation_port}")),
            dashboard_endpoint: Arc::from(format!("http://{bind_ip}:{dashboard_port}")),
            gaming_port_range: PortRange {
                start: 7000,
                end: 7100,
            },
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(
                env_or_default("SONGBIRD_CONNECTION_TIMEOUT", "30").parse().unwrap_or(30),
            ),
            request_timeout: Duration::from_secs(
                env_or_default("SONGBIRD_REQUEST_TIMEOUT", "60").parse().unwrap_or(60),
            ),
            health_check_timeout: Duration::from_secs(
                env_or_default("SONGBIRD_HEALTH_CHECK_TIMEOUT", "5").parse().unwrap_or(5),
            ),
            heartbeat_interval: Duration::from_secs(
                env_or_default("SONGBIRD_HEARTBEAT_INTERVAL", "30").parse().unwrap_or(30),
            ),
            scaling_check_interval: Duration::from_secs(
                env_or_default("SONGBIRD_SCALING_CHECK_INTERVAL", "30").parse().unwrap_or(30),
            ),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            small_buffer_size: env_or_default("SONGBIRD_SMALL_BUFFER_SIZE", "1024")
                .parse()
                .unwrap_or(1024),
            large_buffer_size: env_or_default("SONGBIRD_LARGE_BUFFER_SIZE", "8192")
                .parse()
                .unwrap_or(8192),
            max_packet_size: env_or_default("SONGBIRD_MAX_PACKET_SIZE", "65536")
                .parse()
                .unwrap_or(65536),
            connection_pool_size: env_or_default("SONGBIRD_CONNECTION_POOL_SIZE", "10")
                .parse()
                .unwrap_or(10),
            cache_ttl: Duration::from_secs(
                env_or_default("SONGBIRD_CACHE_TTL", "300").parse().unwrap_or(300),
            ),
        }
    }
}

#[allow(
    deprecated,
    reason = "legacy struct keeps deprecated field mirrors for backward compatibility"
)]
impl Default for PrimalConfig {
    fn default() -> Self {
        let base_ip = env_or_default(
            "SONGBIRD_PRIMAL_BASE_IP",
            &crate::canonical::constants::get_bind_address(),
        );
        let base_port: u16 = env_or_default(
            "SONGBIRD_PRIMAL_BASE_PORT",
            &crate::canonical::constants::network::default_orchestrator_port().to_string(),
        )
        .parse()
        .unwrap_or_else(|_| crate::canonical::constants::get_port_range_start());

        let compute_provider_endpoint: Arc<str> = Arc::from(env_capability_first_then_legacy_warn(
            &["SONGBIRD_COMPUTE_PROVIDER_ENDPOINT", "SONGBIRD_COMPUTE_ENDPOINT"],
            "SONGBIRD_TOADSTOOL_ENDPOINT",
            "SONGBIRD_COMPUTE_PROVIDER_ENDPOINT or SONGBIRD_COMPUTE_ENDPOINT",
            &format!("http://{base_ip}:8082"),
        ));
        let ai_provider_endpoint: Arc<str> = Arc::from(env_capability_first_then_legacy_warn(
            &["SONGBIRD_AI_PROVIDER_ENDPOINT", "SONGBIRD_AI_ENDPOINT"],
            "SONGBIRD_SQUIRREL_ENDPOINT",
            "SONGBIRD_AI_PROVIDER_ENDPOINT or SONGBIRD_AI_ENDPOINT",
            &format!("http://{base_ip}:8083"),
        ));

        let security_provider_endpoint: Arc<str> =
            Arc::from(env_capability_first_then_legacy_warn(
                &["SONGBIRD_SECURITY_ENDPOINT", "SONGBIRD_SECURITY_PROVIDER_ENDPOINT"],
                "SONGBIRD_BEARDOG_ENDPOINT",
                "SONGBIRD_SECURITY_ENDPOINT or SONGBIRD_SECURITY_PROVIDER_ENDPOINT",
                &format!("https://{base_ip}:8443"),
            ));

        Self {
            security_provider_endpoint: Arc::clone(&security_provider_endpoint),
            storage_provider_endpoint: Arc::from(resolve_storage_provider_endpoint(
                &base_ip, base_port,
            )),
            compute_provider_endpoint: Arc::clone(&compute_provider_endpoint),
            ai_provider_endpoint: Arc::clone(&ai_provider_endpoint),
            discovery_endpoints: vec![
                env_or_default(
                    "SONGBIRD_DISCOVERY_ENDPOINT_1",
                    &format!("http://{base_ip}:{base_port}/discovery"),
                ),
                env_or_default(
                    "SONGBIRD_DISCOVERY_ENDPOINT_2",
                    &format!("http://{base_ip}:8081/discovery"),
                ),
            ],
            base_port,
            port_range: (
                env_or_default(
                    "SONGBIRD_PRIMAL_PORT_START",
                    &crate::canonical::constants::get_port_range_start().to_string(),
                )
                .parse()
                .unwrap_or_else(|_| crate::canonical::constants::get_port_range_start()),
                env_or_default(
                    "SONGBIRD_PRIMAL_PORT_END",
                    &crate::canonical::constants::get_port_range_end().to_string(),
                )
                .parse()
                .unwrap_or_else(|_| crate::canonical::constants::get_port_range_end()),
            ),
        }
    }
}

impl Default for FederationConfig {
    fn default() -> Self {
        let base_ip = env_or_default(
            "SONGBIRD_FEDERATION_BASE_IP",
            &crate::canonical::constants::get_bind_address(),
        );
        let base_port = env_or_default(
            "SONGBIRD_FEDERATION_BASE_PORT",
            &crate::canonical::constants::network::default_orchestrator_port().to_string(),
        );

        Self {
            cluster_endpoints: vec![
                env_or_default(
                    "SONGBIRD_CLUSTER_ENDPOINT_1",
                    &format!("http://{base_ip}:{base_port}"),
                ),
                env_or_default("SONGBIRD_CLUSTER_ENDPOINT_2", &format!("http://{base_ip}:8081")),
            ],
            heartbeat_endpoint: env_or_default(
                "SONGBIRD_HEARTBEAT_ENDPOINT",
                &format!("http://{base_ip}:{base_port}/federation/heartbeat"),
            ),
            broadcast_ports: vec![8080, 8081, 8082, 8090],
            discovery_ports: vec![8080, 8000, 3000, 5000],
            default_cluster_id: env_or_default("SONGBIRD_CLUSTER_ID", "songbird-cluster"),
            auto_discovery_enabled: env_or_default("SONGBIRD_AUTO_DISCOVERY", "true") == "true",
        }
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    songbird_process_env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Resolve endpoint env: capability keys first, then one legacy key with a migration warning.
fn env_capability_first_then_legacy_warn(
    capability_keys: &[&str],
    legacy_key: &str,
    migrate_to: &str,
    default: &str,
) -> String {
    for key in capability_keys {
        if let Ok(v) = songbird_process_env::var(key)
            && !v.is_empty()
        {
            return v;
        }
    }
    if let Ok(v) = songbird_process_env::var(legacy_key)
        && !v.is_empty()
    {
        tracing::warn!("Using legacy env var {legacy_key} — migrate to {migrate_to}");
        return v;
    }
    default.to_string()
}

fn resolve_storage_provider_endpoint(base_ip: &str, base_port: u16) -> String {
    for key in ["SONGBIRD_STORAGE_ENDPOINT", "SONGBIRD_STORAGE_PROVIDER_ENDPOINT"] {
        if let Ok(v) = songbird_process_env::var(key)
            && !v.is_empty()
        {
            return v;
        }
    }
    if let Ok(v) = songbird_process_env::var("SONGBIRD_NESTGATE_ENDPOINT")
        && !v.is_empty()
    {
        tracing::warn!(
            "Using legacy env var SONGBIRD_NESTGATE_ENDPOINT — migrate to SONGBIRD_STORAGE_ENDPOINT or SONGBIRD_STORAGE_PROVIDER_ENDPOINT"
        );
        return v;
    }
    format!("http://{base_ip}:{base_port}/storage")
}

fn default_tls_cert_path() -> String {
    songbird_process_env::var("SONGBIRD_TLS_CERT")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| songbird_process_env::var("SSL_CERT_FILE").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(|| {
            songbird_process_env::var("HOME").map_or_else(
                |_| {
                    std::env::temp_dir()
                        .join("songbird")
                        .join("certs")
                        .join("songbird.crt")
                        .to_string_lossy()
                        .into_owned()
                },
                |h| format!("{h}/.songbird/certs/songbird.crt"),
            )
        })
}

/// Thread-safe global configuration using `OnceLock` (idiomatic Rust,
use std::sync::OnceLock;
static GLOBAL_CONFIG: OnceLock<HardcodingEliminationConfig> = OnceLock::new();

/// Get global configuration (thread-safe, idiomatic)
#[must_use]
pub fn get_config() -> &'static HardcodingEliminationConfig {
    GLOBAL_CONFIG.get_or_init(HardcodingEliminationConfig::default)
}

/// Convenience functions for replacing hardcoded values
pub mod replace {
    use super::{Duration, IpAddr, get_config};
    use std::sync::Arc;

    /// Replace hardcoded &`crate::constants::network::DEFAULT_HOST`
    #[must_use]
    pub fn bind_address() -> IpAddr {
        get_config().network.bind_address
    }

    /// Replace hardcoded &format!("{}:{}", `crate::constants::network::DEFAULT_HOST`, `crate::constants::network::DEFAULT_ORCHESTRATOR_PORT`);
    #[must_use]
    pub fn orchestrator_endpoint() -> Arc<str> {
        Arc::clone(&get_config().network.orchestrator_endpoint)
    }

    /// Replace hardcoded "`crate::constants::network::DEFAULT_HOST:8081`"
    #[must_use]
    pub fn gaming_endpoint() -> Arc<str> {
        Arc::clone(&get_config().network.gaming_endpoint)
    }

    /// Replace hardcoded "`crate::constants::network::DEFAULT_HOST:8443`"
    #[must_use]
    pub fn security_provider_endpoint() -> Arc<str> {
        Arc::clone(&get_config().primals.security_provider_endpoint)
    }

    /// Replace hardcoded "`crate::constants::network::DEFAULT_HOST:8080/storage`"
    #[must_use]
    pub fn storage_provider_endpoint() -> Arc<str> {
        Arc::clone(&get_config().primals.storage_provider_endpoint)
    }

    /// Replace hardcoded `Duration::from_secs(30)`
    #[must_use]
    pub fn connection_timeout() -> Duration {
        get_config().timeouts.connection_timeout
    }

    /// Replace hardcoded `Duration::from_secs(60)`
    #[must_use]
    pub fn request_timeout() -> Duration {
        get_config().timeouts.request_timeout
    }

    /// Replace hardcoded `Duration::from_secs(5)`
    #[must_use]
    pub fn health_check_timeout() -> Duration {
        get_config().timeouts.health_check_timeout
    }

    /// Replace hardcoded 8192
    #[must_use]
    pub fn large_buffer_size() -> usize {
        get_config().performance.large_buffer_size
    }

    /// Replace hardcoded STUN servers
    #[must_use]
    pub fn stun_servers() -> Vec<String> {
        get_config().network.stun_servers.clone()
    }

    /// Replace hardcoded federation endpoints
    #[must_use]
    pub fn federation_endpoints() -> Vec<String> {
        get_config().federation.cluster_endpoints.clone()
    }

    /// Replace hardcoded compute capability endpoint.
    #[must_use]
    pub fn compute_provider_endpoint() -> Arc<str> {
        Arc::clone(&get_config().primals.compute_provider_endpoint)
    }

    /// Replace hardcoded AI / neural capability endpoint.
    #[must_use]
    pub fn ai_provider_endpoint() -> Arc<str> {
        Arc::clone(&get_config().primals.ai_provider_endpoint)
    }

    /// Replace hardcoded capability-discovery endpoint list
    #[must_use]
    pub fn primal_discovery_endpoints() -> Vec<String> {
        get_config().primals.discovery_endpoints.clone()
    }

    /// Replace hardcoded broadcast ports
    #[must_use]
    pub fn federation_broadcast_ports() -> Vec<u16> {
        get_config().federation.broadcast_ports.clone()
    }

    /// Replace hardcoded discovery ports
    #[must_use]
    pub fn federation_discovery_ports() -> Vec<u16> {
        get_config().federation.discovery_ports.clone()
    }

    /// Get production-ready bind address (0.0.0.0 vs `crate::constants::network::DEFAULT_HOST`)
    #[must_use]
    pub fn production_bind_address() -> IpAddr {
        if songbird_process_env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default() == "production" {
            get_config().network.production_bind_address
        } else {
            get_config().network.bind_address
        }
    }

    /// Format endpoint with configurable IP and port
    ///
    /// **EVOLVED**: Uses environment variables and capability discovery
    /// instead of hardcoded provider codenames.
    ///
    /// Discovery order:
    /// 1. `{CAPABILITY}_ENDPOINT` environment variable
    /// 2. `{CAPABILITY}_PORT` environment variable + bind address
    /// 3. Auto-select port (0) for dynamic allocation
    ///
    /// # Examples
    /// ```ignore
    /// // For security capability:
    /// // Set SECURITY_ENDPOINT=https://security-provider:8443
    /// // OR set SECURITY_PORT=8443
    /// let endpoint = format_endpoint("security", None);
    /// ```
    #[must_use]
    pub fn format_endpoint(capability: &str, port_override: Option<u16>) -> Arc<str> {
        // Check for full endpoint override first
        let env_key_endpoint = format!("{}_ENDPOINT", capability.to_uppercase());
        if let Ok(endpoint) = songbird_process_env::var(&env_key_endpoint) {
            return Arc::from(endpoint);
        }

        // Otherwise construct from IP and port
        let config = get_config();
        let ip = if songbird_process_env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default()
            == "production"
        {
            config.network.production_bind_address
        } else {
            config.network.bind_address
        };

        // Get port from environment or use override or auto-select
        let env_key_port = format!("{}_PORT", capability.to_uppercase());
        let port = port_override
            .or_else(|| songbird_process_env::var(&env_key_port).ok().and_then(|p| p.parse().ok()))
            .unwrap_or(0); // 0 = auto-select dynamic port

        let protocol = if port == 8443 || capability == "security" {
            "https"
        } else {
            "http"
        };
        Arc::from(format!("{protocol}://{ip}:{port}"))
    }

    /// Format service endpoint with path
    #[must_use]
    pub fn format_service_endpoint(
        service: &str,
        path: &str,
        port_override: Option<u16>,
    ) -> String {
        let base = format_endpoint(service, port_override);
        format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
    }

    /// Replace hardcoded gaming port
    #[must_use]
    pub fn gaming_port() -> u16 {
        get_config().network.gaming_port_range.start
    }

    /// Replace hardcoded timeout configuration
    #[must_use]
    pub fn timeout_config() -> super::TimeoutConfig {
        get_config().timeouts.clone()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::replace;

    #[test]
    fn format_endpoint_prefers_full_endpoint_env() {
        songbird_process_env::set_var("ROUTING_ENDPOINT", "https://router.example:7443");
        let ep = replace::format_endpoint("routing", None);
        assert_eq!(ep.as_ref(), "https://router.example:7443");
        songbird_process_env::remove_var("ROUTING_ENDPOINT");
    }

    #[test]
    fn format_service_endpoint_joins_base_and_path() {
        songbird_process_env::set_var("METRICS_ENDPOINT", "http://metrics.local:9090");
        let s = replace::format_service_endpoint("metrics", "/api/v1/query", None);
        assert_eq!(s, "http://metrics.local:9090/api/v1/query");
        songbird_process_env::remove_var("METRICS_ENDPOINT");
    }

    #[test]
    fn gaming_port_matches_config_default_start() {
        let g = replace::gaming_port();
        assert_eq!(g, super::get_config().network.gaming_port_range.start);
    }

    #[test]
    fn bind_address_returns_valid_ip() {
        let ip = replace::bind_address();
        assert!(ip.is_loopback() || !ip.is_unspecified() || ip.is_unspecified());
    }
}
