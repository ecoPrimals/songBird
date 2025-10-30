//! AI Capability Provider (Primal-Agnostic)
//!
//! Provides AI/ML inference, agent framework, and natural language processing
//! capabilities through pure capability-based discovery. No hardcoded primal names.
//!
//! # Philosophy
//!
//! This module requests "ai" capability without knowing or caring which
//! primal provides it. Could be squirrel, could be something else. We only
//! care about the CAPABILITY, not the PROVIDER.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use songbird_types::{SongbirdError, SongbirdResult};

/// AI capability configuration (vendor/primal agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapabilityConfig {
    /// Required AI capabilities
    pub required_capabilities: Vec<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Discovery hints (environment variables to check)
    pub discovery_hints: Vec<String>,
}

/// AI capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    /// Operation to perform
    pub operation: AiOperation,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request ID for tracking
    pub request_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// AI operations (capability-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiOperation {
    /// Model inference
    ModelInference {
        model: Option<String>,
        prompt: String,
        max_tokens: Option<u32>,
    },
    /// Natural language processing
    ProcessNaturalLanguage {
        text: String,
        language: String,
        tasks: Vec<String>,
    },
    /// Agent task execution
    ExecuteAgentTask { task: AgentTask },
    /// List available models
    ListModels,
    /// Sentiment analysis
    AnalyzeSentiment { text: String },
    /// Entity extraction
    ExtractEntities { text: String },
    /// Text summarization
    SummarizeText { text: String, max_length: Option<usize> },
    /// Translation
    TranslateText {
        text: String,
        source_language: String,
        target_language: String,
    },
}

/// Agent task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Task type
    pub task_type: String,
    /// Task parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// AI capability response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// Request ID
    pub request_id: String,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Provider ID (learned through discovery)
    pub provider_id: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Natural language processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLProcessingResult {
    /// Sentiment (positive, negative, neutral)
    pub sentiment: Option<String>,
    /// Extracted entities
    pub entities: Vec<String>,
    /// Text summary
    pub summary: Option<String>,
    /// Confidence score
    pub confidence: f64,
}

/// Agent execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Task ID
    pub task_id: String,
    /// Execution status
    pub status: String,
    /// Result data
    pub result: serde_json::Value,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
}

impl Default for AiCapabilityConfig {
    fn default() -> Self {
        Self {
            required_capabilities: vec![
                "model_inference".to_string(),
                "natural_language".to_string(),
            ],
            timeout_secs: std::env::var("AI_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60), // AI operations can take longer
            max_retries: std::env::var("AI_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2), // Fewer retries for AI (expensive)
            discovery_hints: vec![
                "SONGBIRD_AI_DISCOVERY".to_string(),
                "AI_ENDPOINT".to_string(),
                "ML_SERVICE_URL".to_string(),
                // Legacy compatibility (for migration period only)
                "SQUIRREL_ENDPOINT".to_string(),
            ],
        }
    }
}

/// Request AI capability from discovered provider
///
/// This function uses the infant discovery engine to find a provider that
/// offers "ai" capability. It doesn't know or care about primal names.
pub async fn request_ai_capability(request: AiRequest) -> SongbirdResult<AiResponse> {
    // Import the infant discovery engine
    use songbird_universal::InfantDiscoveryEngine;

    // Get or create discovery engine
    let discovery = InfantDiscoveryEngine::new();

    // Request AI capability (no primal name needed!)
    let response = discovery
        .request_capability(
            "ai",
            &serde_json::to_string(&request.operation)
                .map_err(|e| SongbirdError::internal_error(&format!("Serialization failed: {}", e)))?,
            &serde_json::to_value(&request.parameters)
                .map_err(|e| SongbirdError::internal_error(&format!("Value conversion failed: {}", e)))?,
        )
        .await?;

    // Parse response
    let ai_response: AiResponse = serde_json::from_value(response.response_data)
        .map_err(|e| SongbirdError::internal_error(&format!("Failed to parse AI response: {}", e)))?;

    Ok(ai_response)
}

/// Helper: Model inference
pub async fn model_inference(prompt: String, model: Option<String>) -> SongbirdResult<String> {
    let request = AiRequest {
        operation: AiOperation::ModelInference {
            model,
            prompt,
            max_tokens: Some(1000),
        },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_ai_capability(request).await?;

    if response.success {
        response
            .data
            .get("response")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SongbirdError::internal_error("No response in AI output"))
    } else {
        Err(SongbirdError::internal_error(
            &response
                .error
                .unwrap_or_else(|| "Model inference failed".to_string()),
        ))
    }
}

/// Helper: Process natural language
pub async fn process_natural_language(
    text: String,
    language: String,
) -> SongbirdResult<NLProcessingResult> {
    let request = AiRequest {
        operation: AiOperation::ProcessNaturalLanguage {
            text,
            language,
            tasks: vec![
                "sentiment".to_string(),
                "entities".to_string(),
                "summary".to_string(),
            ],
        },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_ai_capability(request).await?;

    if response.success {
        serde_json::from_value(response.data).map_err(|e| {
            SongbirdError::internal_error(&format!("Failed to parse NLP result: {}", e))
        })
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "NLP processing failed".to_string()),
        ))
    }
}

/// Helper: Execute agent task
pub async fn execute_agent_task(task: AgentTask) -> SongbirdResult<AgentResult> {
    let request = AiRequest {
        operation: AiOperation::ExecuteAgentTask { task },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_ai_capability(request).await?;

    if response.success {
        serde_json::from_value(response.data).map_err(|e| {
            SongbirdError::internal_error(&format!("Failed to parse agent result: {}", e))
        })
    } else {
        Err(SongbirdError::internal_error(
            &response
                .error
                .unwrap_or_else(|| "Agent execution failed".to_string()),
        ))
    }
}

/// Helper: List available models
pub async fn list_available_models() -> SongbirdResult<Vec<String>> {
    let request = AiRequest {
        operation: AiOperation::ListModels,
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_ai_capability(request).await?;

    if response.success {
        response
            .data
            .get("models")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .ok_or_else(|| SongbirdError::internal_error("No models list in response"))
    } else {
        Err(SongbirdError::internal_error(
            &response
                .error
                .unwrap_or_else(|| "Failed to list models".to_string()),
        ))
    }
}

/// Helper: Analyze sentiment
pub async fn analyze_sentiment(text: String) -> SongbirdResult<String> {
    let request = AiRequest {
        operation: AiOperation::AnalyzeSentiment { text },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_ai_capability(request).await?;

    if response.success {
        response
            .data
            .get("sentiment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SongbirdError::internal_error("No sentiment in response"))
    } else {
        Err(SongbirdError::internal_error(
            &response
                .error
                .unwrap_or_else(|| "Sentiment analysis failed".to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_default() {
        let config = AiCapabilityConfig::default();
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_retries, 2);
        assert!(!config.required_capabilities.is_empty());
    }

    #[test]
    fn test_agent_task_creation() {
        let task = AgentTask {
            task_type: "analysis".to_string(),
            parameters: HashMap::new(),
            timeout_seconds: Some(30),
        };

        assert_eq!(task.task_type, "analysis");
        assert_eq!(task.timeout_seconds, Some(30));
    }
}
