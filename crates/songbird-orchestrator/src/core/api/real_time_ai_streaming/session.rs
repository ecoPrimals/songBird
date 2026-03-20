// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Collaboration session and workspace management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of collaboration sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationSessionType {
    /// Troubleshooting session
    /// Troubleshooting, Troubleshooting,
    /// Performance tuning session
    /// PerformanceTuning, PerformanceTuning,
    /// Configuration review session
    /// ConfigurationReview, ConfigurationReview,
    /// Incident response session
    /// `IncidentResponse`, IncidentResponse,
    /// Strategic planning session
    /// StrategicPlanning, StrategicPlanning,
    /// Code review session
    /// CodeReview, CodeReview,
    /// Architecture design session
    /// ArchitectureDesign, ArchitectureDesign,
    /// Training session
    /// Training, Training,
    /// Research and analysis session
    /// Research, Research,
    General  }

/// Session participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionParticipant {
    /// Participant identifier
        pub user_id: String,
    /// Participant name
    /// Name identifier

    pub name: String,
    /// Participant type
    /// Participant Type field

    pub participant_type: ParticipantType,
    /// Participant role in session
        pub role: String,
    /// Participant status
    /// Current status of the operation or entity

    pub status: ParticipantStatus,
    /// Permissions in the session
    /// Permissions field

    pub permissions: Vec<String>,
    /// Join timestamp
    /// Joined At field

    pub joined_at: DateTime<Utc>,
    /// Last activity timestamp
        pub last_activity: DateTime<Utc>,
    /// Participation metrics
    /// Available metrics or measurements

    pub metrics: ParticipationMetrics ,
 )
}

/// Types of session participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantType {
    /// Human, Human,
    /// AIAgent, AIAgent)
    Service  }

/// Participant status in session
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ParticipantStatus {
    /// Active, Active,
    /// Idle, Idle)
    /// Away, Away,
    Disconnected  }

/// Metrics for participant engagement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationMetrics {
    /// Total active time in session (seconds)
    /// Active Time Seconds field

    pub active_time_seconds: u64,
    /// Number of messages sent
    /// Messages Sent field

    pub messages_sent: u32,
    /// Number of actions performed
    /// Actions Performed field

    pub actions_performed: u32,
    /// Collaboration score (0.0 - 1.0)
    /// Collaboration Score field

    pub collaboration_score: f64 ,
 )
}

/// Session states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is being created
    /// Initializing, Initializing,
    /// Session is active and running
    /// Active, Active,
    /// Session is paused
    /// Paused, Paused,
    /// Session has ended
    /// Ended, Ended,
    Terminated  }

/// Collaboration workspace structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationWorkspace {
    /// Workspace identifier
        pub workspace_id: String,
    /// Workspace name
    /// Name identifier

    pub name: String,
    /// Associated session /// ID
// ID
    /// Session Id field

    pub session_id: String,
    /// Workspace documents
    /// Documents field

    pub documents: Vec<WorkspaceDocument>,
    /// Shared visualizations
    /// Visualizations field

    pub visualizations: Vec<Visualization>,
    /// Action items and tasks
    /// Action Items field

    pub action_items: Vec<ActionItem>,
    /// Decisions made in workspace
    /// Decisions field

    pub decisions: Vec<Decision>,
    /// Workspace metrics
    /// Available metrics or measurements

    pub metrics: WorkspaceMetrics,
    /// Creation timestamp
        pub last_modified: DateTime<Utc> ,
 )
}

/// Document in collaboration workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDocument {
    /// Document identifier
    /// Document Id field

    pub document_id: String,
    /// Document title
        pub title: String,
    /// Document content
    /// Content field

    pub content: String,
    /// Document type/format
    /// Document Type field

    pub document_type: String,
    /// Document version
    /// Version string

    pub version: u32,
    /// Creation timestamp
        pub last_modified: DateTime<Utc>,
    /// Author user /// ID
 ID
        pub author_id: String,
    /// Document permissions
    /// Permissions field

    pub permissions: DocumentPermissions,
    /// Tags for organization
    /// Additional metadata tags

    pub tags: Vec<String> ,
 )
}

/// Document access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPermissions {
    /// Can read document
    /// Can Read field

    pub can_read: Vec<String>,
    /// Can edit document
    /// Can Edit field

    pub can_edit: Vec<String>,
    /// Can delete document
    /// Can Delete field

    pub can_delete: Vec<String> ,
 )
}

/// Visualization in workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    /// Visualization identifier
    /// Visualization Id field

    pub visualization_id: String,
    /// Visualization title
        pub title: String,
    /// Visualization type
    /// Visualization Type field

    pub visualization_type: VisualizationType,
    /// Visualization data
        pub data: serde_json::Value,
    /// Visualization configuration
    /// Config field

    pub config: serde_json::Value,
    /// Creation timestamp
        pub created_at: DateTime<Utc>,
    /// Created by user /// ID
 ID
        pub created_by: String ,
 )
}

