use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::Arc;
use futures_util::Stream;
use crate::{
    communication::{
        CommunicationLayer, CommunicationResponse, CommunicationStats, HttpCommunication,
        InMemoryCommunication, ServiceAddress, ServiceMessage, WebSocketCommunication,
    },
    config::constants::network,
    errors::Result,
    traits::service::ServiceInfo,
};
use std::env;
use tracing;

/// Multi-protocol communication router that automatically selects the best protocol for each service
#[derive(Clone)]
pub struct ProtocolRouter {
    http_layer: Arc<HttpCommunication>,
    websocket_layer: Arc<WebSocketCommunication>,
    in_memory_layer: Arc<InMemoryCommunication>,
    protocol_preferences: Arc<parking_lot::RwLock<HashMap<String, CommunicationProtocol>>>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommunicationProtocol {
    Http,
    WebSocket,
    InMemory,
    Auto, // Let the router decide
impl ProtocolRouter {
    pub fn new() -> Self {
        // Use environment variables with fallback to constants
        let websocket_host = env::var("SONGBIRD_WEBSOCKET_HOST")
            .unwrap_or_else(|_| network::DEFAULT_BIND_ADDRESS.to_string());
        let websocket_port = env::var("SONGBIRD_WEBSOCKET_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(0);
            
        Self {
            http_layer: Arc::new(HttpCommunication::new(
                crate::config::SongbirdConfig::default()
                    .network.default_endpoint()
                    .map(|addr| format!("http://{}", addr))
                    .unwrap_or_else(|_| { let env_config = crate::config::environment::EnvironmentConfig::default(); format!("http://{}:{}", env_config.bind_address, env_config.bind_port) }.to_string())
            ).unwrap_or_else(|e| {
                tracing::error!("Failed to create HTTP layer: {}", e);
                tracing::error!("Critical: Cannot create any HTTP communication layer");
                std::process::exit(1);
            })),
            websocket_layer: Arc::new(WebSocketCommunication::new(websocket_host, websocket_port)),
            in_memory_layer: Arc::new(InMemoryCommunication::new()),
            protocol_preferences: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
    /// Create with custom configuration
    pub fn with_config(
        http_base_url: Option<String>,
        websocket_host: Option<String>,
        websocket_port: Option<u16>,
    ) -> Self {
        let host = websocket_host
            .or_else(|| env::var("SONGBIRD_WEBSOCKET_HOST").ok())
            .unwrap_or_else(|| network::DEFAULT_BIND_ADDRESS.to_string());
        let port = websocket_port
            .or_else(|| env::var("SONGBIRD_WEBSOCKET_PORT").ok().and_then(|p| p.parse().ok()))
            http_layer: Arc::new(HttpCommunication::new(
                http_base_url.unwrap_or_else(|| "".to_string())
            ).unwrap_or_else(|e| {
            websocket_layer: Arc::new(WebSocketCommunication::new(host, port)),
    /// Register a service's communication preferences
    pub fn register_service_protocol(&self, service_id: &str, service_info: &ServiceInfo) {
        let protocol = self.detect_protocol_from_service(service_info);
        self.protocol_preferences.write().insert(service_id.to_string(), protocol);
        tracing::info!("Registered service {} with protocol {:?}", service_id, protocol);
    /// Unregister a service's communication preferences
    pub fn unregister_service_protocol(&self, service_id: &str) {
        self.protocol_preferences.write().remove(service_id);
        tracing::info!("Unregistered service {} protocol", service_id);
    /// Auto-detect the best communication protocol for a service based on its info
    fn detect_protocol_from_service(&self, service_info: &ServiceInfo) -> CommunicationProtocol {
        // Check service type
        if service_info.service_type == "test" || service_info.service_type == "mock" {
            return CommunicationProtocol::InMemory;
        // Check capabilities
        if service_info.capabilities.contains(&"websocket".to_string()) {
            return CommunicationProtocol::WebSocket;
        // Check endpoints for protocol hints
        for endpoint in &service_info.endpoints {
            if endpoint.path.starts_with("http://") || endpoint.path.starts_with("https://") {
                return CommunicationProtocol::Http;
            }
            if endpoint.path.starts_with("ws://") || endpoint.path.starts_with("wss://") {
                return CommunicationProtocol::WebSocket;
        // Check tags
        if let Some(protocol_tag) = service_info.tags.get("protocol") {
            match protocol_tag.as_str() {
                "http" | "https" => return CommunicationProtocol::Http,
                "websocket" | "ws" => return CommunicationProtocol::WebSocket,
                "memory" | "test" => return CommunicationProtocol::InMemory,
                _ => {}
        // Default to HTTP for services with HTTP-like endpoints
        if service_info.endpoints.iter().any(|e| 
            e.method.to_uppercase() == "GET" || 
            e.method.to_uppercase() == "POST" ||
            e.path.contains("/")
        ) {
            return CommunicationProtocol::Http;
        // Ultimate fallback
        CommunicationProtocol::InMemory
    /// Get the appropriate communication layer for a service
    fn get_communication_layer(&self, service_address: &ServiceAddress) -> Arc<dyn CommunicationLayer> {
        let protocol = self.protocol_preferences
            .read()
            .get(&service_address.service_id)
            .cloned()
            .unwrap_or(CommunicationProtocol::Auto);
        match protocol {
            CommunicationProtocol::Http => Arc::clone(&self.http_layer) as Arc<dyn CommunicationLayer>,
            CommunicationProtocol::WebSocket => Arc::clone(&self.websocket_layer) as Arc<dyn CommunicationLayer>,
            CommunicationProtocol::InMemory => Arc::clone(&self.in_memory_layer) as Arc<dyn CommunicationLayer>,
            CommunicationProtocol::Auto => {
                // Try to detect from the endpoint
                if let Some(endpoint) = &service_address.endpoint {
                    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                        Arc::clone(&self.http_layer) as Arc<dyn CommunicationLayer>
                    } else if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
                        Arc::clone(&self.websocket_layer) as Arc<dyn CommunicationLayer>
                    } else {
                        Arc::clone(&self.in_memory_layer) as Arc<dyn CommunicationLayer>
                    }
                } else {
                    // Default to in-memory for testing
                    Arc::clone(&self.in_memory_layer) as Arc<dyn CommunicationLayer>
                }
    /// Get statistics for all communication layers
    pub async fn get_all_stats(&self) -> Result<HashMap<String, CommunicationStats>> {
        let mut stats = HashMap::new();
        
        stats.insert("http".to_string(), self.http_layer.get_stats().await?);
        stats.insert("websocket".to_string(), self.websocket_layer.get_stats().await?);
        stats.insert("in_memory".to_string(), self.in_memory_layer.get_stats().await?);
        Ok(stats)
    /// Start all communication layers
    pub async fn start_all(&self) -> Result<()> {
        // Start WebSocket server if needed
        if let Err(e) = self.websocket_layer.connect().await {
            tracing::warn!("Failed to start WebSocket layer: {}", e);
        // HTTP layer is always ready (stateless)
        if let Err(e) = self.http_layer.connect().await {
            tracing::warn!("Failed to initialize HTTP layer: {}", e);
        // In-memory layer is always ready
        self.in_memory_layer.connect().await?;
        Ok(())
    /// Stop all communication layers
    pub async fn stop_all(&self) -> Result<()> {
        // Stop WebSocket server
        if let Err(e) = self.websocket_layer.disconnect().await {
            tracing::warn!("Failed to stop WebSocket layer: {}", e);
        // HTTP layer cleanup
        if let Err(e) = self.http_layer.disconnect().await {
            tracing::warn!("Failed to cleanup HTTP layer: {}", e);
        // In-memory cleanup
        self.in_memory_layer.disconnect().await?;
#[async_trait]
impl CommunicationLayer for ProtocolRouter {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        let layer = self.get_communication_layer(&target);
        tracing::debug!(
            "Routing message {} to service {} via {:?}", 
            message.id, 
            target.service_id,
            self.protocol_preferences.read().get(&target.service_id)
        );
        layer.send_message(target, message).await
    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // Broadcast to all active communication layers
        let mut all_responses = Vec::new();
        // Broadcast via WebSocket
        match self.websocket_layer.broadcast(message.clone()).await {
            Ok(mut responses) => all_responses.append(&mut responses),
            Err(e) => tracing::warn!("WebSocket broadcast failed: {}", e),
        // HTTP doesn't support broadcast, skip
        // In-memory broadcast
        match self.in_memory_layer.broadcast(message).await {
            Err(e) => tracing::warn!("In-memory broadcast failed: {}", e),
        Ok(all_responses)
    async fn listen(&self) -> Result<Box<dyn Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>> {
        // For now, return the WebSocket listener (most active)
        self.websocket_layer.listen().await
    async fn subscribe(&self, topic: &str) -> Result<()> {
        // Subscribe on all layers that support it
        let _ = self.websocket_layer.subscribe(topic).await;
        let _ = self.in_memory_layer.subscribe(topic).await;
        // HTTP doesn't support pub/sub
    async fn unsubscribe(&self, topic: &str) -> Result<()> {
        // Unsubscribe from all layers
        let _ = self.websocket_layer.unsubscribe(topic).await;
        let _ = self.in_memory_layer.unsubscribe(topic).await;
    async fn connect(&self) -> Result<()> {
        self.start_all().await
    async fn disconnect(&self) -> Result<()> {
        self.stop_all().await
    async fn is_connected(&self) -> bool {
        // Consider connected if any layer is connected
        self.http_layer.is_connected().await ||
        self.websocket_layer.is_connected().await ||
        self.in_memory_layer.is_connected().await
    async fn get_stats(&self) -> Result<CommunicationStats> {
        // Aggregate stats from all layers
        let all_stats = self.get_all_stats().await?;
        let mut aggregated = CommunicationStats::default();
        for stats in all_stats.values() {
            aggregated.messages_sent += stats.messages_sent;
            aggregated.messages_received += stats.messages_received;
            aggregated.bytes_sent += stats.bytes_sent;
            aggregated.bytes_received += stats.bytes_received;
            aggregated.active_connections += stats.active_connections;
            aggregated.failed_connections += stats.failed_connections;
        aggregated.last_activity = Some(chrono::Utc::now());
        Ok(aggregated)
impl Default for ProtocolRouter {
    fn default() -> Self {
        // Build configurable endpoint instead of hardcoded localhost:8080
        let env_config = crate::config::environment::EnvironmentConfig::default();
        let http_endpoint = format!("http://{}:{}", env_config.bind_address, env_config.bind_port);
        
        let http_comm = HttpCommunication::new(http_endpoint).unwrap_or_else(|_| {
            tracing::error!("Critical: Cannot create any HTTP communication layer");
            std::process::exit(1);
        });
        
        Self {
            http_layer: Arc::new(http_comm),
            websocket_layer: Arc::new(WebSocketCommunication::new(
                env::var("SONGBIRD_WEBSOCKET_HOST").unwrap_or_else(|| network::DEFAULT_BIND_ADDRESS.to_string()),
                env::var("SONGBIRD_WEBSOCKET_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(0)
            )),
            in_memory_layer: Arc::new(InMemoryCommunication::new()),
            protocol_preferences: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
} 
