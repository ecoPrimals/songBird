//! WebSocket connection management and handling

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::messages::MessageFormat;

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct StreamingConnection {
    /// Connection identifier
    pub connection_id: String,
    /// User identifier
    pub user_id: Option<String>,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Subscription topics
    pub subscriptions: Vec<String>,
    /// Connection established timestamp
    pub established_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
}

/// Types of streaming connections
#[derive(Debug, Clone)]
pub enum ConnectionType {
    Human,
    AIAgent,
    Service,
    Monitor,
}

/// Request to establish a new connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRequest {
    /// User identifier (optional for anonymous connections)
    pub user_id: Option<String>,
    /// Connection type identifier
    pub connection_type: String,
    /// Topics to subscribe to initially
    pub initial_subscriptions: Vec<String>,
    /// Connection metadata
    pub metadata: HashMap<String, String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Client capabilities
    pub client_capabilities: Vec<String>,
    /// Authentication token (if required)
    pub auth_token: Option<String>,
    /// Preferred message format
    pub preferred_message_format: MessageFormat,
}

/// Response to connection establishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResponse {
    /// Assigned connection identifier
    pub connection_id: String,
    /// Connection status
    pub status: ConnectionStatus,
    /// Available topics for subscription
    pub available_topics: Vec<String>,
    /// Connection configuration
    pub connection_config: ConnectionConfig,
    /// Welcome message
    pub welcome_message: String,
    /// Session token for future authentication
    pub session_token: String,
    /// Server capabilities
    pub server_capabilities: Vec<String>,
    /// Quality metrics for this connection
    pub connection_quality: ConnectionQualityMetrics,
}

/// Connection status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Reconnecting,
    Failed,
    Unauthorized,
    RateLimited,
}

/// Connection configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Heartbeat interval in seconds
    pub heartbeat_interval_seconds: u32,
    /// Maximum message size in bytes
    pub max_message_size_bytes: u32,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u32,
    /// Reconnection policy
    pub reconnection_policy: ReconnectionPolicy,
    /// Quality monitoring enabled
    pub quality_monitoring_enabled: bool,
    /// Compression enabled
    pub compression_enabled: bool,
}

/// Reconnection policy for failed connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectionPolicy {
    /// Maximum reconnection attempts
    pub max_attempts: u32,
    /// Initial delay between attempts (seconds)
    pub initial_delay_seconds: u32,
    /// Maximum delay between attempts (seconds)
    pub max_delay_seconds: u32,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Enable jitter to prevent thundering herd
    pub use_jitter: bool,
}

/// Connection quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQualityMetrics {
    /// Connection latency in milliseconds
    pub latency_ms: f64,
    /// Packet loss rate (0.0 - 1.0)
    pub packet_loss_rate: f64,
    /// Throughput in Mbps
    pub throughput_mbps: f64,
    /// Connection stability score (0.0 - 1.0)
    pub connection_stability: f64,
}

/// Request to start a collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSessionRequest {
    /// Session type
    pub session_type: super::session::CollaborationSessionType,
    /// Participants
    pub participants: Vec<String>,
    /// Session objective
    pub objective: String,
    /// Estimated duration in minutes
    pub estimated_duration_minutes: u32,
    /// Record session
    pub record_session: Option<bool>,
    /// AI assistance level
    pub ai_assistance_level: Option<super::session::AIAssistanceLevel>,
    /// Initial workspace configuration
    pub workspace_config: Option<super::session::WorkspaceConfiguration>,
}

/// Response for collaboration session creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSessionResponse {
    /// Created session identifier
    pub session_id: String,
    /// Workspace URL
    pub workspace_url: String,
    /// Initial workspace state
    pub workspace: super::session::CollaborationWorkspace,
    /// Session configuration
    pub session_config: super::session::SessionConfiguration,
    /// AI facilitator information
    pub ai_facilitator: super::session::AIFacilitator,
}

impl StreamingConnection {
    /// Create new streaming connection
    pub fn new(
        connection_id: String,
        user_id: Option<String>,
        connection_type: ConnectionType,
    ) -> Self {
        let now = Utc::now();
        Self {
            connection_id,
            user_id,
            connection_type,
            subscriptions: vec![],
            established_at: now,
            last_activity: now,
            metadata: HashMap::new(),
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Add subscription topic
    pub fn add_subscription(&mut self, topic: String) {
        if !self.subscriptions.contains(&topic) {
            self.subscriptions.push(topic);
        }
        self.update_activity();
    }

    /// Remove subscription topic
    pub fn remove_subscription(&mut self, topic: &str) {
        self.subscriptions.retain(|t| t != topic);
        self.update_activity();
    }

    /// Check if connection is active (activity within last 5 minutes)
    pub fn is_active(&self) -> bool {
        let five_minutes_ago = Utc::now() - chrono::Duration::minutes(5);
        self.last_activity > five_minutes_ago
    }

    /// Get connection age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.established_at).num_seconds()
    }

    /// Get idle time in seconds
    pub fn idle_seconds(&self) -> i64 {
        (Utc::now() - self.last_activity).num_seconds()
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_seconds: 30,
            max_message_size_bytes: 1024 * 1024, // 1MB
            connection_timeout_seconds: 300,     // 5 minutes
            reconnection_policy: ReconnectionPolicy::default(),
            quality_monitoring_enabled: true,
            compression_enabled: true,
        }
    }
}

impl Default for ReconnectionPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_seconds: 1,
            max_delay_seconds: 60,
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl Default for ConnectionQualityMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            packet_loss_rate: 0.0,
            throughput_mbps: 0.0,
            connection_stability: 1.0,
        }
    }
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "connected"),
            ConnectionStatus::Connecting => write!(f, "connecting"),
            ConnectionStatus::Disconnected => write!(f, "disconnected"),
            ConnectionStatus::Reconnecting => write!(f, "reconnecting"),
            ConnectionStatus::Failed => write!(f, "failed"),
            ConnectionStatus::Unauthorized => write!(f, "unauthorized"),
            ConnectionStatus::RateLimited => write!(f, "rate_limited"),
        }
    }
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::Human => write!(f, "human"),
            ConnectionType::AIAgent => write!(f, "ai_agent"),
            ConnectionType::Service => write!(f, "service"),
            ConnectionType::Monitor => write!(f, "monitor"),
        }
    }
}
