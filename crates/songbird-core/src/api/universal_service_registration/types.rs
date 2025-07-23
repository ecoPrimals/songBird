//! Universal Service Registration Types
//!
//! Type definitions and data structures for universal service registration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for universal service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistrationConfig {
    /// Maximum number of services that can be registered
    pub max_services: usize,

    /// Default health check interval in seconds
    pub default_health_check_interval: u64,

    /// Enable automatic service discovery
    pub enable_auto_discovery: bool,

    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,

    /// Registry storage backend configuration
    pub storage_config: HashMap<String, serde_json::Value>,
}

impl Default for UniversalServiceRegistrationConfig {
    fn default() -> Self {
        Self {
            max_services: 1000,
            default_health_check_interval: 30,
            enable_auto_discovery: true,
            enable_performance_monitoring: true,
            storage_config: HashMap::new(),
        }
    }
}

/// Service type enumeration for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    /// Web service (HTTP/REST API)
    WebService,
    /// Database service
    Database,
    /// Cache service
    Cache,
    /// Message queue service
    MessageQueue,
    /// File storage service
    FileStorage,
    /// Authentication service
    AuthService,
    /// Monitoring service
    MonitoringService,
    /// Load balancer service
    LoadBalancer,
    /// API gateway service
    ApiGateway,
    /// Orchestrator service
    Orchestrator,
    /// Custom service type
    Custom(String),
}

/// Service metadata structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceMetadata {
    /// Service description
    pub description: Option<String>,

    /// Service tags for categorization
    pub tags: Vec<String>,

    /// Service owner/team
    pub owner: Option<String>,

    /// Service documentation URL
    pub documentation_url: Option<String>,

    /// Service source code repository
    pub repository_url: Option<String>,

    /// Additional custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

/// Human notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanNotificationPreferences {
    /// Email notifications enabled
    pub email_enabled: bool,

    /// Slack notifications enabled
    pub slack_enabled: bool,

    /// SMS notifications enabled
    pub sms_enabled: bool,

    /// Notification frequency (minutes)
    pub notification_frequency_minutes: u32,

    /// Escalation timeout (minutes)
    pub escalation_timeout_minutes: u32,
}

impl Default for HumanNotificationPreferences {
    fn default() -> Self {
        Self {
            email_enabled: true,
            slack_enabled: false,
            sms_enabled: false,
            notification_frequency_minutes: 15,
            escalation_timeout_minutes: 60,
        }
    }
}

/// Service registration request structure (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationRequest {
    /// Service name
    pub service_name: String,

    /// Service version
    pub service_version: String,

    /// Service type
    pub service_type: ServiceType,

    /// Service endpoints
    pub endpoints: ServiceEndpoints,

    /// Service metadata
    pub metadata: ServiceMetadata,
}

/// Flexible service endpoints structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Primary service endpoint
    pub primary: String,

    /// Health check endpoint
    pub health: Option<String>,

    /// Metrics endpoint
    pub metrics: Option<String>,

    /// Admin/management endpoint
    pub admin: Option<String>,

    /// WebSocket endpoint (if applicable)
    pub websocket: Option<String>,

    /// Custom endpoints (extensible)
    pub custom: HashMap<String, String>,
}

/// Resource requirements for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores (can be fractional)
    pub cpu_cores: Option<f64>,

    /// Memory in MB
    pub memory_mb: Option<u64>,

    /// Storage in MB
    pub storage_mb: Option<u64>,

    /// Network bandwidth in Mbps
    pub network_mbps: Option<u64>,

    /// GPU count
    pub gpu_count: Option<u32>,

    /// Custom resource requirements
    pub custom_resources: HashMap<String, serde_json::Value>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfiguration {
    /// Health check interval in seconds
    pub interval_seconds: u64,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Number of consecutive successes to mark healthy again
    pub success_threshold: u32,

    /// Custom health check parameters
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

/// Human service interaction preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanServiceInteractionPreferences {
    /// Whether human approval is required for service registration
    pub registration_approval_required: bool,

    /// Whether humans should be notified of service health changes
    pub health_change_notifications: bool,

    /// Confidence threshold for automatic service operations
    pub auto_operation_confidence_threshold: f64,

    /// Human escalation triggers
    pub escalation_triggers: Vec<String>,

    /// Human notification preferences
    pub notification_preferences: HumanNotificationPreferences,
}

