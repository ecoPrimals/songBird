//! Communication Module
//!
//! Basic communication infrastructure for Songbird

use chrono;
use parking_lot;
use serde_json;
use songbird_errors::{SongbirdError, SongbirdResult as Result};
use std::collections::HashMap;
use tracing::debug;

pub use songbird_discovery::traits::communication::MessageType;

// Import the proper hyper client and circuit breaker
use crate::communication::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

/// Service address for routing messages
#[derive(Debug, Clone)]
pub struct ServiceAddress {
    pub service_id: String,
    pub endpoint: Option<String>,
}

/// Service message for communication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceMessage {
    pub id: String,
    pub source: String,
    pub target: String,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_type: String,
}

/// Communication response wrapper
#[derive(Debug, Clone)]
pub struct CommunicationResponse {
    pub id: String,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// Communication statistics
#[derive(Debug, Clone, Default)]
pub struct CommunicationStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Communication layer trait
#[async_trait::async_trait]
pub trait CommunicationLayer: Send + Sync {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse>;
    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>>;
    async fn listen(
        &self,
    ) -> Result<Box<dyn futures::Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>;
    async fn subscribe(&self, topic: &str) -> Result<()>;
    async fn unsubscribe(&self, topic: &str) -> Result<()>;
    async fn get_stats(&self) -> Result<CommunicationStats>;
    async fn connect(&self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn is_connected(&self) -> bool;
}

/// HTTP client error type
#[derive(Debug, thiserror::Error)]
// HTTP client implementation is now in hyper_client.rs module

/// HTTP communication layer
pub struct HttpCommunication {
    client: self::hyper_client::HyperHttpClient,
    circuit_breaker: CircuitBreaker,
    stats: parking_lot::RwLock<CommunicationStats>,
}

impl HttpCommunication {
    pub fn new(_base_url: String) -> Result<Self> {
        let client = self::hyper_client::HyperHttpClient::new().map_err(|e| {
            SongbirdError::network(format!("Failed to create HTTP client: {e}").to_string())
        })?;

        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        Ok(Self {
            client,
            circuit_breaker,
            stats: parking_lot::RwLock::new(CommunicationStats {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
            }),
        })
    }
}

#[async_trait::async_trait]
impl CommunicationLayer for HttpCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        // Check circuit breaker
        if !self.circuit_breaker.should_allow_request().await {
            return Err(SongbirdError::network(
                "Circuit breaker is open, request rejected".to_string(),
            ));
        }

        // Construct URL from target
        let url = if let Some(endpoint) = &target.endpoint {
            format!("{}/{}", endpoint, target.service_id)
        } else {
            target.service_id.clone()
        };

        // Send HTTP request
        let result = self.client.post_json(&url, &message).await;

        match result {
            Ok(response) => {
                // Update stats
                {
                    let mut stats = self.stats.write();
                    stats.messages_sent += 1;
                    stats.bytes_sent += response.body().len() as u64;
                }

                // Record success
                self.circuit_breaker.record_success().await;

                Ok(CommunicationResponse {
                    id: message.id,
                    status: response.status().as_u16(),
                    body: response.text().unwrap_or_default(),
                    headers: response
                        .headers()
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                })
            }
            Err(e) => {
                // Record failure
                self.circuit_breaker.record_failure().await;

                Err(SongbirdError::network(format!("HTTP request failed: {e}")))
            }
        }
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // For HTTP, broadcast isn't typically supported, but we can simulate it
        // by sending to a broadcast endpoint
        let broadcast_target = ServiceAddress {
            service_id: "broadcast".to_string(),
            endpoint: None,
        };

        let response = self.send_message(broadcast_target, message).await?;
        Ok(vec![response])
    }

