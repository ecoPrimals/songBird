// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Adapter /// Configuration capability Configuration
//!
//! Configuration for universal primal adapters and ecosystem integration.
//! Enhanced with comprehensive features from `UniversalPrimalConfig`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::health::CanonicalHealthConfig;

/// **CANONICAL**: Universal adapter configuration
///
/// Enhanced version that consolidates features from:
/// - `songbird-universal-primals::UniversalPrimalConfig`
/// - Previous `CanonicalUniversalAdapterConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalUniversalAdapterConfig {
    /// Auto-discovery settings
    pub auto_discovery: bool,
    /// Primal instance configurations
    pub primal_instances: HashMap<String, CanonicalPrimalInstanceConfig>,
    /// Multi-instance management
    /// Multi Instance field
    pub multi_instance: CanonicalMultiInstanceConfig,
    /// Instance lifecycle management
    pub lifecycle: CanonicalInstanceLifecycleConfig,
    /// Port management
    /// Port Management field
    pub port_management: CanonicalPortManagementConfig,
    /// Security configuration
    pub security: CanonicalAdapterSecurityConfig,
    /// Global timeout settings
    pub timeouts: CanonicalTimeoutConfig,
    /// Logging and monitoring configuration
    /// Monitoring field
    pub monitoring: CanonicalAdapterMonitoringConfig,
}

impl Default for CanonicalUniversalAdapterConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            primal_instances: HashMap::new(),
            multi_instance: CanonicalMultiInstanceConfig::default(),
            lifecycle: CanonicalInstanceLifecycleConfig::default(),
            port_management: CanonicalPortManagementConfig::default(),
            security: CanonicalAdapterSecurityConfig::default(),
            timeouts: CanonicalTimeoutConfig::default(),
            monitoring: CanonicalAdapterMonitoringConfig::default(),
        }
    }
}

/// **CANONICAL**: Enhanced primal instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrimalInstanceConfig {
    /// Base URL for the primal service
    pub base_url: String,
    /// Instance identifier
    /// Instance Id field
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
    /// Timeout Seconds field
    pub timeout_seconds: u64,
    /// Connection pool settings
    /// Connection Pool field
    pub connection_pool: CanonicalConnectionPoolConfig,
    /// Health check configuration
    pub health_check: CanonicalHealthConfig,
}

impl Default for CanonicalPrimalInstanceConfig {
    fn default() -> Self {
        let base_host = songbird_process_env::var("DEFAULT_PRIMAL_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let base_port = songbird_process_env::var("DEFAULT_PRIMAL_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);

        Self {
            base_url: format!("http://{base_host}:{base_port}"),
            instance_id: "default-instance".to_string(),
            user_id: "default-user".to_string(),
            device_id: "default-device".to_string(),
            security_level: "standard".to_string(),
            api_key: None,
            headers: HashMap::new(),
            timeout_seconds: 30,
            connection_pool: CanonicalConnectionPoolConfig::default(),
            health_check: CanonicalHealthConfig::default(),
        }
    }
}

/// **CANONICAL**: Enhanced multi-instance management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMultiInstanceConfig {
    /// Enable multi-instance support
    /// Enabled field
    pub enabled: bool,
    /// Maximum number of instances per primal type
    /// Max Instances Per Type field
    pub max_instances_per_type: usize,
    /// Maximum number of instances per user
    /// Max Instances Per User field
    pub max_instances_per_user: usize,
    /// Load balancing strategy
    /// Load Balancing Strategy field
    pub load_balancing_strategy: CanonicalLoadBalancingStrategy,
    /// Instance selection strategy
    /// Selection Strategy field
    pub selection_strategy: String,
    /// Health check interval for instances
    /// Health Check Interval field
    pub health_check_interval: Duration,
    /// Instance timeout before removal
    /// Instance Timeout field
    pub instance_timeout: Duration,
    /// Instance failover configuration
    pub failover: CanonicalFailoverConfig,
    /// Instance scaling configuration
    /// Whether auto-scaling is supported
    pub scaling: CanonicalScalingConfig,
}

