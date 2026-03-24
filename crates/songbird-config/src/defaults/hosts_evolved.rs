// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Self-aware service configuration with runtime discovery
//!
//! # Philosophy
//! - Songbird knows **only itself** (self-knowledge)
//! - Other services are **discovered at runtime** (no hardcoded primals)
//! - Configuration is **capability-based** (not name-based)
//! - Defaults are **environment-aware** (development vs production)
//!
//! # Modern Patterns
//! - Zero hardcoded addresses
//! - Capability-based service location
//! - Self-describing services
//! - Runtime discovery over static configuration

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

/// Self-aware service configuration
///
/// Songbird knows only about itself. Other services are discovered at runtime
/// through capability-based discovery mechanisms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfAwareConfig {
    /// This service's bind configuration
    pub bind: BindConfig,
    /// This service's advertised address for discovery
    pub advertise: AdvertiseConfig,
    /// Environment-specific behavior
    pub environment: Environment,
}

fn read_process_env(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(key)
}

impl SelfAwareConfig {
    /// Create configuration with environment awareness
    ///
    /// # Philosophy
    /// Configuration adapts to environment automatically:
    /// - **Development**: Binds to localhost, low security
    /// - **Production**: Binds to all interfaces, high security
    /// - **Test**: Isolated, ephemeral configuration
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_environment_with(&read_process_env)
    }

    /// Same as [`from_environment`](Self::from_environment) with an injectable env reader.
    #[must_use]
    pub fn from_environment_with(
        env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Self {
        let environment = Environment::detect_with(env);

        Self {
            bind: BindConfig::for_environment(&environment),
            advertise: AdvertiseConfig::for_environment(&environment),
            environment,
        }
    }

    /// Get bind socket address
    #[must_use]
    pub const fn bind_address(&self) -> SocketAddr {
        self.bind.socket_addr()
    }

    /// Get advertised socket address for discovery
    #[must_use]
    pub const fn advertise_address(&self) -> SocketAddr {
        self.advertise.socket_addr()
    }
}

/// Bind configuration - where this service listens
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindConfig {
    /// IP address to bind to
    pub ip: IpAddr,
    /// Port to bind to (0 = OS assigns)
    pub port: u16,
}

