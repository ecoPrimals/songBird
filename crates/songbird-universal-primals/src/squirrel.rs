//! Squirrel AI Primal Implementation

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::errors::PrimalError;
use crate::traits::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints,
    PrimalHealth, PrimalProvider, PrimalType,
};
use crate::types::{PrimalRequest, PrimalResponse, PrimalResponseType};

/// Squirrel AI Primal - Advanced AI coordination and MCP (Model Context Protocol) integration
#[derive(Debug, Clone)]
pub struct SquirrelPrimal {
    id: String,
    context: PrimalContext,
    capabilities: Vec<PrimalCapability>,
    endpoints: PrimalEndpoints,
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
            endpoints: PrimalEndpoints {
                primary: songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                    .to_string(),
                health: format!(
                    "{}/health",
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                ),
                metrics: Some(format!(
                    "{}/metrics",
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                )),
                admin: Some(format!(
                    "{}/admin",
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                )),
                websocket: Some(format!(
                    "ws://{}:{}/ws",
                    songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS,
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_PORT
                )),
                custom: {
                    let mut map = HashMap::new();
                    let base_endpoint =
                        songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT;
                    map.insert("mcp".to_string(), format!("{}/mcp", base_endpoint));
                    map.insert(
                        "inference".to_string(),
                        format!("{}/inference", base_endpoint),
                    );
                    map.insert("agents".to_string(), format!("{}/agents", base_endpoint));
                    map
                },
            },
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
            endpoints: PrimalEndpoints {
                primary: songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                    .to_string(),
                health: format!(
                    "{}/health",
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                ),
                metrics: Some(format!(
                    "{}/metrics",
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                )),
                admin: Some(format!(
                    "{}/admin",
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT
                )),
                websocket: Some(format!(
                    "ws://{}:{}/ws",
                    songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS,
                    songbird_config::config::constants::network::DEFAULT_SQUIRREL_PORT
                )),
                custom: {
                    let mut map = HashMap::new();
                    let base_endpoint =
                        songbird_config::config::constants::network::DEFAULT_SQUIRREL_ENDPOINT;
                    map.insert("mcp".to_string(), format!("{}/mcp", base_endpoint));
                    map.insert(
                        "inference".to_string(),
                        format!("{}/inference", base_endpoint),
                    );
                    map.insert("agents".to_string(), format!("{}/agents", base_endpoint));
                    map
                },
            },
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

