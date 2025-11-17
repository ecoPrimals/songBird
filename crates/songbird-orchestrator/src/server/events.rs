/// Real-Time Event Broadcasting System
///
/// This module provides a pub-sub event system for broadcasting real-time
/// events to connected WebSocket clients. Events include service updates,
/// health changes, and federation status changes.
///
/// Version: 0.2.1
/// Last Updated: November 11, 2025 - Phase 4
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

/// Event types that can be broadcast
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Service registration/update
    ServiceUpdate,

    /// Health status change
    HealthUpdate,

    /// Federation status change
    FederationStatus,

    /// Peer connection/disconnection
    PeerUpdate,

    /// Task execution updates
    TaskUpdate,
}

impl EventType {
    /// Convert event type to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ServiceUpdate => "service_update",
            Self::HealthUpdate => "health_update",
            Self::FederationStatus => "federation_status",
            Self::PeerUpdate => "peer_update",
            Self::TaskUpdate => "task_update",
        }
    }

    /// Parse event type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "service_update" => Some(Self::ServiceUpdate),
            "health_update" => Some(Self::HealthUpdate),
            "federation_status" => Some(Self::FederationStatus),
            "peer_update" => Some(Self::PeerUpdate),
            "task_update" => Some(Self::TaskUpdate),
            _ => None,
        }
    }
}

/// Event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event type
    pub event_type: String,

    /// Event payload as JSON
    pub payload: serde_json::Value,

    /// Timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, payload: serde_json::Value) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            event_type: event_type.as_str().to_string(),
            payload,
            timestamp,
        }
    }

    /// Create a service update event
    pub fn service_update(service_name: String, status: String, address: String) -> Self {
        Self::new(
            EventType::ServiceUpdate,
            serde_json::json!({
                "type": "service_update",
                "service_name": service_name,
                "status": status,
                "address": address,
            }),
        )
    }

    /// Create a health update event
    pub fn health_update(service_name: String, healthy: bool, message: Option<String>) -> Self {
        let mut payload = serde_json::json!({
            "type": "health_update",
            "service_name": service_name,
            "healthy": healthy,
        });

        if let Some(msg) = message {
            payload["message"] = serde_json::Value::String(msg);
        }

        Self::new(EventType::HealthUpdate, payload)
    }

    /// Create a federation status event
    pub fn federation_status(
        total_services: usize,
        total_peers: usize,
        uptime_seconds: u64,
    ) -> Self {
        Self::new(
            EventType::FederationStatus,
            serde_json::json!({
                "type": "federation_status",
                "total_services": total_services,
                "total_peers": total_peers,
                "uptime_seconds": uptime_seconds,
            }),
        )
    }
}

/// Subscription information for a client
#[derive(Debug, Clone)]
struct Subscription {
    /// Client ID
    client_id: String,

    /// Event types the client is subscribed to
    event_types: HashSet<String>,

    /// Sender for broadcasting events to this client
    sender: broadcast::Sender<Event>,
}

/// Event broadcaster
///
/// Manages WebSocket client subscriptions and broadcasts events
pub struct EventBroadcaster {
    /// Active subscriptions by client ID
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,

    /// Global event channel
    global_tx: broadcast::Sender<Event>,

    /// Event statistics
    stats: Arc<RwLock<BroadcasterStats>>,
}

/// Broadcaster statistics
#[derive(Debug, Clone, Default)]
pub struct BroadcasterStats {
    /// Total events broadcast
    pub total_events: u64,

    /// Events by type
    pub events_by_type: HashMap<String, u64>,

