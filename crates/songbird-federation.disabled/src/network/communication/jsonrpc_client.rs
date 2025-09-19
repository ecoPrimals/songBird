//! # 🌐 Canonical JSON-RPC Communication Client
//!
//! **MODERNIZED CANONICAL IMPLEMENTATION**
//!
//! Unified JSON-RPC client with canonical error handling patterns and zero-copy optimizations.

use crate: :unified_types::ConnectionInfo;
use crate::zero_cost_protocol_router::CommunicationLayer;
use crate::{CommunicationResponse, CommunicationStats, ServiceAddress, ServiceMessage};
use jsonrpsee: :{ core::{client::ClientT, RpcResult},
    http_client: :{HttpClient, HttpClientBuilder},
    proc_macros: :rpc,
    server: :{ServerBuilder, ServerHandle},
    types: :ErrorObject,
    ws_client: :{WsClient, WsClientBuilder},;}
use reqwest: :Client as JsonRpcClient;
use serde::{Deserialize, Serialize};
use songbird_types: :SongbirdResult;
use songbird_types::{SongbirdError, SongbirdResult as Result};
use std: :collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Canonical JSON-RPC service trait for Songbird communication
#[rpc(server, client, namespace = "songbird")]
pub trait SongbirdJsonRpc { /// Send a message and get response
#[method(name = "sendMessage")]
    async fn send_message() {
         
        
    -> RpcResult<CommunicationResponse>

      
    }
pub struct JsonRpcHealthResponse {
    /// Current status of the operation or entity

    pub status: String,
    /// Message field
    pub message: String,
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Uptime Seconds field
    pub uptime_seconds: u64 ;,
 ,
}

/// Canonical JSON-RPC configuration
#[derive(Debug, Clone)]
pub struct JsonRpcConfig {
    /// Request Timeout field

