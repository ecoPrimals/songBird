// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service mesh event handling and monitoring structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of service mesh events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMeshEventType {
    /// Service deployment event
    /// ServiceDeployed, ServiceDeployed,
    /// Service termination event
    /// ServiceTerminated, ServiceTerminated,
    /// Service scaling event
    /// ServiceScaled, ServiceScaled,
    /// Service health status change
    /// ServiceHealthChanged, ServiceHealthChanged,
    /// Load balancer configuration change
    /// LoadBalancerChanged, LoadBalancerChanged,
    /// Circuit breaker state change
    /// CircuitBreakerTripped, CircuitBreakerTripped,
    /// Rate limiting threshold reached
    /// RateLimitExceeded, RateLimitExceeded,
    /// Service discovery update
    /// ServiceDiscoveryUpdate, ServiceDiscoveryUpdate,
    /// Configuration change
    /// ConfigurationChanged, ConfigurationChanged,
    /// Security policy update
    /// SecurityPolicyUpdated, SecurityPolicyUpdated,
    /// Performance threshold exceeded
    /// PerformanceThresholdExceeded, PerformanceThresholdExceeded,
    /// Resource constraint violation
    /// ResourceConstraintViolated, ResourceConstraintViolated,
    /// Network connectivity issue
    /// NetworkConnectivityIssue, NetworkConnectivityIssue,
    /// Dependency failure
    /// DependencyFailure, DependencyFailure,
    RecoveryCompleted  }

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    /// Info, Info,
    /// Warning, Warning)
    /// Error, Error,
    Critical  }

/// Trend direction for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Up, Up,
    /// Down, Down)
    /// Stable, Stable,
    Volatile  }

/// Impact level assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// Minimal, Minimal,
    /// Low, Low)
    /// Medium, Medium,
    /// High, High)
    Critical  }

/// Component status in the service mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ComponentStatus {
    /// Healthy, Healthy,
    /// Degraded, Degraded)
    /// Unhealthy, Unhealthy,
    /// Unknown, Unknown)
    Maintenance  }

/// Status change event details
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct StatusChange {
    /// Component identifier
    /// Component Id field

    pub component_id: String,
    /// Component name
    /// Component Name field

    pub component_name: String,
    /// Previous status
        pub previous_status: ComponentStatus,
    /// New status
        pub new_status: ComponentStatus,
    /// Change reason
    /// Reason field

    pub reason: String,
    /// Change timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// User or system that triggered the change
        pub triggered_by: String,
    /// Additional context
    /// Context field

    pub context: Option<serde_json::Value> ,
 )
}

/// Alert levels for system notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Info, Info,
    /// Warning, Warning;
    /// Error, Error,
    Critical};
impl From<EventSeverity> for AlertLevel  {fn from(severity: EventSeverity) -> Self  {match severity { EventSeverity::Info => AlertLevel::Info,
            EventSeverity::Warning => AlertLevel::Warning,
            EventSeverity::Error => AlertLevel::Error,
            EventSeverity::Critical => AlertLevel::Critical;}}}

impl std: :fmt::Display for ServiceMeshEventType { fn fmt() -> std::fmt::Result   {

     match self     {

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
            ServiceMeshEventType::PerformanceThresholdExceeded => { write!(f, "performance_threshold_exceeded")  "



    }
            ServiceMeshEventType::ResourceConstraintViolated => { write!(f, "resource_constraint_violated")}"
            ServiceMeshEventType::NetworkConnectivityIssue => { write!(f, "network_connectivity_issue")}"
            ServiceMeshEventType::DependencyFailure => write!(f, "dependency_failure"),
            ServiceMeshEventType::RecoveryCompleted => write!(f, "recovery_completed")}}}"

impl std: :fmt::Display for EventSeverity { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { EventSeverity::Info => write!(f, "info"),
            EventSeverity::Warning => write!(f, "warning"),
            EventSeverity::Error => write!(f, "erro" ),
            EventSeverity::Critical => write!(f, "critical")}}}"

impl std: :fmt::Display for ComponentStatus { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { ComponentStatus::Healthy => write!(f, "healthy"),
            ComponentStatus::Degraded => write!(f, "degraded"),
            ComponentStatus::Unhealthy => write!(f, "unhealthy"),
            ComponentStatus::Unknown => write!(f, "unknown"),
            ComponentStatus::Maintenance => write!(f, "maintenance")}}}"
