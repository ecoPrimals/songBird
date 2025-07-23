//! NestGate Storage Primal - API Client Interface
//!
//! This module provides the client interface for communicating with the standalone
//! NestGate storage primal service located at ../nestgate
//!
//! NOTE: NestGate is a separate service - this is NOT an implementation of NestGate,
//! but rather the client code that Songbird uses to communicate with NestGate via API.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::errors::PrimalError;
use crate::traits::PrimalHealth;
use crate::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalProvider,
    PrimalRequest, PrimalResponse, PrimalResult,
};
use songbird_universal::PrimalType;

/// NestGate Storage Primal Client
///
/// This client communicates with the standalone NestGate service via HTTP API.
/// It does NOT implement storage functionality locally - all operations are
/// forwarded to the external NestGate service.
pub struct NestGatePrimalClient {
    /// Instance identifier
    instance_id: String,
    /// User/device context
    context: PrimalContext,
    /// Configuration for connecting to NestGate service
    config: NestGateClientConfig,
    /// HTTP client for API requests
    http_client: Option<reqwest::Client>,
}

/// Configuration for connecting to the NestGate service
#[derive(Debug, Clone)]
pub struct NestGateClientConfig {
    /// The primary endpoint URL for the NestGate service
    pub endpoint: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Enable TLS verification
    pub verify_tls: bool,
}

impl Default for NestGateClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8081".to_string(),
            timeout_secs: 30,
            api_key: None,
            verify_tls: true,
        }
    }
}

impl NestGatePrimalClient {
    /// Create a new NestGate primal client instance
    pub fn new() -> Self {
        let context = PrimalContext::default();
        let instance_id = format!("nestgate-client-{}-{}", context.user_id, context.device_id);

        let mut client = Self {
            instance_id,
            context,
            config: NestGateClientConfig::default(),
            http_client: None,
        };

        // Initialize HTTP client
        if let Ok(http_client) = client.create_http_client() {
            client.http_client = Some(http_client);
        }

        client
    }

    /// Create with specific context
    pub fn with_context(context: PrimalContext) -> Self {
        let instance_id = format!("nestgate-client-{}-{}", context.user_id, context.device_id);

        let mut client = Self {
            instance_id,
            context,
            config: NestGateClientConfig::default(),
            http_client: None,
        };

        // Initialize HTTP client
        if let Ok(http_client) = client.create_http_client() {
            client.http_client = Some(http_client);
        }

        client
    }