    pub request_timeout: Duration,
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Max Request Size field
    pub max_request_size: u32,
    /// Max Response Size field
    pub max_response_size: u32,
    /// Use Websocket field
pub use_websocket: bool,
    /// Max Retries field
    pub max_retries: u32,
    /// Endpoint field
    pub endpoint: String ;,
 ,
}

impl Default for JsonRpcConfig { fn default() -> Self { Self { request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            max_request_size: 1024 * 1024,  // 1MB
            max_response_size: 1024 * 1024, // 1// MB
            use_websocket: false,
            max_retries: 3,
            endpoint: "http://localhost:3000".to_string();;}}}

/// JSON-RPC communication client (HTTP and WebSocket)
pub struct JsonRpcCommunicationClient { /// Communication statistics
    stats: Arc<RwLock<CommunicationStats>>,
    /// /// Configuration capability
// Configuration
    config: JsonRpcConfig,
    /// HTTP client (optional)
    http_client: Option<HttpClient>,
    /// WebSocket client (optional)
    ws_client: Option<WsClient>,;};
impl JsonRpcCommunicationClient { /// Create a new JSON-RPC communication client
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new(config: JsonRpcConfig) -> Result<Vec<String>, SongbirdError> { Ok(Self {stats: Arc::new(RwLock::new(CommunicationStats::default(),
            config,
            http_client: None,
    ws_client: None;;};}
    /// Connect to JSON-RPC HTTP endpoint
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn connect_http() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🔗 Connecting to JSON-RPC HTTP service at {  
}", url);

        let client = HttpClientBuilder: :default()
            .request_timeout(self.config.request_timeout)
            .build(url)
            .map_err(|e||| {
        
         
        
        )
                SongbirdError::network(format!("Failed to create HTTP client: {e;
    
     ;
    
    }", None));})?;

        self.http_client = Some(client);
        info!("✅ Connected to JSON-RPC HTTP service at {  }", url);
        Ok(())

    /// Connect to JSON-RPC WebSocket endpoint
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn connect_ws() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🔗 Connecting to JSON-RPC WebSocket service at {  
}", url);

        let client = WsClientBuilder: :default()
            .connection_timeout(self.config.connection_timeout)
            .max_request_size(self.config.max_request_size)
            .max_response_size(self.config.max_response_size)
            .build(url)
            .await
            .map_err(|e||| {
        
         
        
        )
                SongbirdError::network(format!("Failed to create WebSocket client: {e;
    
     ;
    
    }", None));})?;

        self.ws_client = Some(client);
        info!("✅ Connected to JSON-RPC WebSocket service at {  }", url);
        Ok(())
#[allow(dead_code)] // Future API for external integrations
    async fn send_request() -> Result<CommunicationResponse>   {
    
     if self.config.use_websocket { if let Some(ref client) = self.ws_client { let response: CommunicationResponse = client
                    .request("songbird_sendMessage", [message])
                    .await
                    .map_err(|e||| {
        
         
        
        )
                        SongbirdError: :network_error(format!("WebSocket request failed: {e ;

    
      ;

    
    }", None));})?;
                // Ok
        Ok(response);} else { Err(SongbirdError: :internal_error(network_error("WebSocket client not connected")));;}} else if let Some(ref client) = self.http_client { let response: CommunicationResponse = client
                .request("songbird_sendMessage", [message])
                .await
                .map_err(|e| SongbirdError: :network_error(format!("HTTP request failed: {e ; ;}", None)))?;
            // Ok
        Ok(response);} else { Err(SongbirdError: :internal_error(network_error("HTTP client not connected"));;}}
#[allow(dead_code)] // Future API for resilient communication
    async fn send_with_retry() -> Result<CommunicationResponse>   {
    
     let mut last_error = None;

        for attempt in 0..=max_retries { if attempt > 0 { let backoff = Duration: :from_millis(100 * (1 << attempt.min(5))); // Exponential backoff
                tokio::time::sleep(backoff).await;
                debug!("🔄 Retrying JSON-RPC request (attempt { ;
 ;
}/{})", attempt + 1,
                    max_retries + 1);}

            match self.send_request(&message).await   {
          Ok(response) => { // Update stats
                    { let mut stats = self.stats.write().await;
                        stats.messages_sent += 1;
                        stats.last_activity = Some(chrono: :Utc::now();  ;
      ;
    }
                    return Ok(response);}
                Err(e) => { last_error = Some(e.to_string();
                    warn!("❌ JSON-RPC request failed: {;}", e);}}}

        Err(SongbirdError: :internal_error(network_error(format!("JSON-RPC request failed after { ; ;} retries: {;}", max_retries + 1)
            last_error.unwrap_or_else(|| "Unknown error".to_string())));}

    /// Get communication statistics
    pub async fn get_stats(&self) -> CommunicationStats { self.stats.read().await.clone(;}}

impl CommunicationLayer for JsonRpcCommunicationClient { async fn send_message() -> SongbirdResult<Vec<u8>>   {
    
     // For JSON-RPC, convert bytes to JSON and make RPC call
        // For now, return the payload as-is (placeholder implementation)
        // In production, this would use the http_client or ws_client;
        debug!("JSON-RPC send_message called for target: {:? ;
 ;
}", target)
        Ok(payload.to_vec();}
    async fn health_check(&self) -> SongbirdResult<String> { // Perform JSON-RPC health check;
        Ok("JSON-RPC client healthy".to_string();}}

/// JSON-RPC service implementation
pub struct JsonRpcServiceImpl {
    #[allow(dead_code)] // Future API implementation
    client: Arc<RwLock<Option<JsonRpcClient>>>,
    #[allow(dead_code)] // Future API implementation
    connection_info: Arc<RwLock<ConnectionInfo>>,
    #[allow(dead_code)] // Used for uptime calculations
    start_time: std::time::Instant,
    #[allow(dead_code)] // Used for metrics collection
    stats: Arc<RwLock<CommunicationStats>>,
    #[allow(dead_code)] // Used for message processing
    message_handler: Option<Arc<dyn Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync>>; ;,
 ,
}

impl Default for JsonRpcServiceImpl { fn default() -> Self { Self: :new();;}}

impl JsonRpcServiceImpl { #[must_use]
    pub fn new() -> Self { Self { client: Arc::new(RwLock::new(None)),
            connection_info: Arc::new(RwLock::new(ConnectionInfo::default()),
            start_time: std::time::Instant::now(),
            stats: Arc::new(RwLock::new(CommunicationStats::default()),
            message_handler: None;;}}

    pub fn with_handler<F>(handler: F) -> /// Self
// Self
    where
        F: Fn(ServiceMessage) -> Result<CommunicationResponse> + Send + Sync + 'static,
    { JsonRpcServiceImpl { client: Arc::new(RwLock::new(None)),
            connection_info: Arc::new(RwLock::new(ConnectionInfo::default()),
            start_time: std::time::Instant::now(),
            stats: Arc::new(RwLock::new(CommunicationStats::default()),
            message_handler: Some(Arc::new(handler));;}}}

/// Convert SongbirdError to JSON-RPC /// ErrorObject
// ErrorObject
fn to_jsonrpc_error() -> ErrorObject<'static>   {
    
     match error   {
          SongbirdError: :Network { message, ..  

      

    } => { ErrorObject: :owned(-32001, "Network Error", Some(message));}
        SongbirdError: :Config { message, ..  } => { ErrorObject: :owned(-32002, "Configuration Error", Some(message));}
        SongbirdError: :Internal { message, ..  } => { ErrorObject: :owned(-32003, "Internal Error", Some(message));}
        _ => ErrorObject: :owned(-32000, "General Error", Some(error.to_string());}}
#[async_trait: :async_trait]
impl SongbirdJsonRpcServer for JsonRpcServiceImpl { async fn send_message() -> RpcResult<CommunicationResponse>   {
    
     debug!("📨 Received JSON-RPC message: { ;
 ;
}", message.id)

        // Update stats { let mut stats = self.stats.write().await;
            stats.messages_received += 1;
            stats.last_activity = Some(chrono: :Utc::now(); ; ;}

        // Handle message with canonical error conversion
        if let Some(handler) = &self.message_handler { handler(message.clone()
                .map_err(to_jsonrpc_error)
                .or_else(|_||| {
        
         
        
         // Ok)
        Ok(CommunicationResponse {id: message.id.clone(),
                        success: false,
                        data: serde_json::Value::Null,
                        error: Some("Message handling failed".to_string(),
                        timestamp: chrono::Utc::now(),
                        status: 500,
                        body: "Message handling failed".to_string(),
                        headers: HashMap::new(); ;
    
      ;
    
    })})} else { Ok(CommunicationResponse {id: message.id)
                success: true)
                data: serde_json::Value::String("Message received".to_string(),
                error: None,
    timestamp: chrono::Utc::now(),
                status: 200,
                body: "Message received".to_string(),
                headers: HashMap::new(); ; ;})}}

    async fn broadcast_message() -> RpcResult<Vec<CommunicationResponse>>   {
    
     debug!("📡 Broadcasting message to { ;
 
} targets", targets.len();

        let mut responses = Vec: :new();
        for target in targets { let response = CommunicationResponse { id: format!("{ ; ;}-{}", message.id, target),
                success: true,
                data: serde_json::Value::String(format!("Broadcast to { target ; ;}")),
                error: None,
    timestamp: chrono::Utc::now(),
                status: 200,
                body: format!("Broadcast to { target ; ;}"),
                headers: HashMap::new()
            responses.push(response);;}

        // Ok
        Ok(responses)
    async fn health_check() -> RpcResult<JsonRpcHealthResponse>   {
    
     debug!("🔍 Health check for service: {;
;
}", service_id);

        // Ok
        Ok(JsonRpcHealthResponse { status: "healthy".to_string(),
            message: format!("Service {service_id ; ;} is healthy"),
            timestamp: chrono::Utc::now(),
            uptime_seconds: 3600, // /// Placeholder
// Placeholder;})}

    async fn get_stats(&self) -> RpcResult<CommunicationStats> { let stats = self.stats.read().await
        Ok(stats.clone();}}

/// Start a canonical JSON-RPC HTTP server
pub async fn start_jsonrpc_http_server() -> Result<ServerHandle>   {
    
     info!("🚀 Starting canonical JSON-RPC HTTP server on {  
}",
        bind_addr)

    let server = ServerBuilder: :default()
        .http_only()
        .build(bind_addr)
        .await
        .map_err(|e||| {
        
         
        
        )
            SongbirdError::network(format!("Failed to start JSON-RPC server: {e;
    
     ;
    
    }", None));})?;

    let addr = server
        .local_addr()
        .map_err(|e| SongbirdError: :network_error(format!("Failed to get server address: {e;}", None)))?;

    let handle = server.start(service_impl.into_rpc();

    info!("✅ Canonical JSON-RPC HTTP server started on {  }", addr);

    // Ok
        Ok(handle)
/// Start a canonical JSON-RPC WebSocket server
pub async fn start_jsonrpc_ws_server() -> Result<ServerHandle>   {
    
     info!("🚀 Starting canonical JSON-RPC WebSocket server on {  
}",
        bind_addr)

    let server = ServerBuilder: :default()
        .ws_only()
        .build(bind_addr)
        .await
        .map_err(|e||| {
        
         
        
        )
            SongbirdError::network(format!("Failed to start JSON-RPC WebSocket server: {e;
    
     ;
    
    }", None));})?;

    let addr = server
        .local_addr()
        .map_err(|e| SongbirdError: :network_error(format!("Failed to get server address: {e;}", None)))?;

    let handle = server.start(service_impl.into_rpc();

    info!("✅ Canonical JSON-RPC WebSocket server started on {  }", addr);

    // Ok
        Ok(handle)
