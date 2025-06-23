//! Communication Layer Module
//!
//! Implementation of real-time communication layer using WebSocket and HTTP protocols

pub mod protocol_router;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use futures_util::{SinkExt, Stream, StreamExt};
use serde::{Serialize, Deserialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use reqwest;
use parking_lot::RwLock as ParkingRwLock;

use crate::errors::{Result, SongbirdError};
pub use crate::traits::communication::*;

// Re-export the protocol router for tests
pub use protocol_router::ProtocolRouter;

/// WebSocket communication manager
#[derive(Clone)]
pub struct WebSocketCommunication {
    address: String,
    port: u16,
    connections: Arc<DashMap<String, Arc<WebSocketConnection>>>,
    server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    message_sender: Arc<broadcast::Sender<ServiceMessage>>,
    metrics: Arc<CommunicationMetrics>,
    config: WebSocketConfig,
    running: Arc<AtomicBool>,
}

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub message_buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(30),
            message_buffer_size: 1000,
        }
    }
}

/// Active WebSocket connection
pub struct WebSocketConnection {
    pub id: String,
    pub address: ServiceAddress,
    pub connected_at: Instant,
    pub last_heartbeat: Arc<Mutex<Instant>>,
    pub message_count: AtomicU64,
    pub is_healthy: AtomicBool,
    pub outgoing_tx: mpsc::UnboundedSender<WsMessage>,
}