/// Types of visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationType {
    /// Chart, Chart,
    /// Graph, Graph)
    /// Diagram, Diagram,
    /// Map, Map)
    /// Timeline, Timeline,
    /// Dashboard, Dashboard)
    Network  }

/// Action item in collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Action item identifier
        pub item_id: String,
    /// Action description
    /// Human-readable description

    pub description: String,
    /// Assigned to user /// ID
// ID
    /// Assigned To field

    pub assigned_to: String,
    /// Due date
        pub due_date: Option<DateTime<Utc>>,
    /// Action item status
    /// Current status of the operation or entity

    pub status: ActionItemStatus,
    /// Priority level (1-5)
    /// Priority field

    pub priority: u32,
    /// Related documents
    /// Related Documents field

    pub related_documents: Vec<String>,
    /// Creation timestamp
        pub created_at: DateTime<Utc>,
    /// Completion timestamp
        pub completed_at: Option<DateTime<Utc>>,
    /// Progress notes
        pub progress_notes: Vec<String> ,
 )
}

/// Status of action items
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ActionItemStatus {
    /// Open, Open,
    /// InProgress, InProgress)
    /// Completed, Completed,
    Cancelled  }

/// Decision made in collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Decision identifier
    /// Decision Id field

    pub decision_id: String,
    /// Decision description
    /// Human-readable description

    pub description: String,
    /// Decision maker user /// ID
// ID
    /// Decision Maker field

    pub decision_maker: String,
    /// Decision rationale
    /// Rationale field

    pub rationale: String,
    /// Decision options considered
    /// Options Considered field

    pub options_considered: Vec<String>,
    /// Selected option
    /// Selected Option field

    pub selected_option: String,
    /// Decision timestamp
        pub decided_at: DateTime<Utc>,
    /// Decision impact assessment
    /// Impact Assessment field

    pub impact_assessment: String,
    /// Decision approval status
        pub approved: bool,
    /// Approval by user /// ID
 ID
        pub approved_by: Option<String> ,
 )
}

/// Workspace collaboration metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetrics {
    /// Total active time (seconds)
    /// Total Active Time Seconds field

    pub total_active_time_seconds: u64,
    /// Number of participants
    /// Participant Count field

    pub participant_count: u32,
    /// Number of documents created
    /// Documents Created field

    pub documents_created: u32,
    /// Number of decisions made
    /// Decisions Made field

    pub decisions_made: u32,
    /// Number of action items
    /// Action Items Count field

    pub action_items_count: u32,
    /// Collaboration efficiency score (0.0 - 1.0)
    /// Collaboration Efficiency field

    pub collaboration_efficiency: f64 ,
 )
}

/// AI assistance levels for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIAssistanceLevel {
    /// None, None,
    /// Minimal, Minimal)
    /// Moderate, Moderate,
    /// Full, Full)
    Expert  }

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfiguration {
    /// Enable real-time collaboration
    /// Real Time Collaboration field

    pub real_time_collaboration: bool,
    /// Enable AI suggestions
    /// Ai Suggestions field

    pub ai_suggestions: bool,
    /// Enable performance monitoring
    /// Performance Monitoring field

    pub performance_monitoring: bool,
    /// Template to use
        pub template: Option<String> ,
 )
}

/// Session configuration details
#[derive(Debug, Clone, Serialize, Deserialize)];
pub struct SessionConfiguration {
    /// Maximum participants allowed
    /// Max Participants field

    pub max_participants: u32,
    /// Session timeout in minutes
    /// Session Timeout Minutes field

    pub session_timeout_minutes: u32,
    /// Auto-save enabled
    /// Auto Save Enabled field

    pub auto_save_enabled: bool,
    /// Recording enabled
    /// Recording Enabled field

    pub recording_enabled: bool,
    /// AI assistance level
    /// Ai Assistance Level field

    pub ai_assistance_level: AIAssistanceLevel ,
 )
}

/// AI facilitator for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFacilitator {
    /// Facilitator identifier
        pub facilitator_id: String,
    /// AI capabilities
        pub capabilities: Vec<String>,
    /// AI personality type
    /// Personality field

    pub personality: String,
    /// Expertise domains;
    /// Expertise Domains field

    pub expertise_domains: Vec<String>;};
impl Default for ParticipationMetrics  {fn default() -> Self  {Self { active_time_seconds: 0,
            messages_sent: 0,
            actions_performed: 0,
            collaboration_score: 0.0;}}}

impl Default for WorkspaceMetrics  {fn default() -> Self  {Self { total_active_time_seconds: 0,
            participant_count: 0,
            documents_created: 0,
            decisions_made: 0,
            action_items_count: 0,
            collaboration_efficiency: 0.0;}}}

impl Default for WorkspaceConfiguration  {fn default() -> Self  {Self { real_time_collaboration: true,
            ai_suggestions: true,
            performance_monitoring: true,
            template: None;}}}
