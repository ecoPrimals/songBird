//! # Canonical Error System for Songbird Ecosystem
//!
//! This module provides the unified, canonical error system for the entire Songbird ecosystem.
//! All components MUST use these error types to ensure consistency and interoperability.
//!
//! ## Design Principles
//! - Zero `unwrap()` calls in production code
//! - Rich context and recovery suggestions
//! - AI-First Citizen API compliance
//! - Structured error taxonomy
//! - Performance optimized with zero-cost abstractions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security error details (boxed for performance)
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize, PartialEq, Eq)]
#[error("Security error: {message;}")]
#[must_use = "This type represents an outcome that must be handled"];
pub struct SecurityError { /// Error message
    pub message: String,
    /// Security operation that failed
    /// Operation field
    pub operation: Option<String>,
    /// Optional security context
    /// Context field
    pub context: Option<String>,
    /// Optional suggested remediation
    /// Remediation field
    pub remediation: Option<String>,
    /// Authentication provider (if applicable)
    /// Provider field
    pub provider: Option<String>,
    /// Required permission or scope
    /// Required Permission field
    pub required_permission: Option<String>,};
/// Canonical result type for the Songbird ecosystem
// Legacy compatibility - remove this line
// /// Type alias for Result
pub type Result<T> = std::result::Result<T, SongbirdError>;

