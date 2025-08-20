use songbird_config::unified::*;
//! Storage configuration types and defaults

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Universal storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate UniversalStorageConfig to songbird_config::unified
pub struct UniversalStorageConfig {
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    /// Default timeout for storage operations
    pub operation_timeout: Duration,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Cache configuration
    pub cache_config: super::cache::CacheConfig,
    /// Health check configuration
    pub health_check_config: HealthCheckConfig,
    /// Performance monitoring configuration
    pub monitoring_config: MonitoringConfig,
    /// Security configuration
    pub security_config: SecurityConfig,
}

/// Retry configuration for storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate RetryConfig to songbird_config::unified
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff strategy for retries
    pub backoff_strategy: RetryBackoffStrategy,
    /// Whether to retry on specific error types
    pub retry_on_timeout: bool,
    pub retry_on_network_error: bool,
    pub retry_on_service_unavailable: bool,
}

/// Retry backoff strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryBackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear increase in delay
    Linear,
    /// Exponential backoff
    Exponential,
    /// Exponential backoff with jitter
    ExponentialWithJitter,
    /// Custom backoff strategy
    Custom { multiplier: f64, jitter: bool },
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate HealthCheckConfig to songbird_config::unified
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub check_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
    /// Health check endpoints
    pub endpoints: Vec<String>,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate MonitoringConfig to songbird_config::unified
pub struct MonitoringConfig {
    /// Enable performance monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Performance alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Enable detailed tracing
    pub detailed_tracing: bool,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: f64,
    /// Maximum acceptable error rate (0.0 to 1.0)
    pub max_error_rate: f64,
    /// Minimum acceptable throughput (operations per second)
    pub min_throughput_ops_per_sec: f64,
    /// Maximum memory usage in bytes
    pub max_memory_usage_bytes: u64,
    /// Maximum CPU utilization (0.0 to 1.0)
    pub max_cpu_utilization: f64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate SecurityConfig to songbird_config::unified
pub struct SecurityConfig {
    /// Enable encryption at rest
    pub encryption_at_rest: bool,
    /// Enable encryption in transit
    pub encryption_in_transit: bool,
    /// Encryption key rotation interval
    pub key_rotation_interval: Duration,
    /// Access control configuration
    pub access_control: AccessControlConfig,
    /// Audit logging configuration
    pub audit_logging: AuditLoggingConfig,
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate AccessControlConfig to songbird_config::unified
pub struct AccessControlConfig {
    /// Enable access control
    pub enabled: bool,
    /// Default access permissions
    pub default_permissions: Vec<Permission>,
    /// Authentication required
    pub authentication_required: bool,
    /// Authorization required
    pub authorization_required: bool,
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
// TODO: Migrate AuditLoggingConfig to songbird_config::unified
pub struct AuditLoggingConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log all operations
    pub log_all_operations: bool,
    /// Log only sensitive operations
    pub log_sensitive_operations: bool,
    /// Audit log retention period
    pub retention_period: Duration,
    /// Audit log storage location
    pub storage_location: String,
}

/// Access permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Delete,
    List,
    Admin,
}

impl Default for UniversalStorageConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 100,
            operation_timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            cache_config: super::cache::CacheConfig::default(),
            health_check_config: HealthCheckConfig::default(),
            monitoring_config: MonitoringConfig::default(),
            security_config: SecurityConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_strategy: RetryBackoffStrategy::ExponentialWithJitter,
            retry_on_timeout: true,
            retry_on_network_error: true,
            retry_on_service_unavailable: true,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            endpoints: vec!["/health".to_string()],
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60),
            retention_period: Duration::from_secs(86400 * 7), // 7 days
            alert_thresholds: AlertThresholds::default(),
            detailed_tracing: false,
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_latency_ms: 1000.0,
            max_error_rate: 0.05, // 5%
            min_throughput_ops_per_sec: 10.0,
            max_memory_usage_bytes: 1024 * 1024 * 1024, // 1GB
            max_cpu_utilization: 0.8,                   // 80%
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_at_rest: true,
            encryption_in_transit: true,
            key_rotation_interval: Duration::from_secs(86400 * 90), // 90 days
            access_control: AccessControlConfig::default(),
            audit_logging: AuditLoggingConfig::default(),
        }
    }
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_permissions: vec![Permission::Read],
            authentication_required: true,
            authorization_required: true,
        }
    }
}

