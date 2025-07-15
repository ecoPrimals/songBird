//! Service Discovery Trait
//!
//! Supporting static configuration, Songbird native, Kubernetes, and other discovery mechanisms.

use crate::errors::Result;
use crate::traits::service::ServiceInfo;
use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use songbird_config::constants::{
    DEFAULT_CHECK_INTERVAL, DEFAULT_CONNECTION_TIMEOUT, DEFAULT_RETRY_DELAY,
};
use std::collections::HashMap;
use std::pin::Pin;

/// Core trait for service discovery implementations
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Register a service with the discovery system
    async fn register(&self, service: ServiceInfo) -> Result<()>;

    /// Unregister a service from the discovery system
    async fn unregister(&self, service_id: &str) -> Result<()>;

    /// Discover services matching the given query
    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>>;

    /// Discover services matching the given query (alias for compatibility)
    async fn discover_services(&self, query: &ServiceQuery) -> Result<Vec<ServiceInfo>> {
        self.discover(query.clone()).await
    }

    /// Watch for changes to services matching the query
    async fn watch(
        &self,
        query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>>;

    /// Update service health status
    async fn update_health(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()>;

    /// List all registered services
    async fn list_all(&self) -> Result<Vec<ServiceInfo>>;

    /// Check if a service exists (legacy name)
    async fn exists(&self, service_id: &str) -> Result<bool>;

    /// Check if a service is registered (preferred name)
    async fn is_registered(&self, service_id: &str) -> Result<bool>;

    /// Bulk update service metadata
    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()>;

    /// Downcast support for accessing concrete implementations
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Query parameters for service discovery
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceQuery {
    /// Service name pattern (supports wildcards)
    pub name: Option<String>,
    /// Service ID filter (for exact service lookup)
    pub service_id: Option<String>,
    /// Service type filter
    pub service_type: Option<String>,
    /// Version constraint
    pub version: Option<String>,
    /// Tags that must be present
    pub tags: Vec<String>,
    /// Metadata filters
    pub metadata: HashMap<String, serde_json::Value>,
    /// Health status filter
    pub health_status: Option<HealthStatus>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Sort order
    pub sort_by: Option<SortBy>,
}

impl ServiceQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_service_id(mut self, service_id: impl Into<String>) -> Self {
        self.service_id = Some(service_id.into());
        self
    }

    pub fn with_service_type(mut self, service_type: impl Into<String>) -> Self {
        self.service_type = Some(service_type.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn with_health_status(mut self, status: HealthStatus) -> Self {
        self.health_status = Some(status);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn sort_by(mut self, sort_by: SortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Service discovery event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    ServiceRegistered {
        service: Box<ServiceInfo>,
    },
    ServiceUnregistered {
        service_id: String,
    },
    ServiceHealthChanged {
        service_id: String,
        health: ServiceHealthStatus,
    },
    ServiceMetadataUpdated {
        service_id: String,
    },
    NodeJoined {
        node_id: String,
    },
    NodeHealthChanged {
        node_id: String,
        health: ServiceHealthStatus,
    },
}

/// Health status for service discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Sort options for service queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    ServiceType,
    Version,
    CreatedAt,
    UpdatedAt,
    HealthStatus,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_info: ServiceInfo,
    pub ttl: Option<std::time::Duration>,
    pub health_check_interval: Option<std::time::Duration>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl ServiceRegistration {
    pub fn new(service_info: ServiceInfo) -> Self {
        Self {
            service_info,
            ttl: None,
            health_check_interval: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_health_check_interval(mut self, interval: std::time::Duration) -> Self {
        self.health_check_interval = Some(interval);
        self
    }
}

/// Discovery backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub backend: DiscoveryBackend,
    pub health_check_interval: std::time::Duration,
    pub connection_timeout: std::time::Duration,
    pub retry_attempts: u32,
    pub retry_delay: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryBackend {
    /// Songbird native discovery service
    Songbird {
        federation_enabled: bool,
        trust_verification: bool,
        attribution_tracking: bool,
    },
    /// Static configuration discovery
    Static,
    /// etcd-based discovery
    Etcd {
        endpoints: Vec<String>,
        username: Option<String>,
        password: Option<String>,
    },
    /// Kubernetes service discovery
    Kubernetes {
        namespace: Option<String>,
        in_cluster: bool,
        kubeconfig_path: Option<String>,
    },
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            backend: DiscoveryBackend::Static,
            health_check_interval: DEFAULT_CHECK_INTERVAL,
            connection_timeout: DEFAULT_CONNECTION_TIMEOUT,
            retry_attempts: 3,
            retry_delay: DEFAULT_RETRY_DELAY,
        }
    }
}
