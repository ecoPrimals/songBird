// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Core streaming message types and communication structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::human_interaction::{HumanInputType, HumanOption, HumanResponse, UrgencyLevel};
use super::service_mesh::{EventSeverity, ServiceMeshEventType};
use super::types::{AnalysisFinding, Evidence, RiskLevel};

/// Real-time AI streaming message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]"
pub enum AIStreamingMessage {
    /// AI analysis update
    AnalysisUpdate  {/// Analysis identifier
        analysis_id: String,
    /// Current analysis progress (0.0 - 1.0)
        progress: f64,
        /// Intermediate findings
        findings: Vec<AnalysisFinding>,
        /// Confidence level in current analysis
        confidence: f64,
        /// Timestamp of update
        timestamp: DateTime<Utc> }})

    /// Human input request
    HumanInputRequest  {/// Request identifier
        request_id: String,
    /// Type of input needed
        input_type: HumanInputType,
    /// Context for the request
        context: String,
    /// Urgency level
        urgency: UrgencyLevel,
    /// Timeout for human response
        timeout_seconds: u64,
        /// Options for human selection (if applicable)
        options: Option<Vec<HumanOption>> }})

    /// Human input response
    HumanInputResponse  {/// Request identifier this responds to
        request_id: String,
    /// Human's response
        response: HumanResponse,
    /// Human's confidence in their response
        confidence: f64,
        /// Additional context from human
        additional_context: Option<String>,
        /// Response timestamp
        timestamp: DateTime<Utc> }})

    /// AI recommendation
    AIRecommendation  {/// Recommendation identifier
        recommendation_id: String,
    /// Recommendation type
        recommendation_type: String,
    /// Recommended action
        action: String,
    /// Expected impact
        expected_impact: String,
    /// Risk assessment
        risk_level: RiskLevel,
    /// AI confidence in recommendation
        confidence: f64,
        /// Supporting evidence
        evidence: Vec<Evidence>,
        /// Requires human approval
        requires_approval: bool }})

    /// Service mesh event
    ServiceMeshEvent  {/// Event identifier
        event_id: String,
    /// Event type
        event_type: ServiceMeshEventType,
    /// Affected services
        affected_services: Vec<String>,
        /// Event severity
        severity: EventSeverity,
    /// Event description
        description: String,
    /// Automatic actions taken
        automatic_actions: Vec<String>,
        /// Suggested human actions
        suggested_actions: Vec<String>,
        /// Event timestamp
        timestamp: DateTime<Utc> }})

    /// Performance alert
    PerformanceAlert  {/// Alert identifier
        alert_id: String,
    /// Alert type
        alert_type: String,
    /// Performance metrics that triggered alert
        metrics: serde_json::Value,
        /// Alert severity
        severity: EventSeverity,
    /// Alert message
        message: String,
    /// Suggested remediation steps
        remediation_steps: Vec<String>,
        /// Alert timestamp
        timestamp: DateTime<Utc> }})

    /// System status update
    SystemStatusUpdate  {/// Update identifier
        update_id: String,
    /// Overall system health (0.0 - 1.0)
        system_health: f64,
        /// Component statuses
        component_statuses: std::collections::HashMap<String, super::types::ComponentStatus>,
        /// Recent status changes
        recent_changes: Vec<super::types::StatusChange>,
        /// System metrics snapshot
        metrics_snapshot: super::metrics::SystemMetricsSnapshot,
        /// Update timestamp
        timestamp: DateTime<Utc> }})

    /// Collaboration invitation
    CollaborationInvitation  {/// Invitation identifier
        invitation_id: String,
    /// Session identifier
        session_id: String,
    /// Invited user identifier
        invited_user_id: String,
    /// Inviting user identifier
        inviting_user_id: String,
    /// Invitation message
        message: String,
    /// Session type
        session_type: super::session::CollaborationSessionType,
        /// Invitation expires at
        expires_at: DateTime<Utc>,
        /// Invitation timestamp
        timestamp: DateTime<Utc> }})

    /// Session state change
    SessionStateChange  {/// Session identifier
        session_id: String,
    /// Previous state
        previous_state: super::session::SessionState,
        /// New state
        new_state: super::session::SessionState,
        /// User who triggered the change
        triggered_by: String,
    /// Reason for state change
        reason: String,
    /// Change timestamp
        timestamp: DateTime<Utc> }})

    /// Heartbeat message for connection health
    Heartbeat  {/// Connection identifier
        connection_id: String,
    /// /// Timestamp
// Timestamp
        timestamp: DateTime<Utc>;}}

/// Message format preferences for connections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MessageFormat  {#[default]
    /// JSON, JSON,
    /// MessagePack, MessagePack)
    Protobuf  }