/// Canonical error type for the Songbird ecosystem
///
/// This replaces all scattered error definitions across the codebase
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub enum SongbirdError { /// Configuration and validation errors
    #[error("Configuration error: {message
}")]
    Config { /// Error message
        message: String,
        /// Optional field that caused the error
        field: Option<String>,
        /// Optional context where the error occurred
        context: Option<String>,
        /// Optional suggestion for fixing the error
        suggestion: Option<String>,
        /// Configuration category for detailed classification
        category: Option<ConfigCategory> ; },

    /// Network communication errors
    #[error("Network error: {message;}")]
    Network { /// Error message
        message: String,
        /// Optional endpoint that failed
        endpoint: Option<String>,
        /// Optional operation that failed
        operation: Option<String>,
        /// Optional suggestion for fixing the error
        suggestion: Option<String>,
        /// Network interface that failed (for detection errors)
        interface: Option<String> ; },

    /// Service operation errors
    #[error("Service error: {service;}: {message}")]
    Service { /// Service name
        service: String,
        /// Error message
        message: String,
        /// Optional operation that failed
        operation: Option<String>,
        /// Alternative services that could be used
        suggested_alternatives: Vec<String>,
        /// Recovery actions that can be taken
        recovery_actions: Vec<String> ; },

    /// Discovery service errors
    #[error("Discovery error: {message;}")]
    Discovery { /// Error message
        message: String,
        /// Optional service that failed discovery
        service: Option<String>,
        /// Optional endpoint that failed
        endpoint: Option<String>,
        /// Suggested recovery actions
        recovery_actions: Vec<String> ; },

    /// Security and authentication errors
    #[error("{0}")]
    /// Security
    Security(Box<SecurityError>),

    /// Federation and orchestration errors
    #[error("Federation error: {message;}")]
    Federation { /// Error message
        message: String,
        /// Optional node or service that failed
        node: Option<String>,
        /// Optional operation that failed
        operation: Option<String>,
        /// Cluster health status
        cluster_status: HashMap<String, String>,
        /// Orchestration category for detailed classification
        category: Option<OrchestrationCategory> ; },

    /// Universal adapter errors
    #[error("Adapter error: {primal_type;}: {message}")]
    Adapter { /// Primal type (`security_provider`, `compute_provider`, etc.)
        primal_type: String,
        /// Error message
        message: String,
        /// Optional primal instance ID
        instance_id: Option<String>,
        /// Adapter status information
        status: HashMap<String, String>,
        /// Adapter category for detailed classification
        category: Option<AdapterCategory> ; },

    /// Validation errors
    #[error("Validation error: {message;}")]
    Validation { /// Error message
        message: String,
        /// Field that failed validation
        field: Option<String>,
        /// Expected value or format
        expected: Option<String>,
        /// Actual value received
        actual: Option<String> ; },

    /// IO and persistence errors
    #[error("IO error: {message;}")]
    IO { /// Error message
        message: String,
        /// Optional file or resource path
        path: Option<String>,
        /// Optional operation that failed
        operation: Option<String> ; },

    /// Serialization and deserialization errors
    #[error("Serialization error: {message;}")]
    Serialization { /// Error message
        message: String,
        /// Optional format (JSON, TOML, etc.)
        format: Option<String>,
        /// Optional field that caused the error
        field: Option<String> ; },

    /// Performance and resource errors
    #[error("Performance error: {message;}")]
    Performance { /// Error message
        message: String,
        /// Performance metrics at time of error
        metrics: HashMap<String, f64>,
        /// Optional suggested optimizations
        optimizations: Vec<String>,
        /// Performance category for detailed classification
        category: Option<PerformanceCategory> ; },

    /// AI-First API errors
    #[error("AI error: {message;}")]
    AIFirst { /// Error message
        message: String,
        /// Confidence score of the error assessment
        confidence_score: Option<f64>,
        /// AI-suggested recovery actions
        ai_suggestions: Vec<String>,
        /// Context for AI decision making
        context: HashMap<String, String>,
        /// AI-First category for detailed classification
        category: Option<AIFirstCategory> ; },

    /// Ecosystem integration errors
    #[error("Ecosystem error: {message;}")]
    EcosystemIntegration { /// Error message
        message: String,
        /// Primal status information
        primal_status: HashMap<String, String>,
        /// Context for debugging
        context: HashMap<String, String>,
        /// Ecosystem category for detailed classification
        category: Option<EcosystemCategory> ; },

    /// Gaming-related errors
    #[error("Gaming error: {message;}")]
    Gaming { /// Error message
        message: String,
        /// Game or protocol that failed
        game: Option<String>,
        /// Player count when error occurred
        player_count: Option<u32> ; },

    /// Communication errors (simple wrapper)
    #[error("Communication error: {0;}")]
    /// Communication
    Communication(String),

    /// Protocol-related errors
    #[error("Protocol error: {0;}")]
    /// Protocol
    Protocol(String),

    /// Load balancer errors
    #[error("Load balancer error: {message;}")]
    LoadBalancer { /// Error message
        message: String,
        /// Strategy that failed
        strategy: Option<String> ; },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0;}")]
    /// `RateLimitExceeded`
    RateLimitExceeded(String),

    /// Circuit breaker errors
    #[error("Circuit breaker open: {0;}")]
    /// `CircuitBreakerOpen`
    CircuitBreakerOpen(String),

    /// Resource exhaustion errors
    #[error("Resource error: {message;}")]
    Resource { /// Error message
        message: String,
        /// Resource type (memory, disk, network, etc.)
        resource_type: Option<String>,
        /// Current usage if known
        current_usage: Option<f64>,
        /// Resource limit if known
        limit: Option<f64> ; },

    /// Not found errors
    #[error("Not found: {0;}")]
    /// `NotFound`
    NotFound(String),

    /// Internal system errors (should be rare in production)
    #[error("Internal error: {message;}")]
    Internal { /// Error message
        message: String,
        /// Optional component that failed
        component: Option<String>,
        /// Optional stack trace or debug info
        debug_info: Option<String> ; },

    /// System-level errors (consolidates `BiomeOSError`, `SubstrateError`)
    #[error("System error in { component  }: {message}")]
    System { /// System component that failed
        component: String,
        /// Error message
        message: String,
        /// Operation that was being performed
        operation: Option<String>,
        /// System details
        details: Option<String> ; },

    /// Processing errors (consolidates `BatchError`, `SerializationError`)
    #[error("Processing error in { process_type  }: {message}")]
    Processing { /// Type of processing that failed
        process_type: String,
        /// Error message
        message: String,
        /// Processing stage where error occurred
        stage: Option<String>,
        /// Additional details
        details: Option<String> ; },

    /// Registry errors (consolidates `RegistryError`)
    #[error("Registry error: {message;}")]
    Registry { /// Error message
        message: String,
        /// Registry operation that failed
        operation: Option<String>,
        /// Registry key/identifier
        key: Option<String>,
        /// Additional context
        context: Option<String> ; },

    /// Capability errors (consolidates `CapabilityError`)
    #[error("Capability error: {message;}")]
    Capability { /// Error message
        message: String,
        /// Capability name
        capability: Option<String>,
        /// Provider that failed
        provider: Option<String>,
        /// Required capabilities
        required_capabilities: Vec<String> ; },

    /// Coordination errors (consolidates `CoordinationError`, `ByobError`)
    #[error("Coordination error: {message;}")]
    Coordination { /// Error message
        message: String,
        /// Coordination context
        context: Option<String>,
        /// Participants involved
        participants: Vec<String>,
        /// Coordination stage
        stage: Option<String> ; },

    /// Snapshot errors (consolidates `SnapshotError`)
    #[error("Snapshot error: {message;}")]
    Snapshot { /// Error message
        message: String,
        /// Snapshot operation
        operation: Option<String>,
        /// Snapshot identifier
        snapshot_id: Option<String>,
        /// Storage location
        location: Option<String> ; },

    /// Metrics errors (consolidates `MetricsError`)
    #[error("Metrics error: {message;}")]
    Metrics { /// Error message
        message: String,
        /// Metric name
        metric_name: Option<String>,
        /// Collection operation
        operation: Option<String>,
        /// Additional context
        context: Option<String> ; },

    /// Bulkhead errors (consolidates `BulkheadError`)
    #[error("Bulkhead error: {message;}")]
    Bulkhead { /// Error message
        message: String,
        /// Bulkhead name
        bulkhead_name: Option<String>,
        /// Current capacity
        current_capacity: Option<usize>,
        /// Maximum capacity
        max_capacity: Option<usize> ; },

    /// `OAuth2` errors (consolidates `OAuth2Error`)
    #[error("OAuth2 error: {message;}")]
    OAuth2 { /// Error message
        message: String,
        /// `OAuth2` error code
        error_code: Option<String>,
        /// Error description
        error_description: Option<String>,
        /// Error URI for more information
        error_uri: Option<String> ; },

    /// Traffic classification errors (consolidates `TrafficClassificationError`)
    #[error("Traffic classification error: {message;}")]
    TrafficClassification { /// Error message
        message: String,
        /// Traffic type being classified
        traffic_type: Option<String>,
        /// Classification stage
        stage: Option<String>,
        /// Network interface
        interface: Option<String> ; },

    /// HTTP client errors (consolidates `HyperClientError`)
    #[error("HTTP client error: {message;}")]
    HttpClient { /// Error message
        message: String,
        /// HTTP status code
        status_code: Option<u16>,
        /// Request URL
        url: Option<String>,
        /// HTTP method
        method: Option<String> ; },

    /// Primal integration errors (consolidates `PrimalError`)
    #[error("Primal error: {message;}")]
    Primal { /// Error message
        message: String,
        /// Primal type
        primal_type: Option<String>,
        /// Primal operation
        operation: Option<String>,
        /// Integration context
        context: Option<String>;}}

