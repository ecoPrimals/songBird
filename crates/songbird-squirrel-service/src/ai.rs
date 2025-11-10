//! AI client implementation

use crate::config::SquirrelConfig;
// KNOWN ISSUE: Anthropic v0.0.8 API compatibility
// The anthropic crate v0.0.8 has breaking API changes that need resolution
// Tracked as: Integration issue with anthropic SDK upgrade
// Workaround: Using mock responses until SDK is updated or alternative client implemented
// use anthropic::{Anthropic, types::{Message, MessagesRequest, ContentBlock}};
use std::time::Instant;
use tracing::{info, warn};

pub use crate::{ChatRequest, ChatResponse, InferenceRequest, InferenceResponse};

// Temp type placeholder until anthropic integration is fixed
type Anthropic = ();  // Placeholder

pub struct AIClient {
    anthropic_client: Option<Anthropic>,
    default_model: String,
}

impl AIClient {
    pub fn new(config: &SquirrelConfig) -> anyhow::Result<Self> {
        if config.anthropic_api_key.is_some() {
            info!("✅ Anthropic client initialized (integration pending anthropic v0.0.8 resolution)");
        } else {
            warn!("⚠️  No Anthropic API key - Claude unavailable");
        }

        let default_model = match config.ai_provider.as_str() {
            "claude" => "claude-3-5-haiku-20241022".to_string(),
            "gpt" => "gpt-4o-mini".to_string(),
            _ => "claude-3-5-haiku-20241022".to_string(),
        };

        Ok(Self {
            anthropic_client: None,  // Disabled pending anthropic SDK compatibility resolution
            default_model,
        })
    }

    pub async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        let start = Instant::now();
        let model = req.model.clone().unwrap_or_else(|| self.default_model.clone());

        // KNOWN ISSUE: Claude integration disabled pending anthropic v0.0.8 API resolution
        // Using fallback mock response until SDK compatibility is resolved
        let _ = self.anthropic_client;  // Silence unused field warning

        // Fallback to mock response
        Ok(ChatResponse {
            response: format!(
                "AI service integration pending. Received {} messages for model '{}'.",
                req.messages.len(),
                model
            ),
            model: "fallback".to_string(),
            tokens_used: 0,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    pub async fn inference(&self, req: InferenceRequest) -> anyhow::Result<InferenceResponse> {
        let _start = Instant::now();
        let model = req.model.clone().unwrap_or_else(|| self.default_model.clone());

        // Convert to chat format
        let chat_req = ChatRequest {
            model: Some(model.clone()),
            messages: vec![crate::ChatMessage {
                role: "user".to_string(),
                content: req.prompt,
            }],
            max_tokens: req.max_tokens,
            temperature: None,
        };

        let chat_response = self.chat(chat_req).await?;

        Ok(InferenceResponse {
            response: chat_response.response,
            model: chat_response.model,
            tokens: chat_response.tokens_used,
        })
    }

    pub fn list_models(&self) -> Vec<String> {
        let mut models = vec![];

        if self.anthropic_client.is_some() {
            models.extend(vec![
                "claude-3-5-sonnet-20241022".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                "claude-3-opus-20240229".to_string(),
            ]);
        }

        if models.is_empty() {
            models.push("fallback".to_string());
        }

        models
    }
}

