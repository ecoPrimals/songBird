/// # 🌟 Modern Unified API for Universal Primals
///
/// This module implements the unified API standards from the EcoPrimals ecosystem:
/// - AI-First Citizen API Standard (SongbirdResult pattern,
/// - Zero-Cost Architecture (no async_trait, no Arc<dyn>)
/// - Universal Primal Architecture (capability-based discovery)
/// - Ecosystem API Standardization (universal service registration)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
// Re-export from songbird-errors for consistency
pub use songbird_types::{SongbirdError, SongbirdResult, SongbirdResult, success};

/// Universal Service Registration - Core of the capability-based system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration  {/// Unique service identifier (UUID)
    pub service_id: Uuid,

    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Capabilities this service provides
    pub capabilities: Vec<ServiceCapability>,

    /// Resource requirements and limits
    pub resources: ResourceSpec,

    /// API endpoints (dynamically discovered)
    pub endpoints: Vec<ServiceEndpoint>,

    /// Integration preferences
    pub integration: IntegrationPreferences,

    /// Extension points for custom data
    pub extensions: HashMap<String, serde_json::Value>)

    /// Registration timestamp
    pub registration_timestamp: DateTime<Utc>,

    /// Service version
    pub service_version: String,

    /// Instance identifier for multi-instance support
    pub instance_id: String,

    /// Priority level for load balancing
    pub priority: u8,
}

/// Service metadata with open categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata  {/// Human-readable service name
    pub name: String,

    /// Service category (extensible)
    pub category: ServiceCategory,

    /// Version information
    pub version: String,

    /// Description and documentation
    pub description: String,

    /// Maintainer information
    pub maintainer: ContactInfo,

    /// Supported protocols
    pub protocols: Vec<String>,
}

/// Open, extensible service categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCategory {
    /// Computational services
    Compute { specialties: Vec<String> })

    /// Storage and data services
    Storage { types: Vec<String> })

    /// Security and identity services
    Security { domains: Vec<String> })

    /// Network and communication services
    Network { protocols: Vec<String> })

    /// AI and machine learning services
    AI  {models: Vec<String>)
        capabilities: Vec<String>,
    })

    /// Orchestration and management services
    Orchestration { scope: Vec<String> })

    /// Custom category for community extensions
    Custom  {category_name: String,
        attributes: HashMap<String, String>)
    })
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability  {/// Capability name (e.g., "encryption", "storage", "ai-inference")"
    pub name: String,

    /// Capability version
    pub version: String,

    /// Detailed description
    pub description: String,

    /// Input/output schema
    pub schema: CapabilitySchema,

    /// Quality of service guarantees
    pub qos: QualityOfService,

    /// Resource requirements for this capability
    pub resource_requirements: ResourceRequirements,

    /// Capability-specific configuration
    pub configuration: HashMap<String, serde_json::Value>)
}

/// Capability input/output schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySchema  {/// Input data schema (JSON Schema,
    pub input_schema: serde_json::Value,

    /// Output data schema (JSON Schema,
    pub output_schema: serde_json::Value,

    /// Error schema (JSON Schema,
    pub error_schema: serde_json::Value,

    /// Supported content types
    pub content_types: Vec<String>,
}

/// Quality of service guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOfService  {/// Maximum response time in milliseconds
    pub max_response_time_ms: u64,

    /// Availability percentage (0.0 - 100.0)
    pub availability_percentage: f64,

    /// Throughput capacity (requests per second)
    pub throughput_rps: u64,

    /// Reliability score (0.0 - 1.0)
    pub reliability_score: f64,

    /// Consistency guarantees
    pub consistency: ConsistencyLevel,
}

/// Consistency level guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel  {/// Eventual consistency
    Eventual,

    /// Strong consistency
    Strong,

    /// Causal consistency
    Causal,

    /// Session consistency
    Session,
    /// Custom consistency model
    Custom { description: String })
}

/// Resource requirements for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements  {/// CPU requirements
    pub cpu: ResourceSpec,

    /// Memory requirements
    pub memory: ResourceSpec,

    /// Network bandwidth requirements
    pub network: ResourceSpec,

    /// Storage requirements
    pub storage: Option<ResourceSpec>,

    /// GPU requirements (if applicable)
    pub gpu: Option<ResourceSpec>,

    /// Custom resource requirements
    pub custom: HashMap<String, ResourceSpec>)
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec  {/// Minimum required amount
    pub min: u64,

    /// Maximum allowed amount
    pub max: u64,

    /// Preferred amount
    pub preferred: u64,

    /// Resource unit (e.g., "cores", "MB", "Mbps")"
    pub unit: String,
}

