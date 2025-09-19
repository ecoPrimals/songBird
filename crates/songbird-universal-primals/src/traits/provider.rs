//! Core PrimalProvider trait definition
//!
//! Provides the fundamental trait for all Universal Primals with modern Rust idioms,
//! zero-cost abstractions, and comprehensive error handling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::{PrimalCapability, PrimalContext, PrimalDependency, PrimalHealth};
use songbird_types::{
    errors::SongbirdResult, CanonicalPrimalType, CanonicalRequest, CanonicalResponse,
};

/// Core trait that all Universal Primals must implement
///
/// This trait provides the fundamental interface for primal services with:
/// - Zero-cost abstractions using native async functions
/// - Comprehensive error handling with SongbirdResult
/// - Type-safe capability and dependency management
/// - Modern Rust idioms and patterns
#[async_trait::async_trait]
pub trait PrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "security_provider", "storage_provider")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Get the primal type this provider handles
    fn primal_type(&self) -> CanonicalPrimalType;

    /// Get supported capabilities with their configurations
    fn capabilities(&self) -> Vec<PrimalCapability>;

    /// Get service dependencies
    fn dependencies(&self) -> Vec<PrimalDependency>;

    /// Get current health status
    async fn health(&self) -> SongbirdResult<PrimalHealth>;

    /// Handle a primal request with full error handling
    async fn handle_request(&self, request: CanonicalRequest) -> SongbirdResult<CanonicalResponse>;

    /// Initialize the primal provider
    async fn initialize(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    /// Shutdown the primal provider gracefully
    async fn shutdown(&mut self) -> SongbirdResult<()> {
        Ok(())
    }

    /// Check if the provider is ready to handle requests
    async fn is_ready(&self) -> SongbirdResult<bool> {
        Ok(true)
    }

    /// Validate a request before processing
    async fn validate_request(&self, request: &CanonicalRequest) -> SongbirdResult<()> {
        // Default implementation accepts all requests
        // Override for custom validation logic
        let _ = request;
        Ok(())
    }
}

/// Enhanced primal provider with additional capabilities
#[async_trait::async_trait]
pub trait EnhancedPrimalProvider: PrimalProvider {
    /// Get detailed metrics about the provider
    async fn get_metrics(&self) -> SongbirdResult<ProviderMetrics>;

    /// Get configuration information
    fn get_config(&self) -> SongbirdResult<ProviderConfig>;

    /// Update provider configuration
    async fn update_config(&mut self, config: ProviderConfig) -> SongbirdResult<()>;

    /// Handle batch requests efficiently
    async fn handle_batch_requests(
        &self,
        requests: Vec<CanonicalRequest>,
    ) -> SongbirdResult<Vec<CanonicalResponse>>;

    /// Get provider version information
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Get supported API versions
    fn supported_api_versions(&self) -> Vec<String> {
        vec!["v1".to_string()]
    }
}

/// Metrics for monitoring provider performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    /// Total requests processed
    pub requests_processed: u64,
    /// Total errors encountered
    pub errors_count: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Current active requests
    pub active_requests: u32,
    /// Provider uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Custom metrics specific to the provider
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for ProviderMetrics {
    fn default() -> Self {
        Self {
            requests_processed: 0,
            errors_count: 0,
            avg_response_time_ms: 0.0,
            active_requests: 0,
            uptime_seconds: 0,
            memory_usage_bytes: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

impl ProviderMetrics {
    /// Create new metrics instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            requests_processed: 0,
            errors_count: 0,
            avg_response_time_ms: 0.0,
            active_requests: 0,
            uptime_seconds: 0,
            memory_usage_bytes: 0,
            custom_metrics: HashMap::new(),
        }
    }

    /// Calculate error rate as percentage
    #[must_use]
    pub fn error_rate(&self) -> f64 {
        if self.requests_processed == 0 {
            0.0
        } else {
            (self.errors_count as f64 / self.requests_processed as f64) * 100.0
        }
    }

    /// Check if metrics indicate healthy operation
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.error_rate() < 5.0 && self.avg_response_time_ms < 1000.0
    }

    /// Add a custom metric
    pub fn add_custom_metric(&mut self, name: impl Into<String>, value: f64) {
        self.custom_metrics.insert(name.into(), value);
    }
}

/// Configuration for a primal provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name
    pub name: String,
    /// Configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Feature flags
    pub features: HashMap<String, bool>,
    /// Resource limits
    pub limits: ResourceLimits,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            parameters: HashMap::new(),
            features: HashMap::new(),
            limits: ResourceLimits::default(),
        }
    }
}