impl Default for CanonicalMultiInstanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_instances_per_type: 10,
            max_instances_per_user: 5,
            load_balancing_strategy: CanonicalLoadBalancingStrategy::HealthBased,
            selection_strategy: "health_weighted".to_string(),
            health_check_interval: Duration::from_secs(30),
            instance_timeout: Duration::from_secs(300),
            failover: CanonicalFailoverConfig::default(),
            scaling: CanonicalScalingConfig::default(),
        }
    }
}

/// **CANONICAL**: Load balancing strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalLoadBalancingStrategy {
    /// Round-robin load balancing
    RoundRobin,
    /// Least connections load balancing
    LeastConnections,
    /// Random load balancing
    Random,
    /// Weighted load balancing
    Weighted,
    /// Health-based load balancing (recommended)
    HealthBased,
}

/// **CANONICAL**: Instance lifecycle management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalInstanceLifecycleConfig {
    /// Whether to automatically start instances
    pub auto_start: bool,
    /// Whether to automatically stop unused instances
    pub auto_stop: bool,
    /// Time before stopping unused instances
    /// Idle Timeout Minutes field
    pub idle_timeout_minutes: u64,
    /// Health monitoring configuration
    /// Health Monitoring field
    pub health_monitoring: CanonicalHealthMonitoringConfig,
}

impl Default for CanonicalInstanceLifecycleConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            auto_stop: true,
            idle_timeout_minutes: 30,
            health_monitoring: CanonicalHealthMonitoringConfig::default(),
        }
    }
}

/// **CANONICAL**: Port management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPortManagementConfig {
    /// Port range for dynamic allocation
    /// Port Range field
    pub port_range: CanonicalPortRange,
    /// Port lease duration
    /// Lease Duration Minutes field
    pub lease_duration_minutes: u64,
    /// Port allocation strategy
    /// Allocation Strategy field
    pub allocation_strategy: CanonicalPortAllocationStrategy,
    /// Reserved ports that should not be allocated
    pub reserved_ports: Vec<u16>,
}

impl Default for CanonicalPortManagementConfig {
    fn default() -> Self {
        Self {
            port_range: CanonicalPortRange::default(),
            lease_duration_minutes: 60,
            allocation_strategy: CanonicalPortAllocationStrategy::Sequential,
            reserved_ports: vec![22, 80, 443, 8080, 8443], // Common reserved ports
        }
    }
}

/// **CANONICAL**: Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPortRange {
    /// Starting port number
    pub start: u16,
    /// Ending port number
    /// End field
    pub end: u16,
}

impl Default for CanonicalPortRange {
    fn default() -> Self {
        Self {
            start: 20000,
            end: 30000,
        }
    }
}

/// **CANONICAL**: Port allocation strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalPortAllocationStrategy {
    /// Sequential allocation
    Sequential,
    /// Random allocation
    Random,
    /// Hash-based allocation
    HashBased,
}

/// **CANONICAL**: Adapter security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "intentional pattern; clippy false positive for this API"
)]
pub struct CanonicalAdapterSecurityConfig {
    /// Whether to enable /// TLS
    /// Tls Enabled field
    pub tls_enabled: bool,
    /// TLS certificate verification
    pub verify_certificates: bool,
    /// Client certificate path
    /// Client Cert Path field
    pub client_cert_path: Option<String>,
    /// Client private key path
    /// Client Key Path field
    pub client_key_path: Option<String>,
    /// CA certificate path
    pub ca_cert_path: Option<String>,
    /// API key authentication
    pub api_key_auth: bool,
    /// JWT authentication
    pub jwt_auth: bool,
    /// Encryption settings
    /// Whether encryption is enabled
    pub encryption: CanonicalEncryptionConfig,
}

impl Default for CanonicalAdapterSecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            verify_certificates: true,
            client_cert_path: None,
            client_key_path: None,
            ca_cert_path: None,
            api_key_auth: false,
            jwt_auth: false,
            encryption: CanonicalEncryptionConfig::default(),
        }
    }
}

/// **CANONICAL**: Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEncryptionConfig {
    /// Enable end-to-end encryption
    /// Enabled field
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
}

impl Default for CanonicalEncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
        }
    }
}