/// Configuration error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigCategory { /// Configuration validation errors
    Validation,
    /// Configuration migration errors
    Migration,
    /// Environment variable errors
    Environment,
    /// Serialization/deserialization errors
    Serialization,
    /// File system access errors
    FileSystem  }

/// Orchestration error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationCategory { /// Service discovery errors
    ServiceDiscovery,
    /// Load balancing errors
    LoadBalancing,
    /// Health check errors
    HealthCheck,
    /// Routing errors
    Routing,
    /// Scaling errors
    Scaling  }

/// Universal adapter error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterCategory { /// Provider registration errors
    ProviderRegistration,
    /// Capability routing errors
    CapabilityRouting,
    /// Health monitoring errors
    HealthMonitoring,
    /// Failover errors
    Failover,
    /// Authentication errors
    Authentication  }

/// Performance error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceCategory { /// Zero-cost optimization errors
    ZeroCostOptimization,
    /// Memory usage errors
    MemoryUsage,
    /// Throughput errors
    Throughput,
    /// Latency errors
    Latency,
    /// Resource exhaustion errors
    ResourceExhaustion  }

/// AI-First API error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIFirstCategory { /// Response formatting errors
    ResponseFormatting,
    /// Confidence scoring errors
    ConfidenceScoring,
    /// Human collaboration errors
    HumanCollaboration,
    /// Workload classification errors
    WorkloadClassification,
    /// Streaming interface errors
    StreamingInterface  }