    /// Send MCP (Model Context Protocol) request to Squirrel
    async fn send_mcp_request(&self, payload: Value) -> crate::errors::PrimalResult<Value> {
        let mcp_endpoint = self
            .endpoints
            .custom
            .get("mcp")
            .unwrap_or(&self.endpoints.primary);

        debug!("Sending MCP request to Squirrel at: {}", mcp_endpoint);

        // Get team ID from metadata
        let team_id = self
            .context
            .metadata
            .get("team_id")
            .unwrap_or(&self.context.user_id);

        let response = self
            .http_client
            .post(mcp_endpoint)
            .header("Content-Type", "application/json")
            .header("User-Agent", "songbird-orchestrator/1.0")
            .header("X-Context-User", &self.context.user_id)
            .header("X-Context-Team", team_id)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send MCP request to Squirrel: {}", e);
                crate::errors::PrimalError::Network(format!("MCP request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!(
                "Squirrel MCP request failed with status {}: {}",
                status, error_text
            );
            return Err(crate::errors::PrimalError::Network(format!(
                "Squirrel MCP request failed: {status} - {error_text}"
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            error!("Failed to parse Squirrel MCP response: {}", e);
            crate::errors::PrimalError::Serialization(format!("Response parsing failed: {e}"))
        })?;

        debug!("Received MCP response from Squirrel: {:?}", result);
        Ok(result)
    }

    /// Send AI inference request to Squirrel
    async fn send_inference_request(
        &self,
        model: &str,
        prompt: &str,
        parameters: Option<Value>,
    ) -> crate::errors::PrimalResult<Value> {
        let inference_endpoint = self
            .endpoints
            .custom
            .get("inference")
            .unwrap_or(&self.endpoints.primary);

        // Get team ID from metadata
        let team_id = self
            .context
            .metadata
            .get("team_id")
            .unwrap_or(&self.context.user_id);

        let request_payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "parameters": parameters.unwrap_or_else(|| serde_json::json!({})),
            "context": {
                "user_id": self.context.user_id,
                "team_id": team_id,
                "session_id": uuid::Uuid::new_v4().to_string()
            }
        });

        debug!(
            "Sending inference request to Squirrel: model={}, prompt_len={}",
            model,
            prompt.len()
        );

        let response = self
            .http_client
            .post(format!("{inference_endpoint}/inference"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "songbird-orchestrator/1.0")
            .header("X-Context-User", &self.context.user_id)
            .header("X-Context-Team", team_id)
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send inference request to Squirrel: {}", e);
                crate::errors::PrimalError::Network(format!("Inference request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!(
                "Squirrel inference request failed with status {}: {}",
                status, error_text
            );
            return Err(crate::errors::PrimalError::Network(format!(
                "Squirrel inference request failed: {status} - {error_text}"
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            error!("Failed to parse Squirrel inference response: {}", e);
            crate::errors::PrimalError::Serialization(format!("Response parsing failed: {e}"))
        })?;

        info!(
            "Received inference response from Squirrel for model: {}",
            model
        );
        Ok(result)
    }

    /// Check if Squirrel service is available
    async fn check_service_availability(&self) -> bool {
        match self
            .http_client
            .get(&self.endpoints.health)
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

    /// Convert Value to HashMap for response payload
    fn value_to_hashmap(&self, value: Value) -> HashMap<String, Value> {
        match value {
            Value::Object(map) => map.into_iter().collect(),
            _ => {
                let mut result = HashMap::new();
                result.insert("result".to_string(), value);
                result
            }
        }
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
        PrimalType::AI
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![]
    }

    async fn health_check(&self) -> PrimalHealth {
        if self.check_service_availability().await {
            PrimalHealth::Healthy
        } else {
            PrimalHealth::Unhealthy {
                reason: "Squirrel service unavailable".to_string(),
            }
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        self.endpoints.clone()
    }

    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> crate::errors::PrimalResult<PrimalResponse> {
        info!(
            "Processing Squirrel request: {:?}",
            request.request_type.as_str()
        );

        // Check if service is available before processing
        if !self.check_service_availability().await {
            return Ok(PrimalResponse::error(
                request.id,
                PrimalResponseType::Custom("squirrel".to_string()),
                "Squirrel service is currently unavailable".to_string(),
            ));
        }

        match request.request_type.as_str() {
            "mcp" => {
                // Handle MCP (Model Context Protocol) requests
                let payload_value = serde_json::to_value(&request.payload)?;
                match self.send_mcp_request(payload_value).await {
                    Ok(result) => Ok(PrimalResponse::success(
                        request.id,
                        PrimalResponseType::Custom("mcp".to_string()),
                        self.value_to_hashmap(result),
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        request.id,
                        PrimalResponseType::Custom("mcp".to_string()),
                        format!("MCP request failed: {e}"),
                    )),
                }
            }
            "inference" => {
                // Handle AI inference requests
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
                let parameters = request.payload.get("parameters").cloned();

                match self.send_inference_request(model, prompt, parameters).await {
                    Ok(result) => Ok(PrimalResponse::success(
                        request.id,
                        PrimalResponseType::Custom("inference".to_string()),
                        self.value_to_hashmap(result),
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        request.id,
                        PrimalResponseType::Custom("inference".to_string()),
                        format!("Inference request failed: {e}"),
                    )),
                }
            }
            "agent" => {
                // Handle agent framework requests
                let agent_endpoint = self
                    .endpoints
                    .custom
                    .get("agents")
                    .unwrap_or(&self.endpoints.primary);

                // Get team ID from metadata
                let team_id = self
                    .context
                    .metadata
                    .get("team_id")
                    .unwrap_or(&self.context.user_id);

                match self
                    .http_client
                    .post(format!("{agent_endpoint}/agents"))
                    .header("Content-Type", "application/json")
                    .header("X-Context-User", &self.context.user_id)
                    .header("X-Context-Team", team_id)
                    .json(&request.payload)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.json::<Value>().await {
                                Ok(result) => Ok(PrimalResponse::success(
                                    request.id,
                                    PrimalResponseType::Custom("agent".to_string()),
                                    self.value_to_hashmap(result),
                                )),
                                Err(e) => Ok(PrimalResponse::error(
                                    request.id,
                                    PrimalResponseType::Custom("agent".to_string()),
                                    format!("Failed to parse agent response: {e}"),
                                )),
                            }
                        } else {
                            return Ok(PrimalResponse::error(
                                request.id,
                                PrimalResponseType::Custom("agent".to_string()),
                                format!("Agent request failed with status: {}", response.status()),
                            ));
                        }
                    }
                    Err(e) => Ok(PrimalResponse::error(
                        request.id,
                        PrimalResponseType::Custom("agent".to_string()),
                        format!("Agent request failed: {e}"),
                    )),
                }
            }
            _ => {
                warn!(
                    "Unknown Squirrel request type: {}",
                    request.request_type.as_str()
                );
                Err(PrimalError::Validation(format!(
                    "Unknown request type: {}",
                    request.request_type.as_str()
                )))
            }
        }
    }

    async fn initialize(&mut self, config: serde_json::Value) -> crate::errors::PrimalResult<()> {
        info!("Initializing Squirrel primal with config: {:?}", config);

        // Update endpoints if provided in config
        if let Some(endpoints) = config.get("endpoints") {
            if let Some(primary) = endpoints.get("primary").and_then(|v| v.as_str()) {
                self.endpoints.primary = primary.to_string();
            }
            if let Some(health) = endpoints.get("health").and_then(|v| v.as_str()) {
                self.endpoints.health = health.to_string();
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