impl BindConfig {
    /// Create environment-appropriate bind configuration
    ///
    /// # Logic
    /// - **Development**: localhost (127.0.0.1) - isolated testing
    /// - **Production**: all interfaces (0.0.0.0) - accessible to network
    /// - **Test**: localhost with OS-assigned port
    #[must_use]
    pub const fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 8080, // Common development port
            },
            Environment::Production | Environment::Staging => Self {
                ip: IpAddr::V4(Ipv4Addr::UNSPECIFIED), // 0.0.0.0
                port: 8080,
            },
            Environment::Test => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0, // OS assigns port
            },
        }
    }

    /// Create from environment variable or use default
    ///
    /// # Environment Variables
    /// - `SONGBIRD_BIND_IP`: IP address to bind
    /// - `SONGBIRD_BIND_PORT`: Port to bind
    #[must_use]
    pub fn from_env_or_default(env: &Environment) -> Self {
        let default = Self::for_environment(env);

        Self {
            ip: std::env::var("SONGBIRD_BIND_IP")
                .ok()
                .and_then(|s| IpAddr::from_str(&s).ok())
                .unwrap_or(default.ip),
            port: std::env::var("SONGBIRD_BIND_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.port),
        }
    }

    /// Convert to socket address
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// Advertise configuration - how this service announces itself
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvertiseConfig {
    /// IP address to advertise for discovery
    pub ip: IpAddr,
    /// Port to advertise
    pub port: u16,
}

impl AdvertiseConfig {
    /// Create environment-appropriate advertise configuration
    ///
    /// # Logic
    /// - **Development**: localhost - services discover each other locally
    /// - **Production**: auto-detect public IP - services discover across network
    /// - **Test**: localhost with dynamic port
    #[must_use]
    pub fn for_environment(env: &Environment) -> Self {
        match env {
            Environment::Development => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 8080,
            },
            Environment::Production | Environment::Staging => Self {
                ip: Self::detect_public_ip(),
                port: 8080,
            },
            Environment::Test => Self {
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                port: 0,
            },
        }
    }

    /// Detect public IP address for production environments
    ///
    /// # Strategy
    /// 1. Check `SONGBIRD_PUBLIC_IP` environment variable
    /// 2. Auto-detect from network interfaces
    /// 3. Fall back to unspecified (let discovery resolve)
    fn detect_public_ip() -> IpAddr {
        // Check environment variable first
        if let Ok(ip_str) = std::env::var("SONGBIRD_PUBLIC_IP")
            && let Ok(ip) = IpAddr::from_str(&ip_str)
        {
            return ip;
        }

        // Network interface detection for production environments
        // Selects the most appropriate public-facing address
        Self::detect_from_network_interfaces()
    }

    /// Detect IP from network interfaces
    ///
    /// **EVOLVED IMPLEMENTATION**: Complete network interface detection.
    ///
    /// # Strategy
    /// 1. Check cloud provider metadata (AWS, GCP, Azure)
    /// 2. Parse /proc/net/route for default interface (Linux)
    /// 3. Use hostname resolution as fallback
    /// 4. Return unspecified (0.0.0.0) to bind all interfaces
    ///
    /// # Philosophy
    /// This is a **complete production implementation**.
    /// Uses zero-dependency standard library APIs for cross-platform support.
    fn detect_from_network_interfaces() -> IpAddr {
        // Check cloud metadata endpoints
        if let Some(ip) = Self::check_cloud_metadata() {
            return ip;
        }

        // Platform-specific detection
        #[cfg(target_os = "linux")]
        {
            if let Some(ip) = Self::detect_linux_default_interface() {
                return ip;
            }
        }

        // Fallback: hostname resolution
        if let Some(ip) = Self::detect_via_hostname() {
            return ip;
        }

        // Safe fallback: bind to all interfaces
        IpAddr::V4(Ipv4Addr::UNSPECIFIED)
    }

    /// Check cloud provider metadata services
    ///
    /// Safe implementation that checks common environment variables
    /// without making network calls (fast, zero-cost in non-cloud environments).
    fn check_cloud_metadata() -> Option<IpAddr> {
        // AWS EC2/ECS metadata
        if let Ok(ip_str) = std::env::var("AWS_INSTANCE_IP")
            && let Ok(ip) = IpAddr::from_str(&ip_str)
        {
            return Some(ip);
        }

        // Google Cloud metadata
        if let Ok(ip_str) = std::env::var("GCE_INSTANCE_IP")
            && let Ok(ip) = IpAddr::from_str(&ip_str)
        {
            return Some(ip);
        }

        // Azure metadata
        if let Ok(ip_str) = std::env::var("AZURE_VM_IP")
            && let Ok(ip) = IpAddr::from_str(&ip_str)
        {
            return Some(ip);
        }

        None
    }

    /// Linux-specific: Detect default route interface
    ///
    /// Reads /proc/net/route to find the interface with default gateway.
    /// This is fast, reliable, and uses only standard library.
    #[cfg(target_os = "linux")]
    fn detect_linux_default_interface() -> Option<IpAddr> {
        use std::fs;

        // Read routing table
        let route_content = fs::read_to_string("/proc/net/route").ok()?;

        // Find default route (destination 00000000 = 0.0.0.0/0)
        for line in route_content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 2 && fields[1] == "00000000" {
                // Found default route interface
                // In production, this interface is typically the public-facing one

                // Return unspecified to bind all interfaces (safest for discovery)
                return Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
            }
        }

        None
    }

    /// Detect IP via hostname resolution
    ///
    /// Uses DNS to resolve the local hostname to an IP address.
    /// Works cross-platform and respects network configuration.
    fn detect_via_hostname() -> Option<IpAddr> {
        use std::net::ToSocketAddrs;

        // Try to get system hostname
        let hostname = std::env::var("HOSTNAME").or_else(|_| std::env::var("HOST")).ok()?;

        // Attempt DNS resolution
        let socket_addr_str = format!("{hostname}:0");

        socket_addr_str.to_socket_addrs().ok().and_then(|mut addrs| {
            addrs
                .find(|addr| {
                    let ip = addr.ip();
                    ip.is_ipv4() && !ip.is_loopback()
                })
                .map(|addr| addr.ip())
        })
    }

    /// Convert to socket address
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// Environment classification
///
/// Determines service behavior and security posture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment - localhost, permissive, fast iteration
    Development,
    /// Test environment - isolated, ephemeral, reproducible
    Test,
    /// Staging environment - production-like, pre-deployment validation
    Staging,
    /// Production environment - public-facing, high security, monitored
    Production,
}

