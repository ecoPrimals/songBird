//! Agnostic Primal Integration for Songbird
//!
//! This module provides agnostic integration with any primal through biomeOS.
//! Songbird doesn't need to know about specific primal implementations -
//! it discovers and integrates with whatever primals biomeOS provides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::traits::service::ServiceEndpoint;
use songbird_errors::{NetworkError, Result, SongbirdError};

/// Agnostic primal integration manager
/// Works with any primal through biomeOS SDK
pub struct PrimalIntegrationManager {
    biomeos_client: Arc<BiomeOSClient>,
    discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
    primal_services: Arc<RwLock<HashMap<String, PrimalService>>>,
}

impl PrimalIntegrationManager {
    pub fn new(biomeos_endpoint: String) -> Self {
        Self {
            biomeos_client: Arc::new(BiomeOSClient::new(biomeos_endpoint)),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            primal_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Discover all available primals from biomeOS
    pub async fn discover_primals(&self) -> Result<Vec<DiscoveredPrimal>> {
        info!("🔍 Discovering primals through biomeOS...");

        match self.biomeos_client.discover_primals().await {
            Ok(primals) => {
                info!("✅ Discovered {} primals", primals.len());

                // Update discovered primals cache
                {
                    let mut discovered = self.discovered_primals.write().await;
                    discovered.clear();
                    for primal in &primals {
                        discovered.insert(primal.name.clone(), primal.clone());
                    }
                }

                // Register primals as services
                self.register_primal_services(&primals).await?;

                Ok(primals)
            }
            Err(e) => {
                error!("❌ Failed to discover primals: {}", e);
                Err(SongbirdError::Network(Box::new(NetworkError {
                    message: "Primal Integration - Failed to connect to primal service".to_string(),
                    endpoint: Some(self.biomeos_client.endpoint.clone()),
                    port: None,
                    protocol: Some("HTTP".to_string()),
                })))
            }
        }
    }

    /// Register discovered primals as Songbird services
    async fn register_primal_services(&self, primals: &[DiscoveredPrimal]) -> Result<()> {
        let mut services = self.primal_services.write().await;

        for primal in primals {
            let service = PrimalService {
                name: primal.name.clone(),
                primal_type: primal.primal_type.clone(),
                capabilities: primal.capabilities.clone(),
                endpoints: primal.endpoints.clone(),
                health_status: primal.health_status.clone(),
                metadata: primal.metadata.clone(),
            };

            services.insert(primal.name.clone(), service);
            info!(
                "📝 Registered primal service: {} ({})",
                primal.name, primal.primal_type
            );
        }

        Ok(())
    }

    /// Get all available primal services
    pub async fn get_primal_services(&self) -> HashMap<String, PrimalService> {
        self.primal_services.read().await.clone()
    }

    /// Get specific primal service
    pub async fn get_primal_service(&self, name: &str) -> Option<PrimalService> {
        self.primal_services.read().await.get(name).cloned()
    }

    /// Send request to a primal through biomeOS
    pub async fn send_primal_request(
        &self,
        primal_name: &str,
        method: &str,
        payload: serde_json::Value,
        metadata: HashMap<String, String>,
    ) -> Result<PrimalResponse> {
        debug!("📤 Sending request to primal '{}': {}", primal_name, method);

        let request = PrimalRequest {
            request_id: Uuid::new_v4(),
            method: method.to_string(),
            payload,
            metadata,
        };

        match self
            .biomeos_client
            .send_primal_request(primal_name, request)
            .await
        {
            Ok(response) => {
                debug!(
                    "📥 Received response from primal '{}': {}",
                    primal_name, response.status
                );
                Ok(response)
            }
            Err(e) => {
                error!(
                    "❌ Failed to send request to primal '{}': {}",
                    primal_name, e
                );
                Err(SongbirdError::Network(Box::new(NetworkError {
                    message: "Primal Integration - Failed to connect to primal service".to_string(),
                    endpoint: Some(self.biomeos_client.endpoint.clone()),
                    port: None,
                    protocol: Some("HTTP".to_string()),
                })))
            }
        }
    }

    /// Check health of all primals
    pub async fn check_primal_health(&self) -> Result<HashMap<String, String>> {
        let mut health_status = HashMap::new();

        let services = self.primal_services.read().await;
        for (name, _service) in services.iter() {
            match self.biomeos_client.get_primal_health(name).await {
                Ok(status) => {
                    health_status.insert(name.clone(), status);
                }
                Err(e) => {
                    warn!("⚠️  Failed to check health of primal '{}': {}", name, e);
                    health_status.insert(name.clone(), "unknown".to_string());
                }
            }
        }

        Ok(health_status)
    }

    /// Initialize the primal integration manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("🚀 Initializing primal integration manager...");
        
        // Discover available primals
        match self.discover_primals().await {
            Ok(primals) => {
                info!("✅ Successfully initialized with {} primals", primals.len());
                Ok(())
            }
            Err(e) => {
                warn!("⚠️ Failed to discover primals during initialization: {}", e);
                // Don't fail initialization if discovery fails
                Ok(())
            }
        }
    }

    /// Stop the primal integration manager
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping primal integration manager...");
        
        // Clear discovered primals
        {
            let mut discovered = self.discovered_primals.write().await;
            discovered.clear();
        }
        
        // Clear primal services
        {
            let mut services = self.primal_services.write().await;
            services.clear();
        }
        
        info!("✅ Primal integration manager stopped");
        Ok(())
    }

