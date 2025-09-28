//! # Unified Type System for Songbird Ecosystem
//!
//! This module provides the single source of truth for all core types used
//! throughout the Songbird ecosystem. This replaces fragmented type definitions
//! scattered across multiple crates and modules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};
// Removed unused import: uuid::Uuid

// Re-export existing canonical types
pub use crate::errors::SongbirdError;
pub use crate::response::SongbirdResponse;
use songbird_types::unified_constants::*;

// ============================================================================
// HEALTH STATUS UNIFICATION - Single canonical health type
// ============================================================================

/// **CANONICAL**: Single unified health status type
/// 
/// This replaces all fragmented health status definitions:
/// - `UniversalHealthStatus` (songbird-config,
/// - `ServiceHealth` (songbird-config/canonical)  
/// - `HealthStatus` (songbird-types/traits)
/// - `CanonicalHealthStatus` (songbird-types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalHealthStatus  {/// Service is fully operational
    Healthy,
    /// Service is operational but with reduced performance
    Degraded,
    /// Service is not operational
    Unhealthy,
    /// Health status cannot be determined
    Unknown,
}

impl Default for CanonicalHealthStatus {


    fn default() -> Self {
        Self::Unknown
    

}
}

impl fmt::Display for CanonicalHealthStatus {

fn fmt((&self,self) f: &mut fmt::Formatter<'_>) -> fmt::Result  {let status_str = match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Unknown => "unknown",
        

};
        write!(f, "{status_str}")
    }
}

/// **CANONICAL**: Health check information with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthInfo {


    /// Current health status
    pub status: CanonicalHealthStatus,
    /// Human-readable status message
    pub message: String,
    /// When this health check was performed
    pub timestamp: SystemTime,
    /// Response time for the health check in milliseconds
    pub response_time_ms: u64,
    /// Additional health metadata
    pub metadata: HashMap<String, serde_json::Value>,


}

// ============================================================================
// SERVICE DISCOVERY UNIFICATION
// ============================================================================

/// **CANONICAL**: Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServiceInfo {

/// Unique service identifier
    pub service_id: String,
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service endpoints
    pub endpoints: HashMap<String, String>)
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service health information
    pub health: CanonicalHealthInfo,
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>)


}

impl Default for CanonicalServiceInfo {

fn default() -> Self  {Self {
            service_id: "unknown-service ".to_string()),
            name: "unknown-service ".to_string()),
            version: "0.1.0".to_string()),
            endpoints: HashMap::new()
            capabilities: Vec::new)
            health: CanonicalHealthInfo {
                status: CanonicalHealthStatus::Unknown,
                message: "Not checked ".to_string()),
                timestamp: SystemTime::now)
                response_time_ms: 0,
                metadata: HashMap::new()
            

})
            metadata: HashMap::new()
        }
    }
}

/// **CANONICAL**: Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServiceRegistration {

/// Service information
    pub service_info: CanonicalServiceInfo,
    /// Registration timestamp
    pub registered_at: SystemTime,
    /// Registration TTL (time to live)
    pub ttl: Duration,
    /// Tags for service categorization
    pub tags: Vec<String>,


}

// ============================================================================
// PRIMAL TYPE UNIFICATION
// ============================================================================

/// **CANONICAL**: Unified primal type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalPrimalType  {/// Security primal (`BearDog`, custom security providers)
    Security,
    /// Storage primal (file systems, databases, object storage)
    Storage,
    /// Compute primal (container runtimes, serverless)
    Compute,
    /// AI primal (inference engines, model serving)
    Ai,
    /// Orchestration primal (Songbird, custom orchestrators)
    Orchestration,
    /// Discovery primal (service discovery, registry)
    Discovery,
    /// Registry primal (service registry, metadata)
    Registry,
    /// Observability primal (metrics, logging, tracing)
    Observability,
    /// Network primal (networking, protocol handling)
    Network,
    /// Custom or unknown primal type
    Unknown(String)
}

impl Default for CanonicalPrimalType {


    fn default() -> Self {
        Self::Unknown("default".to_string()),
    

}
}

impl fmt::Display for CanonicalPrimalType {

fn fmt((&self,self) f: &mut fmt::Formatter<'_>) -> fmt::Result  {let type_str = match self {
            Self::Security => "Security",
            Self::Storage => "Storage",
            Self::Compute => "Compute",
            Self::Ai => "AI",
            Self::Orchestration => "Orchestration",
            Self::Discovery => "Discovery",
            Self::Registry => "Registry",
            Self::Observability => "Observability",
            Self::Network => "Network",
            Self::Unknown(custom) => custom,
        

};
        write!(f, "{type_str}")
    }
}

