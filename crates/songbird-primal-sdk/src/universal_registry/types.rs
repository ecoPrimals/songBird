//! Core types for Universal Service Registry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Import the needed types from other modules
use super::config::{IntegrationPreferences, ResourceSpec, ServiceEndpoint};

/// Universal Service Registration - ALL PARTICIPANTS MUST IMPLEMENT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration  {/// Unique service identifier (UUID)
    pub service_id: Uuid,

    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Capabilities this service provides
    pub capabilities: Vec<ServiceCapability>,

    /// Resource requirements and limits
    pub resources: ResourceSpec,

    /// API endpoints (dynamically discovered)
    pub endpoints: Vec<ServiceEndpoint>,

    /// Integration preferences
    pub integration: IntegrationPreferences,

    /// Extension points for custom data
    pub extensions: HashMap<String, serde_json::Value>)

    /// Registration timestamp
    pub registration_timestamp: DateTime<Utc>,

    /// Service version
    pub service_version: String,

    /// Instance identifier for multi-instance support
    pub instance_id: String,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata  {pub name: String,
    pub description: String,
    pub category: ServiceCategory,
    pub tags: Vec<String>,
    pub documentation_url: Option<String>,
    pub contact: ContactInfo,
    pub lifecycle_stage: ServiceLifecycleStage,
    pub compliance_level: ComplianceLevel,
}

/// Service categories for ecosystem organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceCategory  {/// Compute services (containers, serverless, etc.)
    Compute,
    /// Storage services (file systems, object storage, etc.)
    Storage,
    /// Network services (routing, load balancing, etc.)
    Network,
    /// Security services (authentication, encryption, etc.)
    Security,
    /// AI/ML services (inference, training, etc.)
    ArtificialIntelligence,
    /// Data services (databases, analytics, etc.)
    Data,
    /// Orchestration services (workflow management, etc.)
    Orchestration,
    /// Gaming services (game hosting, matchmaking, etc.)
    Gaming,
    /// Community-contributed services
    Community {
        subcategory: String,
    })
    /// Unknown or unclassified services
    Unknown,
}

/// Service capabilities (universal capability model)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceCapability  {/// Compute capabilities
    Compute  {cpu_cores: Option<f64>)
        memory_gb: Option<f64>,
        gpu_support: bool,
        container_runtime: Option<String>,
    })

    /// Storage capabilities
    Storage  {storage_gb: Option<f64>)
        storage_type: StorageType,
        backup_support: bool,
        encryption_support: bool,
    })

    /// Network capabilities
    Network  {bandwidth_mbps: Option<f64>)
        latency_ms: Option<f64>,
        protocols: Vec<String>,
        load_balancing: bool,
    })

    /// Security capabilities
    Security  {authentication_methods: Vec<String>)
        encryption_algorithms: Vec<String>,
        compliance_frameworks: Vec<String>,
        threat_detection: bool,
    })

    /// AI/ML capabilities
    ArtificialIntelligence  {models: Vec<String>)
        inference_support: bool,
        training_support: bool,
        frameworks: Vec<String>,
    })

    /// Generic capability with custom properties
    Custom  {name: String,
        properties: HashMap<String, String>)
    })
}

/// Storage type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StorageType  {FileSystem)
    ObjectStorage,
    BlockStorage,
    Database,
    Cache,
    Other(String)
}

/// Service lifecycle stages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceLifecycleStage  {Development)
    Testing,
    Staging,
    Production,
    Deprecated,
    Retired,
}

/// Compliance levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplianceLevel  {Basic)
    Enhanced,
    Enterprise,
    Sovereign,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo  {pub maintainer: String,
    pub email: Option<String>,
    pub documentation: Option<String>,
    pub support: Option<String>,
}

/// Service handle for tracking registered services
#[derive(Debug, Clone)]
pub struct ServiceHandle  {pub service_id: Uuid,
    pub last_heartbeat: DateTime<Utc>,
}

/// Service information for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo  {pub registration: UniversalServiceRegistration,
    pub health_status: HealthStatus,
    pub performance_metrics: HashMap<String, f64>)
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus  {Healthy)
    Degraded,
    Unhealthy,
    Unknown,
}
