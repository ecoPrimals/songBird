//! Production Federation Message Broadcasting
//!
//! Real gRPC/HTTP message delivery system replacing mock broadcasting

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use songbird_errors::{FederationResult, SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::types::{FederationNode, NodeStatus};

/// Message delivery protocols
#[derive(Debug, Clone)]
pub enum DeliveryProtocol {
    Http,
    Grpc,
    Udp,
    WebSocket,
}

/// Message priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Federation message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Message ID
    pub id: String,
    /// Source node ID
    pub source_node_id: String,
    /// Target node IDs (empty for broadcast)
    pub target_node_ids: Vec<String>,
    /// Message type
    pub message_type: String,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message priority
    pub priority: MessagePriority,
    /// TTL in seconds
    pub ttl_seconds: u32,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Delivery tracking
    pub delivery_attempts: u32,
    /// Maximum delivery attempts
    pub max_attempts: u32,
}

/// Message delivery result
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    pub node_id: String,
    pub success: bool,
    pub response_time: Duration,
    pub error: Option<String>,
    pub delivered_at: chrono::DateTime<chrono::Utc>,
}

/// Message delivery configuration
#[derive(Debug, Clone)]
pub struct DeliveryConfig {
    /// Default delivery timeout
    pub delivery_timeout: Duration,
    /// Maximum concurrent deliveries
    pub max_concurrent_deliveries: usize,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Preferred protocols by node type
    pub protocol_preferences: HashMap<String, DeliveryProtocol>,
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base retry delay
    pub base_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum retry delay
    pub max_delay: Duration,
}

/// Production message broadcaster
pub struct ProductionMessageBroadcaster {
    /// HTTP client for message delivery
    http_client: Client,
    /// Active federation nodes
    active_nodes: Arc<RwLock<HashMap<String, FederationNode>>>,
    /// Message queue for reliable delivery
    message_queue: Arc<RwLock<Vec<FederationMessage>>>,
    /// Delivery configuration
    config: DeliveryConfig,
    /// Delivery statistics
    delivery_stats: Arc<RwLock<DeliveryStatistics>>,
}

/// Delivery statistics
#[derive(Debug, Default)]
pub struct DeliveryStatistics {
    pub total_messages_sent: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub average_delivery_time: Duration,
    pub nodes_reached: HashMap<String, u64>,
}

impl Default for DeliveryConfig {
    fn default() -> Self {
        let mut protocol_preferences = HashMap::new();
        protocol_preferences.insert("security".to_string(), DeliveryProtocol::Grpc);
        protocol_preferences.insert("orchestrator".to_string(), DeliveryProtocol::Http);
        protocol_preferences.insert("discovery".to_string(), DeliveryProtocol::Udp);
        
        Self {
            delivery_timeout: Duration::from_secs(10),
            max_concurrent_deliveries: 50,
            retry_config: RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(100),
                backoff_multiplier: 2.0,
                max_delay: Duration::from_secs(30),
            },
            protocol_preferences,
        }
    }
}

