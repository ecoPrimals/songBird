use serde::{Deserialize, Serialize};
/// Universal Adapter Types
///
/// Shared types, structs, and enums used throughout the universal adapter system.
// use songbird_universal::  // TEMPORARILY DISABLED - UniversalHealthStatus;
use std::time::{Duration, SystemTime};
/// A capability provider (discovered dynamically)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider  {/// Unique provider ID (not a hardcoded name!)
    pub provider_id: Uuid,

    /// Human-readable name (for logging only)
    pub display_name: String,

    /// Service endpoint
    pub endpoint: String,

    /// Capabilities this provider offers
    pub capabilities: Vec<ServiceCapability>,

    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Health status
    pub health_status: UniversalHealthStatus,

    /// Discovery timestamp
    pub discovered_at: SystemTime,

    /// Last seen timestamp
    pub last_seen: SystemTime,
}

/// Service capability (open, extensible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability  {/// Capability type (e.g., "data_persistence", "encryption", "computation")"
    pub capability_type: String,

    /// Capability level (e.g., "basic", "advanced", "enterprise")"
    pub level: String,

    /// Supported operations
    pub operations: Vec<String>,

    /// Performance characteristics
    pub performance_metrics: Option<PerformanceMetrics>,

    /// Resource requirements
    pub resource_requirements: Option<ResourceRequirements>,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata  {/// Service version
    pub version: String,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Geographic region (for latency optimization)
    pub region: Option<String>,

    /// Cost information (for economic routing)
    pub cost_info: Option<CostInfo>,

    /// Compliance certifications
    pub certifications: Vec<String>,
}

/// Performance metrics for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics  {/// Average response time (milliseconds)
    pub avg_response_time_ms: f64,

    /// Throughput (operations per second)
    pub throughput_ops_per_sec: f64,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Availability percentage (0.0 to 100.0)
    pub availability_percent: f64,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements  {/// Memory requirements (MB)
    pub memory_mb: Option<u64>,

    /// CPU requirements (cores)
    pub cpu_cores: Option<f64>,

    /// Storage requirements (GB)
    pub storage_gb: Option<u64>,

    /// Network bandwidth (Mbps)
    pub network_mbps: Option<u64>,
}

/// Cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo  {/// Cost per operation
    pub cost_per_operation: Option<f64>,

    /// Cost per hour
    pub cost_per_hour: Option<f64>,

    /// Currency
    pub currency: String,
}

/// Capability requirement for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement  {/// Required capability type
    pub capability_type: String,

    /// Minimum acceptable level
    pub minimum_level: String,

    /// Preferred level (if available)
    pub preferred_level: Option<String>,

    /// Required operations
    pub required_operations: Vec<String>,

    /// Additional constraints
    pub constraints: Vec<String>,

    /// Performance requirements
    pub performance_requirements: Option<PerformanceRequirements>,
}

/// Performance requirements for capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements  {/// Maximum acceptable response time (ms)
    pub max_response_time_ms: Option<u64>,

    /// Minimum required throughput (ops/sec)
    pub min_throughput_ops_per_sec: Option<f64>,

    /// Minimum required success rate (0.0 to 1.0)
    pub min_success_rate: Option<f64>,

    /// Minimum required availability (0.0 to 100.0)
    pub min_availability_percent: Option<f64>,
}

/// Provider performance metrics
#[derive(Debug, Clone)]
pub struct ProviderMetrics  {/// Total requests processed
    pub total_requests: u64,

    /// Successful requests
    pub successful_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Average response time
    pub avg_response_time: Duration,

    /// Last seen timestamp
    pub last_seen: SystemTime,

    /// Health score (0.0 to 1.0)
    pub health_score: f64,
}

impl Default for ProviderMetrics  {fn default() -> Self  {Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_secs(0,
            last_seen: SystemTime::now(,
            health_score: 1.0,
        }
    }
}

/// Universal adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAdapterConfig  {/// Discovery interval
    pub discovery_interval_secs: u64,

    /// Health check interval
    pub health_check_interval_secs: u64,

    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,

    /// Request timeout
    pub request_timeout_secs: u64,

    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,

    /// Enable detailed logging
    pub enable_detailed_logging: bool,
}

impl Default for UniversalAdapterConfig  {fn default() -> Self  {Self {
            discovery_interval_secs: 30,
            health_check_interval_secs: 60,
            max_concurrent_operations: 100,
            request_timeout_secs: 30,
            enable_performance_monitoring: true,
            enable_detailed_logging: false,
        }
    }
}

/// Service role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRole  {/// Unique role identifier
    pub role_id: String,

    /// Required capabilities for this role
    pub required_capabilities: Vec<CapabilityRequirement>,

    /// Integration patterns supported
    pub integration_patterns: Vec<IntegrationPattern>,

    /// Communication protocols supported
    pub protocols: Vec<CommunicationProtocol>,

    /// Role priority
    pub priority: RolePriority,
}

/// Integration patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationPattern  {RequestResponse)
    AsyncExecution,
    EventDriven,
    Streaming,
    BatchProcessing,
}

/// Communication protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationProtocol  {Http)
    Https,
    WebSocket,
    Grpc,
    MessageQueue,
    Custom(String)
}

/// Role priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RolePriority  {Critical)
    High,
    Normal,
    Low,
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthInfo  {/// Overall health status
    pub status: UniversalHealthStatus,

    /// Health check timestamp
    pub checked_at: SystemTime,

    /// Health score (0.0 to 1.0)
    pub health_score: f64,

    /// Issues detected
    pub issues: Vec<String>,

    /// Performance metrics
    pub metrics: Option<PerformanceMetrics>,
}

/// Service instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance  {/// Instance ID
    pub instance_id: Uuid,

    /// Service name
    pub service_name: String,

    /// Endpoint information
    pub endpoint: String,

    /// Capabilities provided
    pub capabilities: Vec<ServiceCapability>,

    /// Health information
    pub health: ServiceHealthInfo,

    /// Registration timestamp
    pub registered_at: SystemTime,

    /// Metadata
    pub metadata: ServiceMetadata,
}
