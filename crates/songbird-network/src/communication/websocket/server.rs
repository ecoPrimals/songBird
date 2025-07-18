// Module imports
//! WebSocket Server Implementation
//!
//! Core WebSocket server functionality with connection handling

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use futures_util::{SinkExt, Stream, StreamExt};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use songbird_errors::{NetworkError, Result, SongbirdError};
use songbird_discovery::traits::communication::{
    CommunicationLayer, CommunicationResponse, CommunicationStats,
    ServiceAddress, ServiceMessage
use super::{WebSocketConfig, WebSocketConnection};
use crate::communication::metrics::CommunicationMetrics;
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
    /// Start the WebSocket server
    pub async fn start_server(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        let bind_addr = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| SongbirdError::Network(Box::new(NetworkError { message: e.to_string() })))?;
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
        });
        *self.server_handle.write().await = Some(handle);
        Ok(())
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
        let connection_id: Arc<str> = Arc::from(Uuid::new_v4().to_string());
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();
        // Split websocket for concurrent read/write immediately
        let (mut ws_sink, mut ws_stream) = websocket.split();
        // Create connection object without storing the websocket
        let connection = Arc::new(WebSocketConnection {
            id: Arc::clone(&connection_id), // Using Arc<str> for efficient string sharing
            address: ServiceAddress {
                service_id: format!("websocket-{}", connection_id),
                instance_id: Some(connection_id.to_string()),
                endpoint: Some(format!("{}:{}", addr.ip(), addr.port())),
            },
            connected_at: Instant::now(),
            last_heartbeat: Arc::new(tokio::sync::Mutex::new(Instant::now())),
            message_count: std::sync::atomic::AtomicU64::new(0),
            is_healthy: std::sync::atomic::AtomicBool::new(true),
            outgoing_tx,
        });
        
        connections.insert(connection_id.to_string(), Arc::clone(&connection));
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
                metrics_clone.messages_sent.fetch_add(1, Ordering::Relaxed);
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
                Ok(WsMessage::Binary(data)) => {
                        .fetch_add(data.len() as u64, Ordering::Relaxed);
                Ok(WsMessage::Ping(data)) => {
                    // Respond to ping with pong
                    let _ = connection.outgoing_tx.send(WsMessage::Pong(data));
                    *connection.last_heartbeat.lock().await = Instant::now();
                Ok(WsMessage::Pong(_)) => {
                Ok(WsMessage::Close(_)) => {
                    debug!("WebSocket connection closed: {}", connection_id);
                Ok(_) => {
                    // Handle any other message types (e.g., Frame)
                    debug!("Received unhandled WebSocket message type");
                Err(e) => {
                    warn!("WebSocket error for connection {}: {}", connection_id, e);
                    metrics.messages_failed.fetch_add(1, Ordering::Relaxed);
        // Cleanup connection
        connections.remove(&connection_id);
        metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
        info!("WebSocket connection closed: {}", connection_id);
    /// Get the WebSocket server address
    pub fn address(&self) -> &str {
        &self.address
    /// Get the WebSocket server port
    pub fn port(&self) -> u16 {
        self.port
    /// Get the full WebSocket URL
    pub fn url(&self) -> String {
        format!("ws://{}:{}", self.address, self.port)
    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    /// Get communication metrics
    pub fn get_metrics(&self) -> CommunicationMetrics {
        CommunicationMetrics {
            messages_sent: std::sync::atomic::AtomicU64::new(self.metrics.messages_sent.load(Ordering::Relaxed)),
            messages_received: std::sync::atomic::AtomicU64::new(
                self.metrics.messages_received.load(Ordering::Relaxed),
            ),
            messages_failed: std::sync::atomic::AtomicU64::new(self.metrics.messages_failed.load(Ordering::Relaxed)),
            active_connections: std::sync::atomic::AtomicU64::new(
                self.metrics.active_connections.load(Ordering::Relaxed),
            bytes_sent: std::sync::atomic::AtomicU64::new(self.metrics.bytes_sent.load(Ordering::Relaxed)),
            bytes_received: std::sync::atomic::AtomicU64::new(self.metrics.bytes_received.load(Ordering::Relaxed)),
            connection_errors: std::sync::atomic::AtomicU64::new(
                self.metrics.connection_errors.load(Ordering::Relaxed),
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
                .map_err(|_| SongbirdError::Network(Box::new(NetworkError { message: "Connection closed".to_string() })))?;
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
            Err(SongbirdError::Network(Box::new(NetworkError { message: "Connection not found".to_string() })))
    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        let message_json = serde_json::to_string(&message)
            .map_err(|e| SongbirdError::Serialization { message: e.to_string() })?;
        let mut responses = Vec::new();
        let ws_message = WsMessage::Text(message_json.clone());
        for connection in self.connections.iter() {
            let conn = connection.value();
            match conn.outgoing_tx.send(ws_message.clone()) {
                    responses.push(CommunicationResponse {
                        message_id: format!("{}_{}", message.id, conn.id),
                        success: true,
                        payload: Some(serde_json::json!({"status": "sent", "connection": conn.id})),
                        error: None,
                        timestamp: Utc::now(),
                    });
                Err(_) => {
                        success: false,
                        payload: None,
                        error: Some("Connection closed".to_string()),
        self.metrics.bytes_sent.fetch_add(
            (message_json.len() * responses.len()) as u64,
            Ordering::Relaxed,
        );
        Ok(responses)
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
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("WebSocket message listener lagged, skipped {} messages", skipped);
                        continue;
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("WebSocket message broadcast channel closed");
                        break;
        };
        Ok(Box::new(stream.boxed()))
    async fn subscribe(&self, _topic: &str) -> Result<()> {
        // WebSocket subscriptions would be handled at message level
    async fn unsubscribe(&self, _topic: &str) -> Result<()> {
    async fn connect(&self) -> Result<()> {
        self.start_server().await
    async fn disconnect(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        // Close all connections
            connection
                .value()
                .is_healthy
                .store(false, Ordering::Relaxed);
        self.connections.clear();
        // Stop server
        if let Some(handle) = self.server_handle.write().await.take() {
            handle.abort();
    async fn is_connected(&self) -> bool {
        self.running.load(Ordering::Relaxed)
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