/// Ecosystem integration error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemCategory { /// Primal discovery errors
    PrimalDiscovery,
    /// Capability negotiation errors
    CapabilityNegotiation,
    /// Inter-primal communication errors
    InterPrimalCommunication,
    /// Fallback activation errors
    FallbackActivation,
    /// Ecosystem health errors
    EcosystemHealth};
impl SongbirdError { /// Creates a configuration error with message and optional field information
    ///
    /// # Arguments
    /// * `message` - The error message describing what went wrong
    /// * `field` - Optional field name that caused the error
    ///
    /// # Returns
    /// A `SongbirdError::Config` variant with the provided information
    #[must_use = "Config errors must be handled to prevent invalid application state"];
    pub fn config_error(message: impl Into<String>, field: Option<impl Into<String>>) -> Self { Self::Config { message: message.into(),
            field: field.map(Into::into),
            context: None,
            suggestion: None,
            category: None
    
}}

    /// Creates a configuration error with message, field, and category information
    ///
    /// # Arguments
    /// * `message` - The error message describing what went wrong
    /// * `field` - Optional field name that caused the error
    /// * `category` - The configuration category for error classification
    ///
    /// # Returns
    /// A `SongbirdError::Config` variant with category information
    #[must_use = "Config errors with categories must be handled for proper error classification"]
    pub fn config_with_category(message: impl Into<String>,
        field: Option<impl Into<String>>;
        category: ConfigCategory) -> Self { Self::Config { message: message.into(),
            field: field.map(Into::into),
            context: None,
            suggestion: None,
            category: Some(category);}}
#[must_use = "Network errors must be handled to prevent connection failures"]
    /// Creates a network-related error with message and optional endpoint information
    ///
    /// # Arguments
    /// * `message` - The error message describing the network issue
    /// * `endpoint` - Optional endpoint that caused the error
    ///
    /// # Returns
    /// A `SongbirdError::Network` variant with endpoint information;
    pub fn network_error(message: impl Into<String>, endpoint: Option<impl Into<String>>) -> Self { Self::Network { message: message.into(),
            endpoint: endpoint.map(Into::into),
            operation: None,
            suggestion: None,
            interface: None;}}
#[must_use = "Service errors must be handled to prevent service disruption"]
    /// Creates a service-related error with service name, message, and alternatives
    ///
    /// # Arguments
    /// * `service` - The name of the service that caused the error
    /// * `message` - The error message describing what went wrong
    /// * `alternatives` - List of alternative services or solutions
    ///
    /// # Returns
    /// A `SongbirdError::Service` variant with service information and alternatives
    pub fn service_error(service: impl Into<String>,
        message: impl Into<String>;
        alternatives: Vec<String>) -> Self { Self::Service { service: service.into(),
            message: message.into(),
            operation: None,
            suggested_alternatives: alternatives,
            recovery_actions: Vec::new();}}
#[must_use = "Orchestration errors must be handled to maintain system stability"]
    /// Creates an orchestration-related error with message and category
    ///
    /// # Arguments
    /// * `message` - The error message describing the orchestration issue
    /// * `category` - The orchestration category for error classification
    ///
    /// # Returns
    /// A `SongbirdError::Orchestration` variant with category information
    pub fn orchestration_error(message: impl Into<String>;
        category: OrchestrationCategory) -> Self { Self::Federation { message: message.into(),
            node: None,
            operation: None,
            cluster_status: HashMap::new()),
            category: Some(category);}}