/// Communication metrics
#[derive(Debug, Default)]
pub struct CommunicationMetrics {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub messages_failed: AtomicU64,
    pub active_connections: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
    pub connection_errors: AtomicU64,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing - reject all requests
    HalfOpen,  // Testing - allow limited requests
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Timeout before moving from open to half-open
    pub timeout: Duration,
    /// Window size for tracking failures
    pub window_size: Duration,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(60),
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<ParkingRwLock<CircuitState>>,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    last_failure_time: Arc<ParkingRwLock<Option<Instant>>>,
    half_open_requests: Arc<AtomicU64>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(ParkingRwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(ParkingRwLock::new(None)),
            half_open_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if request should be allowed through the circuit breaker
    pub fn should_allow_request(&self) -> bool {
        let state = *self.state.read();
        
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read() {
                    if last_failure.elapsed() >= self.config.timeout {
                        // Move to half-open state
                        *self.state.write() = CircuitState::HalfOpen;
                        self.half_open_requests.store(0, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                        tracing::info!("Circuit breaker moved to HALF_OPEN state");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let current_requests = self.half_open_requests.load(Ordering::Relaxed);
                if current_requests < self.config.half_open_max_requests as u64 {
                    self.half_open_requests.fetch_add(1, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let state = *self.state.read();
        
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold as u64 {
                    // Close the circuit
                    *self.state.write() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    tracing::info!("Circuit breaker moved to CLOSED state");
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                tracing::warn!("Received success while circuit is OPEN");
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        let state = *self.state.read();
        
        *self.last_failure_time.write() = Some(Instant::now());
        
        match state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure_count >= self.config.failure_threshold as u64 {
                    // Open the circuit
                    *self.state.write() = CircuitState::Open;
                    tracing::warn!("Circuit breaker moved to OPEN state after {} failures", failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Failed in half-open, go back to open
                *self.state.write() = CircuitState::Open;
                self.failure_count.fetch_add(1, Ordering::Relaxed);
                tracing::warn!("Circuit breaker moved back to OPEN state from HALF_OPEN");
            }
            CircuitState::Open => {
                // Already open, just record the failure
                self.failure_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get current circuit breaker state
    pub fn get_state(&self) -> CircuitState {
        *self.state.read()
    }

    /// Get circuit breaker statistics
    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.get_state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            half_open_requests: self.half_open_requests.load(Ordering::Relaxed),
            last_failure_time: self.last_failure_time.read()
                .map(|instant| chrono::Utc::now() - chrono::Duration::from_std(instant.elapsed()).unwrap_or_default()),
        }
    }

    /// Reset circuit breaker state
    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_requests.store(0, Ordering::Relaxed);
        *self.last_failure_time.write() = None;
        tracing::info!("Circuit breaker reset to CLOSED state");
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u64,
    pub success_count: u64,
    pub half_open_requests: u64,
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl WebSocketCommunication {
    pub fn new(address: String, port: u16) -> Self {
        let (message_sender, _) = broadcast::channel(1000);

        Self {
            address,
            port,
            connections: Arc::new(DashMap::new()),
            server_handle: Arc::new(RwLock::new(None)),
            message_sender: Arc::new(message_sender),
            metrics: Arc::new(CommunicationMetrics::default()),
            config: WebSocketConfig::default(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_config(address: String, port: u16, config: WebSocketConfig) -> Self {
        let mut comm = Self::new(address, port);
        comm.config = config;
        comm
    }

    /// Start the WebSocket server
    pub async fn start_server(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        let bind_addr = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| SongbirdError::Network { message: e.to_string() })?;

        info!("WebSocket server listening on {}", bind_addr);

        let connections = Arc::clone(&self.connections);
        let message_sender = Arc::clone(&self.message_sender);
        let metrics = Arc::clone(&self.metrics);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);

        running.store(true, Ordering::Relaxed);

        let handle = tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let connections = Arc::clone(&connections);
                        let message_sender = Arc::clone(&message_sender);
                        let metrics = Arc::clone(&metrics);
                        let config = config.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                addr,
                                connections,
                                message_sender,
                                metrics,
                                config,
                            )
                            .await
                            {
                                warn!("Connection handling error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                        metrics.connection_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        *self.server_handle.write().await = Some(handle);
        Ok(())
    }

    /// Handle incoming WebSocket connection
    async fn handle_connection(
        stream: TcpStream,
        addr: std::net::SocketAddr,
        connections: Arc<DashMap<String, Arc<WebSocketConnection>>>,
        message_sender: Arc<broadcast::Sender<ServiceMessage>>,
        metrics: Arc<CommunicationMetrics>,
        _config: WebSocketConfig,
    ) -> Result<()> {
        // Upgrade to WebSocket
        let websocket = accept_async(stream)
            .await
            .map_err(|e| SongbirdError::Network { message: e.to_string() })?;

        let connection_id = Uuid::new_v4().to_string();
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();

        // Split websocket for concurrent read/write immediately
        let (mut ws_sink, mut ws_stream) = websocket.split();

        // Create connection object without storing the websocket
        let connection = Arc::new(WebSocketConnection {
            id: connection_id.clone(),
            address: ServiceAddress {
                service_id: format!("websocket-{}", connection_id),
                instance_id: Some(connection_id.clone()),
                endpoint: Some(format!("{}:{}", addr.ip(), addr.port())),
            },
            connected_at: Instant::now(),
            last_heartbeat: Arc::new(Mutex::new(Instant::now())),
            message_count: AtomicU64::new(0),
            is_healthy: AtomicBool::new(true),
            outgoing_tx,
        });

        connections.insert(connection_id.clone(), Arc::clone(&connection));
        metrics.active_connections.fetch_add(1, Ordering::Relaxed);

        info!("New WebSocket connection: {} from {}", connection_id, addr);

        // Spawn outgoing message handler
        let connection_clone = Arc::clone(&connection);
        let metrics_clone = Arc::clone(&metrics);
        tokio::spawn(async move {
            while let Some(message) = outgoing_rx.recv().await {
                if let Err(e) = ws_sink.send(message).await {
                    warn!("Failed to send message: {}", e);
                    connection_clone.is_healthy.store(false, Ordering::Relaxed);
                    break;
                }
                metrics_clone.messages_sent.fetch_add(1, Ordering::Relaxed);
            }
        });

        // Handle incoming messages
        while let Some(message) = ws_stream.next().await {
            match message {
                Ok(WsMessage::Text(text)) => {
                    metrics.messages_received.fetch_add(1, Ordering::Relaxed);
                    metrics
                        .bytes_received
                        .fetch_add(text.len() as u64, Ordering::Relaxed);
                    connection.message_count.fetch_add(1, Ordering::Relaxed);

                    // Parse and broadcast message
                    if let Ok(service_message) = serde_json::from_str::<ServiceMessage>(&text) {
                        let _ = message_sender.send(service_message);
                    }
                }
                Ok(WsMessage::Binary(data)) => {
                    metrics.messages_received.fetch_add(1, Ordering::Relaxed);
                    metrics
                        .bytes_received
                        .fetch_add(data.len() as u64, Ordering::Relaxed);
                    connection.message_count.fetch_add(1, Ordering::Relaxed);
                }
                Ok(WsMessage::Ping(data)) => {
                    // Respond to ping with pong
                    let _ = connection.outgoing_tx.send(WsMessage::Pong(data));
                    *connection.last_heartbeat.lock().await = Instant::now();
                }
                Ok(WsMessage::Pong(_)) => {
                    *connection.last_heartbeat.lock().await = Instant::now();
                }
                Ok(WsMessage::Close(_)) => {
                    debug!("WebSocket connection closed: {}", connection_id);
                    break;
                }
                Ok(_) => {
                    // Handle any other message types (e.g., Frame)
                    debug!("Received unhandled WebSocket message type");
                }
                Err(e) => {
                    warn!("WebSocket error for connection {}: {}", connection_id, e);
                    metrics.messages_failed.fetch_add(1, Ordering::Relaxed);
                    break;
                }
            }
        }

        // Cleanup connection
        connections.remove(&connection_id);
        metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
        info!("WebSocket connection closed: {}", connection_id);

        Ok(())
    }

    /// Get the WebSocket server address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get the WebSocket server port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the full WebSocket URL
    pub fn url(&self) -> String {
        format!("ws://{}:{}", self.address, self.port)
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Get communication metrics
    pub fn get_metrics(&self) -> CommunicationMetrics {
        CommunicationMetrics {
            messages_sent: AtomicU64::new(self.metrics.messages_sent.load(Ordering::Relaxed)),
            messages_received: AtomicU64::new(
                self.metrics.messages_received.load(Ordering::Relaxed),
            ),
            messages_failed: AtomicU64::new(self.metrics.messages_failed.load(Ordering::Relaxed)),
            active_connections: AtomicU64::new(
                self.metrics.active_connections.load(Ordering::Relaxed),
            ),
            bytes_sent: AtomicU64::new(self.metrics.bytes_sent.load(Ordering::Relaxed)),
            bytes_received: AtomicU64::new(self.metrics.bytes_received.load(Ordering::Relaxed)),
            connection_errors: AtomicU64::new(
                self.metrics.connection_errors.load(Ordering::Relaxed),
            ),
        }
    }
}

#[async_trait]
impl CommunicationLayer for WebSocketCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        // Find connection by service_id
        let connection = self
            .connections
            .iter()
            .find(|entry| entry.value().address.service_id == target.service_id)
            .map(|entry| Arc::clone(entry.value()));

        if let Some(conn) = connection {
            let message_json =
                serde_json::to_string(&message)
                    .map_err(|e| SongbirdError::Serialization { message: e.to_string() })?;

            // Send message
            conn.outgoing_tx
                .send(WsMessage::Text(message_json))
                .map_err(|_| SongbirdError::Network { message: "Connection closed".to_string() })?;

            self.metrics
                .bytes_sent
                .fetch_add(message.payload.to_string().len() as u64, Ordering::Relaxed);

            Ok(CommunicationResponse {
                message_id: message.id.clone(),
                success: true,
                payload: Some(serde_json::json!({"status": "sent"})),
                error: None,
                timestamp: Utc::now(),
            })
        } else {
            Err(SongbirdError::Network { message: "Connection not found".to_string() })
        }
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        let message_json = serde_json::to_string(&message)
            .map_err(|e| SongbirdError::Serialization { message: e.to_string() })?;

        let mut responses = Vec::new();
        let ws_message = WsMessage::Text(message_json.clone());

        for connection in self.connections.iter() {
            let conn = connection.value();
            match conn.outgoing_tx.send(ws_message.clone()) {
                Ok(_) => {
                    responses.push(CommunicationResponse {
                        message_id: format!("{}_{}", message.id, conn.id),
                        success: true,
                        payload: Some(serde_json::json!({"status": "sent", "connection": conn.id})),
                        error: None,
                        timestamp: Utc::now(),
                    });
                }
                Err(_) => {
                    responses.push(CommunicationResponse {
                        message_id: format!("{}_{}", message.id, conn.id),
                        success: false,
                        payload: None,
                        error: Some("Connection closed".to_string()),
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        self.metrics.bytes_sent.fetch_add(
            (message_json.len() * responses.len()) as u64,
            Ordering::Relaxed,
        );

        Ok(responses)
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>> {
        use futures_util::StreamExt;
        
        // Create a receiver for the broadcast channel
        let mut receiver = self.message_sender.subscribe();
        
        // Create a stream that yields messages from WebSocket connections
        let stream = async_stream::stream! {
            loop {
                match receiver.recv().await {
                    Ok(message) => {
                        // Create a dummy service address for the message source
                        // In a real implementation, we'd track which connection sent the message
                        let source_address = ServiceAddress {
                            service_id: "websocket-client".to_string(),
                            instance_id: None,
                            endpoint: None,
                        };
                        
                        yield (source_address, message);
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("WebSocket message listener lagged, skipped {} messages", skipped);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("WebSocket message broadcast channel closed");
                        break;
                    }
                }
            }
        };
        
        Ok(Box::new(stream.boxed()))
    }

    async fn subscribe(&self, _topic: &str) -> Result<()> {
        // WebSocket subscriptions would be handled at message level
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> Result<()> {
        // WebSocket subscriptions would be handled at message level
        Ok(())
    }

    async fn connect(&self) -> Result<()> {
        self.start_server().await
    }

    async fn disconnect(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);

        // Close all connections
        for connection in self.connections.iter() {
            connection
                .value()
                .is_healthy
                .store(false, Ordering::Relaxed);
        }
        self.connections.clear();

        // Stop server
        if let Some(handle) = self.server_handle.write().await.take() {
            handle.abort();
        }

        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(CommunicationStats {
            messages_sent: self.metrics.messages_sent.load(Ordering::Relaxed),
            messages_received: self.metrics.messages_received.load(Ordering::Relaxed),
            bytes_sent: self.metrics.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.metrics.bytes_received.load(Ordering::Relaxed),
            active_connections: self.metrics.active_connections.load(Ordering::Relaxed),
            failed_connections: self.metrics.connection_errors.load(Ordering::Relaxed),
            last_activity: Some(chrono::Utc::now()),
        })
    }
}

/// In-memory communication implementation for testing
pub struct InMemoryCommunication {
    connected: bool,
}

impl InMemoryCommunication {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for InMemoryCommunication {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommunicationLayer for InMemoryCommunication {
    async fn send_message(
        &self,
        _target: ServiceAddress,
        _message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        Ok(CommunicationResponse {
            message_id: "test-response".to_string(),
            success: true,
            payload: Some(serde_json::json!({"status": "ok"})),
            error: None,
            timestamp: Utc::now(),
        })
    }

    async fn broadcast(&self, _message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        Ok(vec![])
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>> {
        // In-memory communication returns empty stream
        Ok(Box::new(futures_util::stream::empty()))
    }

    async fn subscribe(&self, _topic: &str) -> Result<()> {
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> Result<()> {
        Ok(())
    }

    async fn connect(&self) -> Result<()> {
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(CommunicationStats::default())
    }
}

/// HTTP communication implementation with enhanced service resolution
pub struct HttpCommunication {
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
    /// Service registry for endpoint resolution
    service_registry: Option<Arc<dyn ServiceRegistry>>,
    /// Circuit breakers per service
    circuit_breakers: Arc<DashMap<String, Arc<CircuitBreaker>>>,
    /// Default circuit breaker config
    circuit_breaker_config: CircuitBreakerConfig,
    /// Communication metrics with circuit breaker stats
    metrics: Arc<HttpCommunicationMetrics>,
}

/// HTTP communication metrics
#[derive(Debug, Default)]
pub struct HttpCommunicationMetrics {
    pub requests_sent: AtomicU64,
    pub requests_successful: AtomicU64,
    pub requests_failed: AtomicU64,
    pub requests_circuit_breaker_rejected: AtomicU64,
    pub total_response_time_ms: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
}

impl HttpCommunicationMetrics {
    pub fn record_request_sent(&self, bytes: u64) {
        self.requests_sent.fetch_add(1, Ordering::Relaxed);
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_request_success(&self, response_time_ms: u64, bytes: u64) {
        self.requests_successful.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_request_failure(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_circuit_breaker_rejection(&self) {
        self.requests_circuit_breaker_rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> HttpCommunicationStats {
        let requests_sent = self.requests_sent.load(Ordering::Relaxed);
        let avg_response_time = if requests_sent > 0 {
            self.total_response_time_ms.load(Ordering::Relaxed) / requests_sent
        } else {
            0
        };

        HttpCommunicationStats {
            requests_sent,
            requests_successful: self.requests_successful.load(Ordering::Relaxed),
            requests_failed: self.requests_failed.load(Ordering::Relaxed),
            requests_circuit_breaker_rejected: self.requests_circuit_breaker_rejected.load(Ordering::Relaxed),
            average_response_time_ms: avg_response_time,
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCommunicationStats {
    pub requests_sent: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub requests_circuit_breaker_rejected: u64,
    pub average_response_time_ms: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Service registry trait for HTTP communication
#[async_trait]
pub trait ServiceRegistry: Send + Sync {
    async fn get_service_endpoint(&self, service_id: &str) -> Result<Option<String>>;
    async fn get_service_info(&self, service_id: &str) -> Result<Option<crate::traits::service::ServiceInfo>>;
    async fn get_all_endpoints(&self) -> Vec<(String, String)>;
}

/// Simple HTTP service registry implementation
pub struct HttpServiceRegistry {
    /// Map of service_id -> endpoint URL
    service_endpoints: Arc<DashMap<String, String>>,
    /// Map of service_id -> ServiceInfo
    service_info: Arc<DashMap<String, crate::traits::service::ServiceInfo>>,
}

impl HttpServiceRegistry {
    pub fn new() -> Self {
        Self {
            service_endpoints: Arc::new(DashMap::new()),
            service_info: Arc::new(DashMap::new()),
        }
    }

    /// Register a service endpoint
    pub fn register_service_endpoint(&self, service_id: String, endpoint: String) {
        tracing::debug!(
            service_id = %service_id,
            endpoint = %endpoint,
            "Registering service endpoint"
        );
        self.service_endpoints.insert(service_id, endpoint);
    }

    /// Register service info
    pub fn register_service_info(&self, service_info: crate::traits::service::ServiceInfo) {
        let service_id = service_info.id.clone();
        tracing::debug!(
            service_id = %service_id,
            service_type = %service_info.service_type,
            "Registering service info"
        );
        self.service_info.insert(service_id, service_info);
    }

    /// Unregister a service
    pub fn unregister_service(&self, service_id: &str) {
        tracing::debug!(service_id = service_id, "Unregistering service");
        self.service_endpoints.remove(service_id);
        self.service_info.remove(service_id);
    }

    /// Get all registered service endpoints
    pub fn get_all_endpoints(&self) -> Vec<(String, String)> {
        self.service_endpoints
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

#[async_trait]
impl ServiceRegistry for HttpServiceRegistry {
    async fn get_service_endpoint(&self, service_id: &str) -> Result<Option<String>> {
        Ok(self.service_endpoints.get(service_id).map(|e| e.value().clone()))
    }

    async fn get_service_info(&self, service_id: &str) -> Result<Option<crate::traits::service::ServiceInfo>> {
        Ok(self.service_info.get(service_id).map(|info| info.value().clone()))
    }

    async fn get_all_endpoints(&self) -> Vec<(String, String)> {
        self.service_endpoints
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

impl HttpCommunication {
    pub fn new(base_url: String) -> Self {
        let timeout = Duration::from_secs(30);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("songbird-orchestrator/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
            timeout,
            service_registry: None,
            circuit_breakers: Arc::new(DashMap::new()),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            metrics: Arc::new(HttpCommunicationMetrics::default()),
        }
    }

    pub fn with_service_registry(mut self, registry: Arc<dyn ServiceRegistry>) -> Self {
        self.service_registry = Some(registry);
        self
    }

    pub fn with_circuit_breaker_config(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker_config = config;
        self
    }

    /// Get or create circuit breaker for a service
    fn get_circuit_breaker(&self, service_id: &str) -> Arc<CircuitBreaker> {
        self.circuit_breakers
            .entry(service_id.to_string())
            .or_insert_with(|| {
                tracing::debug!(service_id = service_id, "Creating new circuit breaker for service");
                Arc::new(CircuitBreaker::new(self.circuit_breaker_config.clone()))
            })
            .clone()
    }

    /// Get circuit breaker statistics for a service
    pub fn get_circuit_breaker_stats(&self, service_id: &str) -> Option<CircuitBreakerStats> {
        self.circuit_breakers
            .get(service_id)
            .map(|cb| cb.get_stats())
    }

    /// Get all circuit breaker statistics
    pub fn get_all_circuit_breaker_stats(&self) -> Vec<(String, CircuitBreakerStats)> {
        self.circuit_breakers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().get_stats()))
            .collect()
    }

    /// Reset circuit breaker for a service
    pub fn reset_circuit_breaker(&self, service_id: &str) {
        if let Some(cb) = self.circuit_breakers.get(service_id) {
            cb.reset();
        }
    }

    /// Get HTTP communication metrics
    pub fn get_http_metrics(&self) -> HttpCommunicationStats {
        self.metrics.get_stats()
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        // Recreate client with new timeout
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("songbird-orchestrator/0.1.0")
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    async fn build_url(&self, target: &ServiceAddress, path: Option<&str>) -> Result<String> {
        // Priority 1: Use explicit endpoint from target
        if let Some(endpoint) = &target.endpoint {
                if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                // Full URL provided
                return Ok(if let Some(path) = path {
                    let normalized_path = if path.starts_with('/') { 
                        path.to_string() 
                    } else { 
                        format!("/{}", path) 
                    };
                    format!("{}{}", endpoint.trim_end_matches('/'), normalized_path)
                } else {
                    endpoint.clone()
                });
            } else {
                // Relative endpoint - combine with base URL
                let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
                return Ok(if let Some(path) = path {
                    let normalized_path = if path.starts_with('/') { 
                        path.to_string() 
                    } else { 
                        format!("/{}", path) 
                    };
                    format!("{}{}", url, normalized_path)
                } else {
                    url
                });
            }
        }

        // Priority 2: Query service registry for endpoint
        if let Some(registry) = &self.service_registry {
            if let Ok(Some(endpoint)) = registry.get_service_endpoint(&target.service_id).await {
                let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                    endpoint
                } else {
                    format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'))
                };
                
                return Ok(if let Some(path) = path {
                    let normalized_path = if path.starts_with('/') { 
                        path.to_string() 
                    } else { 
                        format!("/{}", path) 
                    };
                    format!("{}{}", url, normalized_path)
                } else {
                    url
                });
            }

            // Try to get service info for more detailed endpoint information
            if let Ok(Some(service_info)) = registry.get_service_info(&target.service_id).await {
                if let Some(first_endpoint) = service_info.endpoints.first() {
                    let endpoint_path = &first_endpoint.path;
                    let url = if endpoint_path.starts_with("http://") || endpoint_path.starts_with("https://") {
                        endpoint_path.clone()
                    } else {
                        format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint_path.trim_start_matches('/'))
                    };
                    
                    return Ok(if let Some(path) = path {
                        let normalized_path = if path.starts_with('/') { 
                            path.to_string() 
                        } else { 
                            format!("/{}", path) 
                        };
                        format!("{}{}", url, normalized_path)
                    } else {
                        url
                    });
                }
            }
        }

        // Priority 3: Default service URL construction
        // Try common service patterns
        let service_patterns = [
            // Direct service access
            format!("http://localhost:8080/services/{}", target.service_id),
            // Service with standard port
            format!("http://127.0.0.1:8080/{}", target.service_id),
            // Use base URL with service path
            format!("{}/services/{}", self.base_url.trim_end_matches('/'), target.service_id),
        ];

        // For now, use the first pattern as default
        let base_service_url = &service_patterns[2]; // Use base URL pattern
        
        Ok(if let Some(path) = path {
            let normalized_path = if path.starts_with('/') { 
                path.to_string() 
            } else { 
                format!("/{}", path) 
            };
            format!("{}{}", base_service_url, normalized_path)
        } else {
            base_service_url.clone()
        })
    }

    /// Test connectivity to a service endpoint
    async fn test_service_connectivity(&self, target: &ServiceAddress) -> Result<bool> {
        let url = self.build_url(target, Some("/health")).await.unwrap_or_else(|_| {
            format!("{}/services/{}/health", self.base_url.trim_end_matches('/'), target.service_id)
        });

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // HTTP doesn't support native broadcast, but we can simulate it
        // by sending to multiple known endpoints if we have a service registry
        if let Some(registry) = &self.service_registry {
            let mut responses = Vec::new();
            let all_endpoints = registry.get_all_endpoints().await;
            
            tracing::debug!(
                "Broadcasting message {} to {} registered services", 
                message.id, 
                all_endpoints.len()
            );
            
            // Send message to all registered services
            for (service_id, endpoint) in all_endpoints {
                let target = ServiceAddress {
                    service_id: service_id.clone(),
                    instance_id: None,
                    endpoint: Some(endpoint),
                };
                
                match self.send_message(target, message.clone()).await {
                    Ok(response) => {
                        responses.push(response);
                    }
                    Err(e) => {
                        tracing::warn!(
                            service_id = %service_id,
                            error = %e,
                            "Failed to broadcast message to service"
                        );
                        
                        // Create error response for failed broadcast
                        responses.push(CommunicationResponse {
                            message_id: message.id.clone(),
                            success: false,
                            payload: Some(serde_json::json!({
                                "error": "Broadcast failed",
                                "service_id": service_id,
                                "details": e.to_string()
                            })),
                            error: Some(format!("Broadcast to {} failed: {}", service_id, e)),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
            
            tracing::info!(
                "Broadcast completed: {}/{} services responded successfully",
                responses.iter().filter(|r| r.success).count(),
                responses.len()
            );
            
            Ok(responses)
        } else {
            tracing::warn!("HTTP broadcast requested but no service registry available");
            
            // Return empty vec indicating no responses
            Ok(vec![])
        }
    }
}

#[async_trait]
impl CommunicationLayer for HttpCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        // Get circuit breaker for this service
        let circuit_breaker = self.get_circuit_breaker(&target.service_id);

        // Check circuit breaker status
        if !circuit_breaker.should_allow_request() {
            tracing::warn!(
                service_id = %target.service_id,
                message_id = %message.id,
                circuit_state = ?circuit_breaker.get_state(),
                "Request rejected by circuit breaker"
            );
            
            self.metrics.record_circuit_breaker_rejection();
            
            return Ok(CommunicationResponse {
                message_id: message.id,
                success: false,
                payload: Some(serde_json::json!({
                    "error": "Circuit breaker open",
                    "details": "Service is currently unavailable due to recent failures",
                    "circuit_state": circuit_breaker.get_state(),
                    "timestamp": chrono::Utc::now()
                })),
                error: Some("Circuit breaker open".to_string()),
                timestamp: chrono::Utc::now(),
            });
        }

        // Extract path from message payload if it contains HTTP-specific routing info
        let request_path = message.headers.get("x-request-path")
            .cloned()
            .or_else(|| {
                // Try to extract path from message payload
                if let Some(path) = message.payload.get("path") {
                    path.as_str().map(String::from)
                } else {
                    None
                }
            });

        let url = match self.build_url(&target, request_path.as_deref()).await {
            Ok(url) => url,
            Err(e) => {
                circuit_breaker.record_failure();
                self.metrics.record_request_failure();
                
                return Ok(CommunicationResponse {
                    message_id: message.id,
                    success: false,
                    payload: Some(serde_json::json!({
                        "error": "URL building failed",
                        "details": e.to_string(),
                        "timestamp": chrono::Utc::now()
                    })),
                    error: Some(format!("URL building failed: {}", e)),
                    timestamp: chrono::Utc::now(),
                });
            }
        };
        
        tracing::debug!(
            service_id = %target.service_id,
            url = %url,
            message_id = %message.id,
            circuit_state = ?circuit_breaker.get_state(),
            "Sending HTTP request"
        );
        
        // Build request
        let mut request_builder = self.client.post(&url);
        
        // Add headers
        for (key, value) in &message.headers {
            // Skip internal headers
            if !key.starts_with("x-request-") {
                request_builder = request_builder.header(key, value);
            }
        }
        
        // Add correlation headers
        request_builder = request_builder.header("x-message-id", &message.id);
        if let Some(correlation_id) = &message.correlation_id {
            request_builder = request_builder.header("x-correlation-id", correlation_id);
        }
        request_builder = request_builder.header("x-message-type", format!("{:?}", message.message_type));
        request_builder = request_builder.header("content-type", "application/json");
        
        // Prepare request payload
        let request_payload = match message.message_type {
            crate::traits::communication::MessageType::Request => {
                // For requests, send the payload directly
                message.payload
            }
            _ => {
                // For other message types, wrap in a message envelope
                serde_json::json!({
                    "message_id": message.id,
                    "message_type": message.message_type,
                    "topic": message.topic,
                    "payload": message.payload,
                    "timestamp": message.timestamp,
                    "correlation_id": message.correlation_id,
                    "ttl": message.ttl
                })
            }
        };
        
        // Estimate request size for metrics
        let request_size = serde_json::to_vec(&request_payload)
            .map(|v| v.len() as u64)
            .unwrap_or(0);
        
        self.metrics.record_request_sent(request_size);
        
        // Send request with timeout
        let start_time = std::time::Instant::now();
        let response_result = request_builder
            .json(&request_payload)
            .send()
            .await;

        let response = match response_result {
            Ok(resp) => resp,
            Err(e) => {
                let elapsed = start_time.elapsed();
                
                tracing::warn!(
                    service_id = %target.service_id,
                    url = %url,
                    error = %e,
                    elapsed_ms = elapsed.as_millis(),
                    "HTTP request failed"
                );
                
                // Record failure in circuit breaker and metrics
                circuit_breaker.record_failure();
                self.metrics.record_request_failure();
                
                return Ok(CommunicationResponse {
                    message_id: message.id,
                    success: false,
                    payload: Some(serde_json::json!({
                        "error": "HTTP request failed",
                        "details": e.to_string(),
                        "url": url,
                        "elapsed_ms": elapsed.as_millis(),
                        "timestamp": chrono::Utc::now()
                    })),
                    error: Some(format!("Network error: {}", e)),
                    timestamp: chrono::Utc::now(),
                });
            }
        };

        let elapsed = start_time.elapsed();
        let success = response.status().is_success();
        let status_code = response.status().as_u16();
        
        tracing::debug!(
            service_id = %target.service_id,
            status_code = status_code,
            elapsed_ms = elapsed.as_millis(),
            success = success,
            "HTTP response received"
        );
        
        // Parse response and get response size
        let (payload, response_size) = if success {
            match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    let size = serde_json::to_vec(&json)
                        .map(|v| v.len() as u64)
                        .unwrap_or(0);
                    (json, size)
                }
                Err(e) => {
                    tracing::warn!(
                        service_id = %target.service_id,
                        error = %e,
                        "Failed to parse response JSON"
                    );
                    let error_json = serde_json::json!({
                        "error": "Failed to parse response",
                        "details": e.to_string(),
                        "status_code": status_code
                    });
                    let size = serde_json::to_vec(&error_json)
                        .map(|v| v.len() as u64)
                        .unwrap_or(0);
                    (error_json, size)
                }
            }
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error_json = serde_json::json!({
                "error": format!("HTTP {} error", status_code),
                "details": error_text,
                "status_code": status_code,
                "url": url
            });
            let size = serde_json::to_vec(&error_json)
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            (error_json, size)
        };

        // Record metrics and circuit breaker state
        if success {
            circuit_breaker.record_success();
            self.metrics.record_request_success(elapsed.as_millis() as u64, response_size);
        } else {
            circuit_breaker.record_failure();
            self.metrics.record_request_failure();
        }

        Ok(CommunicationResponse {
            message_id: message.id,
            success,
            payload: Some(payload),
            error: if success {
                None
            } else {
                Some(format!("HTTP {} error", status_code))
            },
            timestamp: chrono::Utc::now(),
        })
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // HTTP doesn't support native broadcast, but we can simulate it
        // by sending to multiple known endpoints if we have a service registry
        if let Some(registry) = &self.service_registry {
            let mut responses = Vec::new();
            let all_endpoints = registry.get_all_endpoints().await;
            
            tracing::debug!(
                "Broadcasting message {} to {} registered services", 
                message.id, 
                all_endpoints.len()
            );
            
            // Send message to all registered services
            for (service_id, endpoint) in all_endpoints {
                let target = ServiceAddress {
                    service_id: service_id.clone(),
                    instance_id: None,
                    endpoint: Some(endpoint),
                };
                
                match self.send_message(target, message.clone()).await {
                    Ok(response) => {
                        responses.push(response);
                    }
                    Err(e) => {
                        tracing::warn!(
                            service_id = %service_id,
                            error = %e,
                            "Failed to broadcast message to service"
                        );
                        
                        // Create error response for failed broadcast
                        responses.push(CommunicationResponse {
                            message_id: message.id.clone(),
                            success: false,
                            payload: Some(serde_json::json!({
                                "error": "Broadcast failed",
                                "service_id": service_id,
                                "details": e.to_string()
                            })),
                            error: Some(format!("Broadcast to {} failed: {}", service_id, e)),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
            
            tracing::info!(
                "Broadcast completed: {}/{} services responded successfully",
                responses.iter().filter(|r| r.success).count(),
                responses.len()
            );
            
            Ok(responses)
        } else {
            tracing::warn!("HTTP broadcast requested but no service registry available");
            
            // Return empty vec indicating no responses
            Ok(vec![])
        }
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>> {
        // HTTP is request/response, not streaming
        // Return empty stream
        Ok(Box::new(futures_util::stream::empty()))
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        // HTTP doesn't support pub/sub natively
        tracing::debug!(topic = topic, "HTTP subscription requested (no-op)");
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<()> {
        tracing::debug!(topic = topic, "HTTP unsubscription requested (no-op)");
        Ok(())
    }

    async fn connect(&self) -> Result<()> {
        // HTTP is connectionless, but we can do a health check to the base URL
        tracing::info!(base_url = %self.base_url, "Testing HTTP communication layer connectivity");
        
        match self.client.get(&self.base_url).send().await {
            Ok(response) => {
                tracing::info!(
                    base_url = %self.base_url,
                    status = response.status().as_u16(),
                    "HTTP communication layer test successful"
                );
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    base_url = %self.base_url,
                    error = %e,
                    "HTTP communication layer test failed, but continuing (connectionless protocol)"
                );
                // Don't fail - HTTP is connectionless, so this might be expected
                Ok(())
            }
        }
    }

    async fn disconnect(&self) -> Result<()> {
        // HTTP is connectionless
        tracing::debug!("HTTP communication layer disconnect (no-op)");
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // Always considered "connected" for HTTP
        true
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        let http_stats = self.get_http_metrics();
        
        Ok(CommunicationStats {
            messages_sent: http_stats.requests_sent,
            messages_received: http_stats.requests_successful,
            bytes_sent: http_stats.bytes_sent,
            bytes_received: http_stats.bytes_received,
            active_connections: 1, // HTTP is connectionless, but indicate ready
            failed_connections: http_stats.requests_failed + http_stats.requests_circuit_breaker_rejected,
            last_activity: Some(chrono::Utc::now()),
        })
    }
}
