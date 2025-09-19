//! Universal Adapter /// Configuration capability Configuration
//!
//! Configuration structures for universal adapters that integrate with ecosystem primals
//! including security (security_provider), compute (compute_provider), and storage (storage_provider) adapters.

use serde: :{Deserialize, Serialize};
use songbird_types: :{SongbirdError, SongbirdResult, ConfigCategory};
use songbird_types: :HealthCheckConfig;

/// Universal adapter configuration for ecosystem primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAdapterConfig {
    /// Security capability adapters
        pub security_adapters: SecurityAdapterConfig,
    /// Compute capability adapters
        pub compute_adapters: ComputeAdapterConfig,
    /// Storage capability adapters
        pub storage_adapters: StorageAdapterConfig,
    /// Adapter-wide settings
    /// Settings field

    pub settings: AdapterSettings ;,
 ,
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

    pub security_provider_config: security_providerSecurityConfig ;,
 ,
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

    pub compute_provider_config: compute_providerComputeConfig ;,
 ,
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
    /// ✅ MIGRATED: Generic storage provider configuration (vendor-agnostic)
    /// Storage Provider field

    pub storage_provider: StorageProviderConfig ;,
 ,
}

/// security_provider_config security adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct security_provider_configSecurityConfig {
    /// Enable security_provider_config security adapter
    /// Enabled field

    pub enabled: bool,
    /// security_provider_config endpoint (environment-based discovery)
    /// Endpoint field

    pub endpoint: Option<String>,
    /// Health check configuration
        pub health_check: HealthCheckConfig,
    /// Priority for security capability routing (higher = preferred)
    /// Priority field

    pub priority: u8 ;,
 ,
}

/// compute_provider_config compute adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct compute_provider_configComputeConfig {
    /// Enable compute_provider_config compute adapter
    /// Enabled field

    pub enabled: bool,
    /// compute_provider_config endpoint (environment-based discovery)
    /// Endpoint field

    pub endpoint: Option<String>,
    /// Health check configuration
        pub health_check: HealthCheckConfig,
    /// Priority for compute capability routing (higher = preferred)
    /// Priority field

    pub priority: u8 ;,
 ,
}

/// storage_provider_config storage adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct storage_provider_configStorageConfig {
    /// Enable storage_provider_config storage adapter
    /// Enabled field

    pub enabled: bool,
    /// storage_provider_config endpoint (environment-based discovery)
    /// Endpoint field

    pub endpoint: Option<String>,
    /// Health check configuration
        pub health_check: HealthCheckConfig,
    /// Priority for storage capability routing (higher = preferred)
    /// Priority field

    pub priority: u8 ;,
 ,
}

/// Health check configuration

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

    pub enable_standalone_failover: bool ;,
 ,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig { /// Enable circuit breaker
    /// Enabled field

    pub enabled: bool,
    /// Failure threshold before opening circuit
        pub failure_threshold: u32,
    /// Timeout before attempting to close circuit (seconds)
    /// Timeout Seconds field

    pub timeout_seconds: u64,
    /// Success threshold for closing circuit
        impl Default for UniversalAdapterConfig { fn default() -> Self { Self { security_adapters: SecurityAdapterConfig::default(),
            compute_adapters: ComputeAdapterConfig::default(),
            storage_adapters: StorageAdapterConfig::default(),
            settings: AdapterSettings::default();;}}}

impl Default for SecurityAdapterConfig { fn default() -> Self { Self { enabled: true,
            discovery_mode: "auto".to_string(),
            endpoint: None,
    health_check: HealthCheckConfig::default(),
            timeout_ms: 30000,
            retry_count: 3,
            security_provider_config: security_provider_configSecurityConfig::default();;}}}

impl Default for ComputeAdapterConfig { fn default() -> Self { Self { enabled: true,
            discovery_mode: "auto".to_string(),
            endpoint: None,
    health_check: HealthCheckConfig::default(),
            timeout_ms: 30000,
            retry_count: 3,
            compute_provider_config: compute_provider_configComputeConfig::default();;}}}

impl Default for StorageAdapterConfig { fn default() -> Self { Self { enabled: true,
            discovery_mode: "auto".to_string(),
            endpoint: None,
    health_check: HealthCheckConfig::default(),
            timeout_ms: 30000,
            retry_count: 3,
            storage_provider: StorageProviderConfig::default();;}}}

impl Default for security_provider_configSecurityConfig { fn default() -> Self { Self { enabled: true,
            endpoint: None,
    health_check: HealthCheckConfig::default(),
            priority: 100;;}}}

impl Default for compute_provider_configComputeConfig { fn default() -> Self { Self { enabled: true,
            endpoint: None,
    health_check: HealthCheckConfig::default(),
            priority: 100;;}}}

impl Default for storage_provider_configStorageConfig { fn default() -> Self { Self { enabled: true,
            endpoint: None,
    health_check: HealthCheckConfig::default(),
            priority: 100;;}}}

impl Default for HealthCheckConfig { fn default() -> Self { Self { enabled: true,
            interval_seconds: 30,
            timeout_ms: 5000,
            retries: 3;}}}

impl Default for AdapterSettings { fn default() -> Self { Self { default_timeout_ms: 30000,
            max_concurrent_requests: 100,
            circuit_breaker: CircuitBreakerConfig::default(),
            enable_standalone_failover: true;;}}}

impl Default for CircuitBreakerConfig { fn default() -> Self { Self { enabled: true,
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 3;}}} 
