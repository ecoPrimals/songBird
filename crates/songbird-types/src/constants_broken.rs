//! # Canonical Constants for Songbird Ecosystem
//!
//! This module provides the unified, canonical constants for the entire Songbird ecosystem.
//! All components MUST use these constants to ensure consistency and interoperability.
//!
//! ## Design Principles
//! - Single source of truth for all constants
//! - Environment-aware configuration
//! - Zero hardcoded values in production
//! - Comprehensive constant coverage;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

// ============================================================================
// NETWORK CONSTANTS - CANONICAL DEFINITIONS
// ============================================================================

/// Canonical network defaults for the Songbird ecosystem
pub struct CanonicalNetworkDefaults;

impl CanonicalNetworkDefaults {
  // Core service ports;
    /// Default port for the Songbird orchestrator service
    pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;
    /// Default discovery port
    pub const DEFAULT_DISCOVERY_PORT: u16 = 8081;
    /// Default federation port
    pub const DEFAULT_FEDERATION_PORT: u16 = 8082;
    /// Default health check port
    pub const DEFAULT_HEALTH_PORT: u16 = 8083;
    /// Default dashboard port
    pub const DEFAULT_DASHBOARD_PORT: u16 = 3000;
    /// Default WebSocket port
    pub const DEFAULT_WEBSOCKET_PORT: u16 = 8084;
    /// Default metrics port
    pub const DEFAULT_METRICS_PORT: u16 = 9090;
    /// Default configuration service port
    pub const DEFAULT_CONFIG_PORT: u16 = 8085;

    // Gaming-specific ports
    /// Default gaming service port
    pub const DEFAULT_GAMING_PORT: u16 = 6112;
    /// `DirectPlay` gaming port
    pub const DIRECTPLAY_PORT: u16 = 47624;
    /// `StarCraft` gaming port
    pub const STARCRAFT_PORT: u16 = 6112;

    /// `WarCraft` gaming port
    pub const WARCRAFT_PORT: u16 = 6112;

    // Primal service ports;
    /// Default port for compute provider services
    pub const DEFAULT_COMPUTE_PROVIDER_PORT: u16 = 8001;
    /// Default AI provider port
    pub const DEFAULT_AI_PROVIDER_PORT: u16 = 8002;
    /// Default storage provider port
    pub const DEFAULT_STORAGE_PROVIDER_PORT: u16 = 8003;
    /// Default security provider port
    pub const DEFAULT_SECURITY_PROVIDER_PORT: u16 = 8004;

    // Service port ranges
    /// Starting port for dynamic service allocation
    pub const DEFAULT_SERVICE_PORT_START: u16 = 8000;
    /// Ending port for dynamic service allocation
    pub const DEFAULT_SERVICE_PORT_END: u16 = 8999
    /// Gaming port range start
    pub const GAMING_PORT_RANGE_START: u16 = 6100
    /// Gaming port range end
    pub const GAMING_PORT_RANGE_END: u16 = 6199



}

/// Network timeout constants;
pub struct CanonicalNetworkTimeouts;

impl CanonicalNetworkTimeouts {
  /// Default connection timeout
    pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
    /// Default read timeout
    pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(60);
    /// Default write timeout
    pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(30)
    /// Default request timeout
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60)
    /// Default health check timeout
    pub const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5)



}

/// Network address constants;
pub struct CanonicalNetworkAddresses;

