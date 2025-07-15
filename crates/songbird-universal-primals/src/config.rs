//! Configuration system for the universal primal system
//!
//! This module provides comprehensive configuration options for managing
//! primal instances, multi-instance deployments, and system-wide settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Main configuration for the universal primal system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPrimalConfig {
    /// Whether to enable auto-discovery of primals
    pub auto_discovery_enabled: bool,

    /// Individual primal instance configurations
    pub primal_instances: HashMap<String, PrimalInstanceConfig>,

    /// Multi-instance management settings
    pub multi_instance: MultiInstanceConfig,

    /// Instance lifecycle management
    pub lifecycle: InstanceLifecycleConfig,

    /// Port management configuration
    pub port_management: PortManagementConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Global timeout settings
    pub timeouts: TimeoutConfig,

    /// Logging and monitoring configuration
    pub monitoring: MonitoringConfig,
}

/// Configuration for individual primal instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInstanceConfig {
    /// Base URL for the primal service
    pub base_url: String,

    /// Instance identifier
    pub instance_id: String,

    /// User ID this instance serves
    pub user_id: String,

    /// Device ID this instance serves
    pub device_id: String,

    /// Security level for this instance
    pub security_level: String,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Custom headers for requests
    pub headers: HashMap<String, String>,

    /// Maximum request timeout
    pub timeout_seconds: u64,

    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Multi-instance management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceConfig {
    /// Maximum number of instances per primal type
    pub max_instances_per_type: usize,

    /// Maximum number of instances per user
    pub max_instances_per_user: usize,

    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,

    /// Instance failover configuration
    pub failover: FailoverConfig,

    /// Instance scaling configuration
    pub scaling: ScalingConfig,
}

/// Instance lifecycle management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceLifecycleConfig {
    /// Whether to automatically start instances
    pub auto_start: bool,

    /// Whether to automatically stop unused instances
    pub auto_stop: bool,

    /// Time before stopping unused instances
    pub idle_timeout_minutes: u64,

    /// Health check configuration
    pub health_monitoring: HealthMonitoringConfig,
}

/// Port management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortManagementConfig {
    /// Port range for dynamic allocation
    pub port_range: PortRange,

    /// Port lease duration
    pub lease_duration_minutes: u64,

    /// Port allocation strategy
    pub allocation_strategy: PortAllocationStrategy,

    /// Reserved ports that should not be allocated
    pub reserved_ports: Vec<u16>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether to enable TLS
    pub tls_enabled: bool,

    /// TLS certificate verification
    pub verify_certificates: bool,

    /// Client certificate path
    pub client_cert_path: Option<String>,

    /// Client private key path
    pub client_key_path: Option<String>,

    /// CA certificate path
    pub ca_cert_path: Option<String>,

    /// API key authentication
    pub api_key_auth: bool,

    /// JWT authentication
    pub jwt_auth: bool,

    /// Encryption settings
    pub encryption: EncryptionConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections
    pub max_connections: usize,

    /// Minimum number of connections
    pub min_connections: usize,

    /// Connection timeout
    pub connection_timeout_seconds: u64,

    /// Idle timeout
    pub idle_timeout_seconds: u64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,

    /// Health check interval
    pub interval_seconds: u64,

    /// Health check timeout
    pub timeout_seconds: u64,

    /// Number of retries before marking unhealthy
    pub retry_count: u32,

    /// Whether to enable health checks
    pub enabled: bool,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin load balancing
    RoundRobin,

    /// Least connections load balancing
    LeastConnections,

    /// Random load balancing
    Random,

    /// Weighted load balancing
    Weighted,

    /// Health-based load balancing
    HealthBased,
}

/// Failover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    /// Whether to enable failover
    pub enabled: bool,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Retry delay
    pub retry_delay_seconds: u64,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Whether to enable auto-scaling
    pub auto_scaling_enabled: bool,

    /// Minimum number of instances
    pub min_instances: usize,

    /// Maximum number of instances
    pub max_instances: usize,

    /// CPU usage threshold for scaling up
    pub scale_up_cpu_threshold: f64,

    /// CPU usage threshold for scaling down
    pub scale_down_cpu_threshold: f64,

    /// Memory usage threshold for scaling up
    pub scale_up_memory_threshold: f64,

    /// Memory usage threshold for scaling down
    pub scale_down_memory_threshold: f64,
}

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    /// Start port
    pub start: u16,

    /// End port
    pub end: u16,
}

/// Port allocation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortAllocationStrategy {
    /// Sequential allocation
    Sequential,

    /// Random allocation
    Random,

    /// Least recently used allocation
    LeastRecentlyUsed,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    /// Whether to enable health monitoring
    pub enabled: bool,

    /// Health check interval
    pub check_interval_seconds: u64,

    /// Health check timeout
    pub check_timeout_seconds: u64,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Number of consecutive successes before marking healthy
    pub recovery_threshold: u32,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Whether to enable encryption
    pub enabled: bool,

    /// Encryption algorithm
    pub algorithm: String,

    /// Key size in bits
    pub key_size: u32,

    /// Key derivation function
    pub key_derivation: String,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether to enable circuit breaker
    pub enabled: bool,

    /// Failure threshold
    pub failure_threshold: u32,

    /// Success threshold
    pub success_threshold: u32,

    /// Timeout duration
    pub timeout_seconds: u64,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default request timeout
    pub default_request_timeout_seconds: u64,

    /// Connection timeout
    pub connection_timeout_seconds: u64,

    /// Read timeout
    pub read_timeout_seconds: u64,

    /// Write timeout
    pub write_timeout_seconds: u64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether to enable metrics collection
    pub metrics_enabled: bool,

    /// Metrics endpoint
    pub metrics_endpoint: String,

    /// Metrics port
    pub metrics_port: u16,

    /// Tracing configuration
    pub tracing: TracingConfig,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Whether to enable tracing
    pub enabled: bool,

    /// Tracing level
    pub level: String,

    /// Tracing format
    pub format: String,

    /// Whether to include file and line information
    pub include_location: bool,
}