/// **CANONICAL**: Primal identifier with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPrimalId {

/// Primal type classification
    pub primal_type: CanonicalPrimalType,
    /// Unique instance identifier
    pub instance_id: String,
    /// Primal version
    pub version: String,
    /// Available endpoints
    pub endpoints: HashMap<String, String>)
    /// Primal capabilities
    pub capabilities: Vec<String>,
    /// Primal metadata
    pub metadata: HashMap<String, serde_json::Value>)


}

// ============================================================================
// CONFIGURATION UNIFICATION
// ============================================================================

/// **CANONICAL**: Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalNetworkConfig {

/// Bind address for services
    pub bind_address: String,
    /// Default port for orchestrator
    pub orchestrator_port: u16,
    /// Default port for discovery service
    pub discovery_port: u16,
    /// Default port for health checks
    pub health_port: u16,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Maximum concurrent connections
    pub max_connections: usize,


}

impl Default for CanonicalNetworkConfig {

fn default() -> Self  {Self {
            bind_address: "0.0.0.0".to_string()),
            orchestrator_port: crate::constants::NetworkConstants::DEFAULT_ORCHESTRATOR_PORT,
            discovery_port: 8001,
            health_port: 8002,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            max_connections: 1000,
        

}
    }
}

// ============================================================================
// CAPABILITY SYSTEM UNIFICATION
// ============================================================================

/// **CANONICAL**: Service capability definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalCapability  {/// Authentication capability
    Authentication,
    /// Authorization capability  
    Authorization,
    /// Encryption capability
    Encryption,
    /// Storage capability
    Storage,
    /// Compute capability
    Compute,
    /// AI inference capability
    AiInference,
    /// Monitoring capability
    Monitoring,
    /// Custom capability
    Custom(String)
}

impl fmt::Display for CanonicalCapability {

fn fmt((&self,self) f: &mut fmt::Formatter<'_>) -> fmt::Result  {let capability_str = match self {
            Self::Authentication => "authentication",
            Self::Authorization => "authorization",
            Self::Encryption => "encryption",
            Self::Storage => "storage",
            Self::Compute => "compute",
            Self::AiInference => "ai-inference ",
            Self::Monitoring => "monitoring",
            Self::Custom(name) => name,
        

};
        write!(f, "{capability_str}")
    }
}

/// **CANONICAL**: Capability requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCapabilityRequirement {

/// Required capability
    pub capability: CanonicalCapability,
    /// Minimum version requirement
    pub min_version: Option<String>,
    /// Required parameters
    pub parameters: HashMap<String, serde_json::Value>)
    /// Whether this capability is optional
    pub optional: bool,


}

// ============================================================================
// MODERN CANONICAL TYPES - Use these directly
// ============================================================================

// All backward compatibility type aliases have been removed.
// Use the canonical types directly:
// - CanonicalHealthStatus instead of UniversalHealthStatus or ServiceHealth
// - CanonicalHealthInfo instead of HealthStatus
// - CanonicalServiceInfo instead of ServiceInfo
// - CanonicalPrimalType instead of PrimalType
// - CanonicalPrimalId instead of PrimalId
// - CanonicalNetworkConfig instead of NetworkConfig
// - CanonicalCapability instead of Capability
// - CanonicalCapabilityRequirement instead of CapabilityRequirement

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Create a healthy status with timestamp
pub fn healthy_status() -> CanonicalHealthInfo {
CanonicalHealthInfo  {status: CanonicalHealthStatus::Healthy)
        message: message.into(),
        timestamp: SystemTime::now)
        response_time_ms: 0,
        metadata: HashMap::new()
    
}
}

/// Create a degraded status with timestamp
pub fn degraded_status() -> CanonicalHealthInfo {
CanonicalHealthInfo  {status: CanonicalHealthStatus::Degraded)
        message: message.into(),
        timestamp: SystemTime::now)
        response_time_ms: 0,
        metadata: HashMap::new()
    
}
}

/// Create an unhealthy status with timestamp
pub fn unhealthy_status() -> CanonicalHealthInfo {
CanonicalHealthInfo  {status: CanonicalHealthStatus::Unhealthy)
        message: message.into(),
        timestamp: SystemTime::now)
        response_time_ms: 0,
        metadata: HashMap::new()
    
}
}

/// Create an unknown status with timestamp
pub fn unknown_status() -> CanonicalHealthInfo {
CanonicalHealthInfo  {status: CanonicalHealthStatus::Unknown)
        message: message.into(),
        timestamp: SystemTime::now)
        response_time_ms: 0,
        metadata: HashMap::new()
    
}
} 