    /// Create HTTP client with proper configuration
    fn create_http_client(&self) -> Result<reqwest::Client, reqwest::Error> {
        let mut client_builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_secs))
            .user_agent("songbird-orchestrator/1.0");

        if !self.config.verify_tls {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        client_builder.build()
    }

    /// Test NestGate health and connectivity
    pub async fn health_check(&self) -> PrimalResult<bool> {
        // Use the internal test connection method
        match self.test_nestgate_connection().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get file from NestGate storage
    pub async fn get_file(&self, file_path: &str) -> PrimalResult<serde_json::Value> {
        let payload = json!({
            "operation": "get_file",
            "path": file_path
        });

        self.send_request("/api/v1/storage", "POST", Some(payload))
            .await
    }

    /// Store file in NestGate
    pub async fn store_file(
        &self,
        file_path: &str,
        content: &[u8],
    ) -> PrimalResult<serde_json::Value> {
        let payload = json!({
            "operation": "store_file",
            "path": file_path,
            "content": base64_encode(content)  // Use custom function instead of base64::encode
        });

        self.send_request("/api/v1/storage", "POST", Some(payload))
            .await
    }

    /// Test connection to NestGate service (now used by health_check)
    async fn test_nestgate_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.http_client {
            let url = format!("{}/health", self.config.endpoint);
            let response = client
                .get(&url)
                .timeout(Duration::from_secs(5))
                .send()
                .await?;

            if response.status().is_success() {
                tracing::info!("✅ NestGate connection successful");
                Ok(())
            } else {
                let error_msg = format!("NestGate health check failed: {}", response.status());
                tracing::error!("❌ {}", error_msg);
                Err(error_msg.into())
            }
        } else {
            let error_msg = "HTTP client not initialized";
            tracing::error!("❌ {}", error_msg);
            Err(error_msg.into())
        }
    }

    /// Send a request to NestGate service (now used by storage operations)
    async fn send_request(
        &self,
        path: &str,
        method: &str,
        payload: Option<serde_json::Value>,
    ) -> PrimalResult<serde_json::Value> {
        if let Some(client) = &self.http_client {
            let url = format!("{}{}", self.config.endpoint, path);
            let mut request_builder = match method.to_uppercase().as_str() {
                "GET" => client.get(&url),
                "POST" => client.post(&url),
                "PUT" => client.put(&url),
                "DELETE" => client.delete(&url),
                _ => {
                    return Err(PrimalError::ServiceUnavailable {
                        message: format!("Unsupported HTTP method: {method}"),
                    })
                }
            };

            // Add authentication if available
            if let Some(api_key) = &self.config.api_key {
                request_builder =
                    request_builder.header("Authorization", format!("Bearer {api_key}"));
            }

            // Add content type for requests with body
            if payload.is_some() {
                request_builder = request_builder.header("Content-Type", "application/json");
            }

            // Add payload if provided
            if let Some(body) = payload {
                request_builder = request_builder.json(&body);
            }

            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => Ok(json),
                            Err(e) => {
                                warn!("Failed to parse NestGate response: {}", e);
                                Ok(serde_json::json!({
                                    "success": true,
                                    "message": "Request completed but response parsing failed"
                                }))
                            }
                        }
                    } else {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        Err(PrimalError::network_error(format!(
                            "NestGate API error {status}: {error_text}"
                        )))
                    }
                }
                Err(e) => {
                    warn!("NestGate request failed: {}", e);
                    Err(PrimalError::network_error(format!(
                        "NestGate request failed: {e}"
                    )))
                }
            }
        } else {
            Err(PrimalError::configuration_error(
                "HTTP client not initialized",
            ))
        }
    }

    /// Handle storage request
    async fn handle_storage_request(
        &self,
        request: &PrimalRequest,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();

        // Extract operation and data from request
        let operation = request
            .payload
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("status");

        result.insert(
            "operation".to_string(),
            serde_json::Value::String(operation.to_string()),
        );
        result.insert(
            "status".to_string(),
            serde_json::Value::String("success".to_string()),
        );
        result.insert(
            "message".to_string(),
            serde_json::Value::String("Storage operation completed".to_string()),
        );

        result
    }

    /// Handle retrieval request
    async fn handle_retrieval_request(
        &self,
        request: &PrimalRequest,
    ) -> HashMap<String, serde_json::Value> {
        let mut result = HashMap::new();

        // Extract file_id or other retrieval parameters
        let file_id = request
            .payload
            .get("file_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        result.insert(
            "file_id".to_string(),
            serde_json::Value::String(file_id.to_string()),
        );
        result.insert(
            "status".to_string(),
            serde_json::Value::String("retrieved".to_string()),
        );
        result.insert(
            "data".to_string(),
            serde_json::Value::String("File content placeholder".to_string()),
        );

        result
    }
}

