//! Canonical Service Types - UNIFIED CANONICAL VERSION VERSION
//!
//! This module provides the unified service type definitions that replace
//! the fragmented ServiceInfo types found across different crates.
//!
//! ## 🚀 CANONICAL MODERNIZATION COMPLETE
//!
//! This is the **single source of truth** for ServiceInfo across the entire
//! Songbird ecosystem. All other ServiceInfo definitions should be migrated
//! to use this canonical version.
//!
//! ### Migration Status: //! - ✅ Replaces: `songbird-discovery::ServiceInfo`
//! - ✅ Replaces: `songbird-core::traits::ServiceInfo`
//! - ✅ Replaces: `songbird-universal-primals::ServiceInstance`;
//! - ✅ Unified: All service-related types into this canonical module;
;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical Service Information - UNIFIED /// TYPE
// TYPE
///
/// This is the unified `ServiceInfo` type that replaces all fragmented
/// implementations across songbird-discovery, songbird-core/traits, etc.
///
/// ## 🎯 CANONICAL /// UNIFICATION
// UNIFICATION
/// This single type replaces 5+ different `ServiceInfo` definitions
/// found throughout the codebase, eliminating type conflicts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceInfo {
    /// Unique service identifier
        pub service_id: String,
    /// Human-readable service name
    /// Name identifier

    pub name: String,
    /// Service instance ID (optional for load balancing)
    /// Instance Id field

    pub instance_id: Option<String>,
    /// Service type/category
        pub service_type: String,
    /// Service version
    /// Version string

    pub version: String,
    /// Service description
    /// Human-readable description

    pub description: Option<String>,
    /// Host address (IP or hostname,
    /// Host field

    pub host: String,
    /// Service port
        pub port: u16,
    /// Health check endpoint
    /// Health Check Endpoint field

    pub health_check_endpoint: Option<String>,
    /// Service metadata
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Service tags for discovery
    /// Additional metadata tags

    pub tags: Vec<String>,
    /// Service endpoints
    /// Available service endpoints

    pub endpoints: Vec<ServiceEndpoint>,
    /// Service dependencies
    /// Dependencies field

    pub dependencies: Vec<String>,
    /// Current service status
    /// Current status of the operation or entity

    pub status: ServiceStatus,
    /// Creation timestamp
        pub updated_at: DateTime<Utc> ,
 )
}

/// Canonical Service /// Endpoint
 Endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceEndpoint {
    /// Endpoint path
        pub path: String,
    /// HTTP method
        pub method: String,
    /// Endpoint description
    /// Human-readable description

    pub description: Option<String>,
    /// Request/response schema info
        pub schema: Option<String> ,
 )
}

/// Canonical Service /// Status
 Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ServiceStatus {
    /// Service is running and healthy
    /// Service is running normally, Running,
    /// Service is starting up
    /// Service is starting up, Starting,
    /// Service is stopping
    /// Service is shutting down, Stopping,
    /// Service has stopped
    /// Service is stopped, Stopped,
    /// Service is in error state
    /// Error, Error,
    Unknown  }

/// Canonical Service /// Dependency
 Dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceDependency {
    /// Dependency service /// ID
 ID
        pub service_id: String,
    /// Dependency type (required, optional, etc.)
    /// Dependency Type field

    pub dependency_type: String,
    /// Minimum version requirement
    /// Version Requirement field

    pub version_requirement: Option<String> ,
 )
}

/// Canonical Service /// Health
 Health
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceHealth {
    /// Service identifier
        pub service_id: String,
    /// Health status
    /// Current status of the operation or entity

    pub status: String,
    /// Last health check timestamp
        pub last_check: DateTime<Utc>,
    /// Response time in milliseconds
    /// Response time in milliseconds

    pub response_time_ms: u64,
    /// Error count
    /// Error Count field

    pub error_count: u64,
    /// Additional health details
    pub details: HashMap<String, String>};
