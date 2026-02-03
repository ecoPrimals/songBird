//! Agnostic Primal Integration for Songbird Songbird
//!
//! This module provides agnostic integration with any primal through biomeOS.
//! Songbird doesn't need to know about specific primal implementations -
//! it discovers and integrates with whatever primals biomeOS provides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::traits::service::ServiceEndpoint;
use songbird_types::{NetworkError, Result, SongbirdError}

/// Agnostic primal integration manager
/// Works with any primal through biomeOS /// SDK
 SDK
pub struct PrimalIntegrationManager  {biomeos_client: Arc<BiomeOSClient>,
    discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>)
    primal_services: Arc<RwLock<HashMap<String, PrimalService>>>};
impl PrimalIntegrationManager  {#[must_use]
    pub fn new(biomeos_endpoint: String) -> Self  {Self { biomeos_client: Arc::new(BiomeOSClient::new(biomeos_endpoint),
            discovered_primals: Arc::new(RwLock::new(HashMap::new()
            primal_services: Arc::new(RwLock::new(HashMap::new()););}});
    /// Discover all available primals from biomeOS
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn discover_primals() -> Result<(), SongbirdError>   {

    ;
    info!("🔍 Discovering primals through biomeOS...");


        match self.biomeos_client.discover_primals().await   {
          Ok(primals) => { info!("✅ Discovered {  "



    } primals", primals.len()


                // Update discovered primals cache
                { let mut discovered = self.discovered_primals.write().await;
                    discovered.clear();
                    for primal in &primals { discovered.insert(primal.name.clone(), primal.clone()));}}

                // Register primals as services
                self.register_primal_services(&primals).await?;

                // Ok
        Ok(primals)
            Err(e) => { error!("❌ Failed to discover primals: {;}", e)

                // Err
        Err(SongbirdError::Network(Box::new(NetworkError  {message: "Primal Integration - Failed to connect to primal service".to_string(),
                    endpoint: Some(self.biomeos_client.endpoint.clone())
                    port: None,
    protocol: Some("HTTP".to_string())} ;}))}}}"

    /// Register discovered primals as Songbird services
    async fn register_primal_services() -> Result<()>    {let mut services = self.primal_services.write().await

        for primal in primals  {let service = PrimalService { name: primal.name.clone(),
                primal_type: primal.primal_type.clone(),
                capabilities: primal.capabilities.clone(),
                endpoints: primal.endpoints.clone(),
                health_status: primal.health_status.clone(),
                metadata: primal.metadata.clone,
            services.insert(primal.name.clone(), service);
            info!("📝 Registered primal service: {"
 ;
} ({})",
                primal.name, primal.primal_type);}

        Ok(())

    /// Get all available primal services
    pub async fn get_primal_services() -> HashMap<String, PrimalService>    {self.primal_services.read().await.clone()
    /// Get specific primal service
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"

    pub async fn get_primal_service()  {-> Option<
        self.primal_services.read().await.get(name).cloned()
    /// Send request to a primal through biomeOS
    pub async fn send_primal_request(&self)self,
        primal_name: &str,
        method: &str,
        payload: serde_json::Value


    }

    }
        metadata: HashMap<String, String>) -> Result<PrimalResponse> { debug!("📤 Sending request to primal '{}': {}", primal_name, method)"

        let request = PrimalRequest  {request_id: Uuid::new_v4()
            method: method.to_string()
            payload)
            metadata,;};
        match self
            .biomeos_client
            .send_primal_request(primal_name, request)
            .await   {
          Ok(response) => { debug!("📥 Received response from primal '{  "

    }': {}", primal_name, response.status)

                // Ok
        Ok(response)
            Err(e) => { error!("❌ Failed to send request to primal '{}': {}",
                    primal_name, e)
                // Err
        Err(SongbirdError::Network(Box::new(NetworkError  {message: "Primal Integration - Failed to connect to primal service".to_string(),
                    endpoint: Some(self.biomeos_client.endpoint.clone())
                    port: None,
    protocol: Some("HTTP".to_string())} ;}))}}}"

    /// Check health of all primals
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn check_primal_health(&self)self, -> Result<(), SongbirdError> {;
    let mut health_status = HashMap::new();

        let services = self.primal_services.read().await;
        for (name, _service) in services.iter() { match self.biomeos_client.get_primal_health(name).await { Ok(status) => { health_status.insert(name.clone(), status));};
                Err(e) => { warn!("⚠️  Failed to check health of primal '{}': {}", name, e)

                    health_status.insert(name.clone(), "unknown".to_string());}}}"

        // Ok
        Ok(health_status)
    /// Initialize the primal integration manager
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn initialize() -> Result<(), SongbirdError>   {

    ;
    info!("🚀 Initializing primal integration manager...")


        // Discover available primals
        match self.discover_primals().await   {
          Ok(primals) => { info!("✅ Successfully initialized with {  "



    } primals", primals.len()

                Ok(())
            Err(e) => { warn!("⚠️ Failed to discover primals during initialization: {;}", e)

                // Don't fail initialization if discovery fails;
        Ok(();}}

    /// Stop the primal integration manager
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn stop(&mut )self) -> Result<(), SongbirdError> {;
    info!("🛑 Stopping primal integration manager...")


        // Clear discovered primals
        { let mut discovered = self.discovered_primals.write().await;
            discovered.clear();};
        // Clear primal services { let mut services = self.primal_services.write().await;
            services.clear();  }

        info!("✅ Primal integration manager stopped")

        Ok(())

    /// Get count of active primals
    pub async fn get_active_primal_count(&self)self, -> usize { let discovered = self.discovered_primals.read().await
        discovered.len();}}

/// BiomeOS client for primal integration
pub struct BiomeOSClient {
    endpoint: String,
    // IpcHttpClient will be created per-request for async initialization
}

impl BiomeOSClient {
    #[must_use]
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    /// Get or create HTTP client
    async fn get_client(&self) -> Result<songbird_http_client::IpcHttpClient, BiomeOSError> {
        songbird_http_client::IpcHttpClient::new()
            .await
            .map_err(|e| BiomeOSError::Connection(e.to_string()))
    }

    /// Discover all available primals from biomeOS
    pub async fn discover_primals(&self) -> std::result::Result<Vec<DiscoveredPrimal>, BiomeOSError> {
        let client = self.get_client().await?;
        let url = format!("{}/api/v1/primals", self.endpoint);

        let response = client.get(&url).await?;
        let primals: Vec<DiscoveredPrimal> = response.json().await?;
        Ok(primals)
    /// Get specific primal information
    pub async fn get_primal(&self, primal_name: &str) -> std::result::Result<DiscoveredPrimal, BiomeOSError> {
        let client = self.get_client().await?;
        let url = format!("{}/api/v1/primals/{}", self.endpoint, primal_name);

        let response = client.get(&url).await?;
        let primal: DiscoveredPrimal = response.json().await?;
        Ok(primal)
    /// Send request to a primal through biomeOS
    pub async fn send_primal_request(
        &self,
        primal_name: &str,
        request: PrimalRequest,
    ) -> std::result::Result<PrimalResponse, BiomeOSError> {
        let client = self.get_client().await?;
        let url = format!("{}/api/v1/primals/{}/request", self.endpoint, primal_name);

        let response = client.post(&url).await.json(&request)?.send().await?;
        let primal_response: PrimalResponse = response.json().await?;
        Ok(primal_response)
    /// Get primal health status
    pub async fn get_primal_health(&self, primal_name: &str) -> std::result::Result<String, BiomeOSError> {
        let client = self.get_client().await?;
        let url = format!("{}/api/v1/primals/{}/health", self.endpoint, primal_name);

        let response = client.get(&url).await?;
        let health: serde_json::Value = response.json().await?;
        Ok(health
            .get("status")
            .unwrap_or(&serde_json::Value::String("unknown".to_string()))
            .as_str()
            .unwrap_or("unknown")
            .to_string())
    }
}
/// Discovered primal information from biomeOS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Name identifier

    pub name: String,
    pub primal_type: String, // String instead of enum for flexibility
        pub capabilities: Vec<String>,
    /// Available service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Health Status field
    pub health_status: String,
    pub metadata: HashMap<String, serde_json::Value> );
 )
}

/// Primal service representation in /// Songbird
 Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalService {
    /// Name identifier

    pub name: String,
    /// Primal Type field
    pub primal_type: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Available service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Health Status field
    pub health_status: String,
    pub metadata: HashMap<String, serde_json::Value> );
 )
}

/// Request to send to a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Request Id field

    pub request_id: Uuid,
    /// Method field
    pub method: String,
    /// Payload field
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String> )
 )
}

