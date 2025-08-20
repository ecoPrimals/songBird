/// Songbird Self-Discovery Module
///
/// This module defines ONLY what Songbird knows about itself.
/// Songbird should not have hardcoded knowledge of other primals.
/// All external service discovery is done via capability-based routing.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Songbird's own service capabilities (self-knowledge only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdSelfRegistration {
    /// Unique service identifier for this Songbird instance
    pub service_id: Uuid,

    /// Capabilities that THIS Songbird instance provides
    pub capabilities: Vec<ServiceCapability>,

    /// Endpoints that THIS Songbird instance exposes
    pub endpoints: Vec<ServiceEndpoint>,

    /// Instance metadata
    pub metadata: SongbirdMetadata,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    /// Capability type (dot-notation)
    pub capability_type: String,

    /// Capability level/maturity
    pub level: String,

    /// Specific constraints or features
    pub constraints: Vec<String>,
}

/// Service endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Type of endpoint
    pub endpoint_type: String,

    /// Full URL of the endpoint
    pub url: String,

    /// Health check path (if any)
    pub health_check_path: Option<String>,
}

/// Songbird instance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdMetadata {
    /// Instance name
    pub name: String,

    /// Version
    pub version: String,

    /// Description
    pub description: String,

    /// Additional metadata
    pub extensions: HashMap<String, serde_json::Value>,
}

impl SongbirdSelfRegistration {
    /// Create new Songbird self-registration (knows only itself)
    #[must_use]
    pub fn new() -> Self {
        let base_url = std::env::var("SONGBIRD_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:{}".to_string());

        let api_base_url = std::env::var("SONGBIRD_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:{}/api".to_string());

        Self {
            service_id: Uuid::new_v4(),
            capabilities: vec![
                // Primary orchestration capabilities
                ServiceCapability {
                    capability_type: "orchestration.networking".to_string(),
                    level: "advanced".to_string(),
                    constraints: vec![
                        "service_mesh".to_string(),
                        "load_balancing".to_string(),
                        "federation".to_string(),
                    ],
                },
                // Network discovery capabilities
                ServiceCapability {
                    capability_type: "networking.discovery".to_string(),
                    level: "native".to_string(),
                    constraints: vec![
                        "mdns".to_string(),
                        "dns_sd".to_string(),
                        "manual_registration".to_string(),
                    ],
                },
                // Federation coordination capabilities
                ServiceCapability {
                    capability_type: "orchestration.federation".to_string(),
                    level: "distributed".to_string(),
                    constraints: vec![
                        "multi_node".to_string(),
                        "cluster_management".to_string(),
                        "byob_coordination".to_string(),
                    ],
                },
                // Gaming network capabilities
                ServiceCapability {
                    capability_type: "networking.gaming".to_string(),
                    level: "specialized".to_string(),
                    constraints: vec![
                        "nat_traversal".to_string(),
                        "protocol_translation".to_string(),
                        "performance_optimization".to_string(),
                    ],
                },
            ],
            endpoints: vec![
                ServiceEndpoint {
                    endpoint_type: "orchestration".to_string(),
                    url: base_url.clone(),
                    health_check_path: Some("/health".to_string()),
                },
                ServiceEndpoint {
                    endpoint_type: "api".to_string(),
                    url: api_base_url.clone(),
                    health_check_path: Some("/api/health".to_string()),
                },
            ],
            metadata: SongbirdMetadata {
                name: "Songbird Universal Orchestrator".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "Universal service orchestration and networking primal".to_string(),
                extensions: HashMap::new(),
            },
        }
    }

    /// Get capabilities as string list for easy lookup
    #[must_use]
    pub fn get_capability_types(&self) -> Vec<String> {
        self.capabilities
            .iter()
            .map(|cap| cap.capability_type.clone())
            .collect()
    }

    /// Check if this Songbird instance provides a specific capability
    pub fn provides_capability(&self, capability_type: &str) -> bool {
        self.capabilities
            .iter()
            .any(|cap| cap.capability_type == capability_type)
    }
}

impl Default for SongbirdSelfRegistration {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability-based discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiscoveryConfig {
    /// Discovery methods to use
    pub discovery_methods: Vec<String>,

    /// Discovery timeout
    pub discovery_timeout_secs: u64,

    /// Capability cache TTL
    pub capability_cache_ttl_secs: u64,

    /// Manual service registrations (for services that don't self-advertise)
    pub manual_services: Vec<ManualServiceRegistration>,
}

/// Manual service registration for non-self-advertising services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualServiceRegistration {
    /// Service identifier
    pub service_id: String,

    /// Capabilities this service provides
    pub capabilities: Vec<String>,

    /// Service endpoint
    pub endpoint: String,

    /// Health check path (optional)
    pub health_check_path: Option<String>,
}

impl Default for CapabilityDiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_methods: vec![
                "mdns".to_string(),
                "dns_sd".to_string(),
                "manual".to_string(),
            ],
            discovery_timeout_secs:
                crate::config::constants::network::CAPABILITY_DISCOVERY_TIMEOUT_SECS,
            capability_cache_ttl_secs: crate::config::constants::network::CAPABILITY_CACHE_TTL_SECS,
            manual_services: Vec::new(),
        }
    }
}
