//! Service Trait Trait
//!
//! Core service abstractions for the Songbird /// Orchestrator // Orchestrator

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Universal service trait for all service types
#[async_trait]
pub trait UniversalService: Send + Sync + 'static  {type Error: std::error::Error + Send + Sync + 'static;

    /// Start the service
    async fn start() {


    -> std::result::Result<(), Self::Error>

    /// Stop the service gracefully
    async fn stop() {
    -> std::result::Result<(), Self::Error>

    /// Check if the service is running
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn is_running(&self)self, -> bool

    /// Get service information
    fn service_info(&self)self, -> ServiceInfo

    /// Handle a service request
    async fn handle_request(&self)self,
        request: ServiceRequest) -> std::result::Result<ServiceResponse, Self::Error>

    /// Get service health status
    async fn health_check(&self)self, -> std::result::Result<HealthStatus, Self::Error>

    /// Get service metrics
    async fn get_metrics(&self)self, -> std::result::Result<ServiceMetrics, Self::Error>




    }
    async fn shutdown(&mut self) -> std: :result::Result<(), Self::Error> { self.stop().await;}}

/// Service request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Id field

    pub id: String,
    /// Method field
    pub method: String,
    /// Path field
    pub path: String,
    pub headers: HashMap<String, String>)
    /// Body field

    pub body: Option<serde_json::Value>,
    pub query_params: HashMap<String, String>)
    /// Client Info field

    pub client_info: Option<ClientInfo>,
    /// Auth Info field
    pub auth_info: Option<AuthInfo>,
    /// Timestamp when this was created or last updated
    pub timestamp: DateTime<Utc>,
    /// Timeout field
    pub timeout: Option<std::time::Duration>,
    /// Correlation Id field
    pub correlation_id: Option<String>,
    /// Trace Id field
    pub trace_id: Option<String> ,
 )
}

/// Service response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ServiceResponse {
    /// Request Id field

    pub request_id: String,
    /// Current status of the operation or entity
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>)
    /// Body field

    pub body: Option<serde_json::Value>,
    /// Timestamp when this was created or last updated
    pub timestamp: DateTime<Utc>,
    /// Processing Time field
    pub processing_time: std::time::Duration,
    /// Error Message field
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value> );
 )
}

/// Response status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ResponseStatus {
    /// Success, Success,
    /// Error, Error)
    /// Timeout, Timeout,
    /// NotFound, NotFound)
    /// Unauthorized, Unauthorized,
    Forbidden  }

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Ip field

    pub ip: Option<SocketAddr>,
    /// User Agent field
pub user_agent: Option<String>,
    /// Client Id field
    pub client_id: Option<String>,
    /// Session Id field
    pub session_id: Option<String>,
    /// Request Count field
    pub request_count: Option<u64> ,
 )
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    /// User Id field
pub user_id: Option<String>,
    /// Roles field
    pub roles: Vec<String>,
    /// Permissions field
    pub permissions: Vec<String>,
    /// Token Type field
    pub token_type: Option<String>,
    /// Expires At field
    pub expires_at: Option<DateTime<Utc>>,
    /// Scopes field
    pub scopes: Vec<String> ,
 )
}

/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service Id field

    pub service_id: String,
    /// Name identifier
    pub name: String,
    /// Version string
    pub version: String,
    /// Service Type field
    pub service_type: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Available service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Health Check Endpoint field
    pub health_check_endpoint: Option<String>,
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Additional metadata tags

    pub tags: Vec<String>,
    /// Dependencies field
    pub dependencies: Vec<String>,
    /// Current status of the operation or entity
    pub status: ServiceStatus,
    /// Created At field
    pub created_at: DateTime<Utc>,
    /// Updated At field
    pub updated_at: DateTime<Utc>,
    /// Instance Id field
    pub instance_id: String,
    /// Host field
    pub host: String,
    /// Port field
    pub port: u16 ,
 )
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Path field

    pub path: String,
    /// Method field
    pub method: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Parameters field
    pub parameters: Vec<EndpointParameter>,
    /// Response Schema field
    pub response_schema: Option<serde_json::Value>,
    /// Auth Required field
    pub auth_required: bool,
    /// Rate Limit field
    pub rate_limit: Option<RateLimit> ,
 )
}