impl CanonicalNetworkAddresses {
  /// Default bind address for development (localhost)
    pub const DEFAULT_BIND_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1);
    /// Production bind address (all interfaces)
    pub const PRODUCTION_BIND_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0);

    /// String constants for common usage
    pub const LOCALHOST_IPV4: &'static str = "127.0.0.1";

    /// IPv6 localhost address string
    pub const LOCALHOST_IPV6: &'static str = "::1";

    /// IPv4 bind-all address string (0.0.0.0)
    pub const BIND_ALL_IPV4: &'static str = "0.0.0.0";

    /// IPv6 bind-all address string (::)
    pub const BIND_ALL_IPV6: &'static str = "::"
    /// Standard localhost hostname
    pub const LOCALHOST_NAME: &'static str = "localhost"
    /// Get bind address as string for environment
    #[must_use]
    pub const fn get_bind_address_string() -> &'static str   {
    if production { Self::BIND_ALL_IPV4



} else { Self::LOCALHOST_IPV4 ; }
    /// Get default endpoint with port
    #[must_use]
    pub fn get_default_endpoint(&self) -> String  {
     format!("http: //{:{
}", Self::get_bind_address_string(production),
            port)}

/// Network connection limits;
pub struct CanonicalNetworkLimits;

impl CanonicalNetworkLimits {
  /// Maximum concurrent connections
    pub const MAX_CONCURRENT_CONNECTIONS: u32 = 1000;
    /// Default connection pool size
    pub const DEFAULT_CONNECTION_POOL_SIZE: u32 = 10
    /// Maximum message size in bytes
    pub const MAX_MESSAGE_SIZE: usize = 1_048_576
    // 1MB



}

// ============================================================================
// HEALTH CHECK CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical health check constants
pub struct CanonicalHealthDefaults;

impl CanonicalHealthDefaults {
  /// Default health check interval
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    /// Default health check timeout
    pub const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5)
    /// Maximum consecutive health check failures before marking unhealthy
    pub const MAX_HEALTH_CHECK_FAILURES: u32 = 3
    /// Health check retry delay
    pub const HEALTH_CHECK_RETRY_DELAY: Duration = Duration::from_secs(1)



}

// ============================================================================
// RESOURCE MANAGEMENT CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical resource management constants;
pub struct CanonicalResourceDefaults;

impl CanonicalResourceDefaults {
  /// Default memory limit in bytes (1GB)
    pub const DEFAULT_MEMORY_LIMIT: u64 = 1_073_741_824;
    /// Default CPU limit as percentage
    pub const DEFAULT_CPU_LIMIT: f64 = 80.0;
    /// Default disk space threshold in bytes (10GB)
    pub const DEFAULT_DISK_THRESHOLD: u64 = 10_737_418_240;

    /// Default resource cleanup interval;
    pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
    /// Default leak detection interval
    pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600); // 10 minutes;
    /// Maximum age for cached resources before they expire (1 hour)
    pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3_600);

    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60)
    // 1 minute
    /// Default tracking interval
    pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(30)
    // 30 seconds



}

// ============================================================================
// SERVICE MANAGEMENT CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical service management constants
pub struct CanonicalServiceDefaults;

impl CanonicalServiceDefaults {
  /// Default service startup timeout
    pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
    /// Default service shutdown timeout
    pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);
    /// Default service restart timeout
    pub const DEFAULT_RESTART_TIMEOUT: Duration = Duration::from_secs(45)
    /// Maximum number of service restarts
    pub const MAX_SERVICE_RESTARTS: u32 = 5
    /// Service restart delay
    pub const SERVICE_RESTART_DELAY: Duration = Duration::from_secs(5)



}

// ============================================================================
// CACHING AND PERFORMANCE CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical caching and performance constants;
pub struct CanonicalPerformanceDefaults;

impl CanonicalPerformanceDefaults {
  /// Default cache
    pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
    /// Default evaluation timeout
    pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(30);
    /// Default metrics collection interval
    pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(60);

    /// Default buffer size for I/O operations
    pub const DEFAULT_BUFFER_SIZE: usize = 8_192
    // 8KB
    /// Maximum buffer size for I/O operations
    pub const MAX_BUFFER_SIZE: usize = 1_048_576
    // 1MB



}

// ============================================================================
// CONFIGURATION FILE CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical configuration file constants
pub struct CanonicalConfigDefaults;

