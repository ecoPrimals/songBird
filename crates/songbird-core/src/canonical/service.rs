//! Canonical Service Types - UNIFIED CANONICAL VERSION
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
//! ### Migration Status:
//! - ✅ Replaces: `songbird-discovery::ServiceInfo`
//! - ✅ Replaces: `songbird-core::traits::ServiceInfo`
//! - ✅ Replaces: `songbird-universal-primals::ServiceInstance`
//! - ✅ Unified: All service-related types into this canonical module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical Service Information - UNIFIED TYPE
///
/// This is the unified ServiceInfo type that replaces all fragmented
/// implementations across songbird-discovery, songbird-core/traits, etc.
///
/// ## 🎯 CANONICAL UNIFICATION
/// This single type replaces 5+ different ServiceInfo definitions
/// found throughout the codebase, eliminating type conflicts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub service_id: String,
    /// Human-readable service name
    pub name: String,
    /// Service instance ID (optional for load balancing)
    pub instance_id: Option<String>,
    /// Service type/category
    pub service_type: String,
    /// Service version
    pub version: String,
    /// Service description
    pub description: Option<String>,
    /// Host address (IP or hostname)
    pub host: String,
    /// Service port
    pub port: u16,
    /// Health check endpoint
    pub health_check_endpoint: Option<String>,
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Service tags for discovery
    pub tags: Vec<String>,
    /// Service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Service dependencies
    pub dependencies: Vec<String>,
    /// Current service status
    pub status: ServiceStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Canonical Service Endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceEndpoint {
    /// Endpoint path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Endpoint description
    pub description: Option<String>,
    /// Request/response schema info
    pub schema: Option<String>,
}

/// Canonical Service Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ServiceStatus {
    /// Service is running and healthy
    Running,
    /// Service is starting up
    Starting,
    /// Service is stopping
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service is in error state
    Error,
    /// Service status is unknown
    #[default]
    Unknown,
}

/// Canonical Service Dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceDependency {
    /// Dependency service ID
    pub service_id: String,
    /// Dependency type (required, optional, etc.)
    pub dependency_type: String,
    /// Minimum version requirement
    pub version_requirement: Option<String>,
}

/// Canonical Service Health
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceHealth {
    /// Service identifier
    pub service_id: String,
    /// Health status
    pub status: String,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error count
    pub error_count: u64,
    /// Additional health details
    pub details: HashMap<String, String>,
}

impl Default for ServiceInfo {
    fn default() -> Self {
        Self {
            service_id: String::new(),
            name: String::new(),
            instance_id: None,
            service_type: "unknown".to_string(),
            version: "0.0.0".to_string(),
            description: None,
            host: songbird_config::constants::network::DEFAULT_LOCALHOST.to_string(),
            port: songbird_config::constants::DEFAULT_PORT,
            health_check_endpoint: Some(format!(
                "http://{}:{}/health",
                songbird_config::constants::network::DEFAULT_LOCALHOST,
                songbird_config::constants::DEFAULT_PORT
            )),
            metadata: HashMap::new(),
            tags: Vec::new(),
            endpoints: Vec::new(),
            dependencies: Vec::new(),
            status: ServiceStatus::Unknown,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            service_id: "unknown".to_string(),
            status: "healthy".to_string(),
            last_check: Utc::now(),
            response_time_ms: 0,
            error_count: 0,
            details: HashMap::new(),
        }
    }
}

impl ServiceInfo {
    /// Create a new ServiceInfo with basic information
    pub fn new(
        service_id: impl Into<String>,
        name: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> Self {
        let now = Utc::now();
        let host_str = host.into();
        Self {
            service_id: service_id.into(),
            name: name.into(),
            instance_id: None,
            service_type: "service".to_string(),
            version: "0.0.0".to_string(),
            description: None,
            host: host_str.clone(),
            port,
            health_check_endpoint: Some(format!("http://{host_str}:{port}/health")),
            metadata: HashMap::new(),
            tags: Vec::new(),
            endpoints: Vec::new(),
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the service status
    pub fn set_status(&mut self, status: ServiceStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Add a tag to the service
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
        self.updated_at = Utc::now();
    }

    /// Add metadata to the service
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }

    /// Add an endpoint to the service
    pub fn add_endpoint(&mut self, endpoint: ServiceEndpoint) {
        self.endpoints.push(endpoint);
        self.updated_at = Utc::now();
    }

    /// Check if the service is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ServiceStatus::Running)
    }

    /// Get the service URL
    pub fn get_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl ServiceEndpoint {
    /// Create a new service endpoint
    pub fn new(path: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
            description: None,
            schema: None,
        }
    }

    /// Set the endpoint description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the endpoint schema
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }
}

impl ServiceDependency {
    /// Create a new service dependency
    pub fn new(service_id: impl Into<String>, dependency_type: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            dependency_type: dependency_type.into(),
            version_requirement: None,
        }
    }

    /// Set the version requirement
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version_requirement = Some(version.into());
        self
    }
}

impl ServiceHealth {
    /// Create a new service health record
    pub fn new(service_id: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            status: "healthy".to_string(),
            last_check: Utc::now(),
            response_time_ms: 0,
            error_count: 0,
            details: HashMap::new(),
        }
    }

    /// Check if the service is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == "healthy"
    }

    /// Update the health status
    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
        self.last_check = Utc::now();
    }

    /// Record an error
    pub fn record_error(&mut self) {
        self.error_count += 1;
        self.last_check = Utc::now();
    }

    /// Update response time
    pub fn update_response_time(&mut self, response_time_ms: u64) {
        self.response_time_ms = response_time_ms;
        self.last_check = Utc::now();
    }
}
