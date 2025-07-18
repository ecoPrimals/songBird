// Module imports
//! Communication Traits

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use songbird_errors::Result;
use std::collections::HashMap;
/// Communication layer trait
#[async_trait]
pub trait CommunicationLayer: Send + Sync {
    /// Send a message to a specific service
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse>;
    /// Broadcast a message to all services
    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>>;
    /// Listen for incoming messages
    async fn listen(
        &self,
    ) -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>;
    /// Subscribe to a topic
    async fn subscribe(&self, topic: &str) -> Result<()>;
    /// Unsubscribe from a topic
    async fn unsubscribe(&self, topic: &str) -> Result<()>;
    /// Connect to the communication layer
    async fn connect(&self) -> Result<()>;
    /// Disconnect from the communication layer  
    async fn disconnect(&self) -> Result<()>;
    /// Check if connected
    async fn is_connected(&self) -> bool;
    /// Get communication statistics
    async fn get_stats(&self) -> Result<CommunicationStats>;
}
/// Service address for routing messages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceAddress {
    pub service_id: String,
    pub instance_id: Option<String>,
    pub endpoint: Option<String>,
}
/// Message between services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMessage {
    pub id: String,
    pub message_type: MessageType,
    pub topic: Option<String>,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<ServiceAddress>,
    pub ttl: Option<u64>,
}
/// Communication response (renamed to avoid conflict with service::ServiceResponse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationResponse {
    pub message_id: String,
    pub success: bool,
    pub payload: Option<serde_json::Value>,
    pub error: Option<String>,
}
/// Type of message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Command,
    Notification,
}
/// Communication statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub active_connections: u64,
    pub failed_connections: u64,
    pub last_activity: Option<DateTime<Utc>>,
}