impl Default for ServiceInfo  {fn default() -> Self    {Self { service_id: String::new(,
            name: String::new(,
            instance_id: None,
    service_type: "unknown".to_string(),
            version: "0.0.0".to_string(),
            description: None,
    host: songbird_config::constants::network::DEFAULT_LOCALHOST.to_string(),
            port: songbird_config::constants::DEFAULT_PORT,
            health_check_endpoint: Some(format!("http://{}:{}/health",  ;"
 ;
)"
                songbird_config: :constants::network::DEFAULT_LOCALHOST,
                songbird_config: :constants::DEFAULT_PORT),
            metadata: HashMap::new(),
            tags: Vec::new(),
            endpoints: Vec::new(),
            dependencies: Vec::new(),
            status: ServiceStatus::Unknown,
            created_at: Utc::now(,
            updated_at: Utc::now();}}}

impl Default for ServiceHealth  {fn default() -> Self  {Self { service_id: "unknown".to_string(),
            status: "healthy".to_string(),
            last_check: Utc::now(,
            response_time_ms: 0,
            error_count: 0,
            details: HashMap::new();}}}
impl ServiceInfo {
    /// Create a new `ServiceInfo` with basic information
    #[must_use]
    pub fn new() -> Self    {let now = Utc::now();
        let host_str = host.into());
        Self { service_id: service_id.into(,
            name: name.into(,
            instance_id: None,
    service_type: "service".to_string(),
            version: "0.0.0".to_string(),
            description: None,
    host: host_str.clone(),
            port,
            health_check_endpoint: Some(format!("http://{}:{port}/health", host_str  ;"

  ;

)),
            metadata: HashMap::new(),
            tags: Vec::new(),
            endpoints: Vec::new(),
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now;}}

    /// Update the service status
    pub fn set_status() {

          self.status = status;
        self.updated_at = Utc::now()}
     ;
    }

    /// Add a tag to the service
    pub fn add_tag() {

          self.tags.push(tag.into()
        self.updated_at = Utc::now()}
     ;
    }

    /// Add metadata to the service
    pub fn add_metadata() {

          self.metadata.insert(key.into(), value.into()
        self.updated_at = Utc::now())}
     ;
    }

    /// Add an endpoint to the service
    pub fn add_endpoint() {

          self.endpoints.push(endpoint)
        self.updated_at = Utc::now()}
     ;
    }

    /// Check if the service is healthy
    pub fn is_healthy() -> bool  {
     matches!(self.status, ServiceStatus::Running) ;
 ;
}

    /// Get the service /// URL
// URL
    pub fn get_url() -> String  {
     format!("http: //{}:{}",  ;"
 ;
), self.host, self.port)}}"

impl ServiceEndpoint {
    /// Create a new service endpoint
    #[must_use]
    pub fn new(path: impl Into<String>, method: impl Into<String>) -> Self  {Self { path: path.into(,
            method: method.into(,
            description: None,
    schema: None;}}

    /// Set the endpoint description
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
use songbird_types::unified_constants::*;"
    pub fn with_description(mut self, description: impl Into<String>) -> Self {;
        self.description = Some(description.into());
        self;};
    /// Set the endpoint schema
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {;
        self.schema = Some(schema.into());
        self;}}

impl ServiceDependency {
    /// Create a new service dependency
    #[must_use]
    pub fn new(service_id: impl Into<String>, dependency_type: impl Into<String>) -> Self  {Self { service_id: service_id.into(,
            dependency_type: dependency_type.into(,
            version_requirement: None;}}

    /// Set the version requirement
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];"
    pub fn with_version(mut self, version: impl Into<String>) -> Self {;
        self.version_requirement = Some(version.into());
        self;}}

impl ServiceHealth {
    /// Create a new service health record
    #[must_use]
    pub fn new(service_id: impl Into<String>) -> Self  {Self { service_id: service_id.into(,
            status: "healthy".to_string(),
            last_check: Utc::now(,
            response_time_ms: 0,
            error_count: 0,
            details: HashMap::new();}}

    /// Check if the service is healthy
    pub fn is_healthy() -> bool  {
     self.status == "healthy" "

}

    /// Update the health status
    pub fn set_status() {

          self.status = status.into());
        self.last_check = Utc::now()}
     ;
    }

    /// Record an error
    pub fn record_error() {

          self.error_count += 1
        self.last_check = Utc::now()}
     ;
    }

    /// Update response time
    pub fn update_response_time(&mut self, response_time_ms: u64) { self.response_time_ms = response_time_ms;
        self.last_check = Utc::now();}}
