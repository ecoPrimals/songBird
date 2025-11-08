//! Service Discovery Trait Trait
//!
//! Supporting static configuration, Songbird native, Kubernetes, and other discovery mechanisms.

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use songbird_config::constants::health::DEFAULT_CHECK_INTERVAL;
use songbird_config::constants::network::{DEFAULT_CONNECTION_TIMEOUT, DEFAULT_RETRY_DELAY};
use songbird_discovery::traits::service::ServiceInfo;
use songbird_types::SongbirdResult as Result;
use std::collections::HashMap;
use std::pin::Pin;

/// Core trait for service discovery implementations
#[async_trait]
pub trait ServiceDiscovery: Send + Sync { /// Register a service with the discovery system
    async fn register() {


    -> Result<()>

    /// Unregister a service from the discovery system
    async fn unregister() {
    -> Result<()>




    }
    async fn discover_services() -> Result<Vec<ServiceInfo>>   {

     self.discover(query.clone().await}
;
}

    /// Watch for changes to services matching the query
    async fn watch()  {-> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>>

    /// Update service health status
    async fn update_health()  {-> Result<()>

    /// List all registered services
    async fn list_all() -> Result<Vec<ServiceInfo>>

    /// Check if a service exists (legacy name,
    async fn exists(&self, service_id: &str) -> Result<bool>

    /// Check if a service is registered (preferred name,
    async fn is_registered(&self, service_id: &str) -> Result<bool>

    /// Bulk update service metadata
    async fn update_metadata(&self)self,
        service_id: &str,
        metadata: HashMap<String, String>) -> Result<()>




    }
pub struct ServiceQuery {
    /// Service name pattern (supports wildcards)
    /// Name identifier

    pub name: Option<String>,
    /// Service ID filter (for exact service lookup)
    /// Service Id field

    pub service_id: Option<String>,
    /// Service type filter
        pub service_type: Option<String>,
    /// Version constraint
    /// Version string

    pub version: Option<String>,
    /// Tags that must be present
    /// Additional metadata tags

    pub tags: Vec<String>,
    /// Metadata filters
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Health status filter
        pub health_status: Option<HealthStatus>,
    /// Maximum number of results
        pub limit: Option<usize>,
    /// Sort order
        pub sort_by: Option<SortBy> ;
,

)
}

impl ServiceQuery { #[must_use]
    pub fn new() -> Self { Self::default,
#[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"

    #[must_use]

;
    pub fn with_name(mut self, name: impl Into<String>) -> Self {;
        self.name = Some(name.into());
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
use songbird_types::unified_constants::*;
    pub fn with_service_id(mut self, service_id: impl Into<String>) -> Self {;
        self.service_id = Some(service_id.into());
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_service_type(mut self, service_type: impl Into<String>) -> Self {;
        self.service_type = Some(service_type.into());
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_version(mut self, version: impl Into<String>) -> Self {;
        self.version = Some(version.into());
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {;
        self.tags.push(tag.into());
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {;
        self.metadata.insert(key.into(), value);
        self};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_health_status(mut self, status: HealthStatus) -> Self {;
        self.health_status = Some(status);
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_limit(mut self, limit: usize) -> Self {;
        self.limit = Some(limit);
        self;};
    pub fn sort_by(mut self, sort_by: SortBy) -> Self { self.sort_by = Some(sort_by);
        self;}}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ServiceHealthStatus {
    /// Healthy, Healthy,
    /// Degraded, Degraded)
    /// Unhealthy, Unhealthy,
    Unknown  }

/// Service discovery event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent { ServiceRegistered { service: Box<ServiceInfo> }})
    ServiceUnregistered { service_id: String }})
    ServiceHealthChanged  {service_id: String,
    health: ServiceHealthStatus }})
    ServiceMetadataUpdated { service_id: String }})
    NodeJoined { node_id: String }})
    NodeHealthChanged  {node_id: String,
    health: ServiceHealthStatus;}}

/// Health status for service discovery
/// **CANONICAL**: Use unified health status from songbird-types
pub use songbird_types::health::CanonicalHealthStatus as HealthStatus;

/// Sort options for service queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    /// Name, Name,
    /// ServiceType, ServiceType)
    /// Version, Version,
    /// CreatedAt, CreatedAt)
    /// UpdatedAt, UpdatedAt,
    HealthStatus  }

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Service Info field

    pub service_info: ServiceInfo,
    /// Ttl field
    pub ttl: Option<std::time::Duration>,
    /// Health Check Interval field
    pub health_check_interval: Option<std::time::Duration>,
    /// Additional metadata tags
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String> )
 )
}

impl ServiceRegistration  {#[must_use]
    pub fn new(service_info: ServiceInfo) -> Self  {Self { service_info)
            ttl: None,
    health_check_interval: None,
    tags: Vec::new(),
            metadata: HashMap::new();}}
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_ttl(mut self, ttl: std::time::Duration) -> Self {;
        self.ttl = Some(ttl);
        self;};
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]"
;
    pub fn with_health_check_interval(mut self, interval: std::time::Duration) -> Self {;
        self.health_check_interval = Some(interval);
        self;}}

/// Discovery backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {
    /// Backend field

    pub backend: DiscoveryBackend,
    /// Health Check Interval field
    pub health_check_interval: std::time::Duration,
    /// Connection Timeout field
    pub connection_timeout: std::time::Duration,
    /// Retry Attempts field
    pub retry_attempts: u32,
    /// Retry Delay field
    pub retry_delay: std::time::Duration ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryBackend {
    /// Songbird native discovery service
    Songbird  {federation_enabled: bool,
        trust_verification: bool,
        attribution_tracking: bool }})
    /// Static configuration discovery
    /// Static, Static,
    /// etcd-based discovery
    Etcd  {endpoints: Vec<String>,
        username: Option<String>,
        password: Option<String> }})
    /// Kubernetes service discovery
    container_orchestration  {namespace: Option<String>,
        in_cluster: bool,
        kubeconfig_path: Option<String>;}}

impl Default for CanonicalDiscoveryConfig  {fn default() -> Self  {Self { backend: DiscoveryBackend::Static,
            health_check_interval: DEFAULT_CHECK_INTERVAL,
    connection_timeout: DEFAULT_CONNECTION_TIMEOUT,
    retry_attempts: 3,
            retry_delay: DEFAULT_RETRY_DELAY;}}}
