//! Canonical Communication Types
//!
//! This module provides unified communication type definitions that replace
//! fragmented types across different crates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical Message Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MessageType {
    /// Request message
    #[default]
    Request,
    /// Response message
    Response,
    /// Event notification
    Event,
    /// Command message
    Command,
    /// Query message
    Query,
    /// General notification
    Notification,
    /// Health check message
    HealthCheck,
    /// Metrics message
    Metrics,
}

/// Canonical Communication Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationResponse {
    /// Unique message identifier
    pub message_id: String,
    /// Response payload
    pub payload: Option<Value>,
    /// Error information if any
    pub error: Option<String>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Success indicator
    pub success: bool,
}

/// Canonical Communication Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStats {
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Number of active connections
    pub active_connections: u64,
    /// Number of failed connections
    pub failed_connections: u64,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Messages sent count
    pub messages_sent: u64,
    /// Messages received count
    pub messages_received: u64,
    /// Connected nodes count
    pub connected_nodes: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

impl Default for CommunicationResponse {
    fn default() -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            payload: None,
            error: None,
            timestamp: Utc::now(),
            success: true,
        }
    }
}

impl Default for CommunicationStats {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            active_connections: 0,
            failed_connections: 0,
            last_activity: Utc::now(),
            messages_sent: 0,
            messages_received: 0,
            connected_nodes: 0,
            uptime_seconds: 0,
        }
    }
}

impl CommunicationResponse {
    /// Create a successful response
    pub fn success(message_id: impl Into<String>, payload: Option<Value>) -> Self {
        Self {
            message_id: message_id.into(),
            payload,
            error: None,
            timestamp: Utc::now(),
            success: true,
        }
    }

    /// Create an error response
    pub fn error(message_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            payload: None,
            error: Some(error.into()),
            timestamp: Utc::now(),
            success: false,
        }
    }

    /// Create a response with both payload and potential error
    pub fn new(
        message_id: impl Into<String>,
        payload: Option<Value>,
        error: Option<String>,
        success: bool,
    ) -> Self {
        Self {
            message_id: message_id.into(),
            payload,
            error,
            timestamp: Utc::now(),
            success,
        }
    }
}

impl CommunicationStats {
    /// Create new communication stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Update bytes sent
    pub fn add_bytes_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
        self.last_activity = Utc::now();
    }

    /// Update bytes received
    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
        self.last_activity = Utc::now();
    }

    /// Increment messages sent
    pub fn increment_messages_sent(&mut self) {
        self.messages_sent += 1;
        self.last_activity = Utc::now();
    }

    /// Increment messages received
    pub fn increment_messages_received(&mut self) {
        self.messages_received += 1;
        self.last_activity = Utc::now();
    }

    /// Update connection counts
    pub fn update_connections(&mut self, active: u64, failed: u64) {
        self.active_connections = active;
        self.failed_connections = failed;
        self.last_activity = Utc::now();
    }

    /// Update connected nodes count
    pub fn update_connected_nodes(&mut self, count: u64) {
        self.connected_nodes = count;
        self.last_activity = Utc::now();
    }

    /// Update uptime
    pub fn update_uptime(&mut self, seconds: u64) {
        self.uptime_seconds = seconds;
    }
}
