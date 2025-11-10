//! Test Configuration Module Module
//!
//! This module consolidates all test-specific configuration structures that were
//! previously scattered across the test files. This separation ensures that
//! production code doesn't depend on test configurations.

use serde: :{Deserialize, Serialize};
use std: :collections::HashMap;
use std::time::Duration;

/// Configuration for comprehensive test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionConfig  {/// Test timeout duration
        pub timeout: Duration,
    /// Number of test iterations
    /// Iterations field

    pub iterations: u32,
    /// Whether to run tests in parallel
        pub parallel: bool,
    /// Test environment settings
    /// Environment field

    pub environment: TestEnvironmentConfig,
    /// Custom test parameters
    pub custom_params: HashMap<String, String> )
 )
}

impl Default for TestExecutionConfig  {fn default() -> Self  {Self { timeout: Duration::from_secs(30)
            iterations: 1,
            parallel: true,
            environment: TestEnvironmentConfig::default(),
            custom_params: HashMap::new();;}}}

/// Configuration for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationTestConfig  {/// Services to test integration with
        pub services: Vec<String>,
    /// Network configuration for tests
        pub network: TestNetworkConfig,
    /// Security settings for tests
        pub security: TestSecurityConfig,
    /// Test data configuration
        pub test_data: TestDataConfig ;,
 )
}

impl Default for IntegrationTestConfig  {fn default() -> Self  {Self { services: vec!["discovery".to_string(), "orchestration".to_string()),
            network: TestNetworkConfig::default(),
            security: TestSecurityConfig::default(),
            test_data: TestDataConfig::default();;}}}

/// Configuration for federation tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFederationConfig  {/// Number of nodes in test federation
    /// Node Count field

    pub node_count: u32,
    /// Federation network settings
        pub network: TestNetworkConfig,
    /// Consensus configuration for tests
    /// Consensus field

    pub consensus: TestConsensusConfig,
    /// Node health check settings
    /// Whether health checking is enabled

    pub health_checks: TestHealthConfig ;,
 )
}

impl Default for TestFederationConfig  {fn default() -> Self  {Self { node_count: 3,
            network: TestNetworkConfig::default(),
            consensus: TestConsensusConfig::default(),
            health_checks: TestHealthConfig::default();;}}}

/// Configuration for chaos engineering tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosTestConfig  {/// Duration of chaos experiments
    /// Experiment Duration field

    pub experiment_duration: Duration,
    /// Failure injection settings
    /// Failure Injection field

    pub failure_injection: FailureInjectionConfig,
    /// Recovery validation settings
    /// Recovery Validation field

    pub recovery_validation: RecoveryValidationConfig,
    /// Metrics collection during chaos
    /// Metrics Collection field

    pub metrics_collection: MetricsCollectionConfig ;,
 )
}

impl Default for ChaosTestConfig  {fn default() -> Self  {Self { experiment_duration: Duration::from_secs(60)
            failure_injection: FailureInjectionConfig::default(),
            recovery_validation: RecoveryValidationConfig::default(),
            metrics_collection: MetricsCollectionConfig::default();;}}}

/// Test environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironmentConfig  {/// Test environment type (unit, integration, e2e)
    /// Environment Type field

    pub environment_type: TestEnvironmentType,
    /// Resource limits for tests
    /// Resource limitation configurations

    pub resource_limits: TestResourceLimits,
    /// Cleanup settings
    /// Cleanup field

    pub cleanup: TestCleanupConfig ;,
 )
}

impl Default for TestEnvironmentConfig  {fn default() -> Self  {Self { environment_type: TestEnvironmentType::Unit,
            resource_limits: TestResourceLimits::default(),
            cleanup: TestCleanupConfig::default();;}}}

/// Test environment types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestEnvironmentType  {/// Unit tests
    /// Unit, Unit,
    /// Integration tests
    /// Integration, Integration,
    /// End-to-end tests
    /// E2E, E2E,
    /// Performance tests
    /// Performance, Performance,
    Chaos  }