/// Resource limits for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Rate limit (requests per second)
    pub rate_limit_rps: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            max_memory_bytes: 1_000_000_000, // 1GB
            request_timeout_seconds: 30,
            rate_limit_rps: 1000,
        }
    }
}

/// Enumeration of all possible primal provider types
#[derive(Clone)]
pub enum PrimalProviderEnum {
    /// Security-focused primal provider
    Security(Arc<dyn PrimalProvider>),
    /// Storage-focused primal provider  
    Storage(Arc<dyn PrimalProvider>),
    /// Compute-focused primal provider
    Compute(Arc<dyn PrimalProvider>),
    /// AI/ML-focused primal provider
    AI(Arc<dyn PrimalProvider>),
    /// Network-focused primal provider
    Network(Arc<dyn PrimalProvider>),
    /// Custom primal provider
    Custom {
        /// Provider type name
        provider_type: String,
        /// Provider instance
        provider: Arc<dyn PrimalProvider>,
    },
}

impl PrimalProviderEnum {
    /// Get the provider type as a string
    #[must_use]
    pub fn provider_type(&self) -> &str {
        match self {
            Self::Security(_) => "security",
            Self::Storage(_) => "storage",
            Self::Compute(_) => "compute",
            Self::AI(_) => "ai",
            Self::Network(_) => "network",
            Self::Custom { provider_type, .. } => provider_type,
        }
    }

    /// Get the underlying provider
    #[must_use]
    pub fn provider(&self) -> &Arc<dyn PrimalProvider> {
        match self {
            Self::Security(p)
            | Self::Storage(p)
            | Self::Compute(p)
            | Self::AI(p)
            | Self::Network(p) => p,
            Self::Custom { provider, .. } => provider,
        }
    }

    /// Check if this is a specific provider type
    #[must_use]
    pub fn is_type(&self, provider_type: &str) -> bool {
        self.provider_type() == provider_type
    }
}

/// Builder for creating primal providers with configuration
pub struct PrimalProviderBuilder {
    primal_id: Option<String>,
    instance_id: Option<String>,
    context: Option<PrimalContext>,
    config: ProviderConfig,
    capabilities: Vec<PrimalCapability>,
    dependencies: Vec<PrimalDependency>,
}

impl Default for PrimalProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimalProviderBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            primal_id: None,
            instance_id: None,
            context: None,
            config: ProviderConfig::default(),
            capabilities: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set the primal ID
    #[must_use]
    pub fn with_primal_id(mut self, id: impl Into<String>) -> Self {
        self.primal_id = Some(id.into());
        self
    }

    /// Set the instance ID
    #[must_use]
    pub fn with_instance_id(mut self, id: impl Into<String>) -> Self {
        self.instance_id = Some(id.into());
        self
    }

    /// Set the context
    #[must_use]
    pub fn with_context(mut self, context: PrimalContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Set the configuration
    #[must_use]
    pub fn with_config(mut self, config: ProviderConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a capability
    #[must_use]
    pub fn add_capability(mut self, capability: PrimalCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a dependency
    #[must_use]
    pub fn add_dependency(mut self, dependency: PrimalDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Validate the builder configuration
    pub fn validate(&self) -> SongbirdResult<()> {
        if self.primal_id.is_none() {
            return Err(songbird_errors::SongbirdError::configuration(
                "Primal ID is required",
            ));
        }
        if self.instance_id.is_none() {
            return Err(songbird_errors::SongbirdError::configuration(
                "Instance ID is required",
            ));
        }
        Ok(())
    }
}

/// Utility functions for working with primal providers
pub mod utils {
    use super::*;

    /// Check if a provider supports a specific capability
    pub fn provider_supports_capability(
        provider: &dyn PrimalProvider,
        capability_name: &str,
    ) -> bool {
        provider
            .capabilities()
            .iter()
            .any(|cap| cap.name() == capability_name)
    }

    /// Get all providers of a specific type
    pub fn filter_providers_by_type<'a>(
        providers: &'a [PrimalProviderEnum],
        provider_type: &str,
    ) -> Vec<&'a PrimalProviderEnum> {
        providers
            .iter()
            .filter(|p| p.is_type(provider_type))
            .collect()
    }

    /// Create a provider registry
    #[must_use]
    pub fn create_provider_registry() -> HashMap<String, PrimalProviderEnum> {
        HashMap::new()
    }
}
