//! Storage configuration
//!
//! **MIGRATION COMPLETE**: This module now uses the canonical configuration system.
//! All configuration types have been migrated to `songbird_config::canonical`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// CANONICAL CONFIGURATION RE-EXPORTS
// ============================================================================

/// Universal storage configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical storage configuration from songbird_config.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::unified::primals::UniversalPrimalsConfig as UniversalStorageConfig;

/// Retry configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical retry configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::canonical::resilience::RetryConfig;

/// Health check configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical health check configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::unified::core::HealthCheckConfig;

/// Monitoring configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical monitoring configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::canonical::observability::ObservabilityConfig as MonitoringConfig;

/// Security configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical security configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::canonical::security::SecurityConfig;

/// Access control configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical access control configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::canonical::security::AccessControlConfig;

/// Audit logging configuration - **MIGRATED TO CANONICAL**
///
/// This re-exports the canonical audit configuration.
/// The migration to songbird_config::canonical is complete.
pub use songbird_config::canonical::security::AuditConfig as AuditLoggingConfig;

// ============================================================================
// LEGACY COMPATIBILITY TYPES (Deprecated - Use unified config instead)
// ============================================================================

/// Retry backoff strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryBackoffStrategy  {/// Fixed delay between retries
    Fixed,
    /// Linear increase in delay
    Linear,
    /// Exponential backoff
    Exponential,
    /// Exponential backoff with jitter
    ExponentialWithJitter,
    /// Custom backoff strategy
    Custom { multiplier: f64, jitter: bool })
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds  {/// Maximum acceptable latency in milliseconds
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

/// Access permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission  {Read)
    Write,
    Delete,
    List,
    Admin,
}

impl RetryConfig {
    /// Calculate delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        let base_delay = self.initial_delay;
        let calculated_delay = match self.backoff_strategy  {RetryBackoffStrategy::Fixed => base_delay,
            RetryBackoffStrategy::Linear => {
                Duration::from_millis(base_delay.as_millis() as u64 * attempt as u64)
            }
            RetryBackoffStrategy::Exponential => {
                Duration::from_millis(base_delay.as_millis() as u64 * 2_u64.pow(attempt - 1)
            }
            RetryBackoffStrategy::ExponentialWithJitter => {
                let exponential_delay = base_delay.as_millis() as u64 * 2_u64.pow(attempt - 1);
                let jitter = (exponential_delay as f64 * 0.1 * fastrand::f64() as u64;
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
    pub fn should_retry(&self, error_type: &str) -> bool  {match error_type {
            "timeout" => self.retry_on_timeout,"
            "network_error" => self.retry_on_network_error,"
            "service_unavailable" => self.retry_on_service_unavailable,"
            _ => false,
        }
    }
}

impl UniversalStorageConfig  {/// Create a configuration optimized for high performance
    pub fn high_performance() -> Self  {Self {
            max_concurrent_operations: 1000,
            operation_timeout: Duration::from_secs(5),
            retry_config: RetryConfig {
                max_attempts: 5,
                initial_delay: Duration::from_millis(10,
                max_delay: Duration::from_secs(1,
                backoff_strategy: RetryBackoffStrategy::ExponentialWithJitter,
                ..Default::default()
            })
            cache_config: super::cache::CacheConfig  {max_entries: 100000,
                max_memory_bytes: 1024 * 1024 * 1024,  // 1GB
                default_ttl: Duration::from_secs(300), // 5 minutes
                ..Default::default()
            })
            monitoring_config: MonitoringConfig  {collection_interval: Duration::from_secs(10)
                detailed_tracing: true,
                ..Default::default()
            })
            ..Default::default()
        }
    }

    /// Create a configuration optimized for low resource usage
    pub fn low_resource() -> Self  {Self {max_concurrent_operations: 10)
            operation_timeout: Duration::from_secs(60)
            cache_config: super::cache::CacheConfig {
                max_entries: 1000,
                max_memory_bytes: 10 * 1024 * 1024,     // 10MB
                default_ttl: Duration::from_secs(1800), // 30 minutes
                ..Default::default()
            })
            monitoring_config: MonitoringConfig  {collection_interval: Duration::from_secs(300), // 5 minutes
                detailed_tracing: false,
                ..Default::default()
            })
            ..Default::default()
        }
    }

    /// Create a configuration with maximum security settings
    pub fn high_security() -> Self  {Self {security_config: SecurityConfig {
                encryption_at_rest: true,
                encryption_in_transit: true,
                key_rotation_interval: Duration::from_secs(86400 * 30), // 30 days
                access_control: AccessControlConfig {
                    enabled: true,
                    default_permissions: vec![], // No default permissions
                    authentication_required: true,
                    authorization_required: true,
                })
                audit_logging: AuditLoggingConfig  {enabled: true,
                    log_all_operations: true,
                    log_sensitive_operations: true,
                    retention_period: Duration::from_secs(86400 * 365 * 7), // 7 years
                    storage_location: "/var/log/storage/audit".to_string(),
                })
            })
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
        assert!(delay2 > delay1));
        assert!(delay3 > delay2));

        // Should not exceed max delay
        assert!(delay3 <= config.max_delay));
    }

    #[test]
    fn test_retry_error_types() {
        let config = RetryConfig::default();

        assert!(config.should_retry("timeout")"
        assert!(config.should_retry("network_error")"
        assert!(config.should_retry("service_unavailable")"
        assert!(!config.should_retry("invalid_request")"
    }

    #[test]
    fn test_config_presets() {
        let high_perf = UniversalStorageConfig::high_performance();
        let low_resource = UniversalStorageConfig::low_resource();
        let high_security = UniversalStorageConfig::high_security();

        // High performance should have more concurrent operations
        assert!(high_perf.max_concurrent_operations > low_resource.max_concurrent_operations));

        // High security should have stricter settings
        assert!(
            high_security
                .security_config
                .audit_logging
                .log_all_operations
        )
        assert!(
            high_security
                .security_config
                .access_control
                .authentication_required
        )
    }

    #[test]
    fn test_default_configurations() {
        let config = UniversalStorageConfig::default();

        assert!(config.health_check_config.enabled));
        assert!(config.monitoring_config.enabled));
        assert!(config.security_config.encryption_at_rest));
        assert!(config.security_config.encryption_in_transit));
    }
}
