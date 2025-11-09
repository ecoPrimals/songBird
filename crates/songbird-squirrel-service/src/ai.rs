//! AI client implementation

use crate::config::SquirrelConfig;
use anthropic::{Anthropic, types::{Message, MessagesRequest, ContentBlock}};
use std::time::Instant;
use tracing::{info, warn};

pub use crate::{ChatRequest, ChatResponse, InferenceRequest, InferenceResponse};

pub struct AIClient {
    anthropic_client: Option<Anthropic>,
    default_model: String,
}

impl AIClient {
    pub fn new(config: &SquirrelConfig) -> anyhow::Result<Self> {
        let anthropic_client = if let Some(ref key) = config.anthropic_api_key {
            info!("✅ Anthropic client initialized");
            Some(Anthropic::new(key.clone()))
        } else {
            warn!("⚠️  No Anthropic API key - Claude unavailable");
            None
        };

        let default_model = match config.ai_provider.as_str() {
            "claude" => "claude-3-5-haiku-20241022".to_string(),
            "gpt" => "gpt-4o-mini".to_string(),
            _ => "claude-3-5-haiku-20241022".to_string(),
        };

        Ok(Self {
            anthropic_client,
            default_model,
        })
    }

    pub async fn chat(&self, req: ChatRequest) -> anyhow::Result<ChatResponse> {
        let start = Instant::now();
        let model = req.model.clone().unwrap_or_else(|| self.default_model.clone());

        // Use Claude if available
        if let Some(ref client) = self.anthropic_client {
            let messages: Vec<_> = req
                .messages
                .iter()
                .map(|m| Message {
                    role: m.role.clone(),
                    content: m.content.clone(),
                })
                .collect();

            let response = client
                .messages()
                .create(MessagesRequest {
                    model: model.clone(),
                    max_tokens: req.max_tokens.unwrap_or(1024),
                    messages,
                    temperature: req.temperature,
                    ..Default::default()
                })
                .await?;

            let content = response
                .content
                .first()
                .and_then(|c| match c {
                    ContentBlock::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "No response".to_string());

            let tokens_used = response.usage.output_tokens as u32;
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

            return Ok(ChatResponse {
                response: content,
                model,
                tokens_used,
                latency_ms,
            });
        }

        // Fallback to mock response
        Ok(ChatResponse {
            response: "AI service not configured - please set ANTHROPIC_API_KEY".to_string(),
            model: "fallback".to_string(),
            tokens_used: 0,
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    }

    pub async fn inference(&self, req: InferenceRequest) -> anyhow::Result<InferenceResponse> {
        let start = Instant::now();
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

