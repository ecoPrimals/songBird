// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Service Registration Types Types
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
    /// Default Health Check Interval field

    pub default_health_check_interval: u64,

    /// Enable automatic service discovery
    /// Enable Auto Discovery field

    pub enable_auto_discovery: bool,

    /// Enable performance monitoring
    /// Enable Performance Monitoring field

    pub enable_performance_monitoring: bool,

    /// Registry storage backend configuration
    pub storage_config: HashMap<String, serde_json::Value> );
 )
}

impl Default for UniversalServiceRegistrationConfig  {fn default() -> Self  {Self { max_services: 1000,
            default_health_check_interval: 30,
            enable_auto_discovery: true,
            enable_performance_monitoring: true,
            storage_config: HashMap::new();}}}

/// Service type enumeration for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    /// Web service (HTTP/REST API,
    /// `WebService`, WebService)
    /// Database service
    /// Database capability, Database,
    /// Cache service
    /// Cache, Cache,
    /// Message queue service
    /// MessageQueue, MessageQueue,
    /// File storage service
    /// FileStorage, FileStorage,
    /// Authentication service
    /// `AuthService`, AuthService,
    /// Monitoring service
    /// `MonitoringService`, MonitoringService,
    /// Load balancer service
    /// LoadBalancer, LoadBalancer,
    /// API gateway service
    /// ApiGateway, ApiGateway,
    /// Orchestrator service
    /// Orchestrator, Orchestrator,
    /// Custom service type
        Custom(String)
/// Service metadata structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceMetadata {
    /// Service description
    /// Human-readable description

    pub description: Option<String>,

    /// Service tags for categorization
    /// Additional metadata tags

    pub tags: Vec<String>,

    /// Service owner/team
    /// Owner field

    pub owner: Option<String>,

    /// Service documentation /// URL
// URL
    /// Documentation Url field

    pub documentation_url: Option<String>,

    /// Service source code repository
        pub repository_url: Option<String>,

    /// Additional custom metadata
    pub custom: HashMap<String, serde_json::Value> );
 )
}

/// Human notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanNotificationPreferences {
    /// Email notifications enabled
    /// Email Enabled field

    pub email_enabled: bool,

    /// Slack notifications enabled
    /// Slack Enabled field

    pub slack_enabled: bool,

    /// SMS notifications enabled
    /// Sms Enabled field

    pub sms_enabled: bool,

    /// Notification frequency (minutes)
    /// Notification Frequency Minutes field

    pub notification_frequency_minutes: u32,

    /// Escalation timeout (minutes)
    /// Escalation Timeout Minutes field

    pub escalation_timeout_minutes: u32 ,
 )
}

impl Default for HumanNotificationPreferences  {fn default() -> Self  {Self { email_enabled: true,
            slack_enabled: false,
            sms_enabled: false,
            notification_frequency_minutes: 15,
            escalation_timeout_minutes: 60;}}}

/// Service registration request structure (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationRequest {
    /// Service name
        pub service_name: String,
    /// Service version
    /// Service Version field

    pub service_version: String,
    /// Service type
        pub service_type: ServiceType,
    /// Service endpoints
    /// Available service endpoints

    /// Service endpoints - `ServiceEndpoint`s, ServiceEndpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Service metadata
    pub metadata: ServiceMetadata ,
 )
}

/// Flexible service endpoints structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Primary service endpoint
        pub primary: String,
    /// Health check endpoint
        pub health: Option<String>,

    /// Metrics endpoint
    /// Available metrics or measurements

    pub metrics: Option<String>,

    /// Admin/management endpoint
    /// Admin field

    pub admin: Option<String>,

    /// WebSocket endpoint (if applicable)
    /// Websocket field

    pub websocket: Option<String>,

    /// Custom endpoints (extensible)
    pub custom: HashMap<String, String> )
 )
}