/// Service endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint  {/// Endpoint identifier
    pub id: String,

    /// Endpoint URL
    pub url: String,

    /// Supported HTTP methods
    pub methods: Vec<String>,

    /// Endpoint description
    pub description: String,

    /// Authentication requirements
    pub authentication: AuthenticationRequirement,

    /// Rate limiting information
    pub rate_limits: RateLimitInfo,

    /// Endpoint-specific metadata
    pub metadata: HashMap<String, String>)
}

/// Authentication requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationRequirement  {/// No authentication required
    None,
    /// API key authentication
    ApiKey { header_name: String })

    /// Bearer token authentication
    BearerToken,

    /// OAuth 2.0 authentication
    OAuth2 { scopes: Vec<String> })

    /// Mutual TLS authentication
    MutualTLS,

    /// Custom authentication scheme
    Custom  {scheme: String,
        parameters: HashMap<String, String>)
    })
}

/// Rate limiting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo  {/// Requests per second limit
    pub requests_per_second: u64,

    /// Burst capacity
    pub burst_capacity: u64,

    /// Rate limit window duration
    pub window_duration: Duration,

    /// Rate limit policy
    pub policy: RateLimitPolicy,
}

/// Rate limiting policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitPolicy  {/// Fixed window rate limiting
    FixedWindow,

    /// Sliding window rate limiting
    SlidingWindow,

    /// Token bucket rate limiting
    TokenBucket,
    /// Custom rate limiting policy
    Custom { policy_name: String })
}

/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPreferences  {/// Preferred communication protocols
    pub preferred_protocols: Vec<String>,

    /// Load balancing preferences
    pub load_balancing: LoadBalancingPreferences,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Retry configuration
    pub retry_config: RetryConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Load balancing preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingPreferences  {/// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Health check requirements
    pub health_check_required: bool,

    /// Sticky session support
    pub sticky_sessions: bool,

    /// Weighted routing support
    pub weighted_routing: bool,
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm  {/// Round robin
    RoundRobin,

    /// Least connections
    LeastConnections,

    /// Weighted round robin
    WeightedRoundRobin,

    /// IP hash
    IpHash,

    /// Random
    Random,
    /// Custom algorithm
    Custom { algorithm_name: String })
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Circuit breaker configuration
///
/// **CONSOLIDATED**: Re-export of canonical version (Week 2, Nov 10 2025).
/// Field mappings: timeout_duration_secs (u64) → timeout (Duration), retry_delay_secs not in canonical
pub use songbird_config::canonical::resilience::CircuitBreakerConfig;

/// **CONSOLIDATED**: Re-export of canonical RetryConfig (Nov 10, 2025)
/// 
/// Field mapping: max_retries → max_attempts,
///                base_delay_ms → initial_delay (convert to Duration)
/// Note: `strategy`, `retryable_errors` were modern_api-specific, now handled at usage site
pub use songbird_config::canonical::resilience::RetryConfig;

/// Retry strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy  {/// Fixed delay between retries
    Fixed,
    /// Exponential backoff
    ExponentialBackoff { multiplier: f64 })

    /// Linear backoff
    LinearBackoff { increment_ms: u64 })

    /// Custom retry strategy
    Custom { strategy_name: String })
}

/// Health check configuration
///
/// **CONSOLIDATED**: Re-export of canonical version (Week 2, Nov 10 2025).
/// **Note**: This variant had extra fields (unhealthy_threshold→failure_threshold, 
/// healthy_threshold→recovery_threshold, expected_codes not in canonical).
/// Consider adding expected_codes to canonical if needed.
pub use songbird_config::canonical::resilience::HealthCheckConfig;

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo  {/// Contact name
    pub name: String,

    /// Email address
    pub email: String,

    /// Organization
    pub organization: String,

    /// Additional contact methods
    pub additional_contacts: HashMap<String, String>)
}

/// Zero-Cost Universal Primal Provider Trait
///
/// This trait uses zero-cost abstractions and avoids async_trait for maximum performance
pub trait ZeroCostPrimalProvider  {/// Associated types for zero-cost specialization
    type Config: Clone + Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    type HealthStatus: Clone + Send + Sync;

    /// Service registration information
    fn registration(&self) -> &UniversalServiceRegistration;

