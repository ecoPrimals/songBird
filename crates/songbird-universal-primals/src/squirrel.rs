//! Squirrel AI Primal Implementation

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::traits::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalHealth,
    PrimalProvider,
};
use crate::types::{PrimalRequest, PrimalResponse};
use songbird_universal::PrimalType;

/// Squirrel AI Primal - Advanced AI coordination and MCP (Model Context Protocol) integration
#[derive(Debug, Clone)]
pub struct SquirrelPrimal {
    id: String,
    context: PrimalContext,
    capabilities: Vec<PrimalCapability>,
    endpoints: Vec<String>, // Changed from PrimalEndpoints to Vec<String>
    http_client: Client,
}

impl Default for SquirrelPrimal {
    fn default() -> Self {
        Self {
            id: "squirrel".to_string(),
            context: PrimalContext::default(),
            capabilities: vec![
                PrimalCapability::ModelInference {
                    models: vec!["llama".to_string(), "gpt".to_string(), "claude".to_string()],
                },
                PrimalCapability::AgentFramework { mcp_support: true },
                PrimalCapability::NaturalLanguage {
                    languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()],
                },
            ],
            endpoints: vec![
                songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT.to_string(),
            ],
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
}

impl SquirrelPrimal {
    /// Create a new SquirrelPrimal instance with context
    pub fn new(context: PrimalContext) -> Self {
        Self {
            id: format!("squirrel-{}", context.user_id),
            context,
            capabilities: vec![
                PrimalCapability::ModelInference {
                    models: vec!["llama".to_string(), "gpt".to_string(), "claude".to_string()],
                },
                PrimalCapability::AgentFramework { mcp_support: true },
                PrimalCapability::NaturalLanguage {
                    languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()],
                },
            ],
            endpoints: vec![
                songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT.to_string(),
            ],
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Create a new SquirrelPrimal instance with context
    pub fn with_context(context: PrimalContext) -> Self {
        Self::new(context)
    }

    /// Execute MCP request through Squirrel AI service
    pub async fn execute_mcp(
        &self,
        operation: &str,
        context: HashMap<String, serde_json::Value>,
    ) -> crate::errors::PrimalResult<serde_json::Value> {
        let payload = serde_json::json!({
            "operation": operation,
            "context": context,
            "timestamp": chrono::Utc::now(),
        });

        self.send_mcp_request(payload).await
    }

    /// Get AI inference from Squirrel service
    pub async fn get_inference(
        &self,
        model: &str,
        prompt: &str,
    ) -> crate::errors::PrimalResult<String> {
        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "max_tokens": 1000,
        });

