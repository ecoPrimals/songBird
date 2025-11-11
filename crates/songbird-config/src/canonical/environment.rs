//! Canonical environment types and configuration
//!
//! Unified environment definitions for deployment and configuration management

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Environment type for deployment configuration
///
/// Unified from multiple definitions across the ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging environment for testing
    Staging,
    /// Production environment
    Production,
    /// Testing environment
    Testing,
    /// Local development
    Local,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Development => write!(f, "development"),
            Self::Staging => write!(f, "staging"),
            Self::Production => write!(f, "production"),
            Self::Testing => write!(f, "testing"),
            Self::Local => write!(f, "local"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            "testing" | "test" => Ok(Self::Testing),
            "local" => Ok(Self::Local),
            _ => Err(format!("Unknown environment: {s}")),
        }
    }
}

impl Environment {
    /// Check if this is a production environment
    #[must_use]
    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }

    /// Check if this is a development environment
    #[must_use]
    pub fn is_development(self) -> bool {
        matches!(self, Self::Development | Self::Local)
    }

    /// Check if this environment should enable debug features
    #[must_use]
    pub fn enable_debug(self) -> bool {
        matches!(self, Self::Development | Self::Testing | Self::Local)
    }

    /// Get the log level for this environment
    #[must_use]
    pub fn log_level(self) -> &'static str {
        match self {
            Self::Development | Self::Local => "debug",
            Self::Testing | Self::Staging => "info",
            Self::Production => "warn",
        }
    }

    /// Detect environment from environment variable
    #[must_use]
    pub fn detect() -> Self {
        std::env::var("SONGBIRD_ENV")
            .or_else(|_| std::env::var("ENVIRONMENT"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}

// ============================================================================
// EXTENDED ENVIRONMENT CONFIG TYPES (merged from environment_config_clean.rs)
// ============================================================================

use songbird_types::SafeEnv;

/// Comprehensive environment configuration
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    /// Current environment
    pub environment: Environment,
    /// Service endpoints configuration
    pub service_endpoints: ServiceEndpoints,
    /// Logging configuration
    pub log_config: LogConfig,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Performance parameters
    pub performance_config: PerformanceParameters,
}

/// Service endpoint configuration (capability-based)
#[derive(Debug, Clone)]
pub struct ServiceEndpoints {
    /// Orchestrator endpoint
    pub orchestrator_endpoint: String,
    /// Discovery endpoint
    pub discovery_endpoint: String,
    /// Health check endpoint
    pub health_endpoint: String,
    /// Metrics endpoint
    pub metrics_endpoint: String,
}

impl ServiceEndpoints {
    /// Get endpoint by capability type instead of hardcoded primal names
    #[must_use]
    pub fn get_by_capability(capability_type: &str, default_port: u16) -> String {
        SafeEnv::get_required(&format!("{}_ENDPOINT", capability_type.to_uppercase()))
            .unwrap_or_else(|_| {
                format!("http://127.0.0.1:{default_port}")
            })
    }
}

impl Default for ServiceEndpoints {
    fn default() -> Self {
        Self {
            orchestrator_endpoint: SafeEnv::get_or_default(
                "SONGBIRD_ENDPOINT",
                "http://127.0.0.1:8080"
            ),
            discovery_endpoint: SafeEnv::get_or_default(
                "DISCOVERY_ENDPOINT",
                "http://127.0.0.1:8001"
            ),
            health_endpoint: SafeEnv::get_or_default(
                "HEALTH_ENDPOINT",
                "http://127.0.0.1:8002"
            ),
            metrics_endpoint: SafeEnv::get_or_default(
                "METRICS_ENDPOINT",
                "http://127.0.0.1:8004"
            ),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (debug, info, warn, error)
    pub level: String,
    /// Log format (json, text)
    pub format: String,
    /// Log output (stdout, file)
    pub output: String,
    /// Enable file rotation
    pub file_rotation: bool,
    /// Maximum file size in MB
    pub max_file_size_mb: u32,
}

impl Default for LogConfig {
    fn default() -> Self {
        let env = Environment::detect();
        Self {
            level: env.log_level().to_string(),
            format: SafeEnv::get_or_default("LOG_FORMAT", "json"),
            output: SafeEnv::get_or_default("LOG_OUTPUT", "stdout"),
            file_rotation: SafeEnv::get_bool("LOG_FILE_ROTATION", true),
            max_file_size_mb: u32::from(SafeEnv::get_port("LOG_MAX_FILE_SIZE_MB", 100)),
        }
    }
}

/// Resource limits configuration
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Maximum memory in MB (None = unlimited)
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU cores (None = all available)
    pub max_cpu_cores: Option<f64>,
    /// Maximum file descriptors (None = system default)
    pub max_file_descriptors: Option<u64>,
    /// Maximum threads
    pub max_threads: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: SafeEnv::get_port("MAX_CONNECTIONS", 50) as usize,
            max_memory_mb: std::env::var("MAX_MEMORY_MB")
                .ok()
                .and_then(|s| s.parse().ok()),
            max_cpu_cores: std::env::var("MAX_CPU_CORES")
                .ok()
                .and_then(|s| s.parse().ok()),
            max_file_descriptors: std::env::var("MAX_FILE_DESCRIPTORS")
                .ok()
                .and_then(|s| s.parse().ok()),
            max_threads: num_cpus::get() * 2,
        }
    }
}

