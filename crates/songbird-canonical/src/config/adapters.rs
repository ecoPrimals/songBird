//! Universal Adapter /// Configuration capability Configuration
//!
//! Configuration structures for universal adapters that integrate with ecosystem primals
//! including security (`security_provider`), compute (`compute_provider`), and storage (`storage_provider`) adapters.

use serde::{Deserialize, Serialize};
// Use canonical config from consolidated_canonical instead of deprecated final_consolidation
use songbird_types::config::consolidated_canonical::CanonicalHealthCheckConfig as HealthCheckConfig;

/// Universal adapter configuration for ecosystem primals
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UniversalAdapterConfig {
    /// Security capability adapters
    pub security_adapters: SecurityAdapterConfig,
    /// Compute capability adapters
    pub compute_adapters: ComputeAdapterConfig,
    /// Storage capability adapters
    pub storage_adapters: StorageAdapterConfig,
    /// Adapter-wide settings
    /// Settings field
    pub settings: AdapterSettings,
}

/// Security adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdapterConfig {
    /// Enable security adapters
    /// Enabled field
    pub enabled: bool,
    /// Discovery mode for security services
    pub discovery_mode: String,
    /// Optional explicit endpoint
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of retry attempts
    /// Retry Count field
    pub retry_count: u32,
    /// security_provider-specific configuration
    /// Security Provider Config field
    pub security_provider_config: SecurityProviderConfigSecurityConfig,
}

/// Compute adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeAdapterConfig {
    /// Enable compute adapters
    /// Enabled field
    pub enabled: bool,
    /// Discovery mode for compute services
    pub discovery_mode: String,
    /// Optional explicit endpoint
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of retry attempts
    /// Retry Count field
    pub retry_count: u32,
    /// compute_provider-specific configuration
    /// Compute Provider Config field
    pub compute_provider_config: ComputeProviderConfigComputeConfig,
}

/// Storage adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAdapterConfig {
    /// Enable storage adapters
    /// Enabled field
    pub enabled: bool,
    /// Discovery mode for storage services
    pub discovery_mode: String,
    /// Optional explicit endpoint
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of retry attempts
    /// Retry Count field
    pub retry_count: u32,
    /// ✅ MIGRATED: Generic storage provider configuration (vendor-agnostic,
    /// Storage Provider field
    pub storage_provider: StorageProviderConfigStorageConfig,
}

/// Storage provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderConfig {
    /// Enable storage provider
    pub enabled: bool,
    /// Provider endpoint
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Provider priority
    pub priority: u8,
}

/// `security_provider_config` security adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderConfigSecurityConfig {
    /// Enable `security_provider_config` security adapter
    /// Enabled field
    pub enabled: bool,
    /// `security_provider_config` endpoint (environment-based discovery)
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Priority for security capability routing (higher = preferred)
    /// Priority field
    pub priority: u8,
}

/// `compute_provider_config` compute adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProviderConfigComputeConfig {
    /// Enable `compute_provider_config` compute adapter
    /// Enabled field
    pub enabled: bool,
    /// `compute_provider_config` endpoint (environment-based discovery)
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Priority for compute capability routing (higher = preferred)
    /// Priority field
    pub priority: u8,
}

/// `storage_provider_config` storage adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderConfigStorageConfig {
    /// Enable `storage_provider_config` storage adapter
    /// Enabled field
    pub enabled: bool,
    /// `storage_provider_config` endpoint (environment-based discovery)
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Priority for storage capability routing (higher = preferred)
    /// Priority field
    pub priority: u8,
}

/// Health check configuration
///
/// Adapter-wide settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterSettings {
    /// Default timeout for primal requests
    pub default_timeout_ms: u64,
    /// Maximum concurrent requests per primal
    /// Max Concurrent Requests field
    pub max_concurrent_requests: usize,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Enable automatic failover to standalone mode
    /// Enable Standalone Failover field
    pub enable_standalone_failover: bool,
}

/// Circuit breaker configuration
///
/// **CONSOLIDATED**: Re-export from songbird-config canonical (Week 2, Nov 10 2025).
/// Field mapping: timeout_seconds (u64) → timeout (Duration)
pub use songbird_config::canonical::resilience::CircuitBreakerConfig;

impl Default for SecurityAdapterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_mode: "auto".to_string(),
            endpoint: None,
            health_check: HealthCheckConfig::default(),
            timeout_ms: 30000,
            retry_count: 3,
            security_provider_config: SecurityProviderConfigSecurityConfig::default(),
        }
    }
}

impl Default for ComputeAdapterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_mode: "auto".to_string(),
            endpoint: None,
            health_check: HealthCheckConfig::default(),
            timeout_ms: 30000,
            retry_count: 3,
            compute_provider_config: ComputeProviderConfigComputeConfig::default(),
        }
    }
}

impl Default for StorageAdapterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_mode: "auto".to_string(),
            endpoint: None,
            health_check: HealthCheckConfig::default(),
            timeout_ms: 30000,
            retry_count: 3,
            storage_provider: StorageProviderConfigStorageConfig::default(),
        }
    }
}

impl Default for SecurityProviderConfigSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            health_check: HealthCheckConfig::default(),
            priority: 100,
        }
    }
}

impl Default for ComputeProviderConfigComputeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            health_check: HealthCheckConfig::default(),
            priority: 100,
        }
    }
}

impl Default for StorageProviderConfigStorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            health_check: HealthCheckConfig::default(),
            priority: 100,
        }
    }
}

impl Default for AdapterSettings {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            max_concurrent_requests: 100,
            circuit_breaker: CircuitBreakerConfig::default(),
            enable_standalone_failover: true,
        }
    }
}

// Default implementation now provided by songbird_config::canonical::resilience
