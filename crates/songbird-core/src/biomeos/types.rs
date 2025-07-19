//! BiomeOS integration data types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple service manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdServiceManifest {
    pub name: String,
    pub version: String,
    pub port: u16,
    pub endpoints: Vec<String>,
}

/// BiomeOS connectivity status
#[derive(Debug, Clone, PartialEq)]
pub enum BiomeOSConnectivityStatus {
    /// Successfully connected to BiomeOS
    Connected,
    /// Connection failed or unavailable
    Disconnected,
    /// Connection is being established
    Connecting,
    /// Connection timed out
    TimedOut,
}

/// BiomeOS service registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceRegistration {
    pub service_id: String,
    pub service_name: String,
    pub service_version: String,
    pub endpoints: BiomeOSEndpoints,
    pub capabilities: BiomeOSCapabilities,
    pub security: BiomeOSSecurity,
    pub resource_requirements: BiomeOSResourceRequirements,
    pub health_check: BiomeOSHealthCheckConfig,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// BiomeOS service endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSEndpoints {
    pub main: String,
    pub health: String,
    pub metrics: Option<String>,
    pub management: Option<String>,
    pub additional: HashMap<String, String>,
}

/// BiomeOS service capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSCapabilities {
    pub supported_protocols: Vec<String>,
    pub max_concurrent_requests: Option<u32>,
    pub features: Vec<String>,
}

/// BiomeOS security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSSecurity {
    pub authentication_required: bool,
    pub supported_auth_methods: Vec<String>,
    pub tls_enabled: bool,
    pub certificate_info: Option<String>,
}

/// BiomeOS resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub storage_gb: Option<u64>,
    pub network_bandwidth_mbps: Option<u32>,
}

/// BiomeOS health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSHealthCheckConfig {
    pub endpoint: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

/// BiomeOS BYOB deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSByobDeploymentRequest {
    pub service_name: String,
    pub image: String,
    pub resources: BiomeOSServiceResources,
    pub environment: HashMap<String, String>,
    pub ports: Vec<u16>,
    pub metadata: HashMap<String, String>,
}

/// BiomeOS service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceSpec {
    pub name: String,
    pub version: String,
    pub image: String,
    pub resources: BiomeOSServiceResources,
    pub environment: HashMap<String, String>,
    pub ports: Vec<u16>,
}

/// BiomeOS service resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSServiceResources {
    pub cpu_limit: String,
    pub memory_limit: String,
    pub cpu_request: String,
    pub memory_request: String,
}

/// BiomeOS resource quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceQuotas {
    pub max_cpu_cores: f64,
    pub max_memory_gb: u64,
    pub max_storage_gb: u64,
}

/// BiomeOS BYOB deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSByobDeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub endpoints: Vec<String>,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub estimated_ready_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Primal coordination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationInfo {
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Primal coordination result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCoordinationResult {
    pub success: bool,
    pub coordinated_primals: u32,
    pub failed_coordinations: Vec<String>,
}

/// Songbird ecosystem status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdEcosystemStatus {
    pub total_orchestrators: u32,
    pub active_primals: u32,
    pub total_services: u32,
    pub health_score: f64,
    pub uptime_seconds: u64,
    pub version: String,
    pub biomeos_connected: bool,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Ecosystem message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessage {
    pub id: String,
    pub message_type: EcosystemMessageType,
    pub source: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub priority: u8,
    pub ttl_seconds: Option<u64>,
    pub requires_acknowledgment: bool,
}

/// Ecosystem message type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EcosystemMessageType {
    ServiceRegistration,
    ServiceDeregistration,
    HealthCheck,
    StatusUpdate,
    ConfigurationChange,
    PrimalCoordination,
    BiomeOSSync,
    Alert,
    Metrics,
}

/// Ecosystem message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessageResponse {
    pub message_id: String,
    pub success: bool,
    pub response_payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

/// BiomeOS system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSSystemStatus {
    pub system_id: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub total_services: u32,
    pub active_connections: u32,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub storage_usage_percent: f64,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// BiomeOS deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSDeploymentRequest {
    pub service_name: String,
    pub deployment_type: String,
    pub configuration: HashMap<String, serde_json::Value>,
    pub resource_requirements: BiomeOSResourceRequirements,
    pub metadata: HashMap<String, String>,
}

/// BiomeOS deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSDeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub service_endpoints: Vec<String>,
    pub deployment_details: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

/// BiomeOS resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSResourceInfo {
    pub available_cpu_cores: f64,
    pub available_memory_gb: u64,
    pub available_storage_gb: u64,
    pub network_bandwidth_mbps: u32,
    pub resource_quotas: BiomeOSResourceQuotas,
    pub current_usage: HashMap<String, f64>,
}

impl Default for BiomeOSHealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
            healthy_threshold: 3,
            unhealthy_threshold: 3,
        }
    }
}

impl Default for BiomeOSCapabilities {
    fn default() -> Self {
        Self {
            supported_protocols: vec!["HTTP".to_string(), "HTTPS".to_string()],
            max_concurrent_requests: Some(1000),
            features: vec!["discovery".to_string(), "health-checks".to_string()],
        }
    }
}

impl BiomeOSConnectivityStatus {
    /// Check if status indicates connection is active
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if status indicates connection is in progress
    pub fn is_connecting(&self) -> bool {
        matches!(self, Self::Connecting)
    }

    /// Check if connection has failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Disconnected | Self::TimedOut)
    }
}

impl EcosystemMessage {
    /// Create new ecosystem message
    pub fn new(
        message_type: EcosystemMessageType,
        source: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            source,
            target: None,
            payload,
            timestamp: chrono::Utc::now(),
            priority: 5,            // Default priority
            ttl_seconds: Some(300), // 5 minutes default TTL
            requires_acknowledgment: false,
        }
    }

    /// Create targeted message
    pub fn new_targeted(
        message_type: EcosystemMessageType,
        source: String,
        target: String,
        payload: serde_json::Value,
    ) -> Self {
        let mut message = Self::new(message_type, source, payload);
        message.target = Some(target);
        message.requires_acknowledgment = true;
        message
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let elapsed = chrono::Utc::now()
                .signed_duration_since(self.timestamp)
                .num_seconds() as u64;
            elapsed > ttl
        } else {
            false
        }
    }
}
