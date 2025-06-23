//! Universal Service Trait
//!
//! The core trait that any service can implement to work with Songbird Orchestrator

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

/// Core trait that any service must implement to be orchestrated by Songbird
///
/// This trait provides a universal interface for service lifecycle management,
/// health monitoring, and request handling that works across any domain or project.
#[async_trait]
pub trait UniversalService: Send + Sync + 'static {
    /// Service-specific configuration type
    type Config: Clone + Send + Sync + for<'de> Deserialize<'de> + Debug;

    /// Service-specific health information type
    type Health: Send + Sync + Serialize + Debug;

    /// Service-specific error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Initialize the service with configuration
    ///
    /// This is called once during service setup and should prepare
    /// the service for starting but not actually start it.
    async fn initialize(&mut self, config: Self::Config) -> std::result::Result<(), Self::Error>;

    /// Start the service
    ///
    /// Begin accepting requests and performing the service's core function.
    /// This should be idempotent - calling start on an already started service
    /// should be safe.
    async fn start(&mut self) -> std::result::Result<(), Self::Error>;

    /// Stop the service gracefully
    ///
    /// Cease accepting new requests and finish handling existing ones.
    /// This should be idempotent - calling stop on an already stopped service
    /// should be safe.
    async fn stop(&mut self) -> std::result::Result<(), Self::Error>;

    /// Restart the service
    ///
    /// Default implementation stops then starts, but services can override
    /// for more efficient restart procedures.
    async fn restart(&mut self) -> std::result::Result<(), Self::Error> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Perform a health check on the service
    ///
    /// Returns detailed health information specific to the service type.
    /// This should be fast and lightweight as it may be called frequently.
    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error>;

    /// Handle a service request
    ///
    /// Process a generic service request and return a response.
    /// The request/response format is standardized across all services.
    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse, Self::Error>;

    /// Update the service configuration
    ///
    /// Apply new configuration without restarting the service.
    /// Not all configuration changes may be supported - return an error
    /// if a restart is required.
    async fn update_config(&mut self, config: Self::Config)
        -> std::result::Result<(), Self::Error>;

    /// Get service metrics
    ///
    /// Return performance and operational metrics about the service.
    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics, Self::Error>;

    /// Get service information
    ///
    /// Return static information about this service instance.
    fn service_info(&self) -> ServiceInfo;

    /// Check if the service can handle additional load
    ///
    /// Used by load balancers to determine if requests should be routed here.
    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error>;

    /// Get current load factor (0.0 to 1.0)
    ///
    /// Used by load balancers for weighted routing decisions.
    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error>;

    /// Handle service shutdown signal
    ///
    /// Called when the orchestrator is shutting down.
    /// Should perform cleanup and prepare for process termination.
    async fn shutdown(&mut self) -> std::result::Result<(), Self::Error> {
        self.stop().await
    }
}

/// Generic service request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Unique request identifier
    pub id: String,

    /// Request method/action
    pub method: String,

    /// Request path or endpoint
    pub path: String,

    /// Request headers
    pub headers: HashMap<String, String>,

    /// Request payload
    pub payload: serde_json::Value,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,

    /// Request timeout
    pub timeout: Option<Duration>,

    /// Client information
    pub client_info: Option<ClientInfo>,

    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Generic service response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Response to request ID
    pub request_id: String,

    /// Response status
    pub status: ResponseStatus,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response payload
    pub payload: serde_json::Value,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Processing duration
    pub duration: Duration,

    /// Processing time in milliseconds (for load balancer compatibility)
    pub processing_time: u32,

    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error { code: u16, message: String },
    Timeout,
    RateLimit,
    ServiceUnavailable,
}

/// Client information for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client IP address
    pub ip_address: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Client session ID
    pub session_id: Option<String>,

    /// Authentication information
    pub auth_info: Option<AuthInfo>,
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    /// User/client identifier
    pub user_id: String,

    /// User roles
    pub roles: Vec<String>,

    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub id: String,

    /// Human-readable service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service type/category
    pub service_type: String,

    /// Service description
    pub description: String,

    /// Service endpoints
    pub endpoints: Vec<ServiceEndpoint>,

    /// Service capabilities/features
    pub capabilities: Vec<String>,

    /// Service tags for discovery
    pub tags: HashMap<String, String>,

    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceEndpoint {
    /// Endpoint path
    pub path: String,

    /// HTTP method or operation type
    pub method: String,

    /// Endpoint description
    pub description: String,

    /// Input parameters
    pub parameters: Vec<EndpointParameter>,

    /// Response schema
    pub response_schema: Option<serde_json::Value>,
}

/// Endpoint parameter definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointParameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub param_type: String,

    /// Whether parameter is required
    pub required: bool,

    /// Parameter description
    pub description: String,

    /// Default value
    pub default: Option<serde_json::Value>,
}

/// Service metrics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Total requests processed
    pub request_count: u64,

    /// Total errors encountered
    pub error_count: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// 95th percentile response time
    pub p95_response_time_ms: f64,

    /// 99th percentile response time
    pub p99_response_time_ms: f64,

    /// Current CPU usage (0.0 to 1.0)
    pub cpu_usage: f64,

    /// Current memory usage in bytes
    pub memory_usage: u64,

    /// Current active connections
    pub active_connections: u32,

    /// Current queue depth
    pub queue_depth: u32,

    /// Throughput in requests per second
    pub throughput_rps: f64,

    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Service uptime in seconds
    pub uptime_seconds: u64,

    /// Last metrics update time
    pub last_updated: DateTime<Utc>,

    /// Custom metrics specific to the service
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0,
            active_connections: 0,
            queue_depth: 0,
            throughput_rps: 0.0,
            error_rate: 0.0,
            uptime_seconds: 0,
            last_updated: Utc::now(),
            custom_metrics: HashMap::new(),
        }
    }
}

/// Helper implementations for ServiceRequest
impl ServiceRequest {
    /// Create a new service request
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            method: method.into(),
            path: path.into(),
            headers: HashMap::new(),
            payload: serde_json::Value::Null,
            timestamp: Utc::now(),
            timeout: None,
            client_info: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a header to the request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the request payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add metadata to the request
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Helper implementations for ServiceResponse
impl ServiceResponse {
    /// Create a successful response
    pub fn success(request_id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            request_id: request_id.into(),
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            payload,
            timestamp: Utc::now(),
            duration: Duration::from_millis(0),
            processing_time: 0,
            metadata: HashMap::new(),
        }
    }

    /// Create an error response
    pub fn error(request_id: impl Into<String>, code: u16, message: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ResponseStatus::Error {
                code,
                message: message.into(),
            },
            headers: HashMap::new(),
            payload: serde_json::Value::Null,
            timestamp: Utc::now(),
            duration: Duration::from_millis(0),
            processing_time: 0,
            metadata: HashMap::new(),
        }
    }

    /// Set response duration
    /// Set response duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self.processing_time = duration.as_millis() as u32;
        self
    }

    /// Add a response header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}