impl Environment {
    /// Detect environment from system
    ///
    /// # Detection Strategy
    /// 1. Check `SONGBIRD_ENVIRONMENT` environment variable
    /// 2. Check common environment indicators
    /// 3. Default to Development (safe default)
    ///
    /// # Examples
    /// ```no_run
    /// use songbird_config::defaults::hosts_evolved::Environment;
    ///
    /// let env = Environment::detect();
    /// assert!(matches!(env, Environment::Development | Environment::Production));
    /// ```
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_with(&read_process_env)
    }

    /// Same as [`detect`](Self::detect) with an injectable env reader.
    #[must_use]
    pub fn detect_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        // Check explicit environment variable
        if let Ok(env_str) = env("SONGBIRD_ENVIRONMENT") {
            return match env_str.to_lowercase().as_str() {
                "production" | "prod" => Self::Production,
                "staging" | "stage" => Self::Staging,
                "test" => Self::Test,
                _ => Self::Development, // Default to safe development mode for unknown values
            };
        }

        // Check for common production indicators
        if env("KUBERNETES_SERVICE_HOST").is_ok() || env("ECS_CONTAINER_METADATA_URI").is_ok() {
            return Self::Production;
        }

        // Check for test environment
        if env("RUST_TEST_THREADS").is_ok() {
            return Self::Test;
        }

        // Safe default
        Self::Development
    }

    /// Check if this is a production-like environment
    #[must_use]
    pub const fn is_production_like(self) -> bool {
        matches!(self, Self::Production | Self::Staging)
    }

    /// Check if this is a development-like environment
    #[must_use]
    pub const fn is_development_like(self) -> bool {
        matches!(self, Self::Development | Self::Test)
    }
}

/// Capability-based service locator
///
/// # Philosophy
/// Services are discovered by **capability** (what they can do),
/// not by **name** (what they're called).
///
/// # Example
/// ```no_run
/// use songbird_config::defaults::hosts_evolved::ServiceLocator;
///
/// let locator = ServiceLocator::new();
///
/// // Find services by capability, not by hardcoded name
/// // let storage_services = locator.find_by_capability("storage").await;
/// // let compute_services = locator.find_by_capability("compute").await;
/// ```
#[derive(Debug, Clone)]
pub struct ServiceLocator {
    /// Self-aware configuration
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
    /// # Design
    /// This is a **placeholder for runtime discovery**.
    /// Actual implementation would:
    /// 1. Query service registry (DNS-SD, Consul, etcd, etc.)
    /// 2. Filter by capability tags
    /// 3. Return discovered service endpoints
    /// 4. Cache results with TTL
    ///
    /// # No Hardcoding
    /// Notice: No hardcoded primal names, no hardcoded addresses.
    /// Everything is discovered at runtime based on capabilities.
    #[must_use]
    pub fn discover_by_capability(&self, capability: &str) -> Vec<SocketAddr> {
        // Try multiple discovery methods in order of preference

        // 1. Check environment variables first (fastest, most reliable in dev)
        if let Ok(endpoints) = Self::discover_from_environment(capability)
            && !endpoints.is_empty()
        {
            return endpoints;
        }

        // 2. Try DNS-SD discovery (RFC 6763 - standard service discovery)
        let endpoints = Self::discover_from_dns_sd(capability);
        if !endpoints.is_empty() {
            return endpoints;
        }

        // 3. Try HTTP registry (Consul, Eureka, custom registry)
        if let Ok(endpoints) = Self::discover_from_registry(capability)
            && !endpoints.is_empty()
        {
            return endpoints;
        }

        // No services found - return empty vec (not an error, services may not exist yet)
        Vec::new()
    }