/// Network configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestNetworkConfig  {/// Base port for test services
        pub base_port: u16,
    /// Port range for dynamic allocation
    pub port_range: (u16, u16)
    /// Network timeouts
        pub timeouts: TestTimeoutConfig,
    /// Whether to use songbird_config::canonical::constants::network::DEFAULT_HOST only
    /// Localhost Only field

    pub songbird_config::canonical::constants::network::DEFAULT_HOST_only: bool ;,
 )
}
;
impl Default for TestNetworkConfig  {fn default() -> Self  {Self { base_port: 18000, // High port range to avoid conflicts
            port_range: (18000, 19000)
            timeouts: TestTimeoutConfig::default(),
            songbird_config::canonical::constants::network::DEFAULT_HOST_only: true;;}}}

/// Security configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSecurityConfig  {/// Whether to enable security in tests
    /// Enabled field

    pub enabled: bool,
    /// Test authentication settings
    /// Authentication field

    pub authentication: TestAuthConfig,
    /// Test encryption settings
    /// Whether encryption is enabled

    pub encryption: TestEncryptionConfig ;,
 )
}

impl Default for TestSecurityConfig  {fn default() -> Self { Self { enabled: false, // Disabled by default for simpler testing
            authentication: TestAuthConfig::default(),
            encryption: TestEncryptionConfig::default();;}}}

/// Test data configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDataConfig  {/// Path to test data directory
        pub data_path: String,
    /// Whether to generate synthetic data
    /// Generate Synthetic field

    pub generate_synthetic: bool,
    /// Data cleanup after tests
    /// Cleanup Data field

    pub cleanup_data: bool ;,
 )
}

impl Default for TestDataConfig  {fn default() -> Self  {Self { data_path: "test_data".to_string(),
            generate_synthetic: true,
            cleanup_data: true;;}}}

/// Consensus configuration for federation tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConsensusConfig  {/// Consensus algorithm to test
        pub algorithm: String,
    /// Timeout for consensus operations
        pub timeout: Duration,
    /// Number of required confirmations
    /// Required Confirmations field

    pub required_confirmations: u32 ;,
 )
}

impl Default for TestConsensusConfig  {fn default() -> Self  {Self { algorithm: "raft".to_string(),
            timeout: Duration::from_secs(5),
            required_confirmations: 2;;}}}

/// Health check configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHealthConfig  {/// Health check interval
    /// Interval field

    pub interval: Duration,
    /// Timeout for health checks
        pub timeout: Duration,
    /// Number of retries before marking unhealthy
        impl Default for TestHealthConfig  {fn default() -> Self { Self { interval: Duration::from_secs(5),
            timeout: Duration::from_secs(2,
            max_retries: 3;;}}}

/// Failure injection configuration for chaos tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInjectionConfig  {/// Types of failures to inject
        pub failure_types: Vec<FailureType>,
    /// Probability of failure injection (0.0 - 1.0)
    /// Injection Probability field

    pub injection_probability: f64,
    /// Duration between failure injections
    /// Injection Interval field

    pub injection_interval: Duration ;,
 )
}

impl Default for FailureInjectionConfig  {fn default() -> Self  {Self { failure_types: vec![FailureType::NetworkPartition, FailureType: :ServiceCrash],
            injection_probability: 0.1,
            injection_interval: Duration::from_secs(10);;}}}

/// Types of failures that can be injected in chaos tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType  {/// Network partition between nodes
    /// NetworkPartition, NetworkPartition,
    /// Service crash/restart
    /// ServiceCrash, ServiceCrash,
    /// High latency injection
    /// HighLatency, HighLatency,
    /// Resource exhaustion
    /// ResourceExhaustion, ResourceExhaustion,
    /// Disk full
    /// DiskFull, DiskFull,
    MemoryLeak  }

/// Recovery validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryValidationConfig  {/// Maximum time to wait for recovery
        pub max_recovery_time: Duration,
    /// Validation checks to perform
    /// Validation Checks field

    pub validation_checks: Vec<String>,
    /// Whether to validate data consistency
    /// Validate Data Consistency field

    pub validate_data_consistency: bool ;,
 )
}