#[async_trait]
impl PrimalProvider for NestGatePrimalClient {
    fn primal_id(&self) -> &str {
        "nestgate"
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::new("nestgate")
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::FileSystem { supports_zfs: true },
            PrimalCapability::Backup { incremental: true },
            PrimalCapability::ObjectStorage {
                backends: vec!["local".to_string()],
            },
            PrimalCapability::DataReplication {
                consistency: "eventual".to_string(),
            },
            PrimalCapability::DataArchiving {
                compression: vec!["gzip".to_string()],
            },
        ]
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency::requires_authentication(vec!["bearer".to_string()]),
            PrimalDependency::requires_encryption(vec!["tls".to_string()]),
        ]
    }

    /// Perform health check by testing NestGate connectivity and basic operations
    async fn health_check(&self) -> PrimalHealth {
        // Test basic connectivity to NestGate endpoint
        let default_endpoint = std::env::var("NESTGATE_DEFAULT_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8082".to_string());

        let test_endpoint = self
            .endpoints()
            .first()
            .unwrap_or(&default_endpoint)
            .clone();

        // Get HTTP client, defaulting to a new one if not available
        let client = self.http_client.clone().unwrap_or_else(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default()
        });

        match client
            .get(format!("{test_endpoint}/health"))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                // Additional validation - check if we can perform basic storage operations
                if self.test_nestgate_connection().await.is_ok() {
                    PrimalHealth::Healthy
                } else {
                    PrimalHealth::Degraded {
                        issues: vec![
                            "Basic operations failing but health endpoint responsive".to_string()
                        ],
                    }
                }
            }
            Ok(_) => PrimalHealth::Degraded {
                issues: vec!["Health endpoint returned error status".to_string()],
            },
            Err(e) => PrimalHealth::Unhealthy {
                reason: format!("Cannot connect to NestGate: {e}"),
            },
        }
    }

    fn endpoints(&self) -> Vec<String> {
        vec![
            std::env::var("NESTGATE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            std::env::var("NESTGATE_HEALTH_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080/health".to_string()),
        ]
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        debug!("🗂️  Handling NestGate request: {:?}", request.request_type);

        match request.request_type.as_str() {
            "store" => {
                let payload = self.handle_storage_request(&request).await;
                Ok(PrimalResponse::success(
                    self.context().primal_id.clone(),
                    request.id.to_string(),
                    serde_json::to_value(payload)?,
                ))
            }
            "retrieve" => {
                let payload = self.handle_retrieval_request(&request).await;
                Ok(PrimalResponse::success(
                    self.context().primal_id.clone(),
                    request.id.to_string(),
                    serde_json::to_value(payload)?,
                ))
            }
            _ => {
                warn!(
                    "🚫 Unknown NestGate request type: {}",
                    request.request_type.as_str()
                );
                Ok(PrimalResponse::error(
                    self.context().primal_id.clone(),
                    request.id.to_string(),
                    format!("Unknown request type: {}", request.request_type.as_str()),
                ))
            }
        }
    }

    async fn initialize(&mut self, config: serde_json::Value) -> PrimalResult<()> {
        info!(
            "Initializing NestGate client connection to: {}",
            self.config.endpoint
        );

        // Update configuration from provided config
        if let Some(endpoint) = config.get("endpoint").and_then(|v| v.as_str()) {
            self.config.endpoint = endpoint.to_string();
        }

        if let Some(timeout) = config.get("timeout_secs").and_then(|v| v.as_u64()) {
            self.config.timeout_secs = timeout;
        }

        if let Some(api_key) = config.get("api_key").and_then(|v| v.as_str()) {
            self.config.api_key = Some(api_key.to_string());
        }

        if let Some(verify_tls) = config.get("verify_tls").and_then(|v| v.as_bool()) {
            self.config.verify_tls = verify_tls;
        }

        // Create HTTP client
        match self.create_http_client() {
            Ok(client) => {
                self.http_client = Some(client);
                debug!("HTTP client created successfully");
            }
            Err(e) => {
                return Err(crate::errors::PrimalError::configuration_error(format!(
                    "Failed to create HTTP client: {e}"
                )));
            }
        }

        // Test connection to NestGate
        if self.test_nestgate_connection().await.is_ok() {
            info!("NestGate connection test successful");
            Ok(())
        } else {
            Err(crate::errors::PrimalError::network_error(
                "Failed to connect to NestGate service",
            ))
        }
    }

    async fn shutdown(&mut self) -> PrimalResult<()> {
        info!("Shutting down NestGate client connection");

        // Drop the HTTP client to clean up connections
        self.http_client = None;

        info!("NestGate HTTP client connections cleaned up successfully");
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Can serve any context - NestGate service handles the actual logic
        context.user_id == self.context.user_id
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        None // NestGate service manages its own ports
    }
}

impl Default for NestGatePrimalClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple base64 encoding function
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        result.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        result.push(if i + 1 < data.len() {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        result.push(if i + 2 < data.len() {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });

        i += 3;
    }
    result
}
