//! Service Trait
//!
//! Core service abstractions for the Songbird Orchestrator

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

/// Universal service trait for all service types
#[async_trait]
pub trait UniversalService: Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Start the service
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the service gracefully
    async fn stop(&mut self) -> Result<(), Self::Error>;

    /// Check if the service is running
    fn is_running(&self) -> bool;

    /// Get service information
    fn service_info(&self) -> ServiceInfo;

    /// Handle a service request
    async fn handle_request(&self, request: ServiceRequest)
        -> Result<ServiceResponse, Self::Error>;

    /// Get service health status
    async fn health_check(&self) -> Result<HealthStatus, Self::Error>;

    /// Get service metrics
    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error>;

    /// Update service configuration
    async fn update_config(&mut self, config: serde_json::Value) -> Result<(), Self::Error>;

    /// Shutdown the service (alias for stop)
    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.stop().await
    }
}

/// Service request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub query_params: HashMap<String, String>,
    pub client_info: Option<ClientInfo>,
    pub auth_info: Option<AuthInfo>,
    pub timestamp: DateTime<Utc>,
    pub timeout: Option<Duration>,
    pub correlation_id: Option<String>,
    pub trace_id: Option<String>,
}

/// Service response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub request_id: String,
    pub status: ResponseStatus,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub processing_time: Duration,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error,
    Timeout,
    NotFound,
    Unauthorized,
    Forbidden,
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub ip: Option<SocketAddr>,
    pub user_agent: Option<String>,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
    pub request_count: Option<u64>,
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub token_type: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub name: String,
    pub version: String,
    pub service_type: String,
    pub description: Option<String>,
    pub endpoints: Vec<ServiceEndpoint>,
    pub health_check_endpoint: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub status: ServiceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub instance_id: String,
    pub host: String,
    pub port: u16,
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub path: String,
    pub method: String,
    pub description: Option<String>,
    pub parameters: Vec<EndpointParameter>,
    pub response_schema: Option<serde_json::Value>,
    pub auth_required: bool,
    pub rate_limit: Option<RateLimit>,
}

/// Endpoint parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointParameter {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub description: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<ParameterValidation>,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    DateTime,
}

/// Parameter validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_size: Option<u32>,
    pub window_size: Duration,
}

/// Service status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Maintenance,
}

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Service metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time: f64,
    pub uptime: Duration,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub active_connections: u64,
    pub custom_metrics: HashMap<String, f64>,
    pub queue_depth: u64,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub last_updated: DateTime<Utc>,
}

impl ServiceRequest {
    /// Create a new service request
    #[must_use]
    pub fn new(method: String, path: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            method,
            path,
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
            client_info: None,
            auth_info: None,
            timestamp: Utc::now(),
            timeout: Some(Duration::from_secs(30)),
            correlation_id: None,
            trace_id: None,
        }
    }

    /// Add a header to the request
    #[must_use]
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set the request body
    #[must_use]
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Add a query parameter
    #[must_use]
    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.insert(key, value);
        self
    }

    /// Set client information
    #[must_use]
    pub fn with_client_info(mut self, client_info: ClientInfo) -> Self {
        self.client_info = Some(client_info);
        self
    }

    /// Set authentication information
    #[must_use]
    pub fn with_auth_info(mut self, auth_info: AuthInfo) -> Self {
        self.auth_info = Some(auth_info);
        self
    }
}

impl ServiceResponse {
    /// Create a successful response
    #[must_use]
    pub fn success(request_id: String) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            processing_time: Duration::from_millis(0),
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(request_id: String, message: String) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            processing_time: Duration::from_millis(0),
            error_message: Some(message),
            metadata: HashMap::new(),
        }
    }

    /// Add a response header
    #[must_use]
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set the response body
    #[must_use]
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Set processing time
    #[must_use]
    pub const fn with_processing_time(mut self, duration: Duration) -> Self {
        self.processing_time = duration;
        self
    }
}
