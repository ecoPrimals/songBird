//! Configuration module for Songbird Orchestrator
//!
//! Re-exports configuration types and provides implementation modules.

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::errors::{Result, SongbirdError};
use crate::config::constants::network;

// Re-export types from traits module

// Configuration provider implementations will go here
pub mod providers;
pub mod environment;
pub mod constants;

use self::environment::{EnvironmentConfig, EnvironmentAware};
use self::constants::{services, health, monitoring, logging};

/// Generic configuration provider trait
///
/// This trait allows the orchestrator to work with any configuration source
/// including files, environment variables, Consul, etcd, etc.
#[async_trait]
pub trait ConfigProvider<T>: Send + Sync {
    /// Load configuration from the provider
    async fn load_config(&self) -> Result<T>;

    /// Reload configuration (useful for file-based configs)
    async fn reload_config(&self) -> Result<T>;

    /// Watch for configuration changes
    async fn watch_config(&self) -> impl Stream<Item = Result<T>>;

    /// Validate configuration before loading
    async fn validate_config(&self, config: &T) -> Result<()>;

    /// Get provider information
    fn provider_info(&self) -> ConfigProviderInfo;
}

/// Information about a configuration provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProviderInfo {
    pub name: String,
    pub provider_type: String,
    pub description: String,
    pub supports_watch: bool,
    pub supports_reload: bool,
}

/// Core orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig<T = DefaultServiceConfig> {
    /// Core orchestrator settings
    pub orchestrator: CoreOrchestratorConfig,

    /// Service-specific configuration
    pub services: ServiceConfig<T>,

    /// Network configuration
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Monitoring configuration
    pub monitoring: MonitoringConfig,

    /// Discovery configuration
    pub discovery: DiscoveryConfig,

    /// Load balancing configuration
    pub load_balancing: LoadBalancingConfig,

    /// Health monitoring configuration
    pub health: HealthConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,
}

/// Observability configuration for built-in monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Enable observability features
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_secs: u64,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Enable simple web dashboard
    pub enable_dashboard: bool,
    /// Dashboard port
    pub dashboard_port: u16,
    /// Export Prometheus-compatible metrics
    pub export_prometheus: bool,
    /// Maximum number of metric data points to keep in memory
    pub max_metric_history: usize,
    /// Enable detailed system metrics
    pub enable_system_metrics: bool,
    /// Enable service-level metrics
    pub enable_service_metrics: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval_secs: 30,
            health_check_interval_secs: 60,
            enable_dashboard: false,
            dashboard_port: 8081,
            export_prometheus: false,
            max_metric_history: 1000,
            enable_system_metrics: true,
            enable_service_metrics: true,
        }
    }
}

impl EnvironmentAware for ObservabilityConfig {
    fn from_env() -> Self {
        Self::from_env_with_config(&EnvironmentConfig::default())
    }
    
    fn from_env_with_config(env_config: &EnvironmentConfig) -> Self {
        let defaults = Self::default();
        Self {
            enabled: env_config.get_bool_env("SONGBIRD_OBSERVABILITY_ENABLED", defaults.enabled),
            metrics_interval_secs: env_config.get_env_or("SONGBIRD_METRICS_INTERVAL_SECS", defaults.metrics_interval_secs),
            health_check_interval_secs: env_config.get_env_or("SONGBIRD_HEALTH_CHECK_INTERVAL_SECS", defaults.health_check_interval_secs),
            enable_dashboard: env_config.get_bool_env("SONGBIRD_ENABLE_DASHBOARD", defaults.enable_dashboard),
            dashboard_port: env_config.get_env_or("SONGBIRD_DASHBOARD_PORT", defaults.dashboard_port),
            export_prometheus: env_config.get_bool_env("SONGBIRD_EXPORT_PROMETHEUS", defaults.export_prometheus),
            max_metric_history: env_config.get_env_or("SONGBIRD_MAX_METRIC_HISTORY", defaults.max_metric_history),
            enable_system_metrics: env_config.get_bool_env("SONGBIRD_ENABLE_SYSTEM_METRICS", defaults.enable_system_metrics),
            enable_service_metrics: env_config.get_bool_env("SONGBIRD_ENABLE_SERVICE_METRICS", defaults.enable_service_metrics),
        }
    }
}

/// Core orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreOrchestratorConfig {
    /// Orchestrator instance ID
    pub id: String,

    /// Bind address for the orchestrator API
    pub bind_address: String,

    /// Port for the orchestrator API
    pub port: u16,

    /// Maximum number of services to manage
    pub max_services: usize,

    /// Health check interval
    pub health_check_interval: Duration,

    /// Service startup timeout
    pub service_startup_timeout: Duration,

    /// Service shutdown timeout
    pub service_shutdown_timeout: Duration,

    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Metrics collection interval
    pub metrics_interval: Duration,

    /// Log level
    pub log_level: String,

    /// Additional orchestrator metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for CoreOrchestratorConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            bind_address: network::DEFAULT_BIND_ADDRESS.to_string(),
            port: network::DEFAULT_PORT,
            max_services: services::DEFAULT_MAX_SERVICES,
            health_check_interval: health::DEFAULT_CHECK_INTERVAL,
            service_startup_timeout: services::DEFAULT_STARTUP_TIMEOUT,
            service_shutdown_timeout: services::DEFAULT_SHUTDOWN_TIMEOUT,
            enable_metrics: true,
            metrics_interval: monitoring::DEFAULT_METRICS_INTERVAL,
            log_level: logging::DEFAULT_LOG_LEVEL.to_string(),
            metadata: HashMap::new(),
        }
    }
}

