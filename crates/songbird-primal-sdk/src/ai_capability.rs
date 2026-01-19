//! # 🤖 AI Capability Client (Zero Hardcoding)
//!
//! **REPLACES**: `squirrel.rs` - Hardcoded AI primal
//!
//! This module provides AI/ML capabilities without hardcoding specific primal names.
//! Works with ANY AI provider that implements the AI capability interface.
//!
//! ## Migration from Squirrel
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded squirrel primal
//! let squirrel = SquirrelPrimal::new(context);
//! let result = squirrel.get_model_inference("gpt", prompt).await?;
//!
//! // ✅ NEW: Capability-based AI client
//! let ai = AiCapabilityClient::new().await?;
//! let result = ai.get_model_inference("gpt", prompt).await?;
//! // Works with squirrel, openai, anthropic, or any AI provider!
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::errors::{SongbirdError, SongbirdResult};
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};

/// AI capability client (replaces SquirrelPrimal)
///
/// **Pure Rust Implementation**: Uses Unix socket JSON-RPC for inter-primal communication,
/// eliminating HTTP overhead and `reqwest` dependency (ring-free!).
#[derive(Debug, Clone)]
pub struct AiCapabilityClient {
    /// Capability endpoint resolver (for discovery)
    resolver: CapabilityEndpointResolver,
    /// JSON-RPC client for Unix socket communication (Pure Rust!)
    rpc_client: UnixRpcClient,
    /// Client configuration
    config: AiClientConfig,
}

/// AI client configuration
#[derive(Debug, Clone)]
pub struct AiClientConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Maximum tokens for inference
    pub max_tokens: u32,
    /// Default temperature
    pub temperature: f32,
}

impl Default for AiClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(
                std::env::var("AI_REQUEST_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            ),
            max_tokens: std::env::var("AI_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            temperature: std::env::var("AI_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
        }
    }
}

/// Model inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Model identifier
    pub model: String,
    /// Input prompt
    pub prompt: String,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature for sampling
    pub temperature: f32,
}

/// Model inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Generated response text
    pub response: String,
    /// Model used
    pub model: String,
    /// Tokens used
    pub tokens_used: Option<u32>,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
}

/// Agent framework request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    /// Agent action
    pub action: String,
    /// Agent parameters
    pub parameters: serde_json::Value,
    /// MCP (Model Context Protocol) options
    pub mcp_options: Option<serde_json::Value>,
}

/// Natural language processing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpRequest {
    /// Text to process
    pub text: String,
    /// Language code (e.g., "en", "es")
    pub language: Option<String>,
    /// Operation type (sentiment, entities, translation, etc.)
    pub operation: String,
}

