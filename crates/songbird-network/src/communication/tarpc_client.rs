//! # 🌐 Canonical TARPC Communication Client
//!
//! **MODERNIZED CANONICAL IMPLEMENTATION**
//!
//! Unified TARPC client with canonical error handling patterns.

use crate::communication::{
    CommunicationLayer, CommunicationResponse, CommunicationStats, ServiceAddress, ServiceMessage,
};
use songbird_errors::{SongbirdResult as Result, SongbirdError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Canonical tarpc service trait for Songbird communication
#[tarpc::service]
pub trait SongbirdCommunication {
    /// Send a message and get response
    async fn send_message(message: ServiceMessage) -> CommunicationResponse;

    /// Broadcast a message to multiple targets
    async fn broadcast_message(
        message: ServiceMessage,
        targets: Vec<String>,
    ) -> Vec<CommunicationResponse>;

    /// Health check for the service
    async fn health_check(service_name: String) -> HealthCheckResponse;
}

/// Canonical health check response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Canonical TARPC communication client
pub struct TarpcCommunicationClient {
    #[allow(dead_code)] // Part of API design for configuration access
    config: TarpcConfig,
    stats: Arc<RwLock<CommunicationStats>>,
}

/// TARPC configuration
#[derive(Debug, Clone)]
pub struct TarpcConfig {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
}

impl Default for TarpcConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl TarpcCommunicationClient {
    /// Create a new canonical TARPC client
    pub fn new(config: TarpcConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(CommunicationStats::default())),
        }
    }

    /// Get communication statistics
    pub async fn get_stats(&self) -> CommunicationStats {
        self.stats.read().await.clone()
    }

    /// Connect to a tarpc service using canonical address parsing
    async fn connect_to_service(
        &self,
        address: &ServiceAddress,
    ) -> Result<SongbirdCommunicationClient> {
        // Parse endpoint URL to extract host and port
        let addr = if let Some(endpoint) = &address.endpoint {
            // Parse URL-like endpoint: "http://host:port" or "host:port"
            let addr_part = endpoint
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .trim_start_matches("tcp://");

            // If no port specified, use default
            if addr_part.contains(':') {
                addr_part.to_string()
            } else {
                format!("{addr_part}:8080") // Default port
            }
        } else {
            return Err(SongbirdError::internal_error(network_error(
                "No endpoint specified in ServiceAddress",
            ));
        };

        debug!("🔗 Connecting to canonical tarpc service at {}", addr);

        let transport = tarpc::serde_transport::tcp::connect(addr.clone(), || {
            tarpc::tokio_serde::formats::Json::default()
        })
        .await
        .map_err(|e| SongbirdError::network(format!("Failed to connect to {addr}: {e}")))?;

        let client =
            SongbirdCommunicationClient::new(tarpc::client::Config::default(), transport).spawn();

        info!("✅ Connected to canonical tarpc service at {}", addr);
        Ok(client)
    }

    /// Send message with retry logic
    async fn send_with_retry(
        &self,
        client: &SongbirdCommunicationClient,
        message: ServiceMessage,
        max_retries: u32,
    ) -> Result<CommunicationResponse> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(100 * (1 << attempt.min(5))); // Exponential backoff
                tokio::time::sleep(backoff).await;
                debug!(
                    "🔄 Retrying tarpc request (attempt {}/{})",
                    attempt + 1,
                    max_retries + 1
                );
            }

            let ctx = tarpc::context::current();
            match client.send_message(ctx, message.clone()).await {
                Ok(response) => {
                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.messages_sent += 1;
                        stats.last_activity = Some(chrono::Utc::now());
                    }
                    return Ok(response);
                }
                Err(e) => {
                    warn!("⚠️ tarpc request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        // Update failed stats
        {
            let mut stats = self.stats.write().await;
            stats.failed_connections += 1;
        }

        Err(SongbirdError::internal_error(network_error(format!(
            "Failed to send message after {} attempts: {:?}",
            max_retries + 1,
            last_error
        )))
    }
}

#[async_trait::async_trait]
impl CommunicationLayer for TarpcCommunicationClient {
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        debug!(
            "📤 Sending canonical tarpc message to {}: {}",
            target.service_id, message.id
        );

        let client = self.connect_to_service(&target).await?;
        self.send_with_retry(&client, message, 3).await
    }

    async fn broadcast(&self, _message: ServiceMessage) -> Result<Vec<CommunicationResponse>> {
        // For now, return empty responses since we need target addresses for real broadcast
        // In a real implementation, this would use service discovery to find all targets
        Ok(vec![])
    }

    async fn listen(
        &self,
    ) -> Result<Box<dyn futures::Stream<Item = (ServiceAddress, ServiceMessage)> + Send + Unpin>>
    {
        // TARPC is typically request-response, not streaming
        // Return an empty stream for now
        use futures::stream;
        Ok(Box::new(stream::empty()))
    }

    async fn subscribe(&self, _topic: &str) -> Result<()> {
        // TARPC subscription would be implemented here
        Ok(())
    }

    async fn unsubscribe(&self, _topic: &str) -> Result<()> {
        // TARPC unsubscription would be implemented here
        Ok(())
    }

    async fn get_stats(&self) -> Result<CommunicationStats> {
        Ok(self.get_stats().await)
    }

    async fn connect(&self) -> Result<()> {
        // TARPC connection logic would be implemented here
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        // TARPC disconnection logic would be implemented here
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        // Check TARPC connection status
        true // Placeholder
    }
}

