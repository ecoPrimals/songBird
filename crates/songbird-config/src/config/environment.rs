//! Environment-based configuration with zero hardcoded values
//!
//! All configuration values are determined dynamically from environment)
//! system capabilities, or calculated defaults.

use crate::config::constants::{
    enable_zero_copy, get_batch_size, get_bind_address, get_buffer_pool_size,
    get_connection_timeout_ms, get_dashboard_port, get_log_level, get_max_connections,
    get_primal_endpoint, get_worker_threads,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file_rotation: bool,
    pub max_file_size_mb: u32,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: get_log_level(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_rotation: true,
            max_file_size_mb: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // Endpoint suffix is intentional and clear
pub struct ServiceEndpoints {
    pub beardog_endpoint: String,
    pub nestgate_endpoint: String,
    pub toadstool_endpoint: String,
    pub squirrel_endpoint: String,
    pub discovery_endpoint: String,
    pub health_endpoint: String,
    pub metrics_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // Max prefix is intentional for limits
pub struct ResourceLimits {
    pub max_connections: usize,
    pub max_memory_mb: Option<u64>,
    pub max_cpu_cores: Option<f64>,
    pub max_file_descriptors: Option<u64>,
    pub max_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceParameters {
    pub worker_threads: usize,
    pub buffer_pool_size: usize,
    pub batch_size: usize,
    pub enable_zero_copy: bool,
    pub connection_pool_size: usize,
    pub request_timeout_ms: u64,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: get_environment(),
            bind_address: get_bind_address(),
            connection_timeout_secs: get_connection_timeout_ms() / 1000,
            require_tls: should_require_tls(),
            dashboard_port: get_dashboard_port(),
            max_connections: get_max_connections(),
            resource_limits: ResourceLimits::default(),
            log_config: LogConfig::default(),
            service_endpoints: ServiceEndpoints::default(),
            performance_config: PerformanceParameters::default(),
            discovery_ports: vec![],
            discovery_timeout_secs: 10,
            bind_port: 0, // Will be set later
            health_check_interval_secs: 30,
            enable_encryption: false,
            session_timeout_secs: 300,
            gaming_port_range: (0, 0),
            metrics_interval_secs: 60,
            log_level: "info".to_string(),
        }
    }
}

impl Default for ServiceEndpoints {
    fn default() -> Self {
        Self {
            beardog_endpoint: get_primal_endpoint("beardog"),
            nestgate_endpoint: get_primal_endpoint("nestgate"),
            toadstool_endpoint: get_primal_endpoint("toadstool"),
            squirrel_endpoint: get_primal_endpoint("squirrel"),
            discovery_endpoint: get_primal_endpoint("discovery"),
            health_endpoint: get_primal_endpoint("health"),
            metrics_endpoint: get_primal_endpoint("metrics"),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: get_max_connections(),
            max_memory_mb: get_memory_limit(),
            max_cpu_cores: get_cpu_limit(),
            max_file_descriptors: get_fd_limit(),
            max_threads: get_worker_threads() * 2, // 2x worker threads for total thread limit
        }
    }
}

impl Default for PerformanceParameters {
    fn default() -> Self {
        Self {
            worker_threads: get_worker_threads(),
            buffer_pool_size: get_buffer_pool_size(),
            batch_size: get_batch_size(),
            enable_zero_copy: enable_zero_copy(),
            connection_pool_size: get_max_connections() / 10, // 10% of max connections for pool
            request_timeout_ms: get_connection_timeout_ms(),
        }
    }
}

/// Environment configuration with complete adaptability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Current environment (production, staging, development, testing)
    pub environment: String,

    /// Bind address calculated from environment and security requirements
    pub bind_address: String,

    /// Connection timeout calculated from network conditions
    pub connection_timeout_secs: u64,

    /// Whether TLS is required (security-first approach)
    pub require_tls: bool,

    /// Dashboard port for observability interface
    pub dashboard_port: u16,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Resource limits based on system capabilities
    pub resource_limits: ResourceLimits,

    /// Log configuration based on environment
    pub log_config: LogConfig,

    /// Service endpoints dynamically configured
    pub service_endpoints: ServiceEndpoints,

    /// Performance tuning parameters
    pub performance_config: PerformanceParameters,

    // Gaming network specific fields
    /// Discovery ports for gaming network
    pub discovery_ports: Vec<u16>,

    /// Discovery timeout in seconds
    pub discovery_timeout_secs: u64,

    /// Bind port for network services
    pub bind_port: u16,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,

    /// Whether encryption is enabled
    pub enable_encryption: bool,

    /// Session timeout in seconds
    pub session_timeout_secs: u64,

    /// Gaming port range
    pub gaming_port_range: (u16, u16),

    /// Metrics collection interval in seconds
    pub metrics_interval_secs: u64,

    /// Log level setting
    pub log_level: String,
}

/// Get current environment from multiple sources
#[must_use]
pub fn get_environment() -> String {
    env::var("SONGBIRD_ENV")
        .or_else(|_| env::var("NODE_ENV"))
        .or_else(|_| env::var("RAILS_ENV"))
        .or_else(|_| env::var("ENVIRONMENT"))
        .unwrap_or_else(|_| {
            // Detect environment from system characteristics
            detect_environment_from_system()
        })
}

/// Detect environment from system characteristics and context
fn detect_environment_from_system() -> String {
    // Check for container/orchestration environments
    if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return "production".to_string();
    }

    if env::var("DOCKER_CONTAINER").is_ok() || env::var("CONTAINER").is_ok() {
        return "staging".to_string();
    }

    // Check for CI/CD environments
    if env::var("CI").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("GITLAB_CI").is_ok()
        || env::var("JENKINS_URL").is_ok()
    {
        return "testing".to_string();
    }

    // Check for development indicators
    if env::var("HOME").map(|h| h.contains("dev") || h.contains("developer")).unwrap_or(false)
        || env::var("USER").map(|u| u == "root").unwrap_or(false)
    {
        return "development".to_string();
    }

    // Default based on system characteristics
    if std::path::Path::new("/proc/version").exists() {
        // Linux system - likely server
        "production".to_string()
    } else {
        // Other systems - likely development
        "development".to_string()
    }
}

