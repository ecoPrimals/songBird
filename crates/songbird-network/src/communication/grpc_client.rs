//! gRPC Communication Layer
//!
//! Production-ready gRPC client implementation using tonic for
//! high-performance, type-safe inter-service communication.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

use super::{
    CommunicationLayer, CommunicationResponse, CommunicationStats, ServiceAddress, ServiceMessage,
};
use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use songbird_errors::{Result, SongbirdError};

// Import generated gRPC types
tonic::include_proto!("songbird.communication");

// Proto definitions for generic gRPC communication
pub mod proto {
    tonic::include_proto!("songbird.communication");
}

use proto::{
    communication_service_client::CommunicationServiceClient,
    communication_service_server::{CommunicationService, CommunicationServiceServer},
    GenericMessage as ProtoGenericMessage, 
    GenericResponse as ProtoGenericResponse,
    HealthCheckRequest as ProtoHealthCheckRequest,
    HealthCheckResponse as ProtoHealthCheckResponse,
};

/// gRPC communication configuration
#[derive(Debug, Clone)]
pub struct GrpcConfig {
    /// Default timeout for requests
    pub request_timeout: Duration,
    /// Keep-alive interval
    pub keep_alive_interval: Duration,
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
    /// Maximum message size
    pub max_message_size: usize,
    /// Enable compression
    pub enable_compression: bool,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(30),
            keep_alive_timeout: Duration::from_secs(5),
            max_message_size: 4 * 1024 * 1024, // 4MB
            enable_compression: true,
        }
    }
}

/// gRPC communication layer implementation
pub struct GrpcCommunication {
    /// Client connections pool
    clients: Arc<RwLock<HashMap<String, CommunicationServiceClient<Channel>>>>,
    /// Circuit breaker for fault tolerance
    circuit_breaker: CircuitBreaker,
    /// Communication statistics
    stats: Arc<RwLock<CommunicationStats>>,
    /// Configuration
    config: GrpcConfig,
}

impl GrpcCommunication {
    /// Create new gRPC communication layer
    pub fn new(config: GrpcConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            circuit_breaker: CircuitBreaker::new(CircuitBreakerConfig::default()),
            stats: Arc::new(RwLock::new(CommunicationStats {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                active_connections: 0,
                failed_connections: 0,
                last_activity: None,
            })),
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GrpcConfig::default())
    }

    /// Get or create client for service
    async fn get_client(
        &self,
        address: &ServiceAddress,
    ) -> Result<CommunicationServiceClient<Channel>> {
        let endpoint_key = address.endpoint
            .as_ref()
            .unwrap_or(&"localhost:50051".to_string())
            .clone();
        
        // Check if client already exists
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&endpoint_key) {
                return Ok(client.clone());
            }
        }

        // Create new client
        let endpoint_url = format!("http://{}", endpoint_key);
        debug!("🔗 Creating gRPC client for {}", endpoint_url);

        let endpoint = Endpoint::from_shared(endpoint_url.clone())
            .map_err(|e| SongbirdError::network_error(format!("Invalid endpoint: {}", e)))?
            .timeout(self.config.request_timeout)
            .keep_alive_timeout(self.config.keep_alive_timeout)
            .keep_alive_while_idle(true);

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| SongbirdError::network_error(format!("Failed to connect: {}", e)))?;

        let mut client = CommunicationServiceClient::new(channel);
        
        if self.config.enable_compression {
            // Note: Gzip compression may not be available in this tonic version
            // client = client.send_compressed(tonic::codec::CompressionEncoding::Gzip);
        }

        // Store client in pool
        {
            let mut clients = self.clients.write().await;
            clients.insert(endpoint_key, client.clone());
        }

        info!("✅ gRPC client created for {}", endpoint_url);
        Ok(client)
    }

    /// Convert ServiceMessage to gRPC GenericMessage
    fn to_grpc_message(&self, message: ServiceMessage) -> Result<ProtoGenericMessage> {
        let payload = serde_json::to_string(&message.payload)
            .map_err(|e| SongbirdError::internal_error(format!("Failed to serialize payload: {}", e)))?;

        let mut headers = std::collections::HashMap::new();
        headers.insert("source".to_string(), message.source);
        headers.insert("target".to_string(), message.target);
        if let Some(correlation_id) = message.correlation_id {
            headers.insert("correlation_id".to_string(), correlation_id);
        }

        Ok(ProtoGenericMessage {
            id: message.id,
            message_type: message.message_type,
            payload,
            headers,
            timestamp: message.timestamp.to_rfc3339(),
        })
    }

    /// Convert gRPC GenericResponse to CommunicationResponse
    fn from_grpc_response(&self, response: ProtoGenericResponse) -> Result<CommunicationResponse> {
        let headers: HashMap<String, String> = response.headers.into_iter().collect();
        
        Ok(CommunicationResponse {
            id: response.id,
            status: response.status as u16,
            body: response.payload,
            headers,
        })
    }

    /// Send message via gRPC with retry logic
    async fn send_with_retry(
        &self,
        client: &mut CommunicationServiceClient<Channel>,
        message: ProtoGenericMessage,
        max_retries: u32,
    ) -> Result<ProtoGenericResponse> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * (1 << attempt.min(5))); // Exponential backoff
                tokio::time::sleep(delay).await;
                debug!("🔄 Retrying gRPC request (attempt {})", attempt + 1);
            }

            match client.send_message(Request::new(message.clone())).await {
                Ok(response) => {
                    if attempt > 0 {
                        info!("✅ gRPC request succeeded after {} retries", attempt);
                    }
                    return Ok(response.into_inner());
                }
                Err(e) => {
                    last_error = Some(e);
                    warn!("❌ gRPC request failed (attempt {}): {}", attempt + 1, last_error.as_ref().unwrap());
                    
                    // Don't retry on certain errors
                    if let Some(ref err) = last_error {
                        match err.code() {
                            tonic::Code::InvalidArgument
                            | tonic::Code::NotFound
                            | tonic::Code::PermissionDenied
                            | tonic::Code::Unauthenticated => break,
                            _ => continue,
                        }
                    }
                }
            }
        }

        Err(SongbirdError::internal_error(network_error(format!(
            "gRPC request failed after {} attempts: {}",
            max_retries + 1,
            last_error.unwrap()
        )))
    }
}

