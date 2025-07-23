//! Toadstool Compute Primal implementation

use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::errors::PrimalResult;
use crate::traits::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints,
    PrimalHealth, PrimalProvider,
};
use crate::types::{PrimalRequest, PrimalResponse};
use songbird_universal::PrimalType;

/// Toadstool Compute Primal - Advanced compute orchestration and serverless execution
#[derive(Debug, Clone)]
pub struct ToadstoolPrimal {
    id: String,
    context: PrimalContext,
    capabilities: Vec<PrimalCapability>,
    endpoints: PrimalEndpoints,
    http_client: Client,
}

impl Default for ToadstoolPrimal {
    fn default() -> Self {
        Self {
            id: "toadstool".to_string(),
            context: PrimalContext::default(),
            capabilities: vec![
                PrimalCapability::ContainerRuntime {
                    orchestrators: vec![
                        "docker".to_string(),
                        "kubernetes".to_string(),
                        "podman".to_string(),
                    ],
                },
                PrimalCapability::ServerlessExecution {
                    languages: vec![
                        "rust".to_string(),
                        "python".to_string(),
                        "node".to_string(),
                        "go".to_string(),
                    ],
                },
                PrimalCapability::LoadBalancing {
                    algorithms: vec![
                        "round_robin".to_string(),
                        "least_connections".to_string(),
                        "weighted".to_string(),
                    ],
                },
            ],
            endpoints: PrimalEndpoints {
                primary: songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                    .to_string(),
                health: format!(
                    "{}/health",
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                ),
                metrics: Some(format!(
                    "{}/metrics",
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                )),
                admin: Some(format!(
                    "{}/admin",
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                )),
                websocket: Some(format!(
                    "ws://{}:{}/ws",
                    songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS,
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_PORT
                )),
                custom: {
                    let mut map = HashMap::new();
                    let base_endpoint =
                        songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT;
                    map.insert(
                        "containers".to_string(),
                        format!("{base_endpoint}/containers"),
                    );
                    map.insert(
                        "serverless".to_string(),
                        format!("{base_endpoint}/serverless"),
                    );
                    map.insert("jobs".to_string(), format!("{base_endpoint}/jobs"));
                    map.insert("scaling".to_string(), format!("{base_endpoint}/scaling"));
                    map
                },
            },
            http_client: Client::builder()
                .timeout(Duration::from_secs(60)) // Longer timeout for compute operations
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
}

impl ToadstoolPrimal {
    /// Create a new ToadstoolPrimal instance with context
    pub fn new(context: PrimalContext) -> Self {
        Self {
            id: format!("toadstool-{}", context.user_id),
            context,
            capabilities: vec![
                PrimalCapability::ContainerRuntime {
                    orchestrators: vec![
                        "docker".to_string(),
                        "kubernetes".to_string(),
                        "podman".to_string(),
                    ],
                },
                PrimalCapability::ServerlessExecution {
                    languages: vec![
                        "rust".to_string(),
                        "python".to_string(),
                        "node".to_string(),
                        "go".to_string(),
                    ],
                },
                PrimalCapability::LoadBalancing {
                    algorithms: vec![
                        "round_robin".to_string(),
                        "least_connections".to_string(),
                        "weighted".to_string(),
                    ],
                },
            ],
            endpoints: PrimalEndpoints {
                primary: songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                    .to_string(),
                health: format!(
                    "{}/health",
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                ),
                metrics: Some(format!(
                    "{}/metrics",
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                )),
                admin: Some(format!(
                    "{}/admin",
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT
                )),
                websocket: Some(format!(
                    "ws://{}:{}/ws",
                    songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS,
                    songbird_config::config::constants::network::DEFAULT_TOADSTOOL_PORT
                )),
                custom: {
                    let mut map = HashMap::new();
                    let base_endpoint =
                        songbird_config::config::constants::network::DEFAULT_TOADSTOOL_ENDPOINT;
                    map.insert(
                        "containers".to_string(),
                        format!("{base_endpoint}/containers"),
                    );
                    map.insert(
                        "serverless".to_string(),
                        format!("{base_endpoint}/serverless"),
                    );
                    map.insert("jobs".to_string(), format!("{base_endpoint}/jobs"));
                    map.insert("scaling".to_string(), format!("{base_endpoint}/scaling"));
                    map
                },
            },
            http_client: Client::builder()
                .timeout(Duration::from_secs(60)) // Longer timeout for compute operations
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Create a new ToadstoolPrimal instance with context
    pub fn with_context(context: PrimalContext) -> Self {
        Self::new(context)
    }

    /// Deploy container workload
    pub async fn deploy_container(
        &self,
        image: &str,
        config: HashMap<String, String>,
    ) -> crate::errors::PrimalResult<String> {
        let operation = serde_json::json!({
            "type": "deploy",
            "image": image,
            "config": config,
        });

        match self.execute_container_operation(operation).await {
            Ok(result) => {
                if let Some(container_id) = result.get("container_id") {
                    Ok(container_id.as_str().unwrap_or("unknown").to_string())
                } else {
                    Err(crate::errors::PrimalError::ServiceUnavailable {
                        message: "No container ID returned".to_string(),
                    })
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Scale container workload
    pub async fn scale_container(
        &self,
        container_id: &str,
        replicas: u32,
    ) -> crate::errors::PrimalResult<HashMap<String, serde_json::Value>> {
        let operation = serde_json::json!({
            "type": "scale",
            "container_id": container_id,
            "replicas": replicas,
        });

        match self.execute_container_operation(operation).await {
            Ok(result) => Ok(self.value_to_hashmap(result)),
            Err(e) => Err(e),
        }
    }

    /// Execute function workload
    pub async fn execute_function_workload(
        &self,
        function_name: &str,
        payload: serde_json::Value,
    ) -> crate::errors::PrimalResult<serde_json::Value> {
        let operation = serde_json::json!({
            "type": "function",
            "function": function_name,
            "payload": payload,
        });

        self.execute_container_operation(operation).await
    }

    /// Stop container workload
    pub async fn stop_container(&self, container_id: &str) -> crate::errors::PrimalResult<bool> {
        let operation = serde_json::json!({
            "type": "stop",
            "container_id": container_id,
        });

        match self.execute_container_operation(operation).await {
            Ok(result) => Ok(result
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)),
            Err(e) => Err(e),
        }
    }

    /// Execute serverless function
    async fn execute_serverless_function(
        &self,
        function_data: serde_json::Value,
    ) -> PrimalResult<serde_json::Value> {
        let serverless_endpoint = self
            .endpoints
            .custom
            .get("serverless")
            .unwrap_or(&self.endpoints.primary);

        // Get team ID from metadata
        let team_id = self
            .context
            .metadata
            .get("team_id")
            .unwrap_or(&self.context.user_id);

        let request_payload = serde_json::json!({
            "function": function_data,
            "context": {
                "user_id": self.context.user_id,
                "team_id": team_id,
                "execution_id": uuid::Uuid::new_v4().to_string()
            }
        });

        debug!("Executing serverless function at: {}", serverless_endpoint);

        let response = self
            .http_client
            .post(format!("{serverless_endpoint}/serverless/execute"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "songbird-orchestrator/1.0")
            .header("X-Context-User", &self.context.user_id)
            .header("X-Context-Team", team_id)
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to execute serverless function: {}", e);
                crate::errors::PrimalError::network_error(format!(
                    "Serverless execution failed: {e}"
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!(
                "Serverless function execution failed with status {}: {}",
                status, error_text
            );
            return Err(crate::errors::PrimalError::network_error(format!(
                "Serverless execution failed: {status} - {error_text}"
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            error!("Failed to parse serverless execution response: {}", e);
            crate::errors::PrimalError::serialization_error(format!("Response parsing failed: {e}"))
        })?;

        info!("Serverless function executed successfully");
        Ok(result)
    }

    /// Manage compute job
    async fn manage_compute_job(
        &self,
        action: &str,
        job_data: serde_json::Value,
    ) -> PrimalResult<serde_json::Value> {
        let jobs_endpoint = self
            .endpoints
            .custom
            .get("jobs")
            .unwrap_or(&self.endpoints.primary);

        debug!("Managing compute job: {} at {}", action, jobs_endpoint);

        // Get team ID from metadata
        let team_id = self
            .context
            .metadata
            .get("team_id")
            .unwrap_or(&self.context.user_id);

        let response = self
            .http_client
            .post(format!("{jobs_endpoint}/jobs/{action}"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "songbird-orchestrator/1.0")
            .header("X-Context-User", &self.context.user_id)
            .header("X-Context-Team", team_id)
            .json(&job_data)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to manage compute job {}: {}", action, e);
                crate::errors::PrimalError::network_error(format!("Job management failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!(
                "Job management {} failed with status {}: {}",
                action, status, error_text
            );
            return Err(crate::errors::PrimalError::network_error(format!(
                "Job management {action} failed: {status} - {error_text}"
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            error!("Failed to parse job management response: {}", e);
            crate::errors::PrimalError::serialization_error(format!("Response parsing failed: {e}"))
        })?;

        info!("Job management {} completed successfully", action);
        Ok(result)
    }

    /// Handle scaling operations
    async fn handle_scaling_operation(
        &self,
        scaling_data: serde_json::Value,
    ) -> PrimalResult<serde_json::Value> {
        let scaling_endpoint = self
            .endpoints
            .custom
            .get("scaling")
            .unwrap_or(&self.endpoints.primary);

        debug!("Handling scaling operation at: {}", scaling_endpoint);

        // Get team ID from metadata
        let team_id = self
            .context
            .metadata
            .get("team_id")
            .unwrap_or(&self.context.user_id);

        let response = self
            .http_client
            .post(format!("{scaling_endpoint}/scaling"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "songbird-orchestrator/1.0")
            .header("X-Context-User", &self.context.user_id)
            .header("X-Context-Team", team_id)
            .json(&scaling_data)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to handle scaling operation: {}", e);
                crate::errors::PrimalError::network_error(format!("Scaling operation failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            warn!(
                "Scaling operation failed with status {}: {}",
                status, error_text
            );
            return Err(crate::errors::PrimalError::network_error(format!(
                "Scaling operation failed: {status} - {error_text}"
            )));
        }

        let result: Value = response.json().await.map_err(|e| {
            error!("Failed to parse scaling operation response: {}", e);
            crate::errors::PrimalError::serialization_error(format!("Response parsing failed: {e}"))
        })?;

        info!("Scaling operation completed successfully");
        Ok(result)
    }

    /// Check if Toadstool service is available
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
                    debug!("Toadstool service is available");
                    true
                } else {
                    warn!(
                        "Toadstool service health check failed with status: {}",
                        response.status()
                    );
                    false
                }
            }
            Err(e) => {
                warn!("Toadstool service health check failed: {}", e);
                false
            }
        }
    }

    /// Execute container operation (now used internally by all container operations)
    async fn execute_container_operation(
        &self,
        operation: serde_json::Value,
    ) -> crate::errors::PrimalResult<serde_json::Value> {
        let container_endpoint = self
            .endpoints
            .custom
            .get("containers")
            .unwrap_or(&self.endpoints.primary);

        let payload = serde_json::json!({
            "operation": operation,
            "timestamp": chrono::Utc::now(),
        });

        debug!(
            "Executing container operation: {} at {}",
            operation, container_endpoint
        );

        // Get team ID from metadata
        let team_id = self
            .context
            .metadata
            .get("team_id")
            .unwrap_or(&self.context.user_id);

        let response = self
            .http_client
            .post(format!("{container_endpoint}/containers/execute"))
            .header("Content-Type", "application/json")
            .header("User-Agent", "songbird-orchestrator/1.0")
            .header("X-Context-User", &self.context.user_id)
            .header("X-Context-Team", team_id)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to execute container operation: {}", e);
                crate::errors::PrimalError::network_error(format!(
                    "Container operation failed: {e}"
                ))
            })?;

        if response.status().is_success() {
            match response.json().await {
                Ok(json) => Ok(json),
                Err(e) => Err(crate::errors::PrimalError::serialization_error(format!(
                    "Failed to parse container operation response: {e}"
                ))),
            }
        } else {
            let error_msg = format!(
                "Container operation failed with status: {}",
                response.status()
            );
            Err(crate::errors::PrimalError::ServiceUnavailable { message: error_msg })
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
                result.insert("result".to_string(), value);
                result
            }
        }
    }
}

/// Test connection to Toadstool service
async fn test_toadstool_connection() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8082/health")
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Health check failed with status: {}", response.status()).into())
    }
}

#[async_trait]
impl PrimalProvider for ToadstoolPrimal {
    fn primal_id(&self) -> &str {
        "toadstool"
    }

    fn instance_id(&self) -> &str {
        &self.id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::new("toadstool")
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![]
    }

    async fn health_check(&self) -> PrimalHealth {
        match test_toadstool_connection().await {
            Ok(_) => PrimalHealth::Healthy,
            Err(_) => PrimalHealth::Unhealthy {
                reason: "Toadstool service unavailable".to_string(),
            },
        }
    }

    fn endpoints(&self) -> Vec<String> {
        vec![
            "http://localhost:8082".to_string(),
            "http://localhost:8082/health".to_string(),
        ]
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        info!(
            "Processing Toadstool request: {:?}",
            request.request_type.as_str()
        );

        // Cache primal_id to avoid repeated clones - ZERO-COPY OPTIMIZATION
        let primal_id = &self.context().primal_id;

        // Check if service is available before processing
        if !self.check_service_availability().await {
            return Ok(PrimalResponse::error(
                primal_id.clone(),      // Only clone when actually needed for the response
                request.id.to_string(), // Convert Uuid to String
                "Toadstool service is currently unavailable".to_string(),
            ));
        }

        if request.request_type != crate::types::PrimalRequestType::Custom("toadstool".to_string())
        {
            return Ok(PrimalResponse::error(
                primal_id.clone(),
                request.id.to_string(), // Convert Uuid to String
                "Invalid request type for Toadstool".to_string(),
            ));
        }

        match request.request_type.as_str() {
            "container" => {
                debug!("🐳 Handling container request");

                // Extract container operation details from payload
                let operation = request
                    .payload
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("deploy");

                let image = request
                    .payload
                    .get("image")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if image.is_empty() {
                    return Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        "Container image is required".to_string(),
                    ));
                }

                let result = match operation {
                    "deploy" => {
                        let config = request
                            .payload
                            .get("config")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        match self.deploy_container(image, config).await {
                            Ok(container_id) => {
                                Ok(serde_json::json!({"container_id": container_id}))
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "scale" => {
                        let container_id = request
                            .payload
                            .get("container_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let replicas = request
                            .payload
                            .get("replicas")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(1) as u32;

                        match self.scale_container(container_id, replicas).await {
                            Ok(response) => Ok(serde_json::to_value(response)?),
                            Err(e) => Err(e),
                        }
                    }
                    "stop" => {
                        let container_id = request
                            .payload
                            .get("container_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        match self.stop_container(container_id).await {
                            Ok(success) => Ok(serde_json::json!({"stopped": success})),
                            Err(e) => Err(e),
                        }
                    }
                    _ => {
                        return Ok(PrimalResponse::error(
                            primal_id.clone(),
                            request.id.to_string(), // Convert Uuid to String
                            format!("Unsupported container operation: {operation}"),
                        ));
                    }
                };

                match result {
                    Ok(result) => Ok(PrimalResponse::success(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        result,
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        format!("Container operation failed: {e}"),
                    )),
                }
            }
            "serverless" => {
                debug!("⚡ Handling serverless request");
                let result = self
                    .execute_serverless_function(serde_json::to_value(&request.payload)?)
                    .await;
                match result {
                    Ok(result) => Ok(PrimalResponse::success(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        result,
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        format!("Serverless execution failed: {e}"),
                    )),
                }
            }
            "job" => {
                debug!("💼 Handling job request");
                let result = self
                    .manage_compute_job("submit", serde_json::to_value(&request.payload)?)
                    .await;
                match result {
                    Ok(result) => Ok(PrimalResponse::success(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        result,
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        format!("Job execution failed: {e}"),
                    )),
                }
            }
            "scale" => {
                debug!("📈 Handling scaling request");
                let result = self
                    .handle_scaling_operation(serde_json::to_value(&request.payload)?)
                    .await;
                match result {
                    Ok(result) => Ok(PrimalResponse::success(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        result,
                    )),
                    Err(e) => Ok(PrimalResponse::error(
                        primal_id.clone(),
                        request.id.to_string(), // Convert Uuid to String
                        format!("Scaling operation failed: {e}"),
                    )),
                }
            }
            _ => Err(crate::errors::PrimalError::validation_error(format!(
                "Unknown request type: {}",
                request.request_type.as_str()
            ))),
        }
    }

    async fn initialize(&mut self, config: serde_json::Value) -> PrimalResult<()> {
        info!("Initializing Toadstool primal with config: {:?}", config);

        // Update endpoints if provided in config
        if let Some(endpoints) = config.get("endpoints") {
            if let Some(primary) = endpoints.get("primary").and_then(|v| v.as_str()) {
                self.endpoints.primary = primary.to_string();
            }
            if let Some(health) = endpoints.get("health").and_then(|v| v.as_str()) {
                self.endpoints.health = health.to_string();
            }
        }

        // Verify connectivity to Toadstool service
        if !self.check_service_availability().await {
            warn!("Toadstool service is not available during initialization");
        } else {
            info!("Toadstool primal initialized successfully");
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> PrimalResult<()> {
        info!("Shutting down Toadstool primal: {}", self.id);
        // Graceful shutdown - stop running jobs if needed
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Check if this Toadstool instance can serve the given context
        // For now, accept contexts from the same user
        context.user_id == self.context.user_id
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        None // Toadstool uses fixed endpoints
    }
}