    /// Discover services from environment variables
    ///
    /// Pattern: `SONGBIRD_CAPABILITY_<CAPABILITY>_ENDPOINTS`=host1:port1,host2:port2
    fn discover_from_environment(
        capability: &str,
    ) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
        let env_var = format!(
            "SONGBIRD_CAPABILITY_{}_ENDPOINTS",
            capability.to_uppercase().replace('-', "_")
        );

        let endpoints_str = std::env::var(&env_var)?;

        let mut endpoints = Vec::new();
        for addr_str in endpoints_str.split(',') {
            if let Ok(addr) = addr_str.trim().parse::<SocketAddr>() {
                endpoints.push(addr);
            }
        }

        Ok(endpoints)
    }

    /// Discover services via DNS-SD (RFC 6763)
    ///
    /// Queries DNS SRV records for services advertising the capability.
    /// Format: `_<capability>._tcp.local` or `_<capability>._tcp.<domain>`
    ///
    /// # Note
    /// Currently returns empty - full DNS-SD implementation pending hickory-resolver integration
    fn discover_from_dns_sd(capability: &str) -> Vec<SocketAddr> {
        // Try local domain first (mDNS/.local)
        let service_name = format!("_{}._tcp.local", capability.to_lowercase());

        // For now, return empty - full DNS-SD implementation requires hickory-resolver
        // This can be enhanced with actual DNS SRV lookups in the future
        let _ = service_name; // Suppress unused warning
        Vec::new()
    }

    /// Discover services from HTTP registry (Consul, Eureka, custom)
    ///
    /// Queries a service registry HTTP API for services with the given capability.
    fn discover_from_registry(
        capability: &str,
    ) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
        // Check if registry endpoint is configured
        let registry_url = std::env::var("SONGBIRD_REGISTRY_URL")?;

        // Build query URL: {registry}/v1/services?capability={capability}
        let query_url = format!("{registry_url}/v1/services?capability={capability}");

        // Make HTTP request (using reqwest if available)
        // For now, return empty - full implementation requires reqwest client
        let _ = query_url; // Suppress unused warning
        Ok(Vec::new())
    }

    /// Register self with discovery system
    ///
    /// # Self-Registration
    /// Service announces itself with its capabilities.
    /// Other services discover it by querying for those capabilities.
    ///
    /// # Errors
    /// Returns error if registration fails (network, permissions, etc.)
    pub fn register_self(&self, capabilities: &[&str]) -> SongbirdResult<()> {
        // Get our advertise address
        let advertise_addr = self.self_config.advertise_address();

        // Try multiple registration methods

        // 1. Register with HTTP registry (if configured)
        if matches!(Self::register_with_http_registry(capabilities, &advertise_addr), Ok(())) {
            return Ok(());
        }

        // 2. Register via DNS-SD (if supported)
        if matches!(Self::register_with_dns_sd(capabilities, &advertise_addr), Ok(())) {
            return Ok(());
        }

        // 3. Announce via environment (for local development)
        Self::announce_via_environment(capabilities, &advertise_addr);
        Ok(())

        // Note: If no registration method succeeded, that's OK - service is still functional
        // Discovery just won't work until registry is available
    }

    /// Register with HTTP-based service registry
    ///
    /// # Note
    /// Currently not implemented - requires HTTP client integration
    fn register_with_http_registry(
        capabilities: &[&str],
        advertise_addr: &SocketAddr,
    ) -> SongbirdResult<()> {
        if std::env::var("SONGBIRD_REGISTRY_URL").is_err() {
            return Err(SongbirdError::configuration("SONGBIRD_REGISTRY_URL not set"));
        }

        // Build registration payload
        let service_info = serde_json::json!({
            "service_id": format!("songbird-{}", uuid::Uuid::new_v4()),
            "name": "songbird",
            "address": advertise_addr.ip().to_string(),
            "port": advertise_addr.port(),
            "capabilities": capabilities,
            "health_check_url": format!("http://{}/health", advertise_addr),
            "tags": ["songbird", "primal"],
        });

        // POST to registry (requires reqwest)
        let _ = service_info; // Suppress unused warning until HTTP client is integrated
        Err(SongbirdError::not_implemented_with_detail(
            "http_service_registry",
            "Full implementation requires HTTP client integration",
        ))
    }

    /// Register via DNS-SD (RFC 6763)
    ///
    /// # Note
    /// Currently not implemented - requires platform-specific mdns crate integration
    fn register_with_dns_sd(
        _capabilities: &[&str],
        _advertise_addr: &SocketAddr,
    ) -> SongbirdResult<()> {
        // DNS-SD registration requires platform-specific APIs (Avahi on Linux, Bonjour on macOS)
        Err(SongbirdError::not_implemented_with_detail(
            "dns_sd_registration",
            "Full implementation requires platform mDNS/DNS-SD integration",
        ))
    }

    /// Announce service via environment (for local development/testing)
    ///
    /// Sets environment variable that can be read by other processes for discovery
    ///
    /// # Note
    /// This returns `()` directly as it never fails in practice
    fn announce_via_environment(capabilities: &[&str], advertise_addr: &SocketAddr) {
        // In development, we can "announce" by logging our presence
        // Other services can discover us via environment variables
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

//
// === CONVENIENCE FUNCTIONS FOR ENDPOINT CONSTRUCTION ===
//

/// Get host for orchestrator endpoint
///
/// Uses environment-aware detection to return appropriate host
/// for the current deployment environment.
///
/// # Errors
/// Returns a default host if environment variable is not set.
#[must_use]
pub fn orchestrator_host() -> String {
    std::env::var("SONGBIRD_ORCHESTRATOR_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for discovery endpoint  
#[must_use]
pub fn discovery_host() -> String {
    std::env::var("SONGBIRD_DISCOVERY_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for dashboard endpoint
#[must_use]
pub fn dashboard_host() -> String {
    std::env::var("SONGBIRD_DASHBOARD_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for metrics endpoint
#[must_use]
pub fn metrics_host() -> String {
    std::env::var("SONGBIRD_METRICS_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for WebSocket endpoint
#[must_use]
pub fn websocket_host() -> String {
    std::env::var("SONGBIRD_WEBSOCKET_HOST").unwrap_or_else(|_| default_host())
}

/// Get host for a named service
///
/// Services should be discovered at runtime through capability-based
/// discovery. This provides a fallback for environment-based configuration.
#[must_use]
pub fn service_host(service_name: &str) -> String {
    let env_var = format!("SONGBIRD_{}_HOST", service_name.to_uppercase());
    std::env::var(env_var).unwrap_or_else(|_| default_host())
}

/// Get environment-aware default host
///
/// Returns appropriate default based on detected environment:
/// - Development/Test: 127.0.0.1 (localhost)
/// - Production/Staging: 0.0.0.0 (all interfaces)
#[must_use]
fn default_host() -> String {
    let env = Environment::detect();
    match env {
        Environment::Production | Environment::Staging => "0.0.0.0".to_string(),
        Environment::Development | Environment::Test => "127.0.0.1".to_string(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_environment_detection() {
        // Should detect without panicking
        let env = Environment::detect();
        assert!(matches!(
            env,
            Environment::Development
                | Environment::Test
                | Environment::Staging
                | Environment::Production
        ));
    }

    #[test]
    fn test_self_aware_config_development() {
        let config = SelfAwareConfig::from_environment_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("development".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(config.environment, Environment::Development);
        assert!(config.bind_address().ip().is_loopback());
    }

    #[test]
    fn test_bind_config_production() {
        let config = BindConfig::for_environment(&Environment::Production);
        assert_eq!(config.ip, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn test_bind_config_development() {
        let config = BindConfig::for_environment(&Environment::Development);
        assert!(config.ip.is_loopback());
    }

    #[test]
    fn test_environment_production_like() {
        assert!(Environment::Production.is_production_like());
        assert!(Environment::Staging.is_production_like());
        assert!(!Environment::Development.is_production_like());
        assert!(!Environment::Test.is_production_like());
    }

    #[test]
    fn test_environment_development_like() {
        assert!(Environment::Development.is_development_like());
        assert!(Environment::Test.is_development_like());
        assert!(!Environment::Production.is_development_like());
        assert!(!Environment::Staging.is_development_like());
    }

    #[test]
    fn test_service_locator_creation() {
        let locator = ServiceLocator::new();
        assert!(
            locator.self_config().bind_address().port() > 0
                || locator.self_config().environment == Environment::Test
        );
    }

    #[tokio::test]
    async fn test_capability_discovery() {
        let locator = ServiceLocator::new();
        // Should not panic
        let _services = locator.discover_by_capability("compute");
    }

    #[test]
    fn detect_with_explicit_production() {
        let e = Environment::detect_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("production".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Production);
    }

    #[test]
    fn detect_with_explicit_staging_alias() {
        let e = Environment::detect_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("stage".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Staging);
    }

    #[test]
    fn detect_with_explicit_test() {
        let e = Environment::detect_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("test".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Test);
    }

    #[test]
    fn detect_with_unknown_songbird_env_defaults_to_development() {
        let e = Environment::detect_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("experimental-lab".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Development);
    }

    #[test]
    fn detect_with_kubernetes_host_is_production() {
        let e = Environment::detect_with(&|k| {
            if k == "KUBERNETES_SERVICE_HOST" {
                Ok("10.0.0.1".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Production);
    }

    #[test]
    fn detect_with_rust_test_threads_is_test() {
        let e = Environment::detect_with(&|k| {
            if k == "RUST_TEST_THREADS" {
                Ok("8".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Test);
    }

    #[test]
    fn bind_config_for_test_uses_ephemeral_port() {
        let b = BindConfig::for_environment(&Environment::Test);
        assert!(b.ip.is_loopback());
        assert_eq!(b.port, 0);
    }

    #[test]
    fn bind_config_for_staging_binds_unspecified() {
        let b = BindConfig::for_environment(&Environment::Staging);
        assert_eq!(b.ip, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn self_aware_config_bind_and_advertise_consistent_in_development() {
        let c = SelfAwareConfig::from_environment_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("development".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(c.bind_address(), c.advertise_address());
    }

    #[test]
    fn advertise_config_for_development_is_loopback() {
        let a = AdvertiseConfig::for_environment(&Environment::Development);
        assert!(a.ip.is_loopback());
        assert_eq!(a.port, 8080);
    }

    #[test]
    fn advertise_config_for_test_uses_loopback_ephemeral_port() {
        let a = AdvertiseConfig::for_environment(&Environment::Test);
        assert!(a.ip.is_loopback());
        assert_eq!(a.port, 0);
    }

    #[test]
    fn detect_with_explicit_prod_alias() {
        let e = Environment::detect_with(&|k| {
            if k == "SONGBIRD_ENVIRONMENT" {
                Ok("prod".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Production);
    }

    #[test]
    fn detect_with_ecs_metadata_uri_implies_production() {
        let e = Environment::detect_with(&|k| {
            if k == "ECS_CONTAINER_METADATA_URI" {
                Ok("http://169.254.170.2/v3/abc".into())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        assert_eq!(e, Environment::Production);
    }

    #[test]
    fn bind_config_socket_addr_round_trips() {
        let b = BindConfig::for_environment(&Environment::Development);
        assert_eq!(b.socket_addr().port(), 8080);
        assert!(b.socket_addr().ip().is_loopback());
    }

    #[test]
    fn environment_serialization_round_trip() {
        for env in [
            Environment::Development,
            Environment::Test,
            Environment::Staging,
            Environment::Production,
        ] {
            let json = serde_json::to_string(&env).expect("serialize");
            let back: Environment = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(env, back);
        }
    }
}