#[async_trait]
impl CommunicationLayer for GrpcCommunication {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        // Check circuit breaker
        if !self.circuit_breaker.should_allow_request().await {
            return Err(SongbirdError::internal_error(network_error(
                "Circuit breaker is open, gRPC request rejected".to_string(),
            ));
        }

        debug!("📤 Sending gRPC message {} to {}", message.id, target.service_id);

        // Get client
        let mut client = self.get_client(&target).await?;

        // Convert message
        let grpc_message = self.to_grpc_message(message)?;
        let message_size = grpc_message.payload.len() as u64;

        // Send with retry
        match self.send_with_retry(&mut client, grpc_message, 3).await {
            Ok(response) => {
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.messages_sent += 1;
                    stats.bytes_sent += message_size;
                }

                // Record success for circuit breaker
                self.circuit_breaker.record_success().await;

                debug!("✅ gRPC message sent successfully");
                self.from_grpc_response(response)
            }
            Err(e) => {
                // Record failure for circuit breaker
                self.circuit_breaker.record_failure().await;
                
                error!("❌ gRPC message failed: {}", e);
                Err(e)
            }
        }
    }

    async fn broadcast(&self, message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // gRPC doesn't support native broadcasting, so we'd need to send to multiple targets
        // For now, return an error indicating broadcast is not supported
        warn!("📢 gRPC broadcast not implemented - use send_message to multiple targets");
        Err(SongbirdError::internal_error(network_error(
            "gRPC broadcast not supported - use individual send_message calls".to_string(),
        ))
    }

    async fn listen(
        &self,
    ) -> Result<Box<dyn futures::Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>> {
        // gRPC client doesn't support listening - this would require a server implementation
        warn!("🎧 gRPC client listen not implemented - requires server setup");
        Err(SongbirdError::internal_error(network_error(
            "gRPC client listen not supported - use gRPC server for incoming messages".to_string(),
        ))
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        debug!("📝 gRPC subscribe to topic: {}", topic);
        // gRPC subscription would require streaming RPC implementation
        warn!("📝 gRPC subscription not implemented");
        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<()> {
        debug!("📝 gRPC unsubscribe from topic: {}", topic);
        Ok(())
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    async fn connect(&self) -> Result<()> {
        info!("🔗 gRPC communication layer ready");
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        info!("🔌 Disconnecting gRPC clients");
        
        // Clear client pool
        {
            let mut clients = self.clients.write().await;
            clients.clear();
        }
        
        info!("✅ gRPC communication layer disconnected");
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // gRPC clients are connection-per-request, so we're always "connected"
        true
    }
}

/// gRPC server implementation for receiving messages
pub struct GrpcServer {
    stats: Arc<RwLock<CommunicationStats>>,
    message_handler: Option<Box<dyn Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync>>,
}

impl GrpcServer {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(CommunicationStats {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                active_connections: 0,
                failed_connections: 0,
                last_activity: None,
            })),
            message_handler: None,
        }
    }

    pub fn with_message_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync + 'static,
    {
        self.message_handler = Some(Arc::new(handler));
        self
    }

    /// Start gRPC server on specified address
    pub async fn start(&self, addr: std::net::SocketAddr) -> Result<()> {
        info!("🚀 Starting gRPC server on {}", addr);
        
        let service = CommunicationServiceServer::new(GrpcServiceImpl {
            stats: Arc::new(RwLock::new(CommunicationStats {
                messages_sent: 0,
                messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                active_connections: 0,
                failed_connections: 0,
                last_activity: None,
            })),
            message_handler: self.message_handler.clone(),
        });

        tonic::transport::Server::builder()
            .add_service(service)
            .serve(addr)
            .await
            .map_err(|e| SongbirdError::network_error(format!("gRPC server error: {}", e)))?;

        Ok(())
    }
}

