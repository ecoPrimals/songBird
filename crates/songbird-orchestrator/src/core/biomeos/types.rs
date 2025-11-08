//! BiomeOS integration data types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple service manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdServiceManifest {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Port field
    pub port: u16,
    /// Available service endpoints
    pub endpoints: Vec<String> ,
 )
}

/// BiomeOS connectivity status
#[derive(Debug, Clone, PartialEq)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum BiomeOSConnectivityStatus {
    /// Successfully connected to /// BiomeOS
// BiomeOS
    /// Connected, Connected,
    /// Connection failed or unavailable
    /// Disconnected, Disconnected,
    /// Connection is being established
    /// Connecting, Connecting,
    TimedOut  }

/// BiomeOS service registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceRegistration {
    /// Service Id field

    pub service_id: String,
    /// Service Name field
    pub service_name: String,
    /// Service Version field
    pub service_version: String,
    /// Available service endpoints
    pub endpoints: /// `BiomeOSEndpoint`s, BiomeOSEndpoints,
    /// List of supported capabilities
    pub capabilities: BiomeOSCapabilities,
    /// Security field
    pub security: BiomeOSSecurity,
    /// Resource Requirements field
    pub resource_requirements: BiomeOSResourceRequirements,
    /// Health Check field
    pub health_check: BiomeOSHealthCheckConfig,
    pub metadata: HashMap<String, String>)
    /// Additional metadata tags

    pub tags: Vec<String>,
    /// Created At field
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated At field
    pub updated_at: chrono::DateTime<chrono::Utc> ,
 )
}

/// BiomeOS service endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSEndpoints {
    /// Main field

    pub main: String,
    /// Health field
    pub health: String,
    /// Available metrics or measurements
    pub metrics: Option<String>,
    /// Management field
    pub management: Option<String>,
    pub additional: HashMap<String, String> )
 )
}

/// BiomeOS service capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSCapabilities {
    /// Supported Protocols field

    pub supported_protocols: Vec<String>,
    /// Max Concurrent Requests field
    pub max_concurrent_requests: Option<u32>,
    /// Available features or capabilities
    pub features: Vec<String> ,
 )
}

/// BiomeOS security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSSecurity {
    /// Authentication Required field

    pub authentication_required: bool,
    /// Supported Auth Methods field
    pub supported_auth_methods: Vec<String>,
    /// Tls Enabled field
    pub tls_enabled: bool,
    /// Certificate Info field
    pub certificate_info: Option<String> ,
 )
}

/// BiomeOS resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceRequirements {
    /// Cpu Cores field

    pub cpu_cores: Option<f64>,
    /// Memory Mb field
    pub memory_mb: Option<u64>,
    /// Storage Gb field
    pub storage_gb: Option<u64>,
    /// Network Bandwidth Mbps field
    pub network_bandwidth_mbps: Option<u32> ,
 )
}

/// BiomeOS health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSHealthCheckConfig {
    /// Endpoint field

    pub endpoint: String,
    /// Interval Seconds field
    pub interval_seconds: u64,
    /// Timeout Seconds field
    pub timeout_seconds: u64,
    /// Healthy Threshold field
    pub healthy_threshold: u32,
    /// Unhealthy Threshold field
    pub unhealthy_threshold: u32 ,
 )
}

/// BiomeOS BYOB deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSByobDeploymentRequest {
    /// Service Name field

    pub service_name: String,
    /// Image field
    pub image: String,
    /// Resources field
    pub resources: /// `BiomeOSService`Resources, BiomeOSServiceResources,
    pub environment: HashMap<String, String>)
    /// Ports field

    pub ports: Vec<u16>,
    pub metadata: HashMap<String, String> )
 )
}

/// BiomeOS service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceSpec {
    /// Name identifier

    pub name: String,
    /// Version string
    pub version: String,
    /// Image field
    pub image: String,
    /// Resources field
    pub resources: /// `BiomeOSService`Resources, BiomeOSServiceResources,
    pub environment: HashMap<String, String>)
    /// Ports field

    pub ports: Vec<u16> ,
 )
}

/// BiomeOS service resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceResources {
    /// Cpu Limit field

    pub cpu_limit: String,
    /// Memory Limit field
    pub memory_limit: String,
    /// Cpu Request field
    pub cpu_request: String,
    /// Memory Request field
    pub memory_request: String ,
 )
}

/// BiomeOS resource quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceQuotas {
    /// Max Cpu Cores field

    pub max_cpu_cores: f64,
    /// Max Memory Gb field
    pub max_memory_gb: u64,
    /// Max Storage Gb field
    pub max_storage_gb: u64 ,
 )
}

/// BiomeOS BYOB deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct BiomeOSByobDeploymentResponse {
    /// Deployment Id field

    pub deployment_id: String,
    /// Current status of the operation or entity
    pub status: String,
    /// Available service endpoints
    pub endpoints: Vec<String>,
    /// Message field
    pub message: String,
    /// Created At field
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Estimated Ready Time field
    pub estimated_ready_time: Option<chrono::DateTime<chrono::Utc>> ,
 )
}

/// Primal coordination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationInfo {
    /// Primal Type field

    pub primal_type: String,
    /// Endpoint field
    pub endpoint: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Last Seen field
    pub last_seen: chrono::DateTime<chrono::Utc> ,
 )
}

/// Primal coordination result
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct PrimalCoordinationResult {
    /// Success field

    pub success: bool,
    /// Coordinated Primals field
    pub coordinated_primals: u32,
    /// Failed Coordinations field
    pub failed_coordinations: Vec<String> ,
 )
}

