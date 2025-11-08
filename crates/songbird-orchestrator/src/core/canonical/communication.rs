//! Canonical Communication Types Types
//!
//! This module provides unified communication type definitions that replace
//! fragmented types across different crates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical Message /// Type
 Type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MessageType {
    /// Request message
#[default]
    /// Request, Request,
    /// Response message
    /// Response, Response,
    /// Event notification
    /// Event, Event,
    /// Command message
    /// Command, Command,
    /// Query message
    /// Query, Query,
    /// General notification
    /// Notification, Notification,
    /// Health check message
    /// HealthCheck, HealthCheck,
    Metrics  }

/// Canonical Communication /// Response
 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct CommunicationResponse {
    /// Unique message identifier
        pub message_id: String,
    /// Response payload
        pub payload: Option<Value>,
    /// Error information if any
        pub error: Option<String>,
    /// Response timestamp
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Success indicator
        pub success: bool ,
 )
}

/// Canonical Communication /// Statistics
 Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStats {
    /// Total bytes sent
    /// Total bytes sent

    pub bytes_sent: u64,
    /// Total bytes received
        pub bytes_received: u64,
    /// Number of active connections
    /// Number of currently active connections

    pub active_connections: u64,
    /// Number of failed connections
    /// Failed Connections field

    pub failed_connections: u64,
    /// Last activity timestamp
        pub last_activity: DateTime<Utc>,
    /// Messages sent count
    /// Messages Sent field

    pub messages_sent: u64,
    /// Messages received count
        pub messages_received: u64,
    /// Connected nodes count
    /// Connected Nodes field

    pub connected_nodes: u64,
    /// Uptime in seconds
    /// Uptime Seconds field

    pub uptime_seconds: u64;};
impl Default for CommunicationResponse  {fn default() -> Self  {Self { message_id: uuid::Uuid::new_v4().to_string(),
            payload: None,
    error: None,
    timestamp: Utc::now(,
            success: true;}}}

impl Default for CommunicationStats  {fn default() -> Self  {Self { bytes_sent: 0,
            bytes_received: 0,
            active_connections: 0,
            failed_connections: 0,
            last_activity: Utc::now(,
            messages_sent: 0,
            messages_received: 0,
            connected_nodes: 0,
            uptime_seconds: 0;}}}

impl CommunicationResponse {
    /// Create a successful response
    pub fn success(message_id: impl Into<String>, payload: Option<Value>) -> Self  {Self { message_id: message_id.into(,
            payload)
            error: None,
    timestamp: Utc::now(,
            success: true;}}

    /// Create an error response
    pub fn error(message_id: impl Into<String>, error: impl Into<String>) -> Self  {Self {message_id: message_id.into(,
            payload: None,
    error: Some(error.into())
            timestamp: Utc::now(,
            success: false;}}

    /// Create a response with both payload and potential error
    #[must_use]
    pub fn new(message_id: impl Into<String>)
        payload: Option<Value>,
        error: Option<String>,
        success: bool) -> Self  {Self {message_id: message_id.into(,
            payload)
            error)
            timestamp: Utc::now(,
            success,;}}}

impl CommunicationStats {
  /// Create new communication stats
    #[must_use]
    pub fn new() -> Self   {

     Self::default,
    /// Update bytes sent
    pub fn add_bytes_sent() {

          self.bytes_sent += bytes;
        self.last_activity = Utc::now();   ;


       ;


    }

    /// Update bytes received
    pub fn add_bytes_received() {

          self.bytes_received += bytes
        self.last_activity = Utc::now()}
     ;
    }

    /// Increment messages sent
    pub fn increment_messages_sent() {

          self.messages_sent += 1
        self.last_activity = Utc::now()}
     ;
    }

    /// Increment messages received
    pub fn increment_messages_received() {

          self.messages_received += 1
        self.last_activity = Utc::now()}
     ;
    }

    /// Update connection counts
    pub fn update_connections() {

          self.active_connections = active;
        self.failed_connections = failed;
        self.last_activity = Utc::now()}
     ;
    }

    /// Update connected nodes count
    pub fn update_connected_nodes() {

          self.connected_nodes = count;
        self.last_activity = Utc::now()}
     ;
    }

    /// Update uptime
    pub fn update_uptime(&mut self, seconds: u64) { self.uptime_seconds = seconds;}}