/// Resource requirements for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores (can be fractional)
    /// Cpu Cores field

    pub cpu_cores: Option<f64>,

    /// Memory in MB
    pub memory_mb: Option<u64>,

    /// Storage in MB
    pub storage_mb: Option<u64>,
        pub storage_mb: Option<u64>,

    /// Network bandwidth in Mbps
    pub network_mbps: Option<u64>,

    /// GPU count
    /// Gpu Count field

    pub gpu_count: Option<u32>,

    /// Custom resource requirements
    pub custom_resources: HashMap<String, serde_json::Value> );
 )
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfiguration {
    /// Health check interval in seconds
    /// Interval Seconds field

    pub interval_seconds: u64,

    /// Request timeout in seconds
    /// Timeout Seconds field

    pub timeout_seconds: u64,

    /// Number of consecutive failures before marking unhealthy
        pub failure_threshold: u32,

    /// Number of consecutive successes to mark healthy again
        pub custom_parameters: HashMap<String, serde_json::Value> );
 )
}

/// Human service interaction preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanServiceInteractionPreferences {
    /// Whether human approval is required for service registration
    /// Registration Approval Required field

    pub registration_approval_required: bool,

    /// Whether humans should be notified of service health changes
    /// Health Change Notifications field

    pub health_change_notifications: bool,

    /// Confidence threshold for automatic service operations
    /// Auto Operation Confidence Threshold field

    pub auto_operation_confidence_threshold: f64,

    /// Human escalation triggers
    /// Escalation Triggers field

    pub escalation_triggers: Vec<String>,

    /// Human notification preferences
    /// Notification Preferences field

    pub notification_preferences: HumanNotificationPreferences ,
 )
}

/// Universal service registration request - supports any primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistrationRequest {
    /// Service identifier - can be any format
        pub service_id: String,
    /// Service name for human reference
        pub service_name: String,
    /// Service version
    /// Version string

    pub version: String,
    /// Primal type (extensible for community primals)
    /// Primal Type field

    pub primal_type: String,
    /// Service capabilities (flexible list)
    /// List of supported capabilities

    pub capabilities: Vec<String>,

    /// Service endpoints
    /// Available service endpoints

    /// Service endpoints - `ServiceEndpoint`s, ServiceEndpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Resource requirements
    pub resource_requirements: Option<ResourceRequirements>,

    /// Health check configuration
        pub health_check: Option<HealthCheckConfiguration>,

    /// Service metadata (completely extensible)
    pub metadata: HashMap<String, serde_json: :Value>,
    /// Human interaction preferences
    /// Human Interaction Preferences field

    pub human_interaction_preferences: Option<HumanServiceInteractionPreferences> ,
 )
}

