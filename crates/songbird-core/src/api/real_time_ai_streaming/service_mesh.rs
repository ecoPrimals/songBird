//! Service mesh event handling and monitoring structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of service mesh events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshEventType {
    /// Service deployment event
    ServiceDeployed,
    /// Service termination event
    ServiceTerminated,
    /// Service scaling event
    ServiceScaled,
    /// Service health status change
    ServiceHealthChanged,
    /// Load balancer configuration change
    LoadBalancerChanged,
    /// Circuit breaker state change
    CircuitBreakerTripped,
    /// Rate limiting threshold reached
    RateLimitExceeded,
    /// Service discovery update
    ServiceDiscoveryUpdate,
    /// Configuration change
    ConfigurationChanged,
    /// Security policy update
    SecurityPolicyUpdated,
    /// Performance threshold exceeded
    PerformanceThresholdExceeded,
    /// Resource constraint violation
    ResourceConstraintViolated,
    /// Network connectivity issue
    NetworkConnectivityIssue,
    /// Dependency failure
    DependencyFailure,
    /// Recovery action completed
    RecoveryCompleted,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Trend direction for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
    Volatile,
}

/// Impact level assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Minimal,
    Low,
    Medium,
    High,
    Critical,
}

/// Component status in the service mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
    Maintenance,
}

/// Status change event details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChange {
    /// Component identifier
    pub component_id: String,
    /// Component name
    pub component_name: String,
    /// Previous status
    pub previous_status: ComponentStatus,
    /// New status
    pub new_status: ComponentStatus,
    /// Change reason
    pub reason: String,
    /// Change timestamp
    pub timestamp: DateTime<Utc>,
    /// User or system that triggered the change
    pub triggered_by: String,
    /// Additional context
    pub context: Option<serde_json::Value>,
}

/// Alert levels for system notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl From<EventSeverity> for AlertLevel {
    fn from(severity: EventSeverity) -> Self {
        match severity {
            EventSeverity::Info => AlertLevel::Info,
            EventSeverity::Warning => AlertLevel::Warning,
            EventSeverity::Error => AlertLevel::Error,
            EventSeverity::Critical => AlertLevel::Critical,
        }
    }
}

impl std::fmt::Display for ServiceMeshEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceMeshEventType::ServiceDeployed => write!(f, "service_deployed"),
            ServiceMeshEventType::ServiceTerminated => write!(f, "service_terminated"),
            ServiceMeshEventType::ServiceScaled => write!(f, "service_scaled"),
            ServiceMeshEventType::ServiceHealthChanged => write!(f, "service_health_changed"),
            ServiceMeshEventType::LoadBalancerChanged => write!(f, "load_balancer_changed"),
            ServiceMeshEventType::CircuitBreakerTripped => write!(f, "circuit_breaker_tripped"),
            ServiceMeshEventType::RateLimitExceeded => write!(f, "rate_limit_exceeded"),
            ServiceMeshEventType::ServiceDiscoveryUpdate => write!(f, "service_discovery_update"),
            ServiceMeshEventType::ConfigurationChanged => write!(f, "configuration_changed"),
            ServiceMeshEventType::SecurityPolicyUpdated => write!(f, "security_policy_updated"),
            ServiceMeshEventType::PerformanceThresholdExceeded => {
                write!(f, "performance_threshold_exceeded")
            }
            ServiceMeshEventType::ResourceConstraintViolated => {
                write!(f, "resource_constraint_violated")
            }
            ServiceMeshEventType::NetworkConnectivityIssue => {
                write!(f, "network_connectivity_issue")
            }
            ServiceMeshEventType::DependencyFailure => write!(f, "dependency_failure"),
            ServiceMeshEventType::RecoveryCompleted => write!(f, "recovery_completed"),
        }
    }
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventSeverity::Info => write!(f, "info"),
            EventSeverity::Warning => write!(f, "warning"),
            EventSeverity::Error => write!(f, "error"),
            EventSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentStatus::Healthy => write!(f, "healthy"),
            ComponentStatus::Degraded => write!(f, "degraded"),
            ComponentStatus::Unhealthy => write!(f, "unhealthy"),
            ComponentStatus::Unknown => write!(f, "unknown"),
            ComponentStatus::Maintenance => write!(f, "maintenance"),
        }
    }
}