    async fn listen(
        &self,
    ) -> Result<Box<dyn futures::Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>
    {
        // HTTP doesn't support streaming/listening, return empty stream
        Ok(Box::new(futures::stream::empty()))
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        tracing::info!("HTTP subscription to topic: {}", topic);
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<()> {
        tracing::info!("HTTP unsubscription from topic: {}", topic);
        Ok(())
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(self.stats.read().clone())
    }

    async fn connect(&self) -> Result<()> {
        // HTTP doesn't require persistent connections
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        // HTTP doesn't require explicit disconnection
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // HTTP is always "connected" if client is available
        true
    }
}

impl std::fmt::Display for HttpCommunication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HttpCommunication")
    }
}

/// WebSocket communication layer
pub struct WebSocketCommunication {
    host: String,
    port: u16,
    circuit_breaker: CircuitBreaker,
    stats: parking_lot::RwLock<CommunicationStats>,
    connected: parking_lot::RwLock<bool>,
}

impl WebSocketCommunication {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            circuit_breaker: CircuitBreaker::new(CircuitBreakerConfig::default()),
            stats: parking_lot::RwLock::new(CommunicationStats {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
            }),
            connected: parking_lot::RwLock::new(false),
        }
    }
}

#[async_trait::async_trait]
impl CommunicationLayer for WebSocketCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        // Check circuit breaker
        if !self.circuit_breaker.should_allow_request().await {
            return Err(SongbirdError::network(
                "Circuit breaker is open, request rejected".to_string(),
            ));
        }

        // Check if connected
        if !self.is_connected().await {
            return Err(SongbirdError::network(
                "WebSocket not connected".to_string(),
            ));
        }

        // Simulate WebSocket message sending
        let payload = serde_json::to_string(&message).map_err(|e| {
            SongbirdError::network(format!("Failed to serialize message: {e}").to_string())
        })?;

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.messages_sent += 1;
            stats.bytes_sent += payload.len() as u64;
        }

        // Record success
        self.circuit_breaker.record_success().await;

        tracing::info!(
            "WebSocket message sent to {}: {}",
            target.service_id,
            payload
        );

        Ok(CommunicationResponse {
            id: message.id,
            status: 200,
            body: "{}".to_string(),
            headers: HashMap::new(),
        })
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // WebSocket supports broadcasting
        let broadcast_target = ServiceAddress {
            service_id: "broadcast".to_string(),
            endpoint: Some(format!("ws://{}:{}/broadcast", self.host, self.port)),
        };

        let response = self.send_message(broadcast_target, message).await?;
        Ok(vec![response])
    }

    async fn listen(
        &self,
    ) -> Result<Box<dyn futures::Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>
    {
        // WebSocket would normally provide a stream of incoming messages
        // For now, return empty stream but log the action
        tracing::info!("WebSocket listening on {}:{}", self.host, self.port);
        Ok(Box::new(futures::stream::empty()))
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        tracing::info!("WebSocket subscription to topic: {}", topic);
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<()> {
        tracing::info!("WebSocket unsubscription from topic: {}", topic);
        Ok(())
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(self.stats.read().clone())
    }

    async fn connect(&self) -> Result<()> {
        // Simulate WebSocket connection
        *self.connected.write() = true;
        tracing::info!("WebSocket connected to {}:{}", self.host, self.port);
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        // Simulate WebSocket disconnection
        *self.connected.write() = false;
        tracing::info!("WebSocket disconnected from {}:{}", self.host, self.port);
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        *self.connected.read()
    }
}

/// In-memory communication layer for testing and local development
pub struct InMemoryCommunication {
    stats: parking_lot::RwLock<CommunicationStats>,
    message_queue: parking_lot::RwLock<Vec<(ServiceAddress, ServiceMessage)>>,
    subscribers: parking_lot::RwLock<HashMap<String, Vec<String>>>, // topic -> subscriber_ids
}