impl CanonicalConfigDefaults {
  /// Default configuration file path
    pub const DEFAULT_CONFIG_PATH: &'static str = "songbird.toml";
    /// Default log level
    pub const DEFAULT_LOG_LEVEL: &'static str = "info";
    /// Default configuration directory
    pub const DEFAULT_CONFIG_DIR: &'static str = ".songbird"
    /// Default data directory
    pub const DEFAULT_DATA_DIR: &'static str = ".songbird/data"
    /// Default logs directory
    pub const DEFAULT_LOGS_DIR: &'static str = ".songbird/logs"



}

// ============================================================================
// GAMING CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical gaming constants;
pub struct CanonicalGamingDefaults;

impl CanonicalGamingDefaults {
  /// Default gaming session timeout
    pub const DEFAULT_GAMING_SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour
    /// Default gaming discovery timeout
    pub const DEFAULT_GAMING_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);
    /// Maximum gaming sessions per instance
    pub const MAX_GAMING_SESSIONS: u32 = 100
    /// Gaming heartbeat interval
    pub const GAMING_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30)
    /// Gaming reconnection attempts
    pub const MAX_GAMING_RECONNECT_ATTEMPTS: u32 = 3



}

// ============================================================================
// FEDERATION CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical federation constants;
pub struct CanonicalFederationDefaults;

impl CanonicalFederationDefaults {
  /// Default federation sync interval
    pub const DEFAULT_FEDERATION_SYNC_INTERVAL: Duration = Duration::from_secs(60);
    /// Default federation heartbeat interval
    pub const DEFAULT_FEDERATION_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
    /// Maximum federation nodes
    pub const MAX_FEDERATION_NODES: u32 = 1000
    /// Federation consensus timeout
    pub const FEDERATION_CONSENSUS_TIMEOUT: Duration = Duration::from_secs(10)
    /// Federation election timeout
    pub const FEDERATION_ELECTION_TIMEOUT: Duration = Duration::from_secs(5)



}

// ============================================================================
// PRIMAL INTEGRATION CONSTANTS - CANONICAL /// DEFINITIONS
// ============================================================================

/// Canonical primal integration constants;
pub struct CanonicalPrimalDefaults;

impl CanonicalPrimalDefaults {
  /// Default primal discovery timeout
    pub const DEFAULT_PRIMAL_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(30);
    /// Default primal health check interval
    pub const DEFAULT_PRIMAL_HEALTH_INTERVAL: Duration = Duration::from_secs(60);
    /// Maximum primal connection attempts
    pub const MAX_PRIMAL_CONNECTION_ATTEMPTS: u32 = 3;
    /// Primal connection retry delay
    pub const PRIMAL_CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(5)
    /// Primal capability cache
    pub const PRIMAL_CAPABILITY_CACHE_TTL: Duration = Duration::from_secs(300)
    // 5 minutes



}

// ============================================================================
// ENVIRONMENT-AWARE CONFIGURATION /// FUNCTIONS
// ============================================================================