    /// Get count of active primals
    pub async fn get_active_primal_count(&self) -> usize {
        let discovered = self.discovered_primals.read().await;
        discovered.len()
    }
}

/// BiomeOS client for primal integration
pub struct BiomeOSClient {
    endpoint: String,
    client: reqwest::Client,
}

impl BiomeOSClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    /// Discover all available primals from biomeOS
    pub async fn discover_primals(
        &self,
    ) -> std::result::Result<Vec<DiscoveredPrimal>, BiomeOSError> {
        let url = format!("{}/api/v1/primals", self.endpoint);
        let response = self.client.get(&url).send().await?;
        let primals: Vec<DiscoveredPrimal> = response.json().await?;
        Ok(primals)
    }

    /// Get specific primal information
    pub async fn get_primal(
        &self,
        primal_name: &str,
    ) -> std::result::Result<DiscoveredPrimal, BiomeOSError> {
        let url = format!("{}/api/v1/primals/{}", self.endpoint, primal_name);
        let response = self.client.get(&url).send().await?;
        let primal: DiscoveredPrimal = response.json().await?;
        Ok(primal)
    }

    /// Send request to a primal through biomeOS
    pub async fn send_primal_request(
        &self,
        primal_name: &str,
        request: PrimalRequest,
    ) -> std::result::Result<PrimalResponse, BiomeOSError> {
        let url = format!("{}/api/v1/primals/{}/request", self.endpoint, primal_name);
        let response = self.client.post(&url).json(&request).send().await?;
        let primal_response: PrimalResponse = response.json().await?;
        Ok(primal_response)
    }

    /// Get primal health status
    pub async fn get_primal_health(
        &self,
        primal_name: &str,
    ) -> std::result::Result<String, BiomeOSError> {
        let url = format!("{}/api/v1/primals/{}/health", self.endpoint, primal_name);
        let response = self.client.get(&url).send().await?;
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
    pub name: String,
    pub primal_type: String, // String instead of enum for flexibility
    pub capabilities: Vec<String>,
    pub endpoints: Vec<ServiceEndpoint>,
    pub health_status: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Primal service representation in Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalService {
    pub name: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<ServiceEndpoint>,
    pub health_status: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request to send to a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    pub request_id: Uuid,
    pub method: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// Response from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub request_id: Uuid,
    pub status: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// BiomeOS integration errors
#[derive(Debug)]
pub enum BiomeOSError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    Api(String),
    PrimalNotFound(String),
    Connection(String),
}

impl std::fmt::Display for BiomeOSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiomeOSError::Http(e) => write!(f, "HTTP request failed: {e}"),
            BiomeOSError::Json(e) => write!(f, "JSON parsing failed: {e}"),
            BiomeOSError::Api(msg) => write!(f, "BiomeOS API error: {msg}"),
            BiomeOSError::PrimalNotFound(name) => write!(f, "Primal not found: {name}"),
            BiomeOSError::Connection(msg) => write!(f, "Connection failed: {msg}"),
        }
    }
}

impl std::error::Error for BiomeOSError {}

impl From<reqwest::Error> for BiomeOSError {
    fn from(error: reqwest::Error) -> Self {
        BiomeOSError::Http(error)
    }
}

impl From<serde_json::Error> for BiomeOSError {
    fn from(error: serde_json::Error) -> Self {
        BiomeOSError::Json(error)
    }
}

impl From<BiomeOSError> for SongbirdError {
    fn from(error: BiomeOSError) -> Self {
        SongbirdError::Network(Box::new(NetworkError {
            message: "Primal Integration - biomeOS communication failure".to_string(),
            endpoint: Some("biomeOS".to_string()),
            port: None,
            protocol: Some("HTTP".to_string()),
        }))
    }
}
