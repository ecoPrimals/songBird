//! Common types, enums, and utility structures

use serde::{Deserialize, Serialize};

/// Risk level assessment for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Supporting evidence for AI recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Evidence type
    pub evidence_type: String,
    /// Evidence description
    pub description: String,
    /// Supporting data
    pub data: serde_json::Value,
    /// Evidence source
    pub source: String,
    /// Evidence reliability score (0.0 - 1.0)
    pub reliability_score: f64,
    /// Evidence timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Analysis finding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisFinding {
    /// Finding identifier
    pub finding_id: String,
    /// Finding type
    pub finding_type: String,
    /// Finding description
    pub description: String,
    /// Finding severity
    pub severity: super::service_mesh::EventSeverity,
    /// Confidence in finding
    pub confidence: f64,
    /// Supporting data
    pub supporting_data: serde_json::Value,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Finding timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User or system that triggered the change
    pub triggered_by: String,
    /// Additional context
    pub context: Option<serde_json::Value>,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Critical => write!(f, "critical"),
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

impl Default for Evidence {
    fn default() -> Self {
        Self {
            evidence_type: "observation".to_string(),
            description: String::new(),
            data: serde_json::Value::Null,
            source: "ai_analysis".to_string(),
            reliability_score: 0.8,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Default for AnalysisFinding {
    fn default() -> Self {
        Self {
            finding_id: uuid::Uuid::new_v4().to_string(),
            finding_type: "general".to_string(),
            description: String::new(),
            severity: super::service_mesh::EventSeverity::Info,
            confidence: 0.7,
            supporting_data: serde_json::Value::Null,
            recommended_actions: vec![],
            timestamp: chrono::Utc::now(),
        }
    }
}