        match self.send_mcp_request(payload).await {
            Ok(response) => {
                if let Some(text) = response.get("text") {
                    Ok(text.as_str().unwrap_or("").to_string())
                } else {
                    Ok("No inference result".to_string())
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Parse structured data using Squirrel AI
    pub async fn parse_data(
        &self,
        data: serde_json::Value,
        schema: &str,
    ) -> crate::errors::PrimalResult<HashMap<String, serde_json::Value>> {
        let payload = serde_json::json!({
            "operation": "parse",
            "data": data,
            "schema": schema,
        });

        match self.send_mcp_request(payload).await {
            Ok(response) => Ok(self.value_to_hashmap(response)),
            Err(e) => Err(e),
        }
    }

    /// Send MCP (Model Context Protocol) request to Squirrel (now used internally)
    async fn send_mcp_request(&self, payload: Value) -> crate::errors::PrimalResult<Value> {
        let default_endpoint = "http://localhost:8084".to_string();
        let mcp_endpoint = self.endpoints.first().unwrap_or(&default_endpoint);

        let response = match self
            .http_client
            .post(format!("{mcp_endpoint}/api/v1/mcp"))
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Err(crate::errors::PrimalError::network_error(format!(
                    "MCP request failed: {e}"
                )))
            }
        };

        if response.status().is_success() {
            match response.json().await {
                Ok(json) => Ok(json),
                Err(e) => Err(crate::errors::PrimalError::serialization_error(format!(
                    "Failed to parse MCP response: {e}"
                ))),
            }
        } else {
            let error_msg = format!("MCP request failed with status: {}", response.status());
            Err(crate::errors::PrimalError::ServiceUnavailable { message: error_msg })
        }
    }

    /// Check if Squirrel service is available
    async fn check_service_availability(&self) -> bool {
        match self
            .http_client
            .get(
                self.endpoints
                    .first()
                    .unwrap_or(&self.endpoints[0])
                    .to_string(),
            ) // Changed from .health to .first()
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Squirrel service is available");
                    true
                } else {
                    warn!(
                        "Squirrel service health check failed with status: {}",
                        response.status()
                    );
                    false
                }
            }
            Err(e) => {
                warn!("Squirrel service health check failed: {}", e);
                false
            }
        }
    }

    /// Convert Value to HashMap for response payload (now used internally)
    fn value_to_hashmap(&self, value: Value) -> HashMap<String, Value> {
        match value {
            Value::Object(map) => {
                let mut result = HashMap::new();
                for (k, v) in map {
                    result.insert(k, v);
                }
                result
            }
            _ => {
                let mut result = HashMap::new();
                result.insert("data".to_string(), value);
                result
            }
        }
    }

    /// Create new Squirrel primal with failsafe fallbacks
    pub fn with_failsafe_fallbacks(context: PrimalContext) -> Self {
        let mut squirrel = Self::new(context);
        squirrel.setup_failsafe_fallbacks();
        squirrel
    }

    /// Setup failsafe fallbacks for Squirrel AI services
    fn setup_failsafe_fallbacks(&mut self) {
        // Add ecosystem discovery fallbacks to endpoints
        self.endpoints.extend(vec![
            "http://localhost:8084".to_string(), // Default Squirrel port
            "http://localhost:3001".to_string(), // Alternative dashboard port
            "http://127.0.0.1:8084".to_string(), // IPv4 explicit
        ]);

        // Try to discover Squirrel in ecosystem
        if std::path::Path::new("../squirrel").exists() {
            self.endpoints.push("http://localhost:8084".to_string());
            // Add metadata about ecosystem path
            self.context
                .metadata
                .insert("ecosystem_path".to_string(), "../squirrel".to_string());
        }

        // Add capability-based fallbacks
        self.capabilities.push(PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()],
        });
    }

    /// Failsafe request handling with automatic fallbacks
    pub async fn failsafe_request(
        &self,
        request: PrimalRequest,
    ) -> Result<PrimalResponse, Box<dyn std::error::Error>> {
        // Try primary endpoint first
        match self.handle_primal_request(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                tracing::warn!("Primary Squirrel endpoint failed: {}", e);
            }
        }

        // Try ecosystem fallback endpoints
        for fallback_url in &self.endpoints {
            match self.try_fallback_endpoint(fallback_url, &request).await {
                Ok(response) => {
                    tracing::info!("Successful fallback to: {}", fallback_url);
                    return Ok(response);
                }
                Err(e) => {
                    tracing::warn!("Fallback {} failed: {}", fallback_url, e);
                }
            }
        }

        // Final fallback: return a basic response indicating service unavailable
        tracing::error!("All Squirrel endpoints failed, using emergency fallback");
        Ok(self.emergency_fallback_response(&request).await)
    }

    async fn try_fallback_endpoint(
        &self,
        endpoint: &str,
        request: &PrimalRequest,
    ) -> Result<PrimalResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Test if endpoint is responsive
        match client.get(format!("{endpoint}/health")).send().await {
            Ok(response) if response.status().is_success() => {
                // Endpoint is healthy, try to forward the request
                self.forward_request_to_endpoint(endpoint, request).await
            }
            _ => Err("Endpoint not responsive".into()),
        }
    }

    async fn forward_request_to_endpoint(
        &self,
        endpoint: &str,
        request: &PrimalRequest,
    ) -> Result<PrimalResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Convert PrimalRequest to HTTP request
        let response = client
            .post(format!("{endpoint}/api/v1/primal"))
            .json(request)
            .send()
            .await?;

        if response.status().is_success() {
            let response_data = response.json::<PrimalResponse>().await?;
            Ok(response_data)
        } else {
            Err(format!("Request failed with status: {}", response.status()).into())
        }
    }

    async fn emergency_fallback_response(&self, request: &PrimalRequest) -> PrimalResponse {
        PrimalResponse::service_unavailable(
            self.context().primal_id.clone(),
            request.id.to_string(),
        )
    }
}