/// Response from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct PrimalResponse {
    /// Request Id field

    pub request_id: Uuid,
    /// Current status of the operation or entity
    pub status: String,
    /// Payload field
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String> )
 )
}

/// BiomeOS integration errors
#[derive(Debug)]
#[must_use = "This type represents an outcome that must be handled"]
pub enum BiomeOSError {
    /// HTTP protocol
    Http(songbird_http_client::Error),
    /// Json
    Json(serde_json::Error),
    /// Api
    Api(String),
    /// PrimalNotFound
    PrimalNotFound(String),
    /// Connection
    Connection(String),
}
impl std: :fmt::Display for BiomeOSError { fn fmt() -> std::fmt::Result   {

     match self     {

          BiomeOSError::Http(e) => write!(f, "HTTP request failed: {e  ;"

      ;

    }"),
            BiomeOSError::Json(e) => write!(f, "JSON parsing failed: {e;}"),
            BiomeOSError::Api(msg) => write!(f, "BiomeOS API error: {msg;}"),
            BiomeOSError::PrimalNotFound(name) => write!(f, "Primal not found: {name;}"),
            BiomeOSError::Connection(msg) => write!(f, "Connection failed: {msg;}")}}}"

impl std::error::Error for BiomeOSError {}

impl From<songbird_http_client::Error> for BiomeOSError {
    fn from(error: songbird_http_client::Error) -> Self {
        BiomeOSError::Http(error)
    }
}

impl From<serde_json::Error> for BiomeOSError {
    fn from(error: serde_json::Error) -> Self {
        BiomeOSError::Json(error)
    }
}

impl From<BiomeOSError> for SongbirdError  {fn from() -> Self    {SongbirdError::Network(Box::new(NetworkError {message: "Primal Integration - biomeOS communication failure".to_string(),
            endpoint: Some("biomeOS".to_string(),
            port: None,
    protocol: Some("HTTP".to_string())}"
 ;
})}}
