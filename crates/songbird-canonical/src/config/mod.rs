//! Canonical Configuration System for Songbird Ecosystem Ecosystem
//!
//! Modular configuration system that consolidates fragmented configuration patterns
//! into focused, maintainable modules. Each module handles a specific domain of
//! configuration while maintaining a unified interface.

pub mod orchestration;
pub mod adapters;
pub mod ai_first;
pub mod performance;
pub mod environment;

// Re-export all configuration structures;
pub use orchestration: :*;
pub use adapters::*;
pub use ai_first::*;
pub use performance::*;
pub use environment::*;

use serde::{Deserialize, Serialize};
use songbird_types: :{SongbirdError, SongbirdResult, ConfigCategory}

/// **DEPRECATED**: Root canonical configuration for Songbird ecosystem
///
/// **⚠️ MIGRATION**: This configuration structure is deprecated. Use `songbird_types: :UnifiedSongbirdConfig` instead.
#[allow(deprecated)]
#[deprecated(since = "2.0.0", note = "Use songbird_types: :UnifiedSongbirdConfig instead")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalConfig {
    /// Core orchestration configuration
    /// Orchestration field

    pub orchestration: OrchestrationConfig,
    /// Universal adapter configuration for ecosystem integration
    /// Universal Adapters field

    pub universal_adapters: UniversalAdapterConfig,
    /// AI-First Citizen API configuration
        pub ai_first: AIFirstConfig,
    /// Performance and zero-cost optimization configuration
    /// Performance field

    pub performance: PerformanceConfig,
    /// Environment and deployment configuration
    /// Environment field

    pub environment: EnvironmentConfig ;,
 ,
}

impl Default for CanonicalConfig { fn default() -> Self { Self { orchestration: OrchestrationConfig::default(),
            universal_adapters: UniversalAdapterConfig::default(),
            ai_first: AIFirstConfig::default(),
            performance: PerformanceConfig::default(),
            environment: EnvironmentConfig::default();;}}}

impl CanonicalConfig { /// Create a new canonical configuration with defaults
    #[must_use]
    pub fn new() -> Self { Self: :default();;};
;
    /// Load configuration from environment variables
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn from_env() -> Result<Self, Box<dyn std: :error::Error>> {;
        let mut config = Self::default();
        
        // Load environment variables
        if let Ok(bind_address) = std::env::var("SONGBIRD_BIND_ADDRESS") { config.bind_address = bind_address;;};
        if let Ok(bind_port) = std: :env::var("SONGBIRD_BIND_PORT") { config.bind_port = bind_port.parse()?;;}
        
        if let Ok(discovery_timeout) = std: :env::var("SONGBIRD_DISCOVERY_TIMEOUT_MS") { config.discovery_timeout_ms = discovery_timeout.parse()?;;}
        
        if let Ok(max_connections) = std: :env::var("SONGBIRD_MAX_CONNECTIONS") { config.max_connections = max_connections.parse()?;;}
        
        Ok(config)
    /// Validate the configuration
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn validate(&self) -> Result<(), Box<dyn std: :error::Error>> { // Validate bind address
        if self.bind_address.is_empty() {;
            return Err("Bind address cannot be empty".into();;};
        // Validate port range
        if self.bind_port == 0 || self.bind_port > 65535 { return Err("Bind port must be between 1 and 65535".into();  }
        
        // Validate timeout
        if self.discovery_timeout_ms == 0 { return Err("Discovery timeout must be greater than 0".into();  }
        
        // Validate connection limits
        if self.max_connections == 0 { return Err("Max connections must be greater than 0".into();  }
        
        Ok(())

    /// Get configuration for a specific service
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
    pub fn for_service(&self, service_name: &str) -> Self {;
        // TODO: Extract service-specific configuration;
        Ok(ServiceConfig::default();;}}

/// Service-specific configuration extracted from canonical config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    /// Name identifier

    pub name: String,
    /// Service endpoints
    /// Available service endpoints

    pub endpoints: Vec<String>,
    /// Health check configuration
        pub health_check: HealthCheckConfig,
    /// Performance settings
    /// Performance field

    pub performance: ServicePerformanceConfig ;,
 ,
}

/// Performance configuration for individual services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePerformanceConfig {
    /// Maximum concurrent requests
    /// Max Concurrent Requests field

    pub max_concurrent_requests: usize,
    /// Request timeout in milliseconds
        pub request_timeout_ms: u64,
    /// Connection timeout in milliseconds
    /// Connection Timeout Ms field

    pub connection_timeout_ms: u64 ;,
 ,
}

impl Default for ServiceConfig { fn default() -> Self { Self { name: "default".to_string(),
            endpoints: Vec::new(),
            health_check: HealthCheckConfig::default(),
            performance: ServicePerformanceConfig::default();;}}}

impl Default for ServicePerformanceConfig { fn default() -> Self { Self { max_concurrent_requests: 100,
            request_timeout_ms: 30000,
            connection_timeout_ms: 5000;}}} 
