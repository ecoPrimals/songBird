// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical environment types and configuration
//!
//! Unified environment definitions for deployment and configuration management

use serde::{Deserialize, Serialize};

use super::constants::read_process_env;

fn env_get_or_default(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

fn env_get_bool(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: bool,
) -> bool {
    env(key)
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => v.parse().ok(),
        })
        .unwrap_or(default)
}

fn env_get_port(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: u16,
) -> u16 {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// **CANONICAL**: Environment type for deployment configuration
///
/// Unified from multiple definitions across the ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Environment {
    /// Development environment
    #[default]
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
    pub const fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }

    /// Check if this is a development environment
    #[must_use]
    pub const fn is_development(self) -> bool {
        matches!(self, Self::Development | Self::Local)
    }

    /// Check if this environment should enable debug features
    #[must_use]
    pub const fn enable_debug(self) -> bool {
        matches!(self, Self::Development | Self::Testing | Self::Local)
    }

    /// Get the log level for this environment
    #[must_use]
    pub const fn log_level(self) -> &'static str {
        match self {
            Self::Development | Self::Local => "debug",
            Self::Testing | Self::Staging => "info",
            Self::Production => "warn",
        }
    }

    /// Detect environment from environment variable
    #[must_use]
    pub fn detect() -> Self {
        Self::detect_with(read_process_env)
    }

    /// Detect environment using an injectable env reader (e.g. for tests).
    #[must_use]
    pub fn detect_with(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        env("SONGBIRD_ENV")
            .or_else(|_| env("ENVIRONMENT"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}

// ============================================================================
// EXTENDED ENVIRONMENT CONFIG TYPES (merged from environment_config_clean.rs)
// ============================================================================

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
        Self::get_by_capability_with(read_process_env, capability_type, default_port)
    }

    /// Same as [`get_by_capability`](Self::get_by_capability) with an injectable env reader.
    #[must_use]
    pub fn get_by_capability_with(
        env: impl Fn(&str) -> Result<String, std::env::VarError>,
        capability_type: &str,
        default_port: u16,
    ) -> String {
        let key = format!("{}_ENDPOINT", capability_type.to_uppercase());
        env(&key).unwrap_or_else(|_| {
            format!("http://{}:{default_port}", songbird_types::constants::LOCALHOST)
        })
    }

    /// Build from environment using an injectable reader.
    #[must_use]
    pub fn from_env_reader(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        Self {
            orchestrator_endpoint: env_get_or_default(
                &env,
                "SONGBIRD_ENDPOINT",
                format!("http://{}:8080", songbird_types::constants::LOCALHOST),
            ),
            discovery_endpoint: env_get_or_default(
                &env,
                "DISCOVERY_ENDPOINT",
                format!("http://{}:8001", songbird_types::constants::LOCALHOST),
            ),
            health_endpoint: env_get_or_default(
                &env,
                "HEALTH_ENDPOINT",
                format!("http://{}:8002", songbird_types::constants::LOCALHOST),
            ),
            metrics_endpoint: env_get_or_default(
                &env,
                "METRICS_ENDPOINT",
                format!("http://{}:8004", songbird_types::constants::LOCALHOST),
            ),
        }
    }
}