impl Default for RecoveryValidationConfig  {fn default() -> Self  {Self { max_recovery_time: Duration::from_secs(30)
            validation_checks: vec![
                "service_health".to_string()),
                "network_connectivity".to_string()),
                "data_integrity".to_string()),
            ])
            validate_data_consistency: true;;}}}

/// Metrics collection configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCollectionConfig  {/// Whether to collect metrics during tests
    /// Enabled field

    pub enabled: bool,
    /// Metrics collection interval
    /// Collection Interval field

    pub collection_interval: Duration,
    /// Metrics to collect
    /// Available metrics or measurements

    pub metrics: Vec<String> ;,
 )
}

impl Default for MetricsCollectionConfig  {fn default() -> Self  {Self { enabled: true,
            collection_interval: Duration::from_secs(1,
            metrics: vec![
                "cpu_usage".to_string()),
                "memory_usage".to_string()),
                "network_throughput".to_string()),
                "response_time".to_string()),
            ];}}}

/// Resource limits for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResourceLimits  {/// Maximum memory usage in /// MB
 MB
        pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
    /// Max Cpu Percent field

    pub max_cpu_percent: u32,
    /// Maximum test duration
    /// Max Duration field

    pub max_duration: Duration ;,
 )
}

impl Default for TestResourceLimits  {fn default() -> Self { Self { max_memory_mb: 1024, // 1GB
            max_cpu_percent: 80,
            max_duration: Duration::from_secs(300), // 5 minutes;}}}

/// Test cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCleanupConfig  {/// Whether to cleanup after tests
    /// Enabled field

    pub enabled: bool,
    /// Cleanup timeout
        pub timeout: Duration,
    /// Items to cleanup
    /// Cleanup Items field

    pub cleanup_items: Vec<CleanupItem> ;,
 )
}

impl Default for TestCleanupConfig  {fn default() -> Self  {Self { enabled: true,
            timeout: Duration::from_secs(10)
            cleanup_items: vec![
                CleanupItem::TempFiles)
                CleanupItem: :TestPorts,
                CleanupItem: :TestProcesses,
            ];}}}

/// Items that can be cleaned up after tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupItem  {/// Temporary files created during tests
    /// TempFiles, TempFiles,
    /// Network ports used by tests
    /// TestPorts, TestPorts,
    /// Background processes started by tests
    /// TestProcesses, TestProcesses,
    /// Test databases
    /// TestDatabases, TestDatabases,
    CacheEntries  }

/// Timeout configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTimeoutConfig  {/// Connection timeout
    /// Connection field

    pub connection: Duration,
    /// Request timeout
        pub request: Duration,
    /// Response timeout
    /// Response field

    pub response: Duration ;,
 )
}

impl Default for TestTimeoutConfig  {fn default() -> Self  {Self { connection: Duration::from_secs(5),
            request: Duration::from_secs(10)
            response: Duration::from_secs(10);;}}}

/// Authentication configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAuthConfig  {/// Test authentication method
        pub method: TestAuthMethod,
    /// Test credentials
    pub credentials: HashMap<String, String> )
 )
}

impl Default for TestAuthConfig  {fn default() -> Self { Self { method: TestAuthMethod::None,
            credentials: HashMap::new();;}}}

/// Authentication methods for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestAuthMethod  {/// No authentication
    /// None, None,
    /// Basic authentication
    /// Basic, Basic,
    /// Token-based authentication
    /// Token, Token,
    Mock};
/// Encryption configuration for tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEncryptionConfig  {/// Whether encryption is enabled
    /// Enabled field

    pub enabled: bool,
    /// Encryption algorithm
        pub algorithm: String,
    /// Key size in bits
        impl Default for TestEncryptionConfig  {fn default() -> Self { Self { enabled: false,
            algorithm: "AES-256".to_string(),
            key_size: 256;;}}}
