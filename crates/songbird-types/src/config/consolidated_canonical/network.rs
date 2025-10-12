//! # Network Configuration Module
//!
//! **CANONICAL NETWORK CONFIGURATION** ✅
//!
//! This module provides network and communication configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// ============================================================================
// NETWORK CONFIGURATION
// ============================================================================

/// **CANONICAL**: Network and communication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalNetworkConfig {
    /// Server binding configuration
    pub bind: CanonicalBindConfig,

    /// Client connection configuration
    pub client: CanonicalClientConfig,

    /// TLS/SSL configuration
    pub tls: Option<CanonicalTlsConfig>,

    /// Proxy configuration
    pub proxy: Option<CanonicalProxyConfig>,

    /// Connection pooling
    pub connection_pool: CanonicalConnectionPoolConfig,

    /// Timeout settings
    pub timeouts: CanonicalTimeoutConfig,

    /// Rate limiting
    pub rate_limiting: CanonicalRateLimitConfig,
}

/// **CANONICAL**: Server binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalBindConfig {
    /// Primary service address
    pub address: String,

    /// Primary service port
    pub port: u16,

    /// Additional service endpoints
    pub additional_endpoints: Vec<CanonicalEndpointConfig>,

    /// Enable IPv6
    pub ipv6_enabled: bool,

    /// Socket options
    pub socket_options: HashMap<String, serde_json::Value>,
}

/// **CANONICAL**: Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEndpointConfig {
    /// Endpoint name/purpose
    pub name: String,

    /// Bind address
    pub address: String,

    /// Port number
    pub port: u16,

    /// Protocol (http, https, grpc, tcp, udp,
    pub protocol: String,

    /// Endpoint-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// **CANONICAL**: Client connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalClientConfig {
    /// Default user agent
    pub user_agent: String,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Maximum concurrent connections
    pub max_connections: u32,

    /// Connection keep-alive settings
    pub keep_alive: CanonicalKeepAliveConfig,

    /// Retry configuration
    pub retry: CanonicalRetryConfig,
}

/// **CANONICAL**: Keep-alive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalKeepAliveConfig {
    /// Enable keep-alive
    pub enabled: bool,

    /// Keep-alive timeout
    pub timeout: Duration,

    /// Keep-alive interval
    pub interval: Duration,

    /// Maximum idle connections
    pub max_idle_connections: u32,
}

/// **CANONICAL**: Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Base retry delay
    pub base_delay: Duration,

    /// Maximum retry delay
    pub max_delay: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: f64,

    /// Jitter factor (0.0-1.0,
    pub jitter_factor: f64,

    /// Retryable status codes
    pub retryable_status_codes: Vec<u16>,
}

/// **CANONICAL**: TLS/SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalTlsConfig {
    /// Enable TLS
    pub enabled: bool,

    /// Certificate file path
    pub cert_file: PathBuf,

    /// Private key file path
    pub key_file: PathBuf,

    /// CA certificate file path
    pub ca_file: Option<PathBuf>,

    /// TLS version (1.2, 1.3,
    pub version: String,

    /// Cipher suites
    pub cipher_suites: Vec<String>,

    /// Client certificate verification
    pub verify_client_cert: bool,
}

/// **CANONICAL**: Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalProxyConfig {
    /// Proxy URL
    pub url: String,

    /// Proxy username
    pub username: Option<String>,

    /// Proxy password
    pub password: Option<String>,

    /// No proxy hosts
    pub no_proxy: Vec<String>,
}

/// **CANONICAL**: Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalConnectionPoolConfig {
    /// Maximum pool size
    pub max_size: u32,

    /// Minimum pool size
    pub min_size: u32,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Idle timeout
    pub idle_timeout: Duration,

    /// Maximum connection lifetime
    pub max_lifetime: Duration,

    /// Health check query
    pub health_check_query: Option<String>,
}

/// **CANONICAL**: Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalTimeoutConfig {
    /// Connection establishment timeout
    pub connect: Duration,

    /// Request/response timeout
    pub request: Duration,

    /// Keep-alive timeout
    pub keep_alive: Duration,

    /// Graceful shutdown timeout
    pub shutdown: Duration,

    /// Health check timeout
    pub health_check: Duration,
}

/// **CANONICAL**: Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Requests per second limit
    pub requests_per_second: f64,

    /// Burst capacity
    pub burst_capacity: u32,

    /// Rate limit window
    pub window: Duration,

    /// Rate limit strategy (token_bucket, sliding_window, fixed_window,
    pub strategy: String,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for CanonicalNetworkConfig {
    fn default() -> Self {
        Self {
            bind: CanonicalBindConfig::default(),
            client: CanonicalClientConfig::default(),
            tls: None,
            proxy: None,
            connection_pool: CanonicalConnectionPoolConfig::default(),
            timeouts: CanonicalTimeoutConfig::default(),
            rate_limiting: CanonicalRateLimitConfig::default(),
        }
    }
}

impl Default for CanonicalBindConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: 8080,
            additional_endpoints: vec![],
            ipv6_enabled: false,
            socket_options: HashMap::new(),
        }
    }
}

impl Default for CanonicalClientConfig {
    fn default() -> Self {
        Self {
            user_agent: "songbird/0.1.0".to_string(),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_connections: 100,
            keep_alive: CanonicalKeepAliveConfig::default(),
            retry: CanonicalRetryConfig::default(),
        }
    }
}

impl Default for CanonicalKeepAliveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout: Duration::from_secs(60),
            interval: Duration::from_secs(30),
            max_idle_connections: 10,
        }
    }
}

impl Default for CanonicalRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retryable_status_codes: vec![500, 502, 503, 504],
        }
    }
}

impl Default for CanonicalConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_size: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
            health_check_query: None,
        }
    }
}

impl Default for CanonicalTimeoutConfig {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            request: Duration::from_secs(30),
            keep_alive: Duration::from_secs(60),
            shutdown: Duration::from_secs(30),
            health_check: Duration::from_secs(5),
        }
    }
}

impl Default for CanonicalRateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 100.0,
            burst_capacity: 200,
            window: Duration::from_secs(1),
            strategy: "token_bucket".to_string(),
        }
    }
}
