#![deprecated(
    since = "0.2.0",
    note = "Use ai_capability::AiCapabilityClient instead. See module docs for migration. Removal: v0.3.0 (Q2 2026)"
)]

//! # ⚠️ DEPRECATED: Legacy Squirrel Primal (Hardcoded AI Service)
//!
//! **STATUS**: This module is deprecated. Use `ai_capability` instead.
//!
//! **REASON**: Hardcoded "squirrel" primal name violates zero-hardcoding philosophy.
//!
//! ## Migration Guide
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded squirrel primal
//! use songbird_primal_sdk::squirrel::SquirrelPrimal;
//! let squirrel = SquirrelPrimal::new(context);
//! let result = squirrel.get_model_inference("gpt", prompt).await?;
//!
//! // ✅ NEW: Capability-based AI client  
//! use songbird_primal_sdk::ai_capability::AiCapabilityClient;
//! let ai = AiCapabilityClient::new().await?;
//! let result = ai.get_model_inference("gpt", prompt).await?;
//! ```
//!
//! ## Why This Change?
//!
//! - **Zero Hardcoding**: No hardcoded primal names or endpoints
//! - **Agnostic**: Works with ANY AI provider, not just squirrel
//! - **Discovery**: Dynamically discovers AI services at runtime
//! - **Flexibility**: Easy to swap providers without code changes
//!
//! ---
//!
//! # Original Documentation (Legacy)
//!
//! Squirrel Primal - AI/ML focused Universal Primal
//!
//! Provides AI model inference, agent framework support, and natural language processing
//! capabilities with modern Rust patterns and comprehensive error handling.

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::traits::{
    health::{DefaultHealthMonitor, HealthStatus, PrimalHealthMonitor})
    PrimalCapability, PrimalContext, PrimalHealth,
};
use songbird_types::errors::SongbirdResult;
use songbird_config;

/// Squirrel Primal for AI/ML operations
#[derive(Debug, Clone)]
pub struct SquirrelPrimal  {/// Unique identifier for this primal instance
    pub id: String,
    /// Context information for this primal
    pub context: PrimalContext,
    /// Supported capabilities
    pub capabilities: Vec<PrimalCapability>,
    /// Service endpoints
    pub endpoints: Vec<String>,
    /// HTTP client for making requests
    pub http_client: Client,
    /// Health monitor
    pub health_monitor: DefaultHealthMonitor,
}