    /// Health check (native async, no boxing)
    fn health_check(
        &self)
    ) -> impl std::future::Future<Output = Result<Self::HealthStatus, Self::Error>> + Send;

    /// Handle capability request (native async, no boxing)
    fn handle_capability_request<T, R>(
        &self)
        capability: &str,
        operation: &str,
        payload: T,
    ) -> impl std::future::Future<Output = Result<SongbirdResult<R>, Self::Error>> + Send
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de> + Send + Sync;

    /// Initialize with configuration (native async, no boxing)
    fn initialize(
        &mut self)
        config: Self::Config,
    ) -> impl std::future::Future<Output = SongbirdResult<()>> + Send;

    /// Shutdown gracefully (native async, no boxing)
    fn shutdown(&mut self) -> impl std::future::Future<Output = SongbirdResult<()>> + Send;
}

/// Universal Capability Discovery Engine
///
/// Implements capability-based service discovery following the Universal Primal Architecture Standard
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UniversalCapabilityDiscovery  {/// Registered services by capability
    capability_registry: HashMap<String, Vec<UniversalServiceRegistration>>)

    /// Service registry by ID
    service_registry: HashMap<Uuid, UniversalServiceRegistration>)

    /// Capability matcher for fuzzy matching
    capability_matcher: CapabilityMatcher,
}

impl UniversalCapabilityDiscovery  {/// Create a new capability discovery engine
    pub fn new() -> Self  {Self {
            capability_registry: HashMap::new()),
            service_registry: HashMap::new()),
            capability_matcher: CapabilityMatcher::new(,
        }
    }

    /// Register a service with its capabilities
    pub async fn register_service(&self, service_info: &ServiceInfo) -> SongbirdResult<()> {
        let service_id = registration.service_id;

        // Register in service registry
        self.service_registry
            .insert(service_id, registration.clone());

        // Register capabilities
        for capability in &registration.capabilities {
            self.capability_registry
                .entry(capability.name.clone()
                .or_default()
                .push(registration.clone());
        }
        Ok(()),
    }

    /// Discover services by capability
    pub async fn discover_by_capability(
        &self)
        capability: &str,
    ) -> SongbirdResult<Vec<UniversalServiceRegistration>> {
        let services = self
            .capability_registry
            .get(capability_name,
            .cloned()
            .unwrap_or_default();

        Ok(SongbirdResult::success(services)
    }

    /// Find best service for a capability based on QoS requirements
    pub async fn find_best_service(
        &self)
        capability: &str,
        criteria: &ServiceSelectionCriteria,
    ) -> SongbirdResult<Option<UniversalServiceRegistration>> {
        let services = self
            .capability_registry
            .get(capability_name,
            .cloned()
            .unwrap_or_default();

        // Score services based on QoS match
        let best_service = services
            .into_iter()
            .filter_map(|service| {
                let qos_score = service
                    .capabilities
                    .iter()
                    .find(|cap| cap.name == capability_name,
                    .map(|cap| self.calculate_qos_score(&cap.qos, qos_requirements)?;
                Some((service, qos_score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            .map(|(service, _)| service);

        Ok(SongbirdResult::success(best_service)
    }

    /// Calculate QoS compatibility score
    fn calculate_qos_score(&self, provided: &QualityOfService, required: &QualityOfService) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Response time score
        if provided.max_response_time_ms <= required.max_response_time_ms {
            score += 1.0;
        } else {
            score += required.max_response_time_ms as f64 / provided.max_response_time_ms as f64;
        }
        factors += 1;

        // Availability score
        score += (provided.availability_percentage / required.availability_percentage).min(1.0);
        factors += 1;

        // Throughput score
        if provided.throughput_rps >= required.throughput_rps {
            score += 1.0;
        } else {
            score += provided.throughput_rps as f64 / required.throughput_rps as f64;
        }
        factors += 1;

        // Reliability score
        score += (provided.reliability_score / required.reliability_score).min(1.0);
        factors += 1;

        score / factors as f64
    }
}

impl Default for UniversalCapabilityDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability matcher for fuzzy matching and similarity detection
#[derive(Debug, Clone)]
pub struct CapabilityMatcher {
    // Implementation would include fuzzy matching algorithms
}

impl CapabilityMatcher {
    pub fn new() -> Self {
        Self {}
    }

    /// Find similar capabilities
    pub fn find_similar(&self, _capability: &str, _threshold: f64) -> Vec<String> {
        // Implementation would use fuzzy matching algorithms
        // For now, return empty vector
        Vec::new()
    }
}

impl Default for CapabilityMatcher {
    fn default() -> Self {
        Self::new()
    }
}