impl Default for AuditLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_all_operations: false,
            log_sensitive_operations: true,
            retention_period: Duration::from_secs(86400 * 365), // 1 year
            storage_location: "/var/log/storage/audit".to_string(),
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        let base_delay = self.initial_delay;
        let calculated_delay = match self.backoff_strategy {
            RetryBackoffStrategy::Fixed => base_delay,
            RetryBackoffStrategy::Linear => {
                Duration::from_millis(base_delay.as_millis() as u64 * attempt as u64)
            }
            RetryBackoffStrategy::Exponential => {
                Duration::from_millis(base_delay.as_millis() as u64 * 2_u64.pow(attempt - 1))
            }
            RetryBackoffStrategy::ExponentialWithJitter => {
                let exponential_delay = base_delay.as_millis() as u64 * 2_u64.pow(attempt - 1);
                let jitter = (exponential_delay as f64 * 0.1 * fastrand::f64()) as u64;
                Duration::from_millis(exponential_delay + jitter)
            }
            RetryBackoffStrategy::Custom { multiplier, jitter } => {
                let custom_delay =
                    base_delay.as_millis() as f64 * multiplier.powi(attempt as i32 - 1);
                let delay = if jitter {
                    let jitter_amount = custom_delay * 0.1 * fastrand::f64();
                    custom_delay + jitter_amount
                } else {
                    custom_delay
                };
                Duration::from_millis(delay as u64)
            }
        };

        calculated_delay.min(self.max_delay)
    }

    /// Check if we should retry for a specific error type
    pub fn should_retry(&self, error_type: &str) -> bool {
        match error_type {
            "timeout" => self.retry_on_timeout,
            "network_error" => self.retry_on_network_error,
            "service_unavailable" => self.retry_on_service_unavailable,
            _ => false,
        }
    }
}

impl UniversalStorageConfig {
    /// Create a configuration optimized for high performance
    pub fn high_performance() -> Self {
        Self {
            max_concurrent_operations: 1000,
            operation_timeout: Duration::from_secs(5),
            retry_config: RetryConfig {
                max_attempts: 5,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_secs(1),
                backoff_strategy: RetryBackoffStrategy::ExponentialWithJitter,
                ..Default::default()
            },
            cache_config: super::cache::CacheConfig {
                max_entries: 100000,
                max_memory_bytes: 1024 * 1024 * 1024,  // 1GB
                default_ttl: Duration::from_secs(300), // 5 minutes
                ..Default::default()
            },
            monitoring_config: MonitoringConfig {
                collection_interval: Duration::from_secs(10),
                detailed_tracing: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a configuration optimized for low resource usage
    pub fn low_resource() -> Self {
        Self {
            max_concurrent_operations: 10,
            operation_timeout: Duration::from_secs(60),
            cache_config: super::cache::CacheConfig {
                max_entries: 1000,
                max_memory_bytes: 10 * 1024 * 1024,     // 10MB
                default_ttl: Duration::from_secs(1800), // 30 minutes
                ..Default::default()
            },
            monitoring_config: MonitoringConfig {
                collection_interval: Duration::from_secs(300), // 5 minutes
                detailed_tracing: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create a configuration with maximum security settings
    pub fn high_security() -> Self {
        Self {
            security_config: SecurityConfig {
                encryption_at_rest: true,
                encryption_in_transit: true,
                key_rotation_interval: Duration::from_secs(86400 * 30), // 30 days
                access_control: AccessControlConfig {
                    enabled: true,
                    default_permissions: vec![], // No default permissions
                    authentication_required: true,
                    authorization_required: true,
                },
                audit_logging: AuditLoggingConfig {
                    enabled: true,
                    log_all_operations: true,
                    log_sensitive_operations: true,
                    retention_period: Duration::from_secs(86400 * 365 * 7), // 7 years
                    storage_location: "/var/log/storage/audit".to_string(),
                },
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig::default();

        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);
        let delay3 = config.calculate_delay(3);

        // Exponential backoff should increase delays
        assert!(delay2 > delay1);
        assert!(delay3 > delay2);

        // Should not exceed max delay
        assert!(delay3 <= config.max_delay);
    }

    #[test]
    fn test_retry_error_types() {
        let config = RetryConfig::default();

        assert!(config.should_retry("timeout"));
        assert!(config.should_retry("network_error"));
        assert!(config.should_retry("service_unavailable"));
        assert!(!config.should_retry("invalid_request"));
    }

    #[test]
    fn test_config_presets() {
        let high_perf = UniversalStorageConfig::high_performance();
        let low_resource = UniversalStorageConfig::low_resource();
        let high_security = UniversalStorageConfig::high_security();

        // High performance should have more concurrent operations
        assert!(high_perf.max_concurrent_operations > low_resource.max_concurrent_operations);

        // High security should have stricter settings
        assert!(
            high_security
                .security_config
                .audit_logging
                .log_all_operations
        );
        assert!(
            high_security
                .security_config
                .access_control
                .authentication_required
        );
    }

    #[test]
    fn test_default_configurations() {
        let config = UniversalStorageConfig::default();

        assert!(config.health_check_config.enabled);
        assert!(config.monitoring_config.enabled);
        assert!(config.security_config.encryption_at_rest);
        assert!(config.security_config.encryption_in_transit);
    }
}