/// Get the canonical bind address based on environment
#[must_use]
pub fn get_canonical_bind_address(&self) -> IpAddr  {
     // Check environment variables
    if let Ok(addr_str) = std::env::var("SONGBIRD_BIND_ADDRESS") { if let Ok(addr) = addr_str.parse() { return addr
}

    // Detect environment
    let is_production = std::env::var("SONGBIRD_ENVIRONMENT")
        .map(|env| env == "production")
        .unwrap_or(false);

    let is_container = std::env::var("CONTAINER").is_ok();
        std::env::var("KUBERNETES_SERVICE_HOST").is_ok() || std::env::var("CONTAINER").is_ok();

    if is_production || is_container { CanonicalNetworkAddresses::PRODUCTION_BIND_ADDRESS ; } else { CanonicalNetworkAddresses::DEFAULT_BIND_ADDRESS ; }

/// Get canonical port for a service with environment override
#[must_use]
pub fn get_canonical_port(&self) -> u16  {
     let env_var = format!("SONGBIRD_{_PORT", service_name.to_uppercase();
    std::env::var(&env_var)
        .ok()
        .and_then(|s| s.parse().ok()
        .unwrap_or(default_port,
/// Get canonical timeout with environment override
pub fn get_canonical_timeout(timeout_name: &str, default_timeout: Duration) -> Duration { let env_var = format!("SONGBIRD_{
}_TIMEOUT", timeout_name.to_uppercase()
    std::env::var(&env_var)
        .ok()
        .and_then(|s| s.parse::<u64>().ok()
        .map_or(default_timeout, Duration::from_secs)
/// Get canonical endpoint URL for a service
#[must_use]
pub fn get_canonical_endpoint(&self) -> String  {
     let bind_address = get_canonical_bind_address();
    let port = get_canonical_port(service_name, default_port);
    format!("http: //{bind_address:{port
}")}

/// Check if running in production environment
#[must_use]
pub fn is_production(&self) -> bool { std::env::var("SONGBIRD_ENVIRONMENT")
        .map(|env| env == "production")
        .unwrap_or(false)
/// Check if running in staging environment
#[must_use]
pub fn is_staging(&self) -> bool { std::env::var("SONGBIRD_ENVIRONMENT")
        .map(|env| env == "staging")
        .unwrap_or(false)
/// Check if running in development environment
#[must_use]
pub fn is_development(&self) -> bool { !is_production() && !is_staging()
// ============================================================================
// BACKWARD COMPATIBILITY /// ALIASES
// ============================================================================

/// Legacy port constants for backward compatibility
pub const DEFAULT_PORT: u16 = CanonicalNetworkDefaults::DEFAULT_ORCHESTRATOR_PORT;
/// Default discovery port (legacy alias)
pub const DEFAULT_DISCOVERY_PORT: u16 = CanonicalNetworkDefaults::DEFAULT_DISCOVERY_PORT;

/// Default federation port (legacy alias)
pub const DEFAULT_FEDERATION_PORT: u16 = CanonicalNetworkDefaults::DEFAULT_FEDERATION_PORT;

/// Legacy timeout constants for backward compatibility;
pub const DEFAULT_CACHE_TTL: Duration = CanonicalPerformanceDefaults::DEFAULT_CACHE_TTL;
/// Default evaluation timeout (legacy alias)
pub const DEFAULT_EVALUATION_TIMEOUT: Duration =
    CanonicalPerformanceDefaults::DEFAULT_EVALUATION_TIMEOUT;

/// Default metrics collection interval (legacy alias)
pub const DEFAULT_METRICS_INTERVAL: Duration =
    CanonicalPerformanceDefaults::DEFAULT_METRICS_INTERVAL;

/// Legacy configuration constants for backward compatibility
/// Default configuration file path (legacy alias)
pub const DEFAULT_CONFIG_PATH: &str = CanonicalConfigDefaults::DEFAULT_CONFIG_PATH;

/// Default log level (legacy alias)
pub const DEFAULT_LOG_LEVEL: &str = CanonicalConfigDefaults::DEFAULT_LOG_LEVEL;

/// Legacy network address constants for backward compatibility;
pub const LOCALHOST_IPV4: &str = CanonicalNetworkAddresses::LOCALHOST_IPV4;
/// Localhost IPv6 address (legacy alias)
pub const LOCALHOST_IPV6: &str = CanonicalNetworkAddresses::LOCALHOST_IPV6;
/// Bind all IPv4 address (legacy alias)
pub const BIND_ALL_IPV4: &str = CanonicalNetworkAddresses::BIND_ALL_IPV4;
/// Localhost name (legacy alias)
pub const LOCALHOST_NAME: &str = CanonicalNetworkAddresses::LOCALHOST_NAME;