/// Determine if TLS should be required based on environment and security context
#[must_use]
pub fn should_require_tls() -> bool {
    env::var("SONGBIRD_REQUIRE_TLS").ok().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        match get_environment().as_str() {
            "production" | "staging" => true, // Always require TLS in production and staging
            "testing" | "development" => false, // Optional in testing and development for flexibility
            _ => {
                // Require TLS if we detect sensitive data or external access
                detect_tls_requirement()
            }
        }
    })
}

/// Detect if TLS should be required based on system context
fn detect_tls_requirement() -> bool {
    // Require TLS if binding to external interfaces
    let bind_address = get_bind_address();
    if bind_address == "0.0.0.0" || !bind_address.starts_with("127.") {
        return true;
    }

    // Require TLS if running with elevated privileges
    if env::var("USER").map(|u| u == "root").unwrap_or(false) || env::var("SUDO_USER").is_ok() {
        return true;
    }

    // Require TLS if external services are configured
    if env::var("DATABASE_URL").is_ok()
        || env::var("REDIS_URL").is_ok()
        || env::var("EXTERNAL_API_KEY").is_ok()
    {
        return true;
    }

    false
}

/// Get memory limit from system or container constraints
fn get_memory_limit() -> Option<u64> {
    // Check container memory limits first
    if let Ok(limit) = env::var("MEMORY_LIMIT") {
        if let Ok(mb) = limit.parse::<u64>() {
            return Some(mb);
        }
    }

    // Check cgroup memory limits
    if let Ok(limit) = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.limit_in_bytes") {
        if let Ok(bytes) = limit.trim().parse::<u64>() {
            if bytes < u64::MAX / 2 {
                // Reasonable limit
                return Some(bytes / 1024 / 1024); // Convert to MB
            }
        }
    }

    // Check system memory (Linux)
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return Some(kb / 1024); // Convert to MB
                    }
                }
            }
        }
    }

    None // Unable to determine
}