impl Default for ServiceEndpoints {
    fn default() -> Self {
        Self::from_env_reader(read_process_env)
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

impl LogConfig {
    /// Build from environment using an injectable reader.
    #[must_use]
    pub fn from_env_reader(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        let profile = Environment::detect_with(&env);
        Self {
            level: profile.log_level().to_string(),
            format: env_get_or_default(&env, "LOG_FORMAT", "json"),
            output: env_get_or_default(&env, "LOG_OUTPUT", "stdout"),
            file_rotation: env_get_bool(&env, "LOG_FILE_ROTATION", true),
            max_file_size_mb: u32::from(env_get_port(&env, "LOG_MAX_FILE_SIZE_MB", 100)),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::from_env_reader(read_process_env)
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

impl ResourceLimits {
    /// Build from environment using an injectable reader.
    #[must_use]
    pub fn from_env_reader(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        Self {
            max_connections: env_get_port(&env, "MAX_CONNECTIONS", 50) as usize,
            max_memory_mb: env("MAX_MEMORY_MB").ok().and_then(|s| s.parse().ok()),
            max_cpu_cores: env("MAX_CPU_CORES").ok().and_then(|s| s.parse().ok()),
            max_file_descriptors: env("MAX_FILE_DESCRIPTORS").ok().and_then(|s| s.parse().ok()),
            max_threads: std::thread::available_parallelism().map_or(1, std::num::NonZero::get) * 2,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::from_env_reader(read_process_env)
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

impl PerformanceParameters {
    /// Build from environment using an injectable reader.
    #[must_use]
    pub fn from_env_reader(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        Self {
            worker_threads: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
            buffer_pool_size: env_get_port(&env, "BUFFER_POOL_SIZE", 1024) as usize,
            batch_size: env_get_port(&env, "BATCH_SIZE", 100) as usize,
            enable_zero_copy: env_get_bool(&env, "ENABLE_ZERO_COPY", true),
            connection_pool_size: env_get_port(&env, "CONNECTION_POOL_SIZE", 10) as usize,
            request_timeout_ms: u64::from(env_get_port(&env, "REQUEST_TIMEOUT_MS", 30000)),
        }
    }
}

impl Default for PerformanceParameters {
    fn default() -> Self {
        Self::from_env_reader(read_process_env)
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self::from_env_reader(read_process_env)
    }
}

impl EnvironmentConfig {
    /// Create from environment variables
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_reader(read_process_env)
    }

    /// Create from an injectable env reader (tests avoid mutating process environment).
    #[must_use]
    pub fn from_env_reader(env: impl Fn(&str) -> Result<String, std::env::VarError>) -> Self {
        Self {
            environment: Environment::detect_with(&env),
            service_endpoints: ServiceEndpoints::from_env_reader(&env),
            log_config: LogConfig::from_env_reader(&env),
            resource_limits: ResourceLimits::from_env_reader(&env),
            performance_config: PerformanceParameters::from_env_reader(&env),
        }
    }

    /// Check if running in production
    #[must_use]
    pub const fn is_production(&self) -> bool {
        self.environment.is_production()
    }

    /// Check if running in development
    #[must_use]
    pub const fn is_development(&self) -> bool {
        self.environment.is_development()
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use songbird_types::SongbirdResult;

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

    #[test]
    fn test_environment_from_str_error() {
        assert!("unknown-env-xyz".parse::<Environment>().is_err());
    }

    #[test]
    fn test_environment_from_str_aliases() {
        assert_eq!("dev".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("test".parse::<Environment>().unwrap(), Environment::Testing);
        assert_eq!("local".parse::<Environment>().unwrap(), Environment::Local);
    }

    #[test]
    fn test_environment_helpers() {
        assert!(Environment::Production.is_production());
        assert!(!Environment::Staging.is_production());
        assert!(!Environment::Production.enable_debug());
        assert!(Environment::Testing.enable_debug());
        assert!(Environment::Development.is_development());
        assert!(Environment::Local.is_development());
        assert!(!Environment::Production.is_development());
        assert_eq!(Environment::Production.log_level(), "warn");
    }

    #[test]
    fn test_environment_detect_songbird_env() {
        let env = |key: &str| match key {
            "SONGBIRD_ENV" => Ok("staging".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        };
        assert_eq!(Environment::detect_with(env), Environment::Staging);
    }

    #[test]
    fn test_environment_detect_environment_fallback() {
        let env = |key: &str| match key {
            "ENVIRONMENT" => Ok("production".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        };
        assert_eq!(Environment::detect_with(env), Environment::Production);
    }

    #[test]
    fn test_service_endpoints_get_by_capability() {
        let env = |key: &str| match key {
            "FOO_ENDPOINT" => Ok("http://cap:1".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        };
        let ep = ServiceEndpoints::get_by_capability_with(env, "foo", 9999);
        assert_eq!(ep, "http://cap:1");
    }

    #[test]
    fn test_service_endpoints_get_by_capability_default_port() {
        let ep = ServiceEndpoints::get_by_capability("missingcap_xyz", 4242);
        assert_eq!(ep, "http://127.0.0.1:4242");
    }

    #[test]
    fn test_environment_config_from_env_matches_detect() {
        let env = |key: &str| match key {
            "SONGBIRD_ENV" => Ok("production".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        };
        let cfg = EnvironmentConfig::from_env_reader(env);
        let detected = Environment::detect_with(|key| match key {
            "SONGBIRD_ENV" => Ok("production".to_string()),
            _ => Err(std::env::VarError::NotPresent),
        });
        assert_eq!(cfg.environment, detected);
        assert_eq!(cfg.is_production(), cfg.environment.is_production());
        assert_eq!(cfg.is_development(), cfg.environment.is_development());
    }
}