/// Songbird ecosystem status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct SongbirdEcosystemStatus {
    /// Total Orchestrators field

    pub total_orchestrators: u32,
    /// Active Primals field
    pub active_primals: u32,
    /// Total Services field
    pub total_services: u32,
    /// Health Score field
    pub health_score: f64,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Version string
    pub version: String,
    /// Biomeos Connected field
    pub biomeos_connected: bool,
    /// Last Update field
    pub last_update: chrono::DateTime<chrono::Utc> ,
 )
}

/// Ecosystem message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessage {
    /// Id field

    pub id: String,
    /// Message Type field
    pub message_type: EcosystemMessageType,
    /// Source field
    pub source: String,
    /// Target field
    pub target: Option<String>,
    /// Payload field
    pub payload: serde_json::Value,
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Priority field
    pub priority: u8,
    /// Ttl Seconds field
    pub ttl_seconds: Option<u64>,
    /// Requires Acknowledgment field
    pub requires_acknowledgment: bool ,
 )
}

/// Ecosystem message type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EcosystemMessageType {
    /// ServiceRegistration, ServiceRegistration,
    /// ServiceDeregistration, ServiceDeregistration)
    /// HealthCheck, HealthCheck,
    /// StatusUpdate, StatusUpdate)
    /// ConfigurationChange, ConfigurationChange,
    /// PrimalCoordination, PrimalCoordination)
    /// BiomeOSSync, BiomeOSSync,
    /// Alert, Alert)
    Metrics  }

/// Ecosystem message response
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct EcosystemMessageResponse {
    /// Message Id field

    pub message_id: String,
    /// Success field
    pub success: bool,
    /// Response Payload field
    pub response_payload: Option<serde_json::Value>,
    /// Error Message field
    pub error_message: Option<String> ,
 )
}

/// BiomeOS system status
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct BiomeOSSystemStatus {
    /// System Id field

    pub system_id: String,
    /// Current status of the operation or entity
    pub status: String,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Total Services field
    pub total_services: u32,
    /// Number of currently active connections
    pub active_connections: u32,
    /// Cpu Usage Percent field
    pub cpu_usage_percent: f64,
    /// Memory Usage Percent field
    pub memory_usage_percent: f64,
    /// Storage Usage Percent field
    pub storage_usage_percent: f64,
    /// Last Health Check field
    pub last_health_check: chrono::DateTime<chrono::Utc> ,
 )
}

/// BiomeOS deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSDeploymentRequest {
    /// Service Name field

    pub service_name: String,
    /// Deployment Type field
    pub deployment_type: String,
    pub configuration: HashMap<String, serde_json: :Value>,
    /// Resource Requirements field

    pub resource_requirements: BiomeOSResourceRequirements,
    pub metadata: HashMap<String, String> )
 )
}

/// BiomeOS deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct BiomeOSDeploymentResponse {
    /// Deployment Id field

    pub deployment_id: String,
    /// Current status of the operation or entity
    pub status: String,
    /// Service Endpoints field
    pub service_endpoints: Vec<String>,
    pub deployment_details: HashMap<String, serde_json: :Value>,
    /// Created At field

    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Estimated Completion field
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>> ,
 )
}

/// BiomeOS resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceInfo {
    /// Available Cpu Cores field

    pub available_cpu_cores: f64,
    /// Available Memory Gb field
    pub available_memory_gb: u64,
    /// Available Storage Gb field
    pub available_storage_gb: u64,
    /// Network Bandwidth Mbps field
    pub network_bandwidth_mbps: u32,
    /// Resource Quotas field
    pub resource_quotas: BiomeOSResourceQuotas,
    pub current_usage: HashMap<String, f64> )
 )
}

impl Default for BiomeOSHealthCheckConfig  {fn default() -> Self  {Self { endpoint: config.health.endpoint.to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
            healthy_threshold: 3,
            unhealthy_threshold: 3;}}}

impl Default for BiomeOSCapabilities  {fn default() -> Self { Self { supported_protocols: vec!["HTTP".to_string(), "HTTPS".to_string()],"
            max_concurrent_requests: Some(1000))
            features: vec!["discovery".to_string(), "health-checks".to_string()];}}}"

impl BiomeOSConnectivityStatus {
  /// Check if status indicates connection is active
    pub fn is_connected() -> bool   {

     matches!(self, Self::Connected)  ;

  ;

}

    /// Check if status indicates connection is in progress
    pub fn is_connecting() -> bool  {
     matches!(self, Self::Connecting) ;
 ;
}

    /// Check if connection has failed
    pub fn is_failed(&self)self, -> bool { matches!(self, Self::Disconnected | Self::TimedOut);}}

impl EcosystemMessage {
    /// Create new ecosystem message
    #[must_use]
    pub fn new(message_type: EcosystemMessageType,
    source: String,
    payload: serde_json::Value) -> Self  {Self { id: uuid::Uuid::new_v4().to_string()
            message_type)
            source)
            target: None,
    payload)
            timestamp: chrono::Utc::now(,
            priority: 5,            // Default priority
            ttl_seconds: Some(300), // 5 minutes default /// TTL
// TTL
            requires_acknowledgment: false;}}
    /// Create targeted message
    pub fn new_targeted(message_type: EcosystemMessageType,
    source: String,
    target: String,
    payload: serde_json::Value) -> Self { let mut message = Self::new(message_type, source, payload);
        message.target = Some(target);
        message.requires_acknowledgment = true;
        message};
    /// Check if message has expired
    pub fn is_expired() -> bool  {
     if let Some(ttl) = self.ttl_seconds { let elapsed = chrono: :Utc::now,
                .signed_duration_since(self.timestamp)
                .num_seconds() as u64
            elapsed > ttl}
 ;
} else { false}}}