/// Universal service registration request - supports any primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistrationRequest {
    /// Service identifier - can be any format
    pub service_id: String,

    /// Service name for human reference
    pub service_name: String,

    /// Service version
    pub version: String,

    /// Primal type (extensible for community primals)
    pub primal_type: String,

    /// Service capabilities (flexible list)
    pub capabilities: Vec<String>,

    /// Service endpoints
    pub endpoints: ServiceEndpoints,

    /// Resource requirements
    pub resource_requirements: Option<ResourceRequirements>,

    /// Health check configuration
    pub health_check: Option<HealthCheckConfiguration>,

    /// Service metadata (completely extensible)
    pub metadata: HashMap<String, serde_json::Value>,

    /// Human interaction preferences
    pub human_interaction_preferences: Option<HumanServiceInteractionPreferences>,
}

/// Universal service registration response with AI-first format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationData {
    /// Registered service ID
    pub service_id: String,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,

    /// Service mesh routing information
    pub routing_info: ServiceMeshRoutingInfo,

    /// Assigned load balancing pool
    pub load_balancing_pool: Option<String>,

    /// Health monitoring configuration
    pub monitoring_config: MonitoringConfiguration,

    /// Predicted service performance
    pub performance_predictions: PerformancePredictions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshRoutingInfo {
    /// Assigned routing rules
    pub routing_rules: Vec<RoutingRule>,

    /// Traffic distribution percentage
    pub traffic_percentage: f64,

    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,

    /// Service priority in mesh
    pub priority: ServicePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule identifier
    pub rule_id: String,

    /// Rule type
    pub rule_type: String,

    /// Rule conditions
    pub conditions: HashMap<String, serde_json::Value>,

    /// Rule actions
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold percentage
    pub failure_threshold_percentage: f64,

    /// Minimum request threshold
    pub minimum_request_threshold: u32,

    /// Sleep window in seconds
    pub sleep_window_seconds: u64,

    /// Request volume threshold
    pub request_volume_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServicePriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfiguration {
    /// Metrics collection interval
    pub metrics_interval_seconds: u64,

    /// Health check frequency
    pub health_check_frequency_seconds: u64,

    /// Performance baseline
    pub performance_baseline: PerformanceBaseline,

    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Expected response time in milliseconds
    pub expected_response_time_ms: f64,

    /// Expected throughput (requests per second)
    pub expected_throughput_rps: f64,

    /// Expected error rate percentage
    pub expected_error_rate_percentage: f64,

    /// Expected resource utilization
    pub expected_cpu_utilization_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Response time threshold for alerts
    pub response_time_threshold_ms: f64,

    /// Error rate threshold for alerts
    pub error_rate_threshold_percentage: f64,

    /// CPU utilization threshold for alerts
    pub cpu_utilization_threshold_percentage: f64,

    /// Memory utilization threshold for alerts
    pub memory_utilization_threshold_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePredictions {
    /// Predicted latency under normal load
    pub predicted_latency_ms: f64,

    /// Predicted throughput capacity
    pub predicted_max_throughput_rps: f64,

    /// Predicted scaling behavior
    pub scaling_predictions: ScalingPredictions,

    /// Resource efficiency score
    pub resource_efficiency_score: f64,

    /// Reliability prediction
    pub predicted_reliability_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPredictions {
    /// Predicted horizontal scaling efficiency
    pub horizontal_scaling_efficiency: f64,

    /// Recommended minimum instances
    pub recommended_min_instances: u32,

    /// Recommended maximum instances
    pub recommended_max_instances: u32,

    /// Auto-scaling triggers
    pub auto_scaling_triggers: Vec<AutoScalingTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingTrigger {
    /// Metric to monitor for scaling
    pub metric: String,

    /// Threshold value
    pub threshold: f64,

    /// Scale up or down
    pub scale_direction: ScaleDirection,

    /// Scale by how many instances
    pub scale_by: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub struct RegisteredServiceInfo {
    pub registration_request: UniversalServiceRegistrationRequest,
    pub registration_data: ServiceRegistrationData,
    pub registered_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub current_status: ServiceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