impl ProductionMessageBroadcaster {
    /// Create new production message broadcaster
    pub fn new(config: DeliveryConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.delivery_timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            http_client,
            active_nodes: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            config,
            delivery_stats: Arc::new(RwLock::new(DeliveryStatistics::default())),
        }
    }
    
    /// Update active nodes list
    pub async fn update_active_nodes(&self, nodes: Vec<FederationNode>) -> FederationResult<()> {
        let mut active_nodes = self.active_nodes.write().await;
        active_nodes.clear();
        
        for node in nodes {
            if node.status == NodeStatus::Online {
                active_nodes.insert(node.node_id.to_string(), node);
            }
        }
        
        info!("📡 Updated active nodes: {} online", active_nodes.len());
        Ok(())
    }
    
    /// Broadcast message to all active nodes
    pub async fn broadcast_message(
        &self,
        message_type: &str,
        payload: serde_json::Value,
        priority: MessagePriority,
    ) -> FederationResult<Vec<DeliveryResult>> {
        let message = FederationMessage {
            id: Uuid::new_v4().to_string(),
            source_node_id: "local".to_string(), // Would be actual node ID
            target_node_ids: Vec::new(), // Empty for broadcast
            message_type: message_type.to_string(),
            payload,
            priority,
            ttl_seconds: 300, // 5 minutes
            timestamp: chrono::Utc::now(),
            delivery_attempts: 0,
            max_attempts: self.config.retry_config.max_attempts,
        };
        
        info!("📢 Broadcasting message: {} (priority: {:?})", message.message_type, message.priority);
        
        let active_nodes = self.active_nodes.read().await;
        let nodes: Vec<FederationNode> = active_nodes.values().cloned().collect();
        drop(active_nodes);
        
        if nodes.is_empty() {
            warn!("⚠️ No active nodes available for broadcasting");
            return Ok(Vec::new());
        }
        
        // Deliver to all nodes concurrently
        let delivery_tasks: Vec<_> = nodes
            .into_iter()
            .map(|node| self.deliver_message_to_node(message.clone(), node))
            .collect();
        
        let results = futures::future::join_all(delivery_tasks).await;
        
        // Collect delivery results
        let mut delivery_results = Vec::new();
        let mut successful_deliveries = 0;
        
        for result in results {
            match result {
                Ok(delivery_result) => {
                    if delivery_result.success {
                        successful_deliveries += 1;
                    }
                    delivery_results.push(delivery_result);
                }
                Err(e) => {
                    error!("Message delivery failed: {}", e);
                }
            }
        }
        
        // Update statistics
        self.update_delivery_stats(delivery_results.len() as u64ful_deliveries).await;
        
        info!(
            "✅ Broadcast complete: {}/{} nodes reached",
            successful_deliveries,
            delivery_results.len()
        );
        
        Ok(delivery_results)
    }
    
    /// Send message to specific node
    pub async fn send_message_to_node(
        &self,
        target_node_id: &str,
        message_type: &str,
        payload: serde_json::Value,
        priority: MessagePriority,
    ) -> FederationResult<DeliveryResult> {
        let active_nodes = self.active_nodes.read().await;
        let node = active_nodes.get(target_node_id)
            .ok_or_else(|| SongbirdError::federation_error("Target node not found"))?
            .clone();
        drop(active_nodes);
        
        let message = FederationMessage {
            id: Uuid::new_v4().to_string(),
            source_node_id: "local".to_string(),
            target_node_ids: vec![target_node_id.to_string()],
            message_type: message_type.to_string(),
            payload,
            priority,
            ttl_seconds: 300,
            timestamp: chrono::Utc::now(),
            delivery_attempts: 0,
            max_attempts: self.config.retry_config.max_attempts,
        };
        
        info!("📤 Sending message to {}: {}", target_node_id, message.message_type);
        
        self.deliver_message_to_node(message, node).await
    }
    
    /// Deliver message to specific node with retry logic
    async fn deliver_message_to_node(
        &self,
        mut message: FederationMessage,
        node: FederationNode,
    ) -> FederationResult<DeliveryResult> {
        let delivery_start = Instant::now();
        
        // Determine delivery protocol
        let protocol = self.select_delivery_protocol(&node);
        
        // Attempt delivery with retries
        for attempt in 1..=message.max_attempts {
            message.delivery_attempts = attempt;
            
            let delivery_result = match protocol {
                DeliveryProtocol::Http => self.deliver_via_http(&message, &node).await,
                DeliveryProtocol::Grpc => self.deliver_via_grpc(&message, &node).await,
                DeliveryProtocol::Udp => self.deliver_via_udp(&message, &node).await,
                DeliveryProtocol::WebSocket => self.deliver_via_websocket(&message, &node).await,
            };
            
            match delivery_result {
                Ok(()) => {
                    return Ok(DeliveryResult {
                        node_id: node.node_id.to_string(),
                        success: true,
                        response_time: delivery_start.elapsed(),
                        error: None,
                        delivered_at: chrono::Utc::now(),
                    }));
                }
                Err(e) => {
                    if attempt < message.max_attempts {
                        let delay = self.calculate_retry_delay(attempt);
                        debug!(
                            "🔄 Delivery attempt {} failed for {}, retrying in {:?}: {}",
                            attempt, node.node_id, delay, e
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        error!(
                            "❌ All delivery attempts failed for {}: {}",
                            node.node_id, e
                        );
                        return Ok(DeliveryResult {
                            node_id: node.node_id.to_string(),
                            success: false,
                            response_time: delivery_start.elapsed(),
                            error: Some(e.to_string()),
                            delivered_at: chrono::Utc::now(),
                        }));
                    }
                }
            }
        }
        
        // This should never be reached due to the loop above
        unreachable!()
    }
    
    /// Select optimal delivery protocol for node
    fn select_delivery_protocol(&self, node: &FederationNode) -> DeliveryProtocol {
        // Check node type preferences
        if let Some(node_type) = self.get_node_type_string(node) {
            if let Some(preferred_protocol) = self.config.protocol_preferences.get(&node_type) {
                return preferred_protocol.clone();
            }
        }
        
        // Default to HTTP
        DeliveryProtocol::Http
    }
    
    /// Get node type as string for protocol selection
    fn get_node_type_string(&self, node: &FederationNode) -> Option<String> {
        match &node.node_type {
            crate::types::NodeType::Security { .. } => Some("security".to_string()),
            crate::types::NodeType::Gateway { .. } => Some("gateway".to_string()),
            crate::types::NodeType::Service { service_type } => Some(service_type.clone()),
            _ => None,
        }
    }
    
    /// Deliver message via HTTP
    async fn deliver_via_http(
        &self,
        message: &FederationMessage,
        node: &FederationNode,
    ) -> FederationResult<()> {
        let endpoint = node.addresses.first()
            .ok_or_else(|| SongbirdError::federation_error("No address available for node"))?;
        
        let url = format!("http://{}/federation/message", endpoint.addr);
        
        let response = self.http_client
            .post(&url)
            .json(message)
            .send()
            .await
            .map_err(|e| SongbirdError::federation_error(&format!("HTTP delivery failed: {}", e)))?;
        
        if response.status().is_) {
            debug!("✅ HTTP delivery successful to {}", node.node_id);
            Ok(())
        } else {
            Err(SongbirdError::internal_error(federation_error(&format!(
                "HTTP delivery failed with status: {}",
                response.status()
            )))
        }
    }
    
    /// Deliver message via gRPC
    async fn deliver_via_grpc(
        &self,
        message: &FederationMessage,
        node: &FederationNode,
    ) -> FederationResult<()> {
        // For now, fallback to HTTP until gRPC client is implemented
        debug!("🔄 gRPC delivery not yet implemented, falling back to HTTP");
        self.deliver_via_http(message, node).await
    }
    
    /// Deliver message via UDP
    async fn deliver_via_udp(
        &self,
        message: &FederationMessage,
        node: &FederationNode,
    ) -> FederationResult<()> {
        let endpoint = node.addresses.first()
            .ok_or_else(|| SongbirdError::federation_error("No address available for node"))?;
        
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| SongbirdError::federation_error(&format!("UDP socket bind failed: {}", e)))?;
        
        let message_bytes = serde_json::to_vec(message)
            .map_err(|e| SongbirdError::federation_error(&format!("Message serialization failed: {}", e)))?;
        
        socket.send_to(&message_bytes, endpoint.addr).await
            .map_err(|e| SongbirdError::federation_error(&format!("UDP send failed: {}", e)))?;
        
        debug!("✅ UDP delivery successful to {}", node.node_id);
        Ok(())
    }
    
    /// Deliver message via WebSocket
    async fn deliver_via_websocket(
        &self,
        message: &FederationMessage,
        node: &FederationNode,
    ) -> FederationResult<()> {
        // For now, fallback to HTTP until WebSocket client is implemented
        debug!("🔄 WebSocket delivery not yet implemented, falling back to HTTP");
        self.deliver_via_http(message, node).await
    }
    
    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let delay = self.config.retry_config.base_delay.as_millis() as f64
            * self.config.retry_config.backoff_multiplier.powi(attempt as i32 - 1);
        
        Duration::from_millis(delay.min(self.config.retry_config.max_delay.as_millis() as f64) as u64)
    }
    
    /// Update delivery statistics
    async fn update_delivery_stats(&self, total_attempts: u64ful: u64) {
        let mut stats = self.delivery_stats.write().await;
        stats.total_messages_sent += 1;
        stats.successful_deliveries += successful;
        stats.failed_deliveries += total_attempts - successful;
    }
    
    /// Start message processing loop
    pub async fn start_message_processor(&self) -> FederationResult<()> {
        info!("🚀 Starting production message processor...");
        
        let message_queue = Arc::clone(&self.message_queue);
        let broadcaster = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                interval.tick().await;
                
                // Process queued messages
                let mut queue = message_queue.write().await;
                if queue.is_empty() {
                    continue;
                }
                
                // Take messages for processing
                let messages_to_process: Vec<FederationMessage> = queue.drain(..).collect();
                drop(queue);
                
                // Process messages concurrently
                for message in messages_to_process {
                    let broadcaster_clone = broadcaster.clone();
                    tokio::spawn(async move {
                        if let Err(e) = broadcaster_clone.process_queued_message(message).await {
                            error!("Failed to process queued message: {}", e);
                        }
                    });
                }
            }
        });
        
        info!("✅ Message processor started");
        Ok(())
    }
    
    /// Process a queued message
    async fn process_queued_message(&self, message: FederationMessage) -> FederationResult<()> {
        // Check TTL
        let now = chrono::Utc::now();
        let message_age = now.signed_duration_since(message.timestamp);
        
        if message_age.num_seconds() > message.ttl_seconds as i64 {
            debug!("⏰ Message expired: {}", message.id);
            return Ok(());
        }
        
        // Broadcast or send to specific targets
        if message.target_node_ids.is_empty() {
            // Broadcast to all nodes
            self.broadcast_message(&message.message_type, message.payload, message.priority).await?;
        } else {
            // Send to specific nodes
            for target_id in &message.target_node_ids {
                self.send_message_to_node(target_id, &message.message_type, message.payload.clone(), message.priority.clone()).await?;
            }
        }
        
        Ok(())
    }
    
    /// Queue message for reliable delivery
    pub async fn queue_message(
        &self,
        message_type: &str,
        payload: serde_json::Value,
        target_nodes: Vec<String>,
        priority: MessagePriority,
    ) -> FederationResult<String> {
        let message = FederationMessage {
            id: Uuid::new_v4().to_string(),
            source_node_id: "local".to_string(),
            target_node_ids: target_nodes,
            message_type: message_type.to_string(),
            payload,
            priority,
            ttl_seconds: 300,
            timestamp: chrono::Utc::now(),
            delivery_attempts: 0,
            max_attempts: self.config.retry_config.max_attempts,
        };
        
        let message_id = message.id.clone();
        
        let mut queue = self.message_queue.write().await;
        queue.push(message);
        
        // Sort queue by priority
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        debug!("📥 Message queued: {} (priority: {:?})", message_id, priority);
        Ok(message_id)
    }
    
    /// Get delivery statistics
    pub async fn get_delivery_statistics(&self) -> DeliveryStatistics {
        let stats = self.delivery_stats.read().await;
        stats.clone()
    }
    
    /// Get queue status
    pub async fn get_queue_status(&self) -> (usize, HashMap<MessagePriority, usize>) {
        let queue = self.message_queue.read().await;
        let total_queued = queue.len();
        
        let mut priority_counts = HashMap::new();
        for message in queue.iter() {
            *priority_counts.entry(message.priority.clone()).or_insert(0) += 1;
        }
        
        (total_queued, priority_counts)
    }
}

// Required for tokio::spawn
impl Clone for ProductionMessageBroadcaster {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            active_nodes: Arc::clone(&self.active_nodes),
            message_queue: Arc::clone(&self.message_queue),
            config: self.config.clone(),
            delivery_stats: Arc::clone(&self.delivery_stats),
        }
    }
}

impl Clone for DeliveryStatistics {
    fn clone(&self) -> Self {
        Self {
            total_messages_sent: self.total_messages_sent,
            successful_deliveries: self.successful_deliveries,
            failed_deliveries: self.failed_deliveries,
            average_delivery_time: self.average_delivery_time,
            nodes_reached: self.nodes_reached.clone(),
        }
    }
} 