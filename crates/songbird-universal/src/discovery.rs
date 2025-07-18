//! Universal discovery types and patterns

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::{CapabilityRequirement, PrimalType, ServiceCapability, ServiceHealth, ServiceMetrics};

/// Universal service query
#[derive(Debug, Clone)]
pub struct UniversalServiceQuery {
    /// Required capabilities for service matching
    pub required_capabilities: Vec<CapabilityRequirement>,

    /// Optional primal type filter
    pub primal_type: Option<PrimalType>,

    /// Only return healthy services
    pub healthy_only: bool,

    /// Metadata filters (key-value pairs)
    pub metadata_filters: HashMap<String, String>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Sort preference for results
    pub sort_preference: SortPreference,

    /// Include services from specific regions
    pub regions: Option<Vec<String>>,

    /// Include services with specific tags
    pub tags: Option<Vec<String>>,
}

impl Default for UniversalServiceQuery {
    fn default() -> Self {
        Self {
            required_capabilities: Vec::new(),
            primal_type: None,
            healthy_only: true,
            metadata_filters: HashMap::new(),
            limit: None,
            sort_preference: SortPreference::Health,
            regions: None,
            tags: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SortPreference {
    Health,
    Performance,
    LastSeen,
    PrimalType,
    Capability,
}

/// Filters to apply when discovering services
#[derive(Debug, Clone, Default)]
pub struct DiscoveryFilters {
    /// Filter by service types
    pub service_types: Option<Vec<String>>,
    /// Filter by primal types
    pub primal_types: Option<Vec<PrimalType>>,
    /// Filter by capabilities
    pub capabilities: Option<Vec<String>>,
    /// Filter by health status
    pub health_status: Option<ServiceHealth>,
    /// Filter by network location
    pub network_location: Option<String>,
    /// Filter by minimum uptime
    pub min_uptime: Option<Duration>,
}

/// Universal service info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub name: String,
    pub primal_type: PrimalType,
    pub instance_id: String,
    pub version: String,
    pub capabilities: Vec<ServiceCapability>,
    pub endpoints: Vec<crate::ServiceEndpoint>,
    pub health: ServiceHealth,
    pub metrics: ServiceMetrics,
    pub metadata: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub region: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Events emitted by the discovery system
#[derive(Debug, Clone)]
pub enum ServiceEvent {
    /// A service has been registered
    ServiceRegistered(Box<ServiceInfo>),
    /// A service has been deregistered
    ServiceDeregistered(String),
    /// A service's health has changed
    ServiceHealthChanged {
        service_id: String,
        health: ServiceHealth,
    },
    /// A service's capabilities have changed
    ServiceCapabilitiesChanged {
        service_id: String,
        capabilities: Vec<String>,
    },
}

/// Universal watch handle
pub struct WatchHandle {
    pub id: Uuid,
    pub query: UniversalServiceQuery,
    pub created_at: DateTime<Utc>,
}

/// Universal service watch callback
pub type ServiceWatchCallback =
    Box<dyn Fn(ServiceEvent) -> Result<(), crate::DiscoveryError> + Send + Sync>;
