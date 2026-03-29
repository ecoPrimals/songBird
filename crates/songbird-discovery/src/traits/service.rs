// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service Trait
//!
//! Core service abstractions for the Songbird Orchestrator

#![expect(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

/// Universal service trait for all service types
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
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            method: method.into(),
            path: path.into(),
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
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
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
    pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(key.into(), value.into());
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
    pub fn success(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
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
    pub fn error(request_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ResponseStatus::Error,
            headers: HashMap::new(),
            body: None,
            timestamp: Utc::now(),
            processing_time: Duration::from_millis(0),
            error_message: Some(message.into()),
            metadata: HashMap::new(),
        }
    }

    /// Add a response header
    #[must_use]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn service_request_new_sets_method_path_and_timeout() {
        let req = ServiceRequest::new("GET", "/health");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/health");
        assert_eq!(req.timeout, Some(Duration::from_secs(30)));
        assert!(!req.id.is_empty());
    }

    #[test]
    fn service_request_builder_chain() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9_000);
        let req = ServiceRequest::new("POST", "/api")
            .with_header("X-Test", "1")
            .with_query_param("q", "x")
            .with_body(serde_json::json!({"a": 1}))
            .with_client_info(ClientInfo {
                ip: Some(addr),
                user_agent: Some("ua".to_string()),
                client_id: None,
                session_id: None,
                request_count: None,
            })
            .with_auth_info(AuthInfo {
                user_id: Some("u".to_string()),
                roles: vec!["r".to_string()],
                permissions: vec![],
                token_type: None,
                expires_at: None,
                scopes: vec![],
            });
        assert_eq!(req.headers.get("X-Test"), Some(&"1".to_string()));
        assert_eq!(req.query_params.get("q"), Some(&"x".to_string()));
        assert_eq!(req.body, Some(serde_json::json!({"a": 1})));
        assert!(req.client_info.is_some());
        assert!(req.auth_info.is_some());
    }

    #[test]
    fn service_response_success_and_error() {
        let ok = ServiceResponse::success("rid-1");
        assert_eq!(ok.status, ResponseStatus::Success);
        assert!(ok.error_message.is_none());

        let err = ServiceResponse::error("rid-2", "boom");
        assert_eq!(err.status, ResponseStatus::Error);
        assert_eq!(err.error_message.as_deref(), Some("boom"));
    }

    #[test]
    fn service_response_with_header_body_processing_time() {
        let r = ServiceResponse::success("r")
            .with_header("H", "v")
            .with_body(serde_json::json!([]))
            .with_processing_time(Duration::from_millis(42));
        assert_eq!(r.headers.get("H"), Some(&"v".to_string()));
        assert_eq!(r.processing_time, Duration::from_millis(42));
    }

    #[test]
    fn response_status_and_service_status_serde_roundtrip() {
        let rs = ResponseStatus::Unauthorized;
        let json = serde_json::to_string(&rs).unwrap();
        let back: ResponseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(rs, back);

        let ss = ServiceStatus::Maintenance;
        let json = serde_json::to_string(&ss).unwrap();
        let back: ServiceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(ss, back);
    }

    #[test]
    fn health_status_serde_roundtrip() {
        let h = HealthStatus::Degraded;
        let json = serde_json::to_string(&h).unwrap();
        let back: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn parameter_type_and_rate_limit_serde_roundtrip() {
        let pt = ParameterType::DateTime;
        let json = serde_json::to_string(&pt).unwrap();
        let back: ParameterType = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);

        let rl = RateLimit {
            requests_per_minute: 60,
            burst_size: Some(10),
            window_size: Duration::from_secs(60),
        };
        let json = serde_json::to_string(&rl).unwrap();
        let back: RateLimit = serde_json::from_str(&json).unwrap();
        assert_eq!(rl.requests_per_minute, back.requests_per_minute);
    }

    #[test]
    fn service_metrics_serde_roundtrip() {
        let m = ServiceMetrics {
            request_count: 1,
            error_count: 0,
            average_response_time: 0.1,
            uptime: Duration::from_secs(1),
            memory_usage: Some(100),
            cpu_usage: Some(0.5),
            active_connections: 2,
            custom_metrics: HashMap::new(),
            queue_depth: 0,
            throughput_rps: 10.0,
            error_rate: 0.0,
            uptime_seconds: 1,
            last_updated: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ServiceMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(m.request_count, back.request_count);
        assert_eq!(m.throughput_rps, back.throughput_rps);
    }

    #[test]
    fn service_endpoint_serde_roundtrip() {
        let ep = ServiceEndpoint {
            path: "/x".to_string(),
            method: "GET".to_string(),
            description: None,
            parameters: vec![EndpointParameter {
                name: "id".to_string(),
                param_type: ParameterType::String,
                required: true,
                description: None,
                default_value: None,
                validation: None,
            }],
            response_schema: None,
            auth_required: false,
            rate_limit: None,
        };
        let json = serde_json::to_string(&ep).unwrap();
        let back: ServiceEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(ep.path, back.path);
        assert_eq!(ep.parameters.len(), back.parameters.len());
    }
}