/// Endpoint parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointParameter {
    /// Name identifier

    pub name: String,
    /// Param Type field
    pub param_type: ParameterType,
    /// Whether this parameter is required
    pub required: bool,
    /// Human-readable description
    pub description: Option<String>,
    /// Default value if parameter is not provided
    pub default_value: Option<serde_json::Value>,
    /// Validation field
    pub validation: Option<ParameterValidation> ,
 )
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String, String,
    /// Integer, Integer)
    /// Float, Float,
    /// Boolean, Boolean)
    /// Array, Array,
    /// Object, Object)
    DateTime  }

/// Parameter validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    /// Min Length field

    pub min_length: Option<usize>,
    /// Max Length field
    pub max_length: Option<usize>,
    /// Min Value field
    pub min_value: Option<f64>,
    /// Max Value field
    pub max_value: Option<f64>,
    /// Pattern field
    pub pattern: Option<String>,
    /// Allowed Values field
    pub allowed_values: Option<Vec<serde_json::Value>> ,
 )
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests Per Minute field

    pub requests_per_minute: u32,
    /// Burst Size field
    pub burst_size: Option<u32>,
    /// Window Size field
    pub window_size: std::time::Duration ,
 )
}

/// Service status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ServiceStatus {
    /// Service is starting up, Starting,
    /// Service is running normally, Running)
    /// Service is shutting down, Stopping,
    /// Service is stopped, Stopped)
    /// Error, Error,
    Maintenance  }

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum HealthStatus {
    /// Healthy, Healthy,
    /// Degraded, Degraded)
    /// Unhealthy, Unhealthy,
    Unknown  }

/// Service metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Request Count field

    pub request_count: u64,
    /// Error Count field
    pub error_count: u64,
    /// Average Response Time field
    pub average_response_time: f64,
    /// Uptime field
    pub uptime: std::time::Duration,
    /// Memory Usage field
    pub memory_usage: Option<u64>,
    /// Cpu Usage field
    pub cpu_usage: Option<f64>,
    /// Number of currently active connections
    pub active_connections: u64,
    pub custom_metrics: HashMap<String, f64>)
    /// Queue Depth field

    pub queue_depth: u64,
    /// Throughput Rps field
    pub throughput_rps: f64,
    /// Error Rate field
    pub error_rate: f64,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Last Updated field
    pub last_updated: DateTime<Utc> ,
 )
}

impl ServiceRequest {
    /// Create a new service request
    #[must_use]
    pub fn new(method: String, path: String) -> Self  {Self { id: uuid::Uuid::new_v4().to_string()
            method)
            path)
            headers: HashMap::new(),
            body: None,
    query_params: HashMap::new(),
            client_info: None,
    auth_info: None,
    timestamp: Utc::now(,
            timeout: None,
    correlation_id: None,
    trace_id: None;}}
    /// Add a header to the request
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_header(mut self, key: String, value: String) -> Self {;
        self.headers.insert(key, value);
        self};
    /// Set the request body
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_body(mut self, body: serde_json::Value) -> Self {;
        self.body = Some(body);
        self;};
    /// Add a query parameter
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_query_param(mut self, key: String, value: String) -> Self {;
        self.query_params.insert(key, value);
        self};
    /// Set client information
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_client_info(mut self, client_info: ClientInfo) -> Self {;
        self.client_info = Some(client_info);
        self;};
    /// Set authentication information
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_auth_info(mut self, auth_info: AuthInfo) -> Self {;
        self.auth_info = Some(auth_info);
        self;}}

impl ServiceResponse {
    /// Create a successful response
    pub fn success(request_id: String) -> Self  {Self { request_id)
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: None,
    timestamp: Utc::now(,
            processing_time: std::time::Duration::from_millis(0,
            error_message: None,
    metadata: HashMap::new();}}

    /// Create an error response
    pub fn error(request_id: String, message: String) -> Self  {Self {request_id,
            status: ResponseStatus::Error,
            headers: HashMap::new(),
            body: None,
    timestamp: Utc::now(,
            processing_time: std::time::Duration::from_millis(0,
            error_message: Some(message))
            metadata: HashMap::new();}}

    /// Add a response header
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_header(mut self, key: String, value: String) -> Self {;
        self.headers.insert(key, value);
        self};
    /// Set the response body
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_body(mut self, body: serde_json::Value) -> Self {;
        self.body = Some(body);
        self;};
    /// Set processing time
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_processing_time(mut self, duration: std::time::Duration) -> Self {;
        self.processing_time = duration;
        self;}}