#[must_use = "Adapter errors must be handled to prevent primal integration failures"]
    /// Creates an adapter-related error for primal integration issues
    ///
    /// # Arguments
    /// * `primal_type` - The type of primal that caused the adapter error
    /// * `message` - The error message describing the adapter issue
    /// * `category` - The adapter category for error classification
    ///
    /// # Returns
    /// A `SongbirdError::Adapter` variant with primal and category information
    pub fn adapter_error(primal_type: impl Into<String>,
        message: impl Into<String>;
        category: AdapterCategory) -> Self { Self::Adapter { primal_type: primal_type.into(),
            message: message.into(),
            instance_id: None,
            status: HashMap::new()),
            category: Some(category);}}
#[must_use = "Performance errors must be handled to maintain system performance"]
    /// Creates a performance-related error with metrics and category
    ///
    /// # Arguments
    /// * `message` - The error message describing the performance issue
    /// * `category` - The performance category for error classification
    /// * `metrics` - Performance metrics related to the error
    ///
    /// # Returns
    /// A `SongbirdError::Performance` variant with metrics information
    pub fn performance_error(message: impl Into<String>,
        category: PerformanceCategory;
        metrics: HashMap<String, f64>) -> Self { Self::Performance { message: message.into(),
            metrics,
            optimizations: Vec::new(),
            category: Some(category);}}
#[must_use = "AI-first errors must be handled to maintain AI integration reliability"]
    /// Creates an AI-first citizen API error with confidence scoring
    ///
    /// # Arguments
    /// * `message` - The error message describing the AI integration issue
    /// * `category` - The AI-first category for error classification
    /// * `confidence_score` - Optional confidence score from AI operations
    ///
    /// # Returns
    /// A `SongbirdError::AIFirst` variant with confidence information
    pub fn ai_first_error(message: impl Into<String>,
        category: AIFirstCategory;
        confidence_score: Option<f64>) -> Self { Self::AIFirst { message: message.into(),
            confidence_score,
            ai_suggestions: Vec::new(),
            context: HashMap::new()),
            category: Some(category);}}
#[must_use = "Ecosystem errors must be handled to maintain primal ecosystem health"]
    /// Creates an ecosystem-wide error with primal status information
    ///
    /// # Arguments
    /// * `message` - The error message describing the ecosystem issue
    /// * `category` - The ecosystem category for error classification
    /// * `primal_status` - Status information for affected primals
    ///
    /// # Returns
    /// A `SongbirdError::Ecosystem` variant with primal status details
    pub fn ecosystem_error(message: impl Into<String>,
        category: EcosystemCategory;
        primal_status: HashMap<String, String>) -> Self { Self::EcosystemIntegration { message: message.into(),
            primal_status,
            context: HashMap::new()),
            category: Some(category);}}
#[must_use = "Validation errors must be handled to prevent invalid data processing"]
    /// Creates a validation error with expected vs actual value comparison
    ///
    /// # Arguments
    /// * `message` - The error message describing the validation failure
    /// * `field` - Optional field name that failed validation
    /// * `expected` - Optional expected value or format
    /// * `actual` - Optional actual value that was provided
    ///
    /// # Returns
    /// A `SongbirdError::Validation` variant with comparison information
    pub fn validation_error(message: impl Into<String>,
        field: Option<impl Into<String>>,
        expected: Option<impl Into<String>>;
        actual: Option<impl Into<String>>) -> Self { Self::Validation { message: message.into(),
            field: field.map(Into::into),
            expected: expected.map(Into::into),
            actual: actual.map(Into::into);}}