/// Get CPU limit from system or container constraints
fn get_cpu_limit() -> Option<f64> {
    // Check container CPU limits
    if let Ok(limit) = env::var("CPU_LIMIT") {
        if let Ok(cores) = limit.parse::<f64>() {
            return Some(cores);
        }
    }

    // Check cgroup CPU limits
    if let Ok(quota) = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us") {
        if let Ok(period) = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_period_us") {
            if let (Ok(quota_val), Ok(period_val)) =
                (quota.trim().parse::<i64>(), period.trim().parse::<i64>())
            {
                if quota_val > 0 && period_val > 0 {
                    #[allow(clippy::cast_precision_loss)] // CPU cores as f64 is acceptable
                    return Some(quota_val as f64 / period_val as f64);
                }
            }
        }
    }

    // Use available parallelism as fallback
    #[allow(clippy::cast_precision_loss)] // CPU cores as f64 is acceptable
    std::thread::available_parallelism().map(|n| n.get() as f64).ok()
}

/// Get file descriptor limit
fn get_fd_limit() -> Option<u64> {
    // Check ulimit for file descriptors
    if let Ok(output) = std::process::Command::new("sh").arg("-c").arg("ulimit -n").output() {
        if let Ok(limit_str) = String::from_utf8(output.stdout) {
            if let Ok(limit) = limit_str.trim().parse::<u64>() {
                return Some(limit);
            }
        }
    }

    // Check /proc/sys/fs/file-max
    if let Ok(limit) = std::fs::read_to_string("/proc/sys/fs/file-max") {
        if let Ok(max_files) = limit.trim().parse::<u64>() {
            return Some(max_files / 10); // Conservative estimate per process
        }
    }

    None
}

impl EnvironmentConfig {
    /// Create configuration optimized for current environment
    pub fn optimized(&mut self) {
        // Apply environment-specific optimizations
        match self.environment.as_str() {
            "production" => {
                self.performance_config.enable_zero_copy = true;
                self.performance_config.buffer_pool_size *= 2; // More aggressive buffering
                self.require_tls = true;
                self.max_connections *= 2; // Double max connections for high-load environments
                self.resource_limits.max_connections *= 2; // Double max connections
                self.log_config.level = "warn".to_string(); // More aggressive logging in prod
            }
            "staging" => {
                self.performance_config.buffer_pool_size =
                    (self.performance_config.buffer_pool_size * 3) / 2; // 1.5x buffering
                                                                        // Scale connections by 1.5x for staging
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                {
                    self.max_connections = (self.max_connections as f32 * 1.5) as usize;
                    self.resource_limits.max_connections =
                        (self.resource_limits.max_connections as f32 * 1.5) as usize;
                }
            }
            "development" => {
                self.performance_config.buffer_pool_size /= 2; // Less memory usage
                self.resource_limits.max_connections /= 2; // Fewer connections
                self.log_config.level = "debug".to_string(); // More verbose logging in dev
            }
            _ => {}
        }
    }

    /// Get connection timeout as Duration
    #[must_use]
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }

    /// Validate configuration consistency
    ///
    /// # Errors
    ///
    /// Returns an error if there are port conflicts or configuration inconsistencies
    pub fn validate(&self) -> Result<(), String> {
        // Validate port ranges don't conflict
        let endpoints = &self.service_endpoints;
        let mut ports = Vec::new();

        for endpoint in [
            &endpoints.beardog_endpoint,
            &endpoints.nestgate_endpoint,
            &endpoints.toadstool_endpoint,
            &endpoints.squirrel_endpoint,
            &endpoints.discovery_endpoint,
            &endpoints.health_endpoint,
            &endpoints.metrics_endpoint,
        ] {
            if let Some(port_str) = endpoint.split(':').next_back() {
                if let Some(path_start) = port_str.find('/') {
                    let port_only = &port_str[..path_start];
                    if let Ok(port) = port_only.parse::<u16>() {
                        if ports.contains(&port) {
                            return Err(format!("Port conflict detected: {port}"));
                        }
                        ports.push(port);
                    }
                } else if let Ok(port) = port_str.parse::<u16>() {
                    if ports.contains(&port) {
                        return Err(format!("Port conflict detected: {port}"));
                    }
                    ports.push(port);
                }
            }
        }

        // Validate resource limits are reasonable
        if self.resource_limits.max_connections == 0 {
            return Err("max_connections cannot be zero".to_string());
        }

        if self.performance_config.worker_threads == 0 {
            return Err("worker_threads cannot be zero".to_string());
        }

        if self.performance_config.buffer_pool_size == 0 {
            return Err("buffer_pool_size cannot be zero".to_string());
        }

        Ok(())
    }
}
