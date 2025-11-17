//! 🐿️ Squirrel: The True MCP - AI/ML Service
//!
//! A production-ready AI service that implements:
//! - Model Context Protocol (MCP) server
//! - AI inference routing (Claude, GPT, local models)
//! - Distributed AI coordination
//! - Health monitoring and metrics

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

mod ai;
mod config;
mod health;
mod mcp;

use config::SquirrelConfig;

/// Main application state
#[derive(Clone)]
pub struct AppState {
    config: Arc<SquirrelConfig>,
    ai_client: Arc<ai::AIClient>,
    health: Arc<health::HealthMonitor>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("squirrel=info,tower_http=debug").init();

    info!("🐿️  Squirrel AI/MCP Service Starting...");

    // Load configuration
    let config = SquirrelConfig::from_env()?;
    info!("✅ Configuration loaded");
    info!("   Port: {}", config.port);
    info!("   AI Provider: {}", config.ai_provider);

    // Initialize AI client
    let ai_client = Arc::new(ai::AIClient::new(&config)?);
    info!("✅ AI client initialized");

    // Initialize health monitor
    let health = Arc::new(health::HealthMonitor::new());
    info!("✅ Health monitor initialized");

    // Create shared state
    let state = AppState {
        config: Arc::new(config.clone()),
        ai_client,
        health,
    };

    // Build router
    let app = Router::new()
        // Health & info
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/metrics", get(metrics_handler))
        // AI endpoints
        .route("/api/ai/chat", post(chat_handler))
        .route("/api/ai/inference", post(inference_handler))
        .route("/api/ai/models", get(models_handler))
        // MCP endpoints
        .route("/mcp/init", post(mcp_init_handler))
        .route("/mcp/tools", get(mcp_tools_handler))
        .route("/mcp/execute", post(mcp_execute_handler))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive());

    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    let actual_addr = listener.local_addr()?;

    info!("🚀 Squirrel listening on {}", actual_addr);
    info!("   Health: http://{}/health", actual_addr);
    info!("   AI Chat: http://{}/api/ai/chat", actual_addr);
    info!("   MCP: http://{}/mcp/*", actual_addr);
    info!("");
    info!("✅ Squirrel AI/MCP Service Ready!");

    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Info handler
async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let info = serde_json::json!({
        "service": "squirrel",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Squirrel AI/MCP Service",
        "capabilities": [
            "ai_chat",
            "ai_inference",
            "mcp_server",
            "model_routing"
        ],
        "ai_provider": state.config.ai_provider,
        "status": "operational"
    });

    Json(info)
}

/// Metrics handler
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.health.get_metrics();
    Json(metrics)
}

// ============================================================================
// AI Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub model: String,
    pub tokens_used: u32,
    pub latency_ms: f64,
}

/// AI chat handler
async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    info!("📨 Chat request: {} messages", req.messages.len());

    match state.ai_client.chat(req).await {
        Ok(response) => {
            info!("✅ Chat response: {} tokens", response.tokens_used);
            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Chat error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InferenceRequest {
    pub model: Option<String>,
    pub prompt: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct InferenceResponse {
    pub response: String,
    pub model: String,
    pub tokens: u32,
}

/// AI inference handler
async fn inference_handler(
    State(state): State<AppState>,
    Json(req): Json<InferenceRequest>,
) -> Result<Json<InferenceResponse>, StatusCode> {
    info!("🧠 Inference request: {} chars", req.prompt.len());

    match state.ai_client.inference(req).await {
        Ok(response) => {
            info!("✅ Inference complete: {} tokens", response.tokens);
            Ok(Json(response))
        }
        Err(e) => {
            error!("❌ Inference error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Available models handler
async fn models_handler(State(state): State<AppState>) -> impl IntoResponse {
    let models = state.ai_client.list_models();
    Json(models)
}

// ============================================================================
// MCP Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
struct McpInitRequest {
    client_info: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct McpInitResponse {
    protocol_version: String,
    capabilities: Vec<String>,
    server_info: serde_json::Value,
}

/// MCP initialization handler
async fn mcp_init_handler(
    State(_state): State<AppState>,
    Json(_req): Json<McpInitRequest>,
) -> Result<Json<McpInitResponse>, StatusCode> {
    info!("🔌 MCP init request");

    let response = McpInitResponse {
        protocol_version: "1.0".to_string(),
        capabilities: vec!["tools".to_string(), "prompts".to_string(), "resources".to_string()],
        server_info: serde_json::json!({
            "name": "squirrel",
            "version": env!("CARGO_PKG_VERSION")
        }),
    };

    Ok(Json(response))
}

/// MCP tools list handler
async fn mcp_tools_handler() -> impl IntoResponse {
    let tools = serde_json::json!({
        "tools": [
            {
                "name": "ai_chat",
                "description": "Chat with AI models",
                "parameters": {
                    "messages": "array",
                    "model": "string (optional)"
                }
            },
            {
                "name": "ai_inference",
                "description": "Run AI inference",
                "parameters": {
                    "prompt": "string",
                    "model": "string (optional)"
                }
            }
        ]
    });

    Json(tools)
}

#[derive(Debug, Deserialize)]
struct McpExecuteRequest {
    tool: String,
    arguments: serde_json::Value,
}

/// MCP tool execution handler
async fn mcp_execute_handler(
    State(state): State<AppState>,
    Json(req): Json<McpExecuteRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("⚡ MCP execute: {}", req.tool);

    match req.tool.as_str() {
        "ai_chat" => {
            // Parse and execute chat
            if let Ok(chat_req) = serde_json::from_value::<ChatRequest>(req.arguments) {
                match state.ai_client.chat(chat_req).await {
                    Ok(response) => serde_json::to_value(response)
                        .map(Json)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
        "ai_inference" => {
            // Parse and execute inference
            if let Ok(inf_req) = serde_json::from_value::<InferenceRequest>(req.arguments) {
                match state.ai_client.inference(inf_req).await {
                    Ok(response) => serde_json::to_value(response)
                        .map(Json)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
        _ => {
            warn!("Unknown MCP tool: {}", req.tool);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