/// Performance tuning parameters
#[derive(Debug, Clone)]
pub struct PerformanceParameters {
    /// Worker threads
    pub worker_threads: usize,
    /// Buffer pool size
    pub buffer_pool_size: usize,
    /// Batch size for bulk operations
    pub batch_size: usize,
    /// Enable zero-copy optimizations
    pub enable_zero_copy: bool,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
}

impl Default for PerformanceParameters {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            buffer_pool_size: SafeEnv::get_port("BUFFER_POOL_SIZE", 1024) as usize,
            batch_size: SafeEnv::get_port("BATCH_SIZE", 100) as usize,
            enable_zero_copy: SafeEnv::get_bool("ENABLE_ZERO_COPY", true),
            connection_pool_size: SafeEnv::get_port("CONNECTION_POOL_SIZE", 10) as usize,
            request_timeout_ms: u64::from(SafeEnv::get_port("REQUEST_TIMEOUT_MS", 30000)),
        }
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: Environment::detect(),
            service_endpoints: ServiceEndpoints::default(),
            log_config: LogConfig::default(),
            resource_limits: ResourceLimits::default(),
            performance_config: PerformanceParameters::default(),
        }
    }
}

impl EnvironmentConfig {
    /// Create from environment variables
    #[must_use]
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Check if running in production
    #[must_use]
    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }

    /// Check if running in development
    #[must_use]
    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdResult;
    // Removed unused imports

    #[test]
    fn test_environment_variants() -> SongbirdResult<()> {
        let envs = vec![
            Environment::Development,
            Environment::Staging,
            Environment::Production,
            Environment::Testing,
            Environment::Local,
        ];

        for env in envs {
            let serialized =
                serde_json::to_string(&env).expect("JSON serialization should succeed");
            let deserialized: Environment =
                serde_json::from_str(&serialized).expect("JSON deserialization should succeed");
            assert_eq!(env, deserialized);
        }
        Ok(())
    }

    #[test]
    fn test_environment_display() {
        assert_eq!(Environment::Development.to_string(), "development");
        assert_eq!(Environment::Staging.to_string(), "staging");
        assert_eq!(Environment::Production.to_string(), "production");
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!("development".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("prod".parse::<Environment>().unwrap(), Environment::Production);
        assert_eq!("staging".parse::<Environment>().unwrap(), Environment::Staging);
    }
}
