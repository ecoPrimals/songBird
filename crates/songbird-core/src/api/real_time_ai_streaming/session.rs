//! Collaboration session and workspace management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of collaboration sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationSessionType {
    /// Troubleshooting session
    Troubleshooting,
    /// Performance tuning session  
    PerformanceTuning,
    /// Configuration review session
    ConfigurationReview,
    /// Incident response session
    IncidentResponse,
    /// Strategic planning session
    StrategicPlanning,
    /// Code review session
    CodeReview,
    /// Architecture design session
    ArchitectureDesign,
    /// Training session
    Training,
    /// Research and analysis session
    Research,
    /// General collaboration session
    General,
}

/// Session participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParticipant {
    /// Participant identifier
    pub user_id: String,
    /// Participant name
    pub name: String,
    /// Participant type
    pub participant_type: ParticipantType,
    /// Participant role in session
    pub role: String,
    /// Participant status
    pub status: ParticipantStatus,
    /// Permissions in the session
    pub permissions: Vec<String>,
    /// Join timestamp
    pub joined_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Participation metrics
    pub metrics: ParticipationMetrics,
}

/// Types of session participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantType {
    Human,
    AIAgent,
    Service,
}

/// Participant status in session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Active,
    Idle,
    Away,
    Disconnected,
}

/// Metrics for participant engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationMetrics {
    /// Total active time in session (seconds)
    pub active_time_seconds: u64,
    /// Number of messages sent
    pub messages_sent: u32,
    /// Number of actions performed
    pub actions_performed: u32,
    /// Collaboration score (0.0 - 1.0)
    pub collaboration_score: f64,
}

/// Session states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is being created
    Initializing,
    /// Session is active and running
    Active,
    /// Session is paused
    Paused,
    /// Session has ended
    Ended,
    /// Session was terminated due to error
    Terminated,
}

/// Collaboration workspace structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationWorkspace {
    /// Workspace identifier
    pub workspace_id: String,
    /// Workspace name
    pub name: String,
    /// Associated session ID
    pub session_id: String,
    /// Workspace documents
    pub documents: Vec<WorkspaceDocument>,
    /// Shared visualizations
    pub visualizations: Vec<Visualization>,
    /// Action items and tasks
    pub action_items: Vec<ActionItem>,
    /// Decisions made in workspace
    pub decisions: Vec<Decision>,
    /// Workspace metrics
    pub metrics: WorkspaceMetrics,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
}

/// Document in collaboration workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDocument {
    /// Document identifier
    pub document_id: String,
    /// Document title
    pub title: String,
    /// Document content
    pub content: String,
    /// Document type/format
    pub document_type: String,
    /// Document version
    pub version: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub last_modified: DateTime<Utc>,
    /// Author user ID
    pub author_id: String,
    /// Document permissions
    pub permissions: DocumentPermissions,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Document access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPermissions {
    /// Can read document
    pub can_read: Vec<String>,
    /// Can edit document
    pub can_edit: Vec<String>,
    /// Can delete document
    pub can_delete: Vec<String>,
}

/// Visualization in workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    /// Visualization identifier
    pub visualization_id: String,
    /// Visualization title
    pub title: String,
    /// Visualization type
    pub visualization_type: VisualizationType,
    /// Visualization data
    pub data: serde_json::Value,
    /// Visualization configuration
    pub config: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Created by user ID
    pub created_by: String,
}

/// Types of visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    Chart,
    Graph,
    Diagram,
    Map,
    Timeline,
    Dashboard,
    Network,
}

/// Action item in collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Action item identifier
    pub item_id: String,
    /// Action description
    pub description: String,
    /// Assigned to user ID
    pub assigned_to: String,
    /// Due date
    pub due_date: Option<DateTime<Utc>>,
    /// Action item status
    pub status: ActionItemStatus,
    /// Priority level (1-5)
    pub priority: u32,
    /// Related documents
    pub related_documents: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Progress notes
    pub progress_notes: Vec<String>,
}

/// Status of action items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionItemStatus {
    Open,
    InProgress,
    Completed,
    Cancelled,
}

/// Decision made in collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Decision identifier
    pub decision_id: String,
    /// Decision description
    pub description: String,
    /// Decision maker user ID
    pub decision_maker: String,
    /// Decision rationale
    pub rationale: String,
    /// Decision options considered
    pub options_considered: Vec<String>,
    /// Selected option
    pub selected_option: String,
    /// Decision timestamp
    pub decided_at: DateTime<Utc>,
    /// Decision impact assessment
    pub impact_assessment: String,
    /// Decision approval status
    pub approved: bool,
    /// Approval by user ID
    pub approved_by: Option<String>,
}

/// Workspace collaboration metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetrics {
    /// Total active time (seconds)
    pub total_active_time_seconds: u64,
    /// Number of participants
    pub participant_count: u32,
    /// Number of documents created
    pub documents_created: u32,
    /// Number of decisions made
    pub decisions_made: u32,
    /// Number of action items
    pub action_items_count: u32,
    /// Collaboration efficiency score (0.0 - 1.0)
    pub collaboration_efficiency: f64,
}

/// AI assistance levels for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIAssistanceLevel {
    None,
    Minimal,
    Moderate,
    Full,
    Expert,
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfiguration {
    /// Enable real-time collaboration
    pub real_time_collaboration: bool,
    /// Enable AI suggestions
    pub ai_suggestions: bool,
    /// Enable performance monitoring
    pub performance_monitoring: bool,
    /// Template to use
    pub template: Option<String>,
}

/// Session configuration details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfiguration {
    /// Maximum participants allowed
    pub max_participants: u32,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// Auto-save enabled
    pub auto_save_enabled: bool,
    /// Recording enabled
    pub recording_enabled: bool,
    /// AI assistance level
    pub ai_assistance_level: AIAssistanceLevel,
}

/// AI facilitator for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFacilitator {
    /// Facilitator identifier
    pub facilitator_id: String,
    /// AI capabilities
    pub capabilities: Vec<String>,
    /// AI personality type
    pub personality: String,
    /// Expertise domains
    pub expertise_domains: Vec<String>,
}

impl Default for ParticipationMetrics {
    fn default() -> Self {
        Self {
            active_time_seconds: 0,
            messages_sent: 0,
            actions_performed: 0,
            collaboration_score: 0.0,
        }
    }
}

impl Default for WorkspaceMetrics {
    fn default() -> Self {
        Self {
            total_active_time_seconds: 0,
            participant_count: 0,
            documents_created: 0,
            decisions_made: 0,
            action_items_count: 0,
            collaboration_efficiency: 0.0,
        }
    }
}

impl Default for WorkspaceConfiguration {
    fn default() -> Self {
        Self {
            real_time_collaboration: true,
            ai_suggestions: true,
            performance_monitoring: true,
            template: None,
        }
    }
}