/// Universal service registration response with AI-first format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationData {
    /// Registered service ID
    pub service_id: String,
    /// Registration timestamp
        pub registered_at: DateTime<Utc>,

    /// Service mesh routing information
    /// Routing Info field

    pub routing_info: ServiceMeshRoutingInfo,
    /// Assigned load balancing pool
    /// Load Balancing Pool field

    pub load_balancing_pool: Option<String>,

    /// Health monitoring configuration
    /// Monitoring Config field

    pub monitoring_config: MonitoringConfiguration,
    /// Predicted service performance
    /// Performance Predictions field

    pub performance_predictions: PerformancePredictions ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshRoutingInfo {
    /// Assigned routing rules
    /// Routing Rules field

    pub routing_rules: Vec<RoutingRule>,

    /// Traffic distribution percentage
    /// Traffic Percentage field

    pub traffic_percentage: f64,

    /// Circuit breaker configuration
    /// Circuit Breaker Config field

    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Service priority in mesh
        pub priority: ServicePriority ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule identifier
        pub rule_id: String,
    /// Rule type
        pub rule_type: String,
    /// Rule conditions
    pub conditions: HashMap<String, serde_json: :Value>,
    /// Rule actions
    /// Actions field

    pub actions: Vec<String> ,
 )
}
/// Circuit breaker configuration
///
/// **CONSOLIDATED**: Re-export of canonical version (Week 2, Nov 10 2025).
/// Note: Original was specialized for service registration (percentage-based, request volumes)
pub use songbird_config::canonical::resilience::CircuitBreakerConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServicePriority {
    /// Critical, Critical,
    /// High, High)
    /// Normal, Normal,
    Low  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfiguration {
    /// Metrics collection interval
    /// Metrics Interval Seconds field

    pub metrics_interval_seconds: u64,

    /// Health check frequency
    /// Health Check Frequency Seconds field

    pub health_check_frequency_seconds: u64,

    /// Performance baseline
    /// Performance Baseline field

    pub performance_baseline: PerformanceBaseline,
    /// Alert thresholds
        pub alert_thresholds: AlertThresholds ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Expected response time in milliseconds
    /// Expected Response Time Ms field

    pub expected_response_time_ms: f64,

    /// Expected throughput (requests per second)
    /// Expected Throughput Rps field

    pub expected_throughput_rps: f64,

    /// Expected error rate percentage
    /// Expected Error Rate Percentage field

    pub expected_error_rate_percentage: f64,

    /// Expected resource utilization
    /// Expected Cpu Utilization Percentage field

    pub expected_cpu_utilization_percentage: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Response time threshold for alerts
    /// Response Time Threshold Ms field

    pub response_time_threshold_ms: f64,

    /// Error rate threshold for alerts
    /// Error Rate Threshold Percentage field

    pub error_rate_threshold_percentage: f64,

    /// CPU utilization threshold for alerts
    /// Cpu Utilization Threshold Percentage field

    pub cpu_utilization_threshold_percentage: f64,

    /// Memory utilization threshold for alerts
    /// Memory Utilization Threshold Percentage field

    pub memory_utilization_threshold_percentage: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePredictions {
    /// Predicted latency under normal load
    /// Predicted Latency Ms field

    pub predicted_latency_ms: f64,

    /// Predicted throughput capacity
        pub predicted_max_throughput_rps: f64,

    /// Predicted scaling behavior
    /// Scaling Predictions field

    pub scaling_predictions: ScalingPredictions,
    /// Resource efficiency score
    /// Resource Efficiency Score field

    pub resource_efficiency_score: f64,

    /// Reliability prediction
    /// Predicted Reliability Percentage field

    pub predicted_reliability_percentage: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPredictions {
    /// Predicted horizontal scaling efficiency
    /// Horizontal Scaling Efficiency field

    pub horizontal_scaling_efficiency: f64,

    /// Recommended minimum instances
    /// Recommended Min Instances field

    pub recommended_min_instances: u32,

    /// Recommended maximum instances
    /// Recommended Max Instances field

    pub recommended_max_instances: u32,

    /// Auto-scaling triggers
    /// Auto Scaling Triggers field

    pub auto_scaling_triggers: Vec<AutoScalingTrigger> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingTrigger {
    /// Metric to monitor for scaling
        pub metric: String,
    /// Threshold value
        pub threshold: f64,

    /// Scale up or down
    /// Scale Direction field

    pub scale_direction: ScaleDirection,
    /// Scale by how many instances
        pub scale_by: u32 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleDirection {
    /// Up, Up,
    Down  }
#[derive(Debug, Clone)]
pub struct RegisteredServiceInfo {
    /// Registration Request field

    pub registration_request: UniversalServiceRegistrationRequest,
    /// Registration Data field
    pub registration_data: ServiceRegistrationData,
    /// Registered At field
    pub registered_at: DateTime<Utc>,
    /// Last Health Check field
    pub last_health_check: Option<DateTime<Utc>>,
    /// Current Status field
    pub current_status: ServiceStatus ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ServiceStatus {
    /// Healthy, Healthy,
    /// Degraded, Degraded)
    /// Unhealthy, Unhealthy,
    Unknown,;};