impl Default for InMemoryCommunication {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryCommunication {
    pub fn new() -> Self {
        Self {
            stats: parking_lot::RwLock::new(CommunicationStats {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
            }),
            message_queue: parking_lot::RwLock::new(Vec::new()),
            subscribers: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Get all messages in the queue (for testing)
    pub fn get_messages(&self) -> Vec<(ServiceAddress, ServiceMessage)> {
        self.message_queue.read().clone()
    }

    /// Clear all messages from the queue
    pub fn clear_messages(&self) {
        self.message_queue.write().clear();
    }
}

#[async_trait::async_trait]
impl CommunicationLayer for InMemoryCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        // Calculate message size for stats
        let message_size = serde_json::to_string(&message)
            .map(|s| s.len())
            .unwrap_or(0) as u64;

        // Store message in queue
        self.message_queue
            .write()
            .push((target.clone(), message.clone()));

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.messages_sent += 1;
            stats.bytes_sent += message_size;
        }

        tracing::debug!(
            "In-memory message sent to {}: {}",
            target.service_id,
            message.id
        );

        let message_id = message.id.clone();
        Ok(CommunicationResponse {
            id: message_id,
            status: 200,
            body: serde_json::to_string(&message).unwrap_or_default(),
            headers: HashMap::new(),
        })
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // For in-memory, broadcast to all subscribers
        let subscribers_snapshot = {
            let subscribers = self.subscribers.read();
            subscribers.clone()
        };

        let mut responses = Vec::new();

        for (topic, subscriber_ids) in subscribers_snapshot.iter() {
            debug!(
                "Broadcasting to topic '{}' with {} subscribers",
                topic,
                subscriber_ids.len()
            );

            for subscriber_id in subscriber_ids {
                let target = ServiceAddress {
                    service_id: subscriber_id.clone(),
                    endpoint: Some(format!("memory://{topic}")),
                };

                let response = self.send_message(target, message.clone()).await?;
                responses.push(response);
            }
        }

        if responses.is_empty() {
            // No subscribers, but still record the broadcast attempt
            let broadcast_target = ServiceAddress {
                service_id: "broadcast".to_string(),
                endpoint: None,
            };
            let response = self.send_message(broadcast_target, message).await?;
            responses.push(response);
        }

        Ok(responses)
    }

    async fn listen(
        &self,
    ) -> Result<Box<dyn futures::Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>
    {
        // In-memory implementation could provide a stream of queued messages
        // For now, return empty stream but this could be enhanced
        tracing::debug!("In-memory listening started");
        Ok(Box::new(futures::stream::empty()))
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        let subscriber_id = format!("subscriber-{}", uuid::Uuid::new_v4());

        {
            let mut subscribers = self.subscribers.write();
            subscribers
                .entry(topic.to_string())
                .or_default()
                .push(subscriber_id.clone());
        }

        tracing::info!(
            "In-memory subscription to topic '{}' with id '{}'",
            topic,
            subscriber_id
        );
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<()> {
        {
            let mut subscribers = self.subscribers.write();
            subscribers.remove(topic);
        }

        tracing::info!("In-memory unsubscription from topic '{}'", topic);
        Ok(())
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        let mut stats = self.stats.read().clone();
        // Add queue size to received messages for completeness
        stats.messages_received = self.message_queue.read().len() as u64;
        Ok(stats)
    }

    async fn connect(&self) -> Result<()> {
        // In-memory is always "connected"
        tracing::debug!("In-memory communication connected");
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        // Clear all state on disconnect
        self.message_queue.write().clear();
        self.subscribers.write().clear();
        tracing::debug!("In-memory communication disconnected");
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        true
    }
}

pub mod benchmarks;
pub mod circuit_breaker;
pub mod hyper_client;
pub mod performance_optimizer;

// Re-export circuit breaker types
pub use circuit_breaker::{CircuitBreakerStats, CircuitState};

// Re-export hyper client types
pub use hyper_client::HyperResponse;

// Make HyperHttpClient public through module system
pub use self::hyper_client::{HyperClientError, HyperHttpClient};

// Re-export protocol router types