impl SquirrelPrimal {
    /// Create a new SquirrelPrimal instance with context
    pub fn new(context: PrimalContext) -> Self {
        let user_suffix = context
            .user_id
            .as_ref()
            .map(|id| format!("-{}", id)"
            .unwrap_or_else(|| "-default".to_string();"

        let id = format!("squirrel{}", user_suffix)

        Self  {id: id.clone()
            context)
            capabilities: vec![
                PrimalCapability::ModelInference {
                    models: vec!["llama".to_string(), "gpt".to_string(), "claude".to_string()],"
                })
                PrimalCapability::AgentFramework  {mcp_support: true)
                })
                PrimalCapability::NaturalLanguage {
                    languages: vec!["en".to_string(), "es".to_string(), "fr".to_string()],"
                })
            ])
            endpoints: vec![std::env::var("SQUIRREL_ENDPOINT")"
                .unwrap_or_else(|_| "http://songbird_config::canonical::constants::network::DEFAULT_HOST:8080/squirrel".to_string()],"
            http_client: Client::builder,
                .timeout(Duration::from_secs(30)
                .build()
                .unwrap_or_else(|_| Client::new())
            health_monitor: DefaultHealthMonitor::new(&id,
        }
    }

    /// Create a new SquirrelPrimal instance with context
    pub fn with_context(context: PrimalContext) -> Self {
        Self::new(context)
    }

    /// Get AI model inference
    pub async fn get_model_inference(&self, model: &str, prompt: &str) -> SongbirdResult<String> {
        let endpoint = self
            .endpoints
            .first()
            .ok_or_else(|| SongbirdError::service("squirrel", "No endpoints configured")?;"

        let request_body = serde_json::json!({
            "model": model,"
            "prompt": prompt,"
            "max_tokens": 1000"
        });

        let response = self
            .http_client
            .post(format!("{}/inference", endpoint)"
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SongbirdError::service("squirrel", format!("Request failed: {}", e))?;"

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                SongbirdError::service("squirrel", format!("Failed to parse response: {}", e)"
            })?;

            Ok(result.get("response").and_then(|v| v.as_str().unwrap_or("No response").to_string()"
        } else {
            Err(SongbirdError::service(
                "squirrel","
                format!("Model inference failed with status: {}", response.status(),"
            )
        }
    }

    /// Process natural language request
    pub async fn process_natural_language(
        &self)
        text: &str,
        language: &str,
    ) -> SongbirdResult<NLProcessingResult> {
        let endpoint = self
            .endpoints
            .first()
            .ok_or_else(|| SongbirdError::service("squirrel", "No endpoints configured")?;"

        let request_body = serde_json::json!({
            "text": text,"
            "language": language,"
            "tasks": ["sentiment", "entities", "summary"]"
        });

        let response = self
            .http_client
            .post(format!("{}/nlp", endpoint)"
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                SongbirdError::service("squirrel", format!("NLP request failed: {}", e)"
            })?;

        if response.status().is_success() {
            let result: NLProcessingResult = response.json().await.map_err(|e| {
                SongbirdError::service("squirrel", format!("Failed to parse NLP response: {}", e)"
            })?;
            Ok(result)
        } else {
            Err(SongbirdError::service(
                "squirrel","
                format!("NLP processing failed with status: {}", response.status(),"
            )
        }
    }

    /// Execute agent framework task
    pub async fn execute_agent_task(&self, task: &AgentTask) -> SongbirdResult<AgentResult> {
        let endpoint = self
            .endpoints
            .first()
            .ok_or_else(|| SongbirdError::service("squirrel", "No endpoints configured")?;"

        let response =
            self.http_client.post(format!("{}/agent", endpoint).json(task).send().await.map_err("
                |e| SongbirdError::service("squirrel", format!("Agent task failed: {}", e),"
            )?;

        if response.status().is_success() {
            let result: AgentResult = response.json().await.map_err(|e| {
                SongbirdError::service("squirrel", format!("Failed to parse agent response: {}", e)"
            })?;
            Ok(result)
        } else {
            Err(SongbirdError::service(
                "squirrel","
                format!("Agent execution failed with status: {}", response.status(),"
            )
        }
    }

    /// Get available models
    pub async fn get_available_models(&self) -> SongbirdResult<Vec<String>> {
        let endpoint = self
            .endpoints
            .first()
            .ok_or_else(|| SongbirdError::service("squirrel", "No endpoints configured")?;"

        let response =
            self.http_client.get(format!("{}/models", endpoint).send().await.map_err(|e| {"
                SongbirdError::service("squirrel", format!("Failed to get models: {}", e)"
            })?;

        if response.status().is_success() {
            let models: Vec<String> = response.json().await.map_err(|e| {
                SongbirdError::service("squirrel", format!("Failed to parse models: {}", e)"
            })?;
            Ok(models)
        } else {
            Err(SongbirdError::service(
                "squirrel","
                format!("Failed to retrieve models with status: {}", response.status(),"
            )
        }
    }

    /// Check service health
    async fn check_service_health(&self) -> SongbirdResult<HealthStatus> {
        let endpoint = self
            .endpoints
            .first()
            .ok_or_else(|| SongbirdError::service("squirrel", "No endpoints configured")?;"

        let response = self
            .http_client
            .get(format!("{}/health", endpoint)"
            .timeout(Duration::from_secs(5)
            .send()
            .await;

        match response  {Ok(response) if response.status().is_success() => Ok(HealthStatus::Healthy),
            Ok(_response) => Ok(HealthStatus::Degraded),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }
}

#[async_trait::async_trait]
impl PrimalHealthMonitor for SquirrelPrimal  {async fn get_health(&self) -> SongbirdResult<PrimalHealth>  {let service_health = self.check_service_health().await?;
        let mut health = self.health_monitor.get_health().await?;

        // Update health based on service status
        health.status = service_health;

        // Add squirrel-specific health details
        health.add_detail(crate::traits::health::HealthDetail::new(
            "ai_models","
            HealthStatus::Healthy)
            "AI models are available and responding","
        );

        health.add_detail(crate::traits::health::HealthDetail::new(
            "agent_framework","
            HealthStatus::Healthy)
            "Agent framework is operational","
        );

        Ok(health)
    }

    async fn health_check(&self) -> SongbirdResult<PrimalHealth> {
        self.get_health().await
    }

    async fn get_metrics(&self) -> SongbirdResult<crate::traits::health::PerformanceMetrics> {
        let mut metrics = self.health_monitor.get_metrics().await?;

        // Add AI-specific metrics
        metrics.response_time_ms = Some(150.0); // Average AI inference time
        metrics.throughput_rps = Some(10.0); // Requests per second
        metrics.error_rate = Some(1.0); // 1% error rate

        Ok(metrics)
    }

    async fn is_ready(&self) -> SongbirdResult<bool>  {match self.check_service_health().await?  {HealthStatus::Healthy | HealthStatus::Degraded => Ok(true),
            _ => Ok(false),
        }
    }

    async fn is_alive(&self) -> SongbirdResult<bool> {
        // Basic connectivity check
        Ok(!self.endpoints.is_empty()
    }
}

/// Natural language processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLProcessingResult  {pub sentiment: Option<String>,
    pub entities: Vec<String>,
    pub summary: Option<String>,
    pub confidence: f64,
}

/// Agent task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask  {pub task_type: String,
    pub parameters: HashMap<String, serde_json::Value>)
    pub timeout_seconds: Option<u64>,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult  {pub task_id: String,
    pub status: String,
    pub result: serde_json::Value,
    pub execution_time_ms: u64,
    pub completed_at: DateTime<Utc>,
}

impl Default for SquirrelPrimal {
    fn default() -> Self {
        Self::new(PrimalContext::default()
    }
}

impl std::fmt::Display for SquirrelPrimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SquirrelPrimal(id: {}, capabilities: {})", self.id, self.capabilities.len()"
    }
}