/// Canonical tarpc service implementation
#[derive(Clone)]
pub struct TarpcServiceImpl {
    #[allow(dead_code)] // Used for metrics collection
    stats: Arc<RwLock<CommunicationStats>>,
    #[allow(dead_code)] // Used for message processing
    message_handler:
        Option<Arc<dyn Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync>>,
}

impl Default for TarpcServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TarpcServiceImpl {
    /// Create new canonical service implementation
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(CommunicationStats::default())),
            message_handler: None,
        }
    }

    /// Create with canonical message handler
    pub fn with_handler<F>(handler: F) -> Self
    where
        F: Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync + 'static,
    {
        Self {
            stats: Arc::new(RwLock::new(CommunicationStats::default())),
            message_handler: Some(Arc::new(handler)),
        }
    }
}

// Temporarily disable tarpc server implementation during canonical modernization
// This will be re-enabled after dependency issues are resolved
/*
#[tarpc::server]
impl SongbirdCommunication for TarpcServiceImpl {
    async fn send_message(
        self,
        _context: tarpc::context::Context,
        message: ServiceMessage,
    ) -> CommunicationResponse {
        debug!("📨 Received canonical tarpc message: {}", message.id);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.messages_received += 1;
            stats.last_activity = Some(chrono::Utc::now());
        }

        // Handle message with canonical response patterns
        if let Some(handler) = &self.message_handler {
            handler(message.clone()).unwrap_or_else(|_| {
                CommunicationResponse::error(
                    message.id.clone(),
                    "Message handling failed".to_string(),
                )
            })
        } else {
            CommunicationResponse::success(
                message.id,
                serde_json::Value::String("Message processed".to_string()),
            )
        }
    }

    async fn broadcast_message(
        self,
        _context: tarpc::context::Context,
        message: ServiceMessage,
        targets: Vec<String>,
    ) -> Vec<CommunicationResponse> {
        debug!("📡 Broadcasting canonical tarpc message to {} targets", targets.len());

        let mut responses = Vec::new();
        for target in targets {
            let response = CommunicationResponse::success(
                format!("{}-{}", message.id, target),
                serde_json::Value::String(format!("Broadcast to {}", target)),
            );
            responses.push(response);
        }

        responses
    }

    async fn health_check(
        self,
        _context: tarpc::context::Context,
        service_name: String,
    ) -> HealthCheckResponse {
        debug!("🔍 Canonical health check for service: {}", service_name);

        HealthCheckResponse {
            status: HealthStatus::Healthy,
            message: "Service is healthy".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Start a canonical tarpc server
pub async fn start_tarpc_server(bind_addr: &str, service_impl: TarpcServiceImpl) -> Result<()> {
    info!("🚀 Starting canonical tarpc server on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .map_err(|e| {
            SongbirdError::network(&format!("Failed to bind to {}: {}", bind_addr, e))
        })?;

    info!("✅ Canonical tarpc server listening on {}", bind_addr);

    loop {
        let (stream, addr) = listener.accept().await.map_err(|e| {
            SongbirdError::network(&format!("Failed to accept connection: {}", e))
        })?;

        debug!("🔗 New canonical tarpc connection from {}", addr);

        let transport = tarpc::serde_transport::new(
            stream,
            tarpc::tokio_serde::formats::Json::default(),
        );

        let server = tarpc::server::BaseChannel::with_defaults(transport);
        let service = service_impl.clone();

        tokio::spawn(async move {
            if let Err(e) = server.execute(service.serve()).await {
                warn!("❌ Tarpc server error: {}", e);
            }
        });
    }
}
*/