    /// Total subscribers
    pub total_subscribers: usize,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new() -> Self {
        let (global_tx, _) = broadcast::channel(1000);

        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            global_tx,
            stats: Arc::new(RwLock::new(BroadcasterStats::default())),
        }
    }

    /// Subscribe a client to events
    ///
    /// # Arguments
    /// * `client_id` - Unique client identifier
    /// * `event_types` - Event types to subscribe to
    ///
    /// # Returns
    /// * Receiver for events
    pub async fn subscribe(
        &self,
        client_id: String,
        event_types: Vec<String>,
    ) -> broadcast::Receiver<Event> {
        let (tx, rx) = broadcast::channel(100);

        let subscription = Subscription {
            client_id: client_id.clone(),
            event_types: event_types.iter().cloned().collect(),
            sender: tx,
        };

        let mut subs = self.subscriptions.write().await;
        subs.insert(client_id.clone(), subscription);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_subscribers = subs.len();

        info!("Client {} subscribed to events: {:?}", client_id, event_types);

        rx
    }

    /// Unsubscribe a client from events
    pub async fn unsubscribe(&self, client_id: &str, event_types: Vec<String>) {
        let mut subs = self.subscriptions.write().await;

        if let Some(subscription) = subs.get_mut(client_id) {
            for event_type in event_types {
                subscription.event_types.remove(&event_type);
            }

            info!(
                "Client {} unsubscribed from some events, remaining: {:?}",
                client_id, subscription.event_types
            );

            // Remove subscription if no event types left
            if subscription.event_types.is_empty() {
                subs.remove(client_id);
                info!("Client {} fully unsubscribed", client_id);
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_subscribers = subs.len();
    }

    /// Remove a client subscription completely
    pub async fn remove_client(&self, client_id: &str) {
        let mut subs = self.subscriptions.write().await;
        subs.remove(client_id);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_subscribers = subs.len();

        info!("Client {} removed from subscriptions", client_id);
    }

    /// Broadcast an event to all subscribed clients
    pub async fn broadcast(&self, event: Event) {
        let event_type = event.event_type.clone();

        debug!("Broadcasting event: {}", event_type);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
            *stats.events_by_type.entry(event_type.clone()).or_insert(0) += 1;
        }

        // Get all subscriptions
        let subs = self.subscriptions.read().await;

        // Send to each subscribed client
        let mut sent_count = 0;
        let mut error_count = 0;

        for subscription in subs.values() {
            // Check if client is subscribed to this event type
            if subscription.event_types.contains(&event_type) {
                match subscription.sender.send(event.clone()) {
                    Ok(_) => sent_count += 1,
                    Err(e) => {
                        error_count += 1;
                        warn!("Failed to send event to client {}: {}", subscription.client_id, e);
                    }
                }
            }
        }

        if sent_count > 0 {
            debug!(
                "Event {} sent to {} client(s) ({} errors)",
                event_type, sent_count, error_count
            );
        }
    }

    /// Get broadcaster statistics
    pub async fn get_stats(&self) -> BroadcasterStats {
        self.stats.read().await.clone()
    }

    /// Get number of active subscribers
    pub async fn subscriber_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_creation() {
        let event = Event::service_update(
            "test-service".to_string(),
            "running".to_string(),
            "localhost:8080".to_string(),
        );

        assert_eq!(event.event_type, "service_update");
        assert!(event.timestamp > 0);
    }

    #[tokio::test]
    async fn test_subscription() {
        let broadcaster = EventBroadcaster::new();

        let _rx =
            broadcaster.subscribe("client1".to_string(), vec!["service_update".to_string()]).await;

        assert_eq!(broadcaster.subscriber_count().await, 1);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let broadcaster = EventBroadcaster::new();

        let mut rx =
            broadcaster.subscribe("client1".to_string(), vec!["service_update".to_string()]).await;

        let event = Event::service_update(
            "test-service".to_string(),
            "running".to_string(),
            "localhost:8080".to_string(),
        );

        broadcaster.broadcast(event.clone()).await;

        // Receive event
        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type, "service_update");
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let broadcaster = EventBroadcaster::new();

        let _rx = broadcaster
            .subscribe(
                "client1".to_string(),
                vec!["service_update".to_string(), "health_update".to_string()],
            )
            .await;

        assert_eq!(broadcaster.subscriber_count().await, 1);

        broadcaster.unsubscribe("client1", vec!["service_update".to_string()]).await;

        // Still subscribed to health_update
        assert_eq!(broadcaster.subscriber_count().await, 1);

        broadcaster.unsubscribe("client1", vec!["health_update".to_string()]).await;

        // Now fully unsubscribed
        assert_eq!(broadcaster.subscriber_count().await, 0);
    }
}