impl EnvironmentAware for CoreOrchestratorConfig {
    fn from_env() -> Self {
        Self::from_env_with_config(&EnvironmentConfig::default())
    }
    
    fn from_env_with_config(env_config: &EnvironmentConfig) -> Self {
        let defaults = Self::default();
        Self {
            id: env_config.get_env_or("orchestrator_id", defaults.id),
            bind_address: env_config.get_env_or("bind_address", defaults.bind_address),
            port: env_config.get_env_or("port", defaults.port),
            max_services: env_config.get_env_or("max_services", defaults.max_services),
            health_check_interval: env_config.get_duration_env("health_check_interval", defaults.health_check_interval),
            service_startup_timeout: env_config.get_duration_env("service_startup_timeout", defaults.service_startup_timeout),
            service_shutdown_timeout: env_config.get_duration_env("service_shutdown_timeout", defaults.service_shutdown_timeout),
            enable_metrics: env_config.get_bool_env("enable_metrics", defaults.enable_metrics),
            metrics_interval: env_config.get_duration_env("metrics_interval", defaults.metrics_interval),
            log_level: env_config.get_env_or("log_level", defaults.log_level),
            metadata: defaults.metadata, // TODO: Support env var for metadata
        }
    }
}

/// Service configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig<T> {
    /// Default service configuration
    pub default: T,

    /// Service-specific overrides
    pub overrides: HashMap<String, T>,

    /// Service discovery settings
    pub discovery: ServiceDiscoveryConfig,

    /// Service deployment settings
    pub deployment: ServiceDeploymentConfig,
}

/// Default service configuration (used when no specific type is provided)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultServiceConfig {
    pub enabled: bool,
    pub restart_policy: RestartPolicy,
    pub resource_limits: ResourceLimits,
    pub environment: HashMap<String, String>,
}

impl Default for DefaultServiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            restart_policy: RestartPolicy::Always,
            resource_limits: ResourceLimits::default(),
            environment: HashMap::new(),
        }
    }
}

/// Service restart policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
    UnlessStopped,
}

/// Resource limits for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_connections: Option<u32>,
    pub max_requests_per_second: Option<f64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: None,
            max_cpu_percent: None,
            max_connections: Some(1000),
            max_requests_per_second: None,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network interface to bind to
    pub interface: String,

    /// Port range for services
    pub port_range: (u16, u16),

    /// Enable TLS
    pub enable_tls: bool,

    /// TLS certificate path
    pub tls_cert_path: Option<PathBuf>,

    /// TLS key path
    pub tls_key_path: Option<PathBuf>,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Enable HTTP/2
    pub enable_http2: bool,

    /// CORS settings
    pub cors: CorsConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        // Enable TLS by default in production environments
        let enable_tls_default = match std::env::var("SONGBIRD_ENVIRONMENT").as_deref() {
            Ok("production") | Ok("prod") => true,
            Ok("staging") => true,
            _ => false,  // Development/test environments
        };

        Self {
            interface: network::DEFAULT_BIND_ADDRESS.to_string(),
            port_range: (8000, 9000),
            enable_tls: enable_tls_default,
            tls_cert_path: None,
            tls_key_path: None,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            enable_http2: true,
            cors: CorsConfig::default(),
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_origins: vec![],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
            ],
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_auth: bool,

    /// Authentication provider
    pub auth_provider: AuthProviderConfig,

    /// Enable authorization
    pub enable_authz: bool,

    /// Authorization provider
    pub authz_provider: AuthzProviderConfig,

    /// API key for service-to-service communication
    pub api_key: Option<String>,

    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,

    /// Audit logging configuration
    pub audit_logging: AuditConfig,
}

/// Authentication provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProviderConfig {
    pub provider_type: String,
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for AuthProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: "none".to_string(),
            config: HashMap::new(),
        }
    }
}

/// Authorization provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzProviderConfig {
    pub provider_type: String,
    pub config: HashMap<String, serde_json::Value>,
}

impl Default for AuthzProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: "none".to_string(),
            config: HashMap::new(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub per_service_limits: HashMap<String, u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_minute: 1000,
            burst_size: 100,
            per_service_limits: HashMap::new(),
        }
    }
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_requests: bool,
    pub log_responses: bool,
    pub log_failures: bool,
    pub log_file: Option<PathBuf>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_requests: true,
            log_responses: false,
            log_failures: true,
            log_file: None,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics
    pub enable_prometheus: bool,

    /// Prometheus metrics endpoint
    pub prometheus_endpoint: String,

    /// Metrics collection interval
    pub metrics_interval: Duration,

    /// Custom metrics configuration
    pub custom_metrics: HashMap<String, MetricConfig>,

    /// Tracing configuration
    pub tracing: TracingConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_prometheus: true,
            prometheus_endpoint: "/metrics".to_string(),
            metrics_interval: Duration::from_secs(30),
            custom_metrics: HashMap::new(),
            tracing: TracingConfig::default(),
        }
    }
}

/// Metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    pub metric_type: String,
    pub description: String,
    pub labels: Vec<String>,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub sample_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            sample_rate: 0.1,
        }
    }
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Discovery backend type
    pub backend: String,

    /// Backend-specific configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Service registration TTL
    pub registration_ttl: Duration,

    /// Health check configuration
    pub health_check: DiscoveryHealthConfig,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            backend: "static".to_string(),
            config: HashMap::new(),
            registration_ttl: Duration::from_secs(30),
            health_check: DiscoveryHealthConfig::default(),
        }
    }
}

/// Discovery health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryHealthConfig {
    pub enabled: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
}

impl Default for DiscoveryHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
        }
    }
}

/// Service discovery configuration within service config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Service deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDeploymentConfig {
    pub auto_start: bool,
    pub restart_on_failure: bool,
    pub max_restart_attempts: u32,
    pub restart_backoff: Duration,
}

impl Default for ServiceDeploymentConfig {
    fn default() -> Self {
        Self {
            auto_start: true,
            restart_on_failure: true,
            max_restart_attempts: 3,
            restart_backoff: Duration::from_secs(10),
        }
    }
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: String,

    /// Algorithm-specific configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Health check integration
    pub health_aware: bool,

    /// Sticky sessions configuration
    pub sticky_sessions: StickySessionsConfig,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            algorithm: "round_robin".to_string(),
            config: HashMap::new(),
            health_aware: true,
            sticky_sessions: StickySessionsConfig::default(),
        }
    }
}

/// Sticky sessions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickySessionsConfig {
    pub enabled: bool,
    pub cookie_name: String,
    pub session_timeout: Duration,
}

impl Default for StickySessionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cookie_name: "SONGBIRD_SESSION".to_string(),
            session_timeout: Duration::from_secs(3600),
        }
    }
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Global health check interval
    pub check_interval: Duration,

    /// Health check timeout
    pub check_timeout: Duration,

    /// Failure threshold before marking unhealthy
    pub failure_threshold: u32,

    /// Recovery threshold before marking healthy
    pub recovery_threshold: u32,

    /// Health check endpoints
    pub endpoints: Vec<HealthEndpointConfig>,

    /// Custom health checks
    pub custom_checks: HashMap<String, CustomHealthCheckConfig>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            endpoints: vec![HealthEndpointConfig {
                path: "/health".to_string(),
                method: "GET".to_string(),
                expected_status: 200,
            }],
            custom_checks: HashMap::new(),
        }
    }
}

/// Health endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEndpointConfig {
    pub path: String,
    pub method: String,
    pub expected_status: u16,
}

/// Custom health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHealthCheckConfig {
    pub check_type: String,
    pub config: HashMap<String, serde_json::Value>,
    pub interval: Duration,
    pub timeout: Duration,
}

/// Default implementation for the main configuration
impl<T: Default> Default for OrchestratorConfig<T> {
    fn default() -> Self {
        Self {
            orchestrator: CoreOrchestratorConfig::default(),
            services: ServiceConfig {
                default: T::default(),
                overrides: HashMap::new(),
                discovery: ServiceDiscoveryConfig::default(),
                deployment: ServiceDeploymentConfig::default(),
            },
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            discovery: DiscoveryConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            health: HealthConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

/// Utility functions for configuration
impl<T> OrchestratorConfig<T> {
    /// Get configuration for a specific service
    pub fn get_service_config(&self, service_id: &str) -> &T {
        self.services
            .overrides
            .get(service_id)
            .unwrap_or(&self.services.default)
    }

    /// Check if the configuration is secure
    pub fn is_secure(&self) -> bool {
        self.security.enable_auth && self.security.enable_authz && self.network.enable_tls
    }

    /// Get the full bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.network.interface, self.orchestrator.port)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Port validation
        if self.orchestrator.port == 0 {
            return Err(SongbirdError::ValidationFailed {
                field: "orchestrator.port".to_string(),
                issue: "Port cannot be zero".to_string(),
            });
        }

        // Port range validation
        if self.network.port_range.0 >= self.network.port_range.1 {
            return Err(SongbirdError::ValidationFailed {
                field: "network.port_range".to_string(),
                issue: "Start port must be less than end port".to_string(),
            });
        }

        // TLS validation
        if self.network.enable_tls
            && (self.network.tls_cert_path.is_none() || self.network.tls_key_path.is_none())
        {
            return Err(SongbirdError::ValidationFailed {
                field: "network.tls".to_string(),
                issue: "TLS enabled but certificate or key path not provided".to_string(),
            });
        }

        Ok(())
    }
}
