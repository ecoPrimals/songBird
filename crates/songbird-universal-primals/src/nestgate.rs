//! NestGate Storage Primal - API Client Interface
//!
//! This module provides the client interface for communicating with the standalone
//! NestGate storage primal service located at ../nestgate
//!
//! NOTE: NestGate is a separate service - this is NOT an implementation of NestGate,
//! but rather the client code that Songbird uses to communicate with NestGate via API.

use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::traits::PrimalHealth;
use crate::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints,
    PrimalProvider, PrimalRequest, PrimalResponse, PrimalResult,
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

impl NestGatePrimalClient {
    /// Create a new NestGate primal client instance
    pub fn new() -> Self {
        let context = PrimalContext::default();
        let instance_id = format!("nestgate-client-{}-{}", context.user_id, context.device_id);

        Self {
            instance_id,
            context,
            config: NestGateClientConfig::default(),
            http_client: None,
        }
    }

    /// Create with specific context
    pub fn with_context(context: PrimalContext) -> Self {
        let instance_id = format!("nestgate-client-{}-{}", context.user_id, context.device_id);

        Self {
            instance_id,
            context,
            config: NestGateClientConfig::default(),
            http_client: None,
        }
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

    /// Test connection to NestGate service
    async fn test_connection(&self) -> bool {
        if let Some(client) = &self.http_client {
            let health_url = format!("{}/health", self.config.endpoint);

            match client.get(&health_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("NestGate health check successful");
                        true
                    } else {
                        warn!(
                            "NestGate health check failed with status: {}",
                            response.status()
                        );
                        false
                    }
                }
                Err(e) => {
                    warn!("NestGate health check failed: {}", e);
                    false
                }
            }
        } else {
            warn!("HTTP client not initialized");
            false
        }
    }

    /// Send a request to NestGate service
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
                    return Err(crate::errors::PrimalError::InvalidRequest(format!(
                        "Unsupported HTTP method: {method}"
                    )))
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
                        Err(crate::errors::PrimalError::Network(format!(
                            "NestGate API error {status}: {error_text}"
                        )))
                    }
                }
                Err(e) => {
                    warn!("NestGate request failed: {}", e);
                    Err(crate::errors::PrimalError::Network(format!(
                        "NestGate request failed: {e}"
                    )))
                }
            }
        } else {
            Err(crate::errors::PrimalError::Configuration(
                "HTTP client not initialized".to_string(),
            ))
        }
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
        PrimalType::NestGate
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
            PrimalDependency::RequiresAuthentication {
                methods: vec!["token".to_string()],
            },
            PrimalDependency::RequiresEncryption {
                algorithms: vec!["AES256".to_string()],
            },
        ]
    }

    async fn health_check(&self) -> PrimalHealth {
        if self.test_connection().await {
            PrimalHealth::Healthy
        } else {
            PrimalHealth::Unhealthy {
                reason: "Failed to connect to NestGate service".to_string(),
            }
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        PrimalEndpoints {
            primary: self.config.endpoint.clone(),
            health: format!("{}/health", self.config.endpoint),
            metrics: Some(format!("{}/metrics", self.config.endpoint)),
            admin: Some(format!("{}/admin", self.config.endpoint)),
            websocket: None,
            custom: HashMap::new(),
        }
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        info!("Forwarding request to NestGate service: {:?}", request);

        // Determine the API path based on request type
        let (path, method, payload) = match request.request_type {
            crate::types::PrimalRequestType::Store => {
                // Extract operation from request payload
                let operation = request
                    .payload
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("status");

                match operation {
                    "list" => ("/api/v1/storage/list".to_string(), "GET", None),
                    "get" => {
                        let file_id = request
                            .payload
                            .get("file_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        (format!("/api/v1/storage/files/{file_id}"), "GET", None)
                    }
                    "upload" => {
                        let upload_data = request.payload.get("data").cloned();
                        ("/api/v1/storage/upload".to_string(), "POST", upload_data)
                    }
                    "delete" => {
                        let file_id = request
                            .payload
                            .get("file_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        (format!("/api/v1/storage/files/{file_id}"), "DELETE", None)
                    }
                    _ => ("/api/v1/storage/status".to_string(), "GET", None),
                }
            }
            _ => ("/api/v1/status".to_string(), "GET", None),
        };

        debug!("Sending request to NestGate: {} {}", method, path);

        // Make the actual HTTP request to NestGate
        match self.send_request(&path, method, payload).await {
            Ok(response_data) => {
                debug!("Received response from NestGate: {:?}", response_data);

                Ok(PrimalResponse {
                    request_id: request.id,
                    response_type: crate::types::PrimalResponseType::Storage,
                    payload: {
                        let mut payload = HashMap::new();
                        payload.insert("data".to_string(), response_data);
                        payload
                    },
                    timestamp: chrono::Utc::now(),
                    success: true,
                    error_message: None,
                    metadata: Some({
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "nestgate_endpoint".to_string(),
                            self.config.endpoint.clone(),
                        );
                        metadata
                    }),
                })
            }
            Err(error) => {
                warn!("NestGate request failed: {}", error);

                Ok(PrimalResponse {
                    request_id: request.id,
                    response_type: crate::types::PrimalResponseType::Storage,
                    payload: {
                        let mut payload = HashMap::new();
                        payload.insert(
                            "error".to_string(),
                            serde_json::Value::String(error.to_string()),
                        );
                        payload
                    },
                    timestamp: chrono::Utc::now(),
                    success: false,
                    error_message: Some(error.to_string()),
                    metadata: Some({
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "nestgate_endpoint".to_string(),
                            self.config.endpoint.clone(),
                        );
                        metadata
                    }),
                })
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
                return Err(crate::errors::PrimalError::Configuration(format!(
                    "Failed to create HTTP client: {e}"
                )));
            }
        }

        // Test connection to NestGate
        if self.test_connection().await {
            info!("NestGate connection test successful");
            Ok(())
        } else {
            Err(crate::errors::PrimalError::Network(
                "Failed to connect to NestGate service".to_string(),
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

impl Default for NestGateClientConfig {
    fn default() -> Self {
        Self {
            endpoint: songbird_config::config::constants::network::DEFAULT_NESTGATE_ENDPOINT
                .to_string(),
            timeout_secs: 30,
            api_key: None,
            verify_tls: true,
        }
    }
}