#[async_trait]
impl PrimalProvider for SquirrelPrimal {
    fn primal_id(&self) -> &str {
        "squirrel"
    }

    fn instance_id(&self) -> &str {
        &self.id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::new("squirrel")
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![]
    }

    async fn health_check(&self) -> PrimalHealth {
        match self
            .http_client
            .get(
                self.endpoints
                    .first()
                    .unwrap_or(&"http://localhost:8084".to_string())
                    .to_string(),
            )
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => PrimalHealth::Healthy,
            _ => PrimalHealth::Unhealthy {
                reason: "Squirrel service unavailable".to_string(),
            },
        }
    }

    fn endpoints(&self) -> Vec<String> {
        // Changed from PrimalEndpoints to Vec<String>
        self.endpoints.clone()
    }

    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> crate::errors::PrimalResult<PrimalResponse> {
        // Cache primal_id to avoid repeated clones - ZERO-COPY OPTIMIZATION
        let primal_id = &self.context().primal_id;

        info!(
            "Processing Squirrel AI request: {:?} for {}",
            request.request_type.as_str(),
            primal_id
        );

        // Check if service is available before processing
        if !self.check_service_availability().await {
            return Ok(PrimalResponse::error(
                primal_id.clone(),
                request.id.to_string(), // Convert Uuid to String
                "Squirrel AI service is currently unavailable".to_string(),
            ));
        }

        if request.request_type != crate::types::PrimalRequestType::Custom("squirrel".to_string()) {
            return Ok(PrimalResponse::error(
                primal_id.clone(),
                request.id.to_string(), // Convert Uuid to String
                "Invalid request type for Squirrel".to_string(),
            ));
        }

        match request.request_type.as_str() {
            "mcp" => {
                debug!("🧠 Handling MCP (Model Context Protocol) request");

                // Use execute_mcp method with operation and context
                let operation = request
                    .payload
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                let context = request
                    .payload
                    .get("context")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let result = self.execute_mcp(operation, context).await;

                match result {
                    Ok(result) => Ok(PrimalResponse::success(
                        primal_id.clone(),
                        request.id.to_string(),
                        result,
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(),
                        format!("MCP operation failed: {e}"),
                    )),
                }
            }
            "inference" => {
                debug!("🔮 Handling AI inference request");

                // Use get_inference method with model and prompt
                let model = request
                    .payload
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("llama");

                let prompt = request
                    .payload
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let result = self.get_inference(model, prompt).await;
                match result {
                    Ok(result) => Ok(PrimalResponse::success(
                        primal_id.clone(),
                        request.id.to_string(),
                        serde_json::json!({"text": result}),
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(),
                        format!("Inference failed: {e}"),
                    )),
                }
            }
            _ => Err(crate::errors::PrimalError::validation_error(format!(
                "Unknown request type: {}",
                request.request_type.as_str()
            ))),
        }
    }

    async fn initialize(&mut self, config: serde_json::Value) -> crate::errors::PrimalResult<()> {
        info!("Initializing Squirrel primal with config: {:?}", config);

        // Update endpoints if provided in config
        if let Some(endpoints) = config.get("endpoints") {
            if let Some(primary) = endpoints.get("primary").and_then(|v| v.as_str()) {
                self.endpoints.push(primary.to_string());
            }
        }

        // Verify connectivity to Squirrel service
        if !self.check_service_availability().await {
            warn!("Squirrel service is not available during initialization");
        } else {
            info!("Squirrel primal initialized successfully");
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> crate::errors::PrimalResult<()> {
        info!("Shutting down Squirrel primal: {}", self.id);
        // Graceful shutdown - notify service if needed
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Check if this Squirrel instance can serve the given context
        // For now, accept contexts from the same user
        context.user_id == self.context.user_id
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        None // Squirrel uses fixed endpoints
    }
}
