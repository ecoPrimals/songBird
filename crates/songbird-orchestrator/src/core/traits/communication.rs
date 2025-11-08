// Module imports
//! Communication /// Traits // Traits

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult as Result;
use std::collections::HashMap;
/// Communication layer trait
#[async_trait]
pub trait CommunicationLayer: Send + Sync { /// Send a message to a specific service
    async fn send_message() {


    -> Result<CommunicationResponse>
    /// Broadcast a message to all services
    async fn broadcast() {
    -> Result<Vec<CommunicationResponse>>
    /// Listen for incoming messages
    async fn listen() -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>
    /// Subscribe to a topic
    async fn subscribe(&self, topic: &str) -> Result<()>
    /// Unsubscribe from a topic
    async fn unsubscribe(&self, topic: &str) -> Result<()>
    /// Connect to the communication layer
    async fn connect(&self)self, -> Result<()>
    /// Disconnect from the communication layer
    async fn disconnect(&self)self, -> Result<()>



    }
pub struct ServiceAddress {
    /// Service Id field

    pub service_id: String,
    /// Instance Id field
    pub instance_id: Option<String>,
    /// Endpoint field
    pub endpoint: Option<String> ;
,

)
}
/// Message between services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMessage {
    /// Id field

    pub id: String,
    /// Message Type field
    pub message_type: MessageType,
    /// Topic field
    pub topic: Option<String>,
    /// Payload field
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>)
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Correlation Id field
    pub correlation_id: Option<String>,
    /// Reply To field
    pub reply_to: Option<ServiceAddress>,
    /// Ttl field
    pub ttl: Option<u64> ,
 )
}
/// Communication response (renamed to avoid conflict with service: :`ServiceResponse`,
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct CommunicationResponse {
    /// Message Id field

    pub message_id: String,
    /// Success field
    pub success: bool,
    /// Payload field
    pub payload: Option<serde_json::Value>,
    /// Error field
    pub error: Option<String> ,
 )
}
/// Type of message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Request, Request,
    /// Response, Response)
    /// Event, Event,
    /// Command, Command)
    Notification,;};
/// Communication statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationStats {
    /// Messages Sent field

    pub messages_sent: u64,
    /// Messages Received field
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Number of currently active connections
    pub active_connections: u64,
    /// Failed Connections field
    pub failed_connections: u64,
    /// Last Activity field
    pub last_activity: Option<DateTime<Utc>> ,
 )
}
