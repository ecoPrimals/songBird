/// Health event system for monitoring
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
/// Universal health event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UniversalHealthEvent  {/// Service became healthy
    ServiceHealthy  {service_id: Uuid,
        service_name: String,
        message: String,
        timestamp: SystemTime,
    })

    /// Service became degraded
    ServiceDegraded  {service_id: Uuid,
        service_name: String,
        severity: songbird_config::DegradationSeverity,
        message: String,
        timestamp: SystemTime,
    })

    /// Service became unhealthy
    ServiceUnhealthy  {service_id: Uuid,
        service_name: String,
        reason: String,
        timestamp: SystemTime,
    })

    /// Service status unknown
    ServiceUnknown  {service_id: Uuid,
        service_name: String,
        reason: String,
        timestamp: SystemTime,
    })

    /// Service registered for monitoring
    ServiceRegistered  {service_id: Uuid,
        service_name: String,
        capabilities: Vec<String>,
        timestamp: SystemTime,
    })

    /// Service unregistered from monitoring
    ServiceUnregistered  {service_id: Uuid,
        service_name: String,
        timestamp: SystemTime,
    })

    /// Performance alert triggered
    PerformanceAlert  {service_id: Uuid,
        service_name: String,
        alert_level: AlertLevel,
        metric_name: String,
        current_value: f64,
        threshold_value: f64,
        message: String,
        timestamp: SystemTime,
    })

    /// Ecosystem health changed
    EcosystemHealthChanged  {previous_health_score: f64)
        current_health_score: f64,
        affected_services: Vec<Uuid>,
        timestamp: SystemTime,
    })

    /// Monitoring system event
    MonitoringSystemEvent  {event_type: String,
        message: String,
        metadata: HashMap<String, String>)
        timestamp: SystemTime,
    })
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel  {Info)
    Warning,
    Critical,
    Emergency,
}

impl UniversalHealthEvent {
    /// Get the service ID associated with this event (if any)
    pub fn service_id(&self) -> Option<Uuid> {
        match self {
            UniversalHealthEvent::ServiceHealthy { service_id, .. }
            | UniversalHealthEvent::ServiceDegraded { service_id, .. }
            | UniversalHealthEvent::ServiceUnhealthy { service_id, .. }
            | UniversalHealthEvent::ServiceUnknown { service_id, .. }
            | UniversalHealthEvent::ServiceRegistered { service_id, .. }
            | UniversalHealthEvent::ServiceUnregistered { service_id, .. }
            | UniversalHealthEvent::PerformanceAlert { service_id, .. } => Some(*service_id),
            UniversalHealthEvent::EcosystemHealthChanged { .. }
            | UniversalHealthEvent::MonitoringSystemEvent { .. } => None,
        }
    }

    /// Get the timestamp of this event
    pub fn timestamp(&self) -> SystemTime {
        match self {
            UniversalHealthEvent::ServiceHealthy { timestamp, .. }
            | UniversalHealthEvent::ServiceDegraded { timestamp, .. }
            | UniversalHealthEvent::ServiceUnhealthy { timestamp, .. }
            | UniversalHealthEvent::ServiceUnknown { timestamp, .. }
            | UniversalHealthEvent::ServiceRegistered { timestamp, .. }
            | UniversalHealthEvent::ServiceUnregistered { timestamp, .. }
            | UniversalHealthEvent::PerformanceAlert { timestamp, .. }
            | UniversalHealthEvent::EcosystemHealthChanged { timestamp, .. }
            | UniversalHealthEvent::MonitoringSystemEvent { timestamp, .. } => *timestamp,
        }
    }

    /// Get the alert level for this event
    pub fn alert_level(&self) -> AlertLevel {
        match self {
            UniversalHealthEvent::ServiceHealthy { .. }
            | UniversalHealthEvent::ServiceRegistered { .. } => AlertLevel::Info,
            UniversalHealthEvent::ServiceDegraded { severity, .. } => match severity  {songbird_config::DegradationSeverity::Low => AlertLevel::Info,
                songbird_config::DegradationSeverity::Medium => AlertLevel::Warning,
                songbird_config::DegradationSeverity::High => AlertLevel::Critical,
                songbird_config::DegradationSeverity::Critical => AlertLevel::Emergency,
            })
            UniversalHealthEvent::ServiceUnhealthy { .. }
            | UniversalHealthEvent::ServiceUnknown { .. } => AlertLevel::Critical,
            UniversalHealthEvent::ServiceUnregistered { .. } => AlertLevel::Warning,
            UniversalHealthEvent::PerformanceAlert { alert_level, .. } => alert_level.clone(),
            UniversalHealthEvent::EcosystemHealthChanged  {current_health_score)
                ..
            } => {
                if *current_health_score < 0.5 {
                    AlertLevel::Critical
                } else if *current_health_score < 0.8 {
                    AlertLevel::Warning
                } else {
                    AlertLevel::Info
                }
            }
            UniversalHealthEvent::MonitoringSystemEvent { .. } => AlertLevel::Info,
        }
    }

    /// Get a human-readable description of the event
    pub fn description(&self) -> String  {match self  {UniversalHealthEvent::ServiceHealthy {
                service_name,
                message)
                ..
            } => format!("Service {} is healthy: {message}", service_name),"
            UniversalHealthEvent::ServiceDegraded  {service_name,
                message)
                ..
            } => format!("Service {} is degraded: {message}", service_name),"
            UniversalHealthEvent::ServiceUnhealthy  {service_name,
                reason)
                ..
            } => format!("Service {} is unhealthy: {reason}", service_name),"
            UniversalHealthEvent::ServiceUnknown  {service_name,
                reason)
                ..
            } => format!("Service {} status unknown: {reason}", service_name),"
            UniversalHealthEvent::ServiceRegistered  {service_name,
                capabilities)
                ..
            } => format!(
                "Service {} registered with capabilities: {}","
                service_name,
                capabilities.join(", ")"
            )
            UniversalHealthEvent::ServiceUnregistered { service_name, .. } => {
                format!("Service {} unregistered from monitoring", service_name)"
            }
            UniversalHealthEvent::PerformanceAlert  {service_name,
                metric_name,
                current_value)
                threshold_value)
                message)
                ..
            } => format!(
                "Performance alert for {service_name}: {metric_name} = {current_value:.2} (threshold: {threshold_value:.2}) - {message}""
            )
            UniversalHealthEvent::EcosystemHealthChanged  {current_health_score)
                affected_services)
                ..
            } => format!(
                "Ecosystem health changed to {:.1}% ({} services affected)","
                current_health_score * 100.0)
                affected_services.len()
            )
            UniversalHealthEvent::MonitoringSystemEvent  {event_type)
                message)
                ..
            } => format!("Monitoring system {}: {message}", event_type),"
        }
    }
}