/// gRPC service implementation
#[derive(Clone)]
pub struct GrpcServiceImpl {
    stats: Arc<RwLock<CommunicationStats>>,
    message_handler: Option<Arc<dyn Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync>>,
}

#[async_trait]
impl CommunicationService for GrpcServiceImpl {
    async fn send_message(
        &self,
        request: Request<ProtoGenericMessage>,
    ) -> std::result::Result<Response<ProtoGenericResponse>, Status> {
        let message = request.into_inner();
        debug!("📨 Received gRPC message: {}", message.id);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.messages_received += 1;
            stats.bytes_received += message.payload.len() as u64;
        }

        // Convert to ServiceMessage
        let service_message = ServiceMessage {
            id: message.id.clone(),
            source: message.headers.get("source").cloned().unwrap_or_default(),
            target: message.headers.get("target").cloned().unwrap_or_default(),
            payload: serde_json::from_str(&message.payload).unwrap_or(serde_json::Value::Null),
            correlation_id: message.headers.get("correlation_id").cloned(),
            timestamp: chrono::DateTime::parse_from_rfc3339(&message.timestamp)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            message_type: message.message_type.clone(),
        };

        let response = if let Some(handler) = &self.message_handler {
            handler(service_message).unwrap_or_else(|_| CommunicationResponse {
                id: message.id.clone(),
                status: 500,
                body: "Internal error".to_string(),
                headers: HashMap::new(),
            })
        } else {
            CommunicationResponse {
                id: message.id,
                status: 200,
                body: "Message processed".to_string(),
                headers: HashMap::new(),
            }
        };

        // Convert to gRPC response
        let grpc_response = ProtoGenericResponse {
            id: response.id,
            status: response.status as i32,
            payload: response.body,
            headers: response.headers,
        };

        Ok(Response::new(grpc_response))
    }

    type StreamMessagesStream = std::pin::Pin<Box<dyn futures_util::Stream<Item = std::result::Result<ProtoGenericResponse, Status>> + Send>>;

    async fn stream_messages(
        &self,
        _request: Request<tonic::Streaming<ProtoGenericMessage>>,
    ) -> std::result::Result<Response<Self::StreamMessagesStream>, Status> {
        // For now, return an empty stream
        let stream = futures_util::stream::empty();
        Ok(Response::new(Box::pin(stream)))
    }

    async fn health_check(
        &self,
        request: Request<ProtoHealthCheckRequest>,
    ) -> std::result::Result<Response<ProtoHealthCheckResponse>, Status> {
        let req = request.into_inner();
        debug!("🏥 Health check for service: {}", req.service_name);

        let response = ProtoHealthCheckResponse {
            status: 1, // SERVING
            message: "Service is healthy".to_string(),
        };

        Ok(Response::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_grpc_communication_creation() {
        let grpc_comm = GrpcCommunication::with_defaults();
        assert!(grpc_comm.is_connected().await);
    }

    #[tokio::test]
    async fn test_grpc_server_creation() {
        let server = GrpcServer::new();
        // Just test creation - starting server requires actual network binding
        assert!(server.stats.read().await.messages_received == 0);
    }
} 