/// **CANONICAL**: Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalConnectionPoolConfig {
    /// Maximum number of connections
    /// Max Connections field
    pub max_connections: usize,
    /// Minimum number of connections
    /// Min Connections field
    pub min_connections: usize,
    /// Connection timeout
    /// Connection Timeout Seconds field
    pub connection_timeout_seconds: u64,
    /// Idle timeout
    /// Idle Timeout Seconds field
    pub idle_timeout_seconds: u64,
}

impl Default for CanonicalConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 300,
        }
    }
}

/// **CANONICAL**: Failover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFailoverConfig {
    /// Whether to enable failover
    /// Enabled field
    pub enabled: bool,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Retry delay
    /// Retry Delay Seconds field
    pub retry_delay_seconds: u64,
    /// Circuit breaker configuration
    pub circuit_breaker: CanonicalCircuitBreakerConfig,
}

impl Default for CanonicalFailoverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            retry_delay_seconds: 5,
            circuit_breaker: CanonicalCircuitBreakerConfig::default(),
        }
    }
}

/// **CANONICAL**: Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCircuitBreakerConfig {
    /// Enable circuit breaker
    /// Enabled field
    pub enabled: bool,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Recovery timeout
    /// Recovery Timeout Seconds field
    pub recovery_timeout_seconds: u64,
}

impl Default for CanonicalCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            recovery_timeout_seconds: 60,
        }
    }
}

/// **CANONICAL**: Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalScalingConfig {
    /// Enable auto-scaling
    /// Enabled field
    pub enabled: bool,
    /// Minimum instances
    /// Min Instances field
    pub min_instances: usize,
    /// Maximum instances
    /// Max Instances field
    pub max_instances: usize,
    /// CPU threshold for scaling up (percentage)
    /// Cpu Scale Up Threshold field
    pub cpu_scale_up_threshold: f32,
    /// CPU threshold for scaling down (percentage)
    /// Cpu Scale Down Threshold field
    pub cpu_scale_down_threshold: f32,
}

impl Default for CanonicalScalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_instances: 1,
            max_instances: 10,
            cpu_scale_up_threshold: 80.0,
            cpu_scale_down_threshold: 30.0,
        }
    }
}

/// **CANONICAL**: Global timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names, reason = "field names mirror canonical timeout schema naming")]
pub struct CanonicalTimeoutConfig {
    /// Default request timeout
    pub default_request_timeout: Duration,
    /// Connection timeout
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Discovery timeout
    pub discovery_timeout: Duration,
}

impl Default for CanonicalTimeoutConfig {
    fn default() -> Self {
        Self {
            default_request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            health_check_timeout: Duration::from_secs(5),
            discovery_timeout: Duration::from_secs(15),
        }
    }
}

/// **CANONICAL**: Adapter monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAdapterMonitoringConfig {
    /// Enable performance monitoring
    /// Performance Monitoring field
    pub performance_monitoring: bool,
    /// Enable detailed logging
    /// Detailed Logging field
    pub detailed_logging: bool,
    /// Metrics collection interval
    /// Metrics Interval field
    pub metrics_interval: Duration,
    /// Log level
    pub log_level: String,
    /// Enable distributed tracing
    /// Distributed Tracing field
    pub distributed_tracing: bool,
}

impl Default for CanonicalAdapterMonitoringConfig {
    fn default() -> Self {
        Self {
            performance_monitoring: true,
            detailed_logging: false,
            metrics_interval: Duration::from_secs(60),
            log_level: "info".to_string(),
            distributed_tracing: true,
        }
    }
}

/// **CANONICAL**: Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthMonitoringConfig {
    /// Health check interval
    /// Check Interval field
    pub check_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
}

impl Default for CanonicalHealthMonitoringConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::collections::HashMap;

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), json);
    }

    #[test]
    fn default_universal_adapter_config() {
        let c = CanonicalUniversalAdapterConfig::default();
        assert!(c.auto_discovery);
        assert!(c.primal_instances.is_empty());
        assert_eq!(c.multi_instance.max_instances_per_type, 10);
        assert_eq!(c.port_management.reserved_ports.len(), 5);
        assert_eq!(c.security.encryption.key_size, 256);
        assert_eq!(c.monitoring.log_level, "info");
    }

    #[test]
    fn default_primal_instance_and_multi_instance() {
        let p = CanonicalPrimalInstanceConfig::default();
        assert!(p.base_url.starts_with("http://"));
        assert_eq!(p.timeout_seconds, 30);
        let m = CanonicalMultiInstanceConfig::default();
        assert!(m.enabled);
        assert_eq!(m.load_balancing_strategy, CanonicalLoadBalancingStrategy::HealthBased);
    }

    #[test]
    fn default_lifecycle_port_pool_security_timeouts() {
        assert!(CanonicalInstanceLifecycleConfig::default().auto_start);
        let pm = CanonicalPortManagementConfig::default();
        assert_eq!(pm.port_range.start, 20000);
        assert_eq!(pm.allocation_strategy, CanonicalPortAllocationStrategy::Sequential);
        let sec = CanonicalAdapterSecurityConfig::default();
        assert!(sec.tls_enabled && sec.verify_certificates);
        let t = CanonicalTimeoutConfig::default();
        assert_eq!(t.default_request_timeout, Duration::from_secs(30));
    }

    #[test]
    fn default_connection_failover_scaling_circuit_breaker() {
        let cp = CanonicalConnectionPoolConfig::default();
        assert_eq!(cp.max_connections, 10);
        assert_eq!(cp.min_connections, 1);
        let f = CanonicalFailoverConfig::default();
        assert!(f.enabled);
        assert!(CanonicalCircuitBreakerConfig::default().enabled);
        let s = CanonicalScalingConfig::default();
        assert!(!s.enabled);
        assert_eq!(s.cpu_scale_up_threshold, 80.0);
    }

    #[test]
    fn default_monitoring_health() {
        let h = CanonicalHealthMonitoringConfig::default();
        assert_eq!(h.failure_threshold, 3);
        assert_eq!(h.success_threshold, 2);
    }

    #[test]
    fn derive_clone_partialeq_load_balancing_and_port_strategy() {
        let a = CanonicalLoadBalancingStrategy::HealthBased;
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(
            CanonicalPortAllocationStrategy::HashBased,
            CanonicalPortAllocationStrategy::HashBased
        );
    }

    #[test]
    fn roundtrip_canonical_universal_adapter_config() {
        assert_json_roundtrip(&CanonicalUniversalAdapterConfig::default());
    }

    #[test]
    fn roundtrip_primal_instance_with_map() {
        let mut c = CanonicalPrimalInstanceConfig::default();
        c.headers.insert("h".into(), "v".into());
        assert_json_roundtrip(&c);
    }

    #[test]
    fn roundtrip_multi_instance_and_lifecycle() {
        assert_json_roundtrip(&CanonicalMultiInstanceConfig::default());
        assert_json_roundtrip(&CanonicalInstanceLifecycleConfig::default());
    }

    #[test]
    fn roundtrip_port_management_and_range() {
        assert_json_roundtrip(&CanonicalPortManagementConfig::default());
        assert_json_roundtrip(&CanonicalPortRange::default());
    }

    #[test]
    fn roundtrip_security_encryption_connection_pool() {
        assert_json_roundtrip(&CanonicalAdapterSecurityConfig::default());
        assert_json_roundtrip(&CanonicalEncryptionConfig::default());
        assert_json_roundtrip(&CanonicalConnectionPoolConfig::default());
    }

    #[test]
    fn roundtrip_failover_circuit_breaker_scaling_timeouts_monitoring() {
        assert_json_roundtrip(&CanonicalFailoverConfig::default());
        assert_json_roundtrip(&CanonicalCircuitBreakerConfig::default());
        assert_json_roundtrip(&CanonicalScalingConfig::default());
        assert_json_roundtrip(&CanonicalTimeoutConfig::default());
        assert_json_roundtrip(&CanonicalAdapterMonitoringConfig::default());
        assert_json_roundtrip(&CanonicalHealthMonitoringConfig::default());
    }

    #[test]
    fn roundtrip_load_balancing_strategies() {
        assert_json_roundtrip(&CanonicalLoadBalancingStrategy::RoundRobin);
        assert_json_roundtrip(&CanonicalPortAllocationStrategy::Random);
    }

    #[test]
    fn roundtrip_empty_hashmap_primal_instances() {
        let mut cfg = CanonicalUniversalAdapterConfig::default();
        cfg.primal_instances = HashMap::new();
        assert_json_roundtrip(&cfg);
    }
}