impl Default for UniversalPrimalConfig {
    fn default() -> Self {
        Self {
            auto_discovery_enabled: true,
            primal_instances: HashMap::new(),
            multi_instance: MultiInstanceConfig::default(),
            lifecycle: InstanceLifecycleConfig::default(),
            port_management: PortManagementConfig::default(),
            security: SecurityConfig::default(),
            timeouts: TimeoutConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for MultiInstanceConfig {
    fn default() -> Self {
        Self {
            max_instances_per_type: 10,
            max_instances_per_user: 5,
            load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
            failover: FailoverConfig::default(),
            scaling: ScalingConfig::default(),
        }
    }
}

impl Default for InstanceLifecycleConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            auto_stop: true,
            idle_timeout_minutes: 30,
            health_monitoring: HealthMonitoringConfig::default(),
        }
    }
}

impl Default for PortManagementConfig {
    fn default() -> Self {
        Self {
            port_range: PortRange {
                start: 20000,
                end: 30000,
            },
            lease_duration_minutes: 60,
            allocation_strategy: PortAllocationStrategy::Sequential,
            reserved_ports: vec![22, 80, 443, 8080, 8443],
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            verify_certificates: true,
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            api_key_auth: true,
            jwt_auth: false,
            encryption: EncryptionConfig::default(),
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            min_connections: 10,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 300,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
            retry_count: 3,
            enabled: true,
        }
    }
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            retry_delay_seconds: 1,
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            auto_scaling_enabled: false,
            min_instances: 1,
            max_instances: 10,
            scale_up_cpu_threshold: 80.0,
            scale_down_cpu_threshold: 20.0,
            scale_up_memory_threshold: 80.0,
            scale_down_memory_threshold: 20.0,
        }
    }
}

impl Default for HealthMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 30,
            check_timeout_seconds: 5,
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            key_derivation: "PBKDF2".to_string(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            success_threshold: 2,
            timeout_seconds: 60,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_request_timeout_seconds: 30,
            connection_timeout_seconds: 10,
            read_timeout_seconds: 30,
            write_timeout_seconds: 30,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_endpoint: "/metrics".to_string(),
            metrics_port: 9090,
            tracing: TracingConfig::default(),
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            format: "json".to_string(),
            include_location: false,
        }
    }
}

impl UniversalPrimalConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Load basic settings from environment
        if let Ok(val) = env::var("PRIMAL_AUTO_DISCOVERY") {
            config.auto_discovery_enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("PRIMAL_MAX_INSTANCES_PER_TYPE") {
            config.multi_instance.max_instances_per_type = val.parse().unwrap_or(10);
        }

        if let Ok(val) = env::var("PRIMAL_MAX_INSTANCES_PER_USER") {
            config.multi_instance.max_instances_per_user = val.parse().unwrap_or(5);
        }

        if let Ok(val) = env::var("PRIMAL_TLS_ENABLED") {
            config.security.tls_enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("PRIMAL_VERIFY_CERTIFICATES") {
            config.security.verify_certificates = val.parse().unwrap_or(true);
        }

        if let Ok(val) = env::var("PRIMAL_PORT_RANGE_START") {
            config.port_management.port_range.start = val.parse().unwrap_or(20000);
        }

        if let Ok(val) = env::var("PRIMAL_PORT_RANGE_END") {
            config.port_management.port_range.end = val.parse().unwrap_or(30000);
        }

        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate port range
        if self.port_management.port_range.start >= self.port_management.port_range.end {
            return Err("Port range start must be less than end".to_string());
        }

        // Validate instance limits
        if self.multi_instance.max_instances_per_type == 0 {
            return Err("Max instances per type must be greater than 0".to_string());
        }

        if self.multi_instance.max_instances_per_user == 0 {
            return Err("Max instances per user must be greater than 0".to_string());
        }

        // Validate timeout values
        if self.timeouts.default_request_timeout_seconds == 0 {
            return Err("Default request timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl PrimalInstanceConfig {
    /// Create a new instance configuration
    pub fn new(base_url: String, instance_id: String, user_id: String, device_id: String) -> Self {
        Self {
            base_url,
            instance_id,
            user_id,
            device_id,
            security_level: "standard".to_string(),
            api_key: None,
            headers: HashMap::new(),
            timeout_seconds: 30,
            connection_pool: ConnectionPoolConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set security level
    pub fn with_security_level(mut self, level: String) -> Self {
        self.security_level = level;
        self
    }

    /// Add custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}