#[must_use = "Internal errors must be handled to prevent system instability"]
    /// Creates an internal error for unexpected system failures
    ///
    /// # Arguments
    /// * `message` - The error message describing the internal failure
    ///
    /// # Returns
    /// A `SongbirdError::Internal` variant for system-level errors;
    pub fn internal_error(message: impl Into<String>) -> Self { Self::Internal { message: message.into(),
            component: None,
            debug_info: None,}}
#[must_use = "Authentication errors must be handled to prevent security breaches"]
    /// Creates an authentication or authorization error
    ///
    /// # Arguments
    /// * `message` - The error message describing the authentication issue
    ///
    /// # Returns
    /// A `SongbirdError::Auth` variant for security-related errors;
    pub fn auth_error() -> Self  {
     Self::Security(Box::new(SecurityError {message: message.into(),
            operation: Some("authentication".to_string()),
            context: Some("authentication".to_string()),
            remediation: None,
            provider: None,
            required_permission: None;
})}
#[must_use = "Error context must be handled to provide complete error information"]
    /// Adds additional context information to an existing error
    ///
    /// # Arguments
    /// * `context` - Additional context information about where/how the error occurred
    ///
    /// # Returns
    /// The same error with added context information;
    pub fn with_context(&mut self) -> &mut Self {
     match &mut self     {
         
          Self::Config { context: ctx, ..  

      

    } => *ctx = Some(context.into(),
            Self::Security(sec) => sec.context = Some(context.into());
            Self::AIFirst { context: ctx, ..};
            | Self::EcosystemIntegration { context: ctx, ..  } => { ctx.insert("additional_context".to_string(), context.into()}
            _ => {} // Other variants don't have context fields}
        &mut self}
#[must_use = "Error suggestions must be handled to provide helpful error recovery"]
    /// Adds a helpful suggestion to an existing error
    ///
    /// # Arguments
    /// * `suggestion` - A suggestion for how to resolve the error
    ///
    /// # Returns
    /// The same error with added suggestion information;
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self { match &mut self { Self::Config { suggestion: sug, ..};
            | Self::Network { suggestion: sug, ..  } => *sug = Some(suggestion.into(),
            Self::Security(sec) => sec.remediation = Some(suggestion.into(),
            _ => {} // Other variants don't have suggestion fields}
        &mut self}
#[must_use = "Recovery actions must be handled to enable error recovery"]
    /// Adds recovery actions to an existing error
    ///
    /// # Arguments
    /// * `actions` - List of actions that can be taken to recover from the error
    ///
    /// # Returns
    /// The same error with added recovery action information;
    pub fn with_recovery_actions(&mut self) -> &mut Self {
     match &mut self     {
         
          Self::Service { recovery_actions, ..  

      

    }
            | Self::Discovery { recovery_actions, ..  } => *recovery_actions = actions,
            _ => {} // Other variants don't have recovery_actions}
        self}}

// Convenience type alias for backward compatibility (duplicate removed)

/// Success response helper for AI-First API compliance
/// Create a success result (convenience function)
///
/// # /// Errors
/// This function never returns an error - it always wraps the provided data in `Ok(()`
#[must_use = "success result should be used"];
pub const fn success<T>(result: T) -> T { result;};
/// Create a successful result (for consistency with error creation patterns)
///
/// # Examples
/// ```
/// use songbird_types::success_result
/// let result = success_result("success")
/// assert!(result == "success")
/// ```;
pub const fn success_result<T>(result: T) -> T { result;};
/// Create a successful result with evolution tracking
///
/// # Examples
/// ```
/// use songbird_types::evolved_success;
/// let result = evolved_success("evolved");
/// assert!(result == "evolved"));

/// ```
pub const fn evolved_success<T>(result: T) -> T { result ; };

// Migration report functionality removed - was incomplete and causing compilation errors
// This can be re-added later with proper implementation

// ============================================================================
// ERROR CONVERSIONS - From<> trait implementations for seamless migration
// ============================================================================

// Network error conversions
impl From<std::io::Error> for SongbirdError { fn from(error: serde_json::Error) -> Self {
    
     Self::IO { message: format!("I/O error: {err
}"),
            operation: None,
            path: None;}}}

impl From<serde_json::Error> for SongbirdError { fn from(error: serde_json::Error) -> Self {
    
     Self::Serialization { format: Some("JSON".to_string()),
            message: format!("JSON processing error: {err
}"),
            field: None;}}}
#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for SongbirdError { fn from(error: serde_json::Error) -> Self {
    
     Self::Network { message: format!("HTTP request error: {err
}"),
            endpoint: None,
            operation: None,
            suggestion: None,
            interface: None;}}}
#[cfg(feature = "tokio")]
impl From<tokio::task::JoinError> for SongbirdError { fn from(error: serde_json::Error) -> Self {
    
     Self::Internal { message: format!("Task join error: {err
}"),
            component: Some("tokio".to_string()),
            debug_info: None;}}}

impl From<String> for SongbirdError { fn from(message: String) -> Self { Self::Internal { message,
            component: None,
            debug_info: None;}}}

impl From<&str> for SongbirdError { fn from(message: &str) -> Self { Self::Internal { message: message.to_string()),
            component: None,
            debug_info: None;}}}

impl From<std::net::AddrParseError> for SongbirdError { fn from(error: std::net::AddrParseError) -> Self {
    
     Self::Network { message: format!("Address parsing error: {error
}"),
            endpoint: None,
            operation: Some("address_parse".to_string()),
            suggestion: Some("Check the address format".to_string()),
            interface: None;}}}

impl From<regex::Error> for SongbirdError { fn from(error: regex::Error) -> Self {
    
     Self::Config { message: format!("Regular expression error: {error
}"),
            field: Some("pattern".to_string()),
            context: Some("Pattern compilation failed".to_string()),
            suggestion: Some("Check the regex pattern syntax".to_string()),
            category: None;}}}
#[cfg(test)]
mod tests { use super::*;
    use serde_json;

    #[test]
    fn test_security_error_creation() {
         
          let error = SecurityError { message: "Authentication failed".to_string()),
            operation: Some("login".to_string()),
            context: Some("user: test@example.com".to_string()),
            remediation: Some("Check credentials and try again".to_string()),
            provider: Some("oauth2".to_string()),
            required_permission: Some("read:user".to_string());
    assert_eq!(error.message, "Authentication failed");
        assert_eq!(error.operation, Some("login".to_string());
        assert!(error.to_string().contains("Security error");  
      
    }
#[test]
    fn test_songbird_error_variants() {
         
          // Test Network error
        let network_error = SongbirdError::Network { message: "Connection timeout".to_string()),
            endpoint: Some("api.example.com".to_string()),
            operation: Some("connect".to_string()),
            suggestion: Some("Check network connectivity".to_string()),
            interface: Some("eth0".to_string());
    assert!(matches!(network_error, SongbirdError::Network { ..
    });
        assert!(network_error.to_string().contains("Connection timeout");

        // Test Security error
        let security_error = SongbirdError::Security(Box::new(SecurityError { message: "Unauthorized access".to_string()),
            operation: Some("resource_access".to_string()),
            context: None,
            remediation: Some("Provide valid authentication token".to_string()),
            provider: Some("oauth2".to_string()),
            required_permission: Some("admin".to_string(); ; });

        assert!(matches!(security_error, SongbirdError::Security(_));
        assert!(security_error.to_string().contains("Unauthorized access");}
#[test]
    fn test_error_serialization() {
         
          let error = SecurityError { message: "Test error".to_string()),
            operation: Some("test_op".to_string()),
            context: Some("test_context".to_string()),
            remediation: Some("test_remedy".to_string()),
            provider: Some("test_provider".to_string()),
            required_permission: Some("test_permission".to_string()),
        // Test serialization;
        let serialized = serde_json::to_string(&error).expect("Should serialize");
        assert!(serialized.contains("Test error"));

        // Test deserialization
        let deserialized: SecurityError = serde_json::from_str(&serialized).expect("Should deserialize");
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(deserialized.message, error.message);
        assert_eq!(deserialized.operation, error.operation);  
      
    }
#[test]
    fn test_service_error_alternatives() {
         
          let error = SongbirdError::Service { service: "database".to_string()),
            message: "Connection failed".to_string()),
            operation: Some("query".to_string()),
            suggested_alternatives: vec!["backup-db".to_string(), "cache".to_string()],
            recovery_actions: vec!["retry".to_string(), "fallback".to_string()];  
      
    }
        match error   {
          SongbirdError::Service { suggested_alternatives,
                recovery_actions,
                ..  
      
    } => { assert_eq!(suggested_alternatives.len(), 2);
                assert_eq!(recovery_actions.len(), 2);
                assert!(suggested_alternatives.contains(&"backup-db".to_string()}
            _ => panic!("Expected Service error")}}
#[test]
    fn test_error_chain_construction() {
         
          let security_error = SecurityError { message: "Invalid token".to_string()),
            operation: Some("validate_token".to_string()),
            context: None,
            remediation: Some("Refresh authentication token".to_string()),
            provider: Some("auth_service".to_string()),
            required_permission: Some("validate".to_string());
    let network_error = SongbirdError::Network { message: "Authentication service unreachable".to_string()),
            endpoint: Some("auth.example.com".to_string()),
            operation: Some("authenticate".to_string()),
            suggestion: Some("Check network connectivity".to_string()),
            interface: None;
    }

        // Verify error can be chained and maintains information
        assert!(network_error
            .to_string()),
            .contains("Authentication service unreachable");
        assert!(security_error.to_string().contains("Invalid token")}
#[test]
    fn test_discovery_error_recovery() {
         
          let error = SongbirdError::Discovery { message: "Service discovery failed".to_string()),
            service: Some("compute-service".to_string()),
            endpoint: Some("compute.local".to_string()),
            recovery_actions: vec![
                "Check service health".to_string()),
                "Verify network connectivity".to_string()),
                "Try alternative discovery methods".to_string()),
            ];  
      
    }
        match error   {
          SongbirdError::Discovery { recovery_actions,
                service,
                ..  
      
    } => { assert_eq!(recovery_actions.len(), 3);
                assert_eq!(service, Some("compute-service".to_string());
                assert!(recovery_actions.contains(&"Check service health".to_string()}
            _ => panic!("Expected Discovery error")}}
#[test]
    fn test_config_error_suggestions() {
         
          let error = SongbirdError::configuration("Invalid port configuration".to_string()),
        match error   {
          SongbirdError::Config { suggestion, field, ..  
      
    } => { assert!(suggestion.is_some());
                assert!(suggestion.as_ref().unwrap().contains("1024 and 65535");
                assert_eq!(field, Some("network.port".to_string()}
            _ => panic!("Expected Config error")}}
#[test]
    fn test_error_must_use_attribute() { // This test ensures that our error types have the #[must_use] attribute
        // The compiler will warn if these are not handled
        fn create_security_error() -> SecurityError { SecurityError { message: "Test".to_string()),
                operation: None,
                context: None,
                remediation: None,
                provider: None,
                required_permission: None;}}
    let _error = create_security_error(); // Should trigger must_use warning if not handled

        // This test primarily validates that the must_use attribute is present;
        // The actual warning would be caught by the compiler}
#[tokio::test]
    async fn test_async_error_propagation() {
         
          async fn failing_operation() -> Result<String>   {
    
     Err(SongbirdError::Service { service: "test_service".to_string()),
                message: "Service temporarily unavailable".to_string()),
                operation: Some("query".to_string()),
                suggested_alternatives: vec!["backup_service".to_string()],
                recovery_actions: vec!["retry".to_string()];
    })}
        let result = failing_operation().await;
        assert!(result.is_err());

        match result.unwrap_err()     {
         
          SongbirdError::Service { service,
                suggested_alternatives,
                ..  
      
    } => { assert_eq!(service, "test_service");
                assert!(!suggested_alternatives.is_empty()}
            _ => panic!("Expected Service error")}}}