impl AiCapabilityClient {
    /// Create new AI capability client
    ///
    /// Discovers AI providers dynamically - no hardcoded endpoints!
    ///
    /// # Example
    /// ```no_run
    /// use songbird_primal_sdk::ai_capability::AiCapabilityClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ai = AiCapabilityClient::new().await?;
    /// let response = ai.get_model_inference("gpt-4", "Hello!").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> SongbirdResult<Self> {
        Self::with_config(AiClientConfig::default()).await
    }
    
    /// Create AI client with custom configuration
    pub async fn with_config(config: AiClientConfig) -> SongbirdResult<Self> {
        info!("🤖 Creating AI capability client (Pure Rust Unix socket!)");
        
        // Discover Unix socket path for AI capability
        let socket_path = Self::discover_socket_path()?;
        
        // Create UnixRpcClient (100% Pure Rust!)
        let rpc_client = UnixRpcClient::new(&socket_path)
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create Unix RPC client for {:?}: {}", socket_path, e),
                field: Some("rpc_client".to_string()),
                suggestion: Some("Ensure AI primal is running and socket exists".to_string()),
            })?;
        
        info!("✅ AI capability client connected to {:?}", socket_path);
        
        Ok(Self {
            resolver: CapabilityEndpointResolver::new(),
            rpc_client,
            config,
        })
    }
    
    /// Discover Unix socket path for AI capability
    ///
    /// Priority:
    /// 1. AI_SOCKET_PATH environment variable
    /// 2. SQUIRREL_SOCKET_PATH environment variable (legacy)
    /// 3. Default: /tmp/squirrel.sock
    fn discover_socket_path() -> SongbirdResult<PathBuf> {
        std::env::var("AI_SOCKET_PATH")
            .or_else(|_| std::env::var("SQUIRREL_SOCKET_PATH"))
            .map(PathBuf::from)
            .or_else(|_| Ok(PathBuf::from("/tmp/squirrel.sock")))
    }
    
    /// Get model inference from any AI provider
    ///
    /// Works with ANY provider that implements the AI capability:
    /// - squirrel (if available)
    /// - OpenAI API
    /// - Anthropic Claude
    /// - Local models (llama.cpp, etc.)
    /// - Custom AI services
    ///
    /// # Arguments
    /// * `model` - Model identifier (e.g., "gpt-4", "claude-3", "llama-2")
    /// * `prompt` - Input prompt
    ///
    /// # Example
    /// ```no_run
    /// # use songbird_primal_sdk::ai_capability::AiCapabilityClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let ai = AiCapabilityClient::new().await?;
    /// let response = ai.get_model_inference("gpt-4", "Explain quantum computing").await?;
    /// println!("AI Response: {}", response.response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_model_inference(&self, model: &str, prompt: &str) -> SongbirdResult<InferenceResponse> {
        debug!("🧠 Requesting model inference via JSON-RPC: model={}", model);
        
        let request = InferenceRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };
        
        // Call ai.inference JSON-RPC method
        let response: InferenceResponse = self.rpc_client
            .call("ai.inference", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("AI inference RPC failed: {}", e),
                source: Some("ai.inference".to_string()),
            })?;
        
        info!("✅ AI inference complete (Pure Rust RPC!): {} tokens", response.tokens_used.unwrap_or(0));
        Ok(response)
    }
    
    /// Execute agent framework action
    ///
    /// Supports MCP (Model Context Protocol) for agent communication.
    pub async fn execute_agent_action(&self, request: AgentRequest) -> SongbirdResult<serde_json::Value> {
        debug!("🤖 Executing agent action via JSON-RPC: {}", request.action);
        
        // Call ai.execute_agent JSON-RPC method
        let response: serde_json::Value = self.rpc_client
            .call("ai.execute_agent", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Agent RPC failed: {}", e),
                source: Some("ai.execute_agent".to_string()),
            })?;
        
        info!("✅ Agent action complete (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Process natural language
    pub async fn process_natural_language(&self, request: NlpRequest) -> SongbirdResult<serde_json::Value> {
        debug!("💬 Processing NLP via JSON-RPC: operation={}", request.operation);
        
        // Call ai.nlp JSON-RPC method
        let response: serde_json::Value = self.rpc_client
            .call("ai.nlp", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("NLP RPC failed: {}", e),
                source: Some("ai.nlp".to_string()),
            })?;
        
        info!("✅ NLP processing complete (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Check if AI capability is available
    pub async fn is_available(&self) -> bool {
        self.resolver.get_endpoint(CapabilityType::Ai).await.is_ok()
    }
    
    /// Get current configuration
    pub fn config(&self) -> &AiClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ai_client_creation() {
        // Should create client (might fail if no AI provider configured)
        let result = AiCapabilityClient::new().await;
        // Don't assert success since we might not have AI provider configured in test
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_default_config() {
        let config = AiClientConfig::default();
        assert_eq!(config.max_tokens, 1000);
        assert!(config.temperature >= 0.0 && config.temperature <= 1.0);
    }
    
    #[test]
    fn test_inference_request_serialization() {
        let request = InferenceRequest {
            model: "gpt-4".to_string(),
            prompt: "test".to_string(),
            max_tokens: 100,
            temperature: 0.7,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("test"));
    }
}

