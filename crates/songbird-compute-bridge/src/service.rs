// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🌉 Songbird Compute Bridge (Capability-Based, Agnostic)
//!
//! **Philosophy:** Zero hardcoding, works with ANY compute service
//!
//! This bridge enables any compute service (Toadstool, K8s, Lambda, etc.) to
//! participate in Songbird federation by exposing a standard API and registering
//! its capabilities dynamically.
//!
//! ## Features
//! - ✅ **Agnostic:** Works with any backend compute service
//! - ✅ **Auto-Discovery:** Detects capabilities from environment/backend
//! - ✅ **Federation Integration:** Registers with Songbird automatically
//! - ✅ **Zero Hardcoding:** All configuration via environment variables
//! - ✅ **Health Monitoring:** Automatic heartbeats and health checks
//!
//! ## Environment Variables
//! ```bash
//! # Required
//! export COMPUTE_SERVICE_TYPE="compute"          # Service type
//! export COMPUTE_SERVICE_NAME="My Compute Node"  # Human-readable name
//! export COMPUTE_HOST="192.168.1.144"           # This node's address
//! export COMPUTE_PORT="9000"                     # This node's port
//! export SONGBIRD_FEDERATION_ENDPOINT="http://192.168.1.144:8080"
//!
//! # Optional - Auto-detected if not provided
//! export COMPUTE_CAPABILITIES="compute,cpu,gpu,batch-processing"
//! export COMPUTE_NODE_ID="auto-generated-uuid"
//! export COMPUTE_CPU_CORES="16"
//! export COMPUTE_MEMORY_GB="64"
//! export COMPUTE_GPU_COUNT="1"
//! export COMPUTE_GPU_MODEL="NVIDIA RTX 2070 SUPER"
//! ```

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Command-line arguments for the compute bridge HTTP service.
#[derive(Parser, Debug)]
#[command(name = "songbird-compute-bridge")]
#[command(about = "Agnostic compute service bridge for Songbird federation")]
pub struct Args {
    /// Bind address
    #[arg(long, env = "COMPUTE_HOST", default_value = "0.0.0.0")]
    host: String,

    /// Bind port
    #[arg(long, env = "COMPUTE_PORT", default_value = "9000")]
    port: u16,

    /// Service name
    #[arg(long, env = "COMPUTE_SERVICE_NAME", default_value = "Compute Service")]
    service_name: String,

    /// Service type
    #[arg(long, env = "COMPUTE_SERVICE_TYPE", default_value = "compute")]
    service_type: String,

    /// Songbird federation endpoint
    #[arg(long, env = "SONGBIRD_FEDERATION_ENDPOINT")]
    songbird_endpoint: Option<String>,

    /// Node ID (auto-generated if not provided)
    #[arg(long, env = "COMPUTE_NODE_ID")]
    node_id: Option<String>,

    /// Tower ID (for federation)
    #[arg(long, env = "COMPUTE_TOWER_ID")]
    tower_id: Option<String>,

    /// Capabilities (comma-separated)
    #[arg(long, env = "COMPUTE_CAPABILITIES")]
    capabilities: Option<String>,

    /// Backend service URL (optional - for proxying to actual compute service)
    #[arg(long, env = "COMPUTE_BACKEND_URL")]
    backend_url: Option<String>,
}

/// Bridge state
#[derive(Clone)]
struct BridgeState {
    config: Arc<BridgeConfig>,
    http_client: IpcHttpClient,
    service_info: Arc<ServiceInfo>,
}

/// Bridge configuration
#[derive(Debug, Clone)]
struct BridgeConfig {
    host: String,
    port: u16,
    service_name: String,
    service_type: String,
    node_id: String,
    tower_id: String,
    songbird_endpoint: Option<String>,
    capabilities: Vec<String>,
    backend_url: Option<String>,
}

/// Service information (auto-detected)
#[derive(Debug, Clone, Serialize)]
struct ServiceInfo {
    cpu_cores: usize,
    memory_gb: usize,
    gpu_count: usize,
    gpu_model: Option<String>,
    storage_gb: Option<usize>,
    platform: String,
}

/// Service registration payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceRegistration {
    service_id: String,
    service_name: String,
    service_type: String,
    tower_id: String,
    tower_name: String,
    endpoint: String,
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
    health_status: String,
    registered_at: String,
    last_seen: String,
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,songbird_compute_bridge=debug".to_string());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Run the compute bridge until shutdown (standalone binary or `songbird compute-bridge`).
pub async fn run(args: Args) -> anyhow::Result<()> {
    init_tracing();

    info!("🌉 Starting Songbird Compute Bridge (Agnostic)");
    info!("================================================");

    // Detect system resources
    let service_info = detect_resources().await;
    info!("📊 Detected Resources:");
    info!("   CPU Cores: {}", service_info.cpu_cores);
    info!("   Memory: {}GB", service_info.memory_gb);
    info!("   GPUs: {}", service_info.gpu_count);
    if let Some(ref gpu_model) = service_info.gpu_model {
        info!("   GPU Model: {}", gpu_model);
    }

    // Build configuration
    let node_id = args.node_id.unwrap_or_else(|| format!("compute-{}", Uuid::new_v4()));

    let tower_id =
        args.tower_id.clone().or_else(|| std::env::var("SERVICE_ID").ok()).unwrap_or_else(|| {
            hostname::get().map_or_else(
                |_| format!("tower-unknown-{}", Uuid::new_v4()),
                |h| format!("tower-{}", h.to_string_lossy()),
            )
        });

    let capabilities = args
        .capabilities
        .unwrap_or_else(|| detect_capabilities(&service_info))
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let config = Arc::new(BridgeConfig {
        host: args.host.clone(),
        port: args.port,
        service_name: args.service_name,
        service_type: args.service_type,
        node_id: node_id.clone(),
        tower_id: tower_id.clone(),
        songbird_endpoint: args.songbird_endpoint,
        capabilities,
        backend_url: args.backend_url,
    });

    info!("🔧 Configuration:");
    info!("   Node ID: {}", config.node_id);
    info!("   Tower ID: {}", config.tower_id);
    info!("   Service: {} ({})", config.service_name, config.service_type);
    info!("   Endpoint: {}:{}", args.host, args.port);
    info!("   Capabilities: {:?}", config.capabilities);
    if let Some(ref songbird) = config.songbird_endpoint {
        info!("   Songbird: {}", songbird);
    }

    // Create state
    let http_client = IpcHttpClient::new().await.map_err(|e| {
        error!("❌ Failed to create IPC HTTP client: {}", e);
        error!("   This is required for compute-bridge operation.");
        error!("   Ensure Songbird IPC socket is available.");
        anyhow::anyhow!("IPC HTTP client is required for compute-bridge operation: {e}")
    })?;

    let state = BridgeState {
        config: config.clone(),
        http_client,
        service_info: Arc::new(service_info),
    };

    // Register with Songbird (if configured)
    if let Some(ref songbird_endpoint) = config.songbird_endpoint {
        match register_with_songbird(&state, songbird_endpoint).await {
            Ok(()) => info!("✅ Registered with Songbird federation"),
            Err(e) => warn!("⚠️  Failed to register with Songbird: {}", e),
        }

        // Start heartbeat task
        let heartbeat_state = state.clone();
        let songbird_url = songbird_endpoint.clone();
        tokio::spawn(async move {
            heartbeat_loop(heartbeat_state, songbird_url).await;
        });
    }

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/capabilities", get(capabilities_handler))
        .route("/resources", get(resources_handler))
        .route("/api/v1/workloads", post(submit_workload_handler))
        .route("/api/v1/workloads/:id", get(get_workload_handler))
        .with_state(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    info!("🚀 Compute Bridge listening on {}", addr);
    info!("================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Auto-detect system resources
async fn detect_resources() -> ServiceInfo {
    use std::process::Command;

    tokio::task::yield_now().await;

    // Detect CPU cores
    let cpu_cores = num_cpus::get();

    // Detect memory (Linux-specific, fallback to estimate)
    let memory_gb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| {
            contents
                .lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|kb| kb.parse::<usize>().ok())
                .map(|kb| kb / 1024 / 1024) // Convert KB to GB
        })
        .unwrap_or(16); // Default estimate

    // Detect GPU (NVIDIA)
    let (gpu_count, gpu_model) = if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=name,count", "--format=csv,noheader"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').collect();
            let model = parts.first().map(|s| s.trim().to_string());
            let count = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1);
            (count, model)
        } else {
            (0, None)
        }
    } else {
        (0, None)
    };

    ServiceInfo {
        cpu_cores,
        memory_gb,
        gpu_count,
        gpu_model,
        storage_gb: Some(100), // Placeholder - would need proper detection
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

/// Auto-detect capabilities based on resources
fn detect_capabilities(info: &ServiceInfo) -> String {
    let mut caps = vec!["compute".to_string(), "cpu".to_string()];

    if info.gpu_count > 0 {
        caps.push("gpu".to_string());
        caps.push("ml-inference".to_string());
    }

    if info.cpu_cores >= 8 {
        caps.push("batch-processing".to_string());
    }

    if info.cpu_cores >= 32 {
        caps.push("parallel-computing".to_string());
    }

    caps.join(",")
}

/// Register with Songbird federation
async fn register_with_songbird(
    state: &BridgeState,
    songbird_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = &state.config;
    let info = &state.service_info;

    let mut metadata = HashMap::new();
    metadata.insert("cpu_cores".to_string(), info.cpu_cores.to_string());
    metadata.insert("memory_gb".to_string(), info.memory_gb.to_string());
    metadata.insert("gpu_count".to_string(), info.gpu_count.to_string());
    if let Some(ref gpu_model) = info.gpu_model {
        metadata.insert("gpu_model".to_string(), gpu_model.clone());
    }
    if let Some(storage) = info.storage_gb {
        metadata.insert("storage_gb".to_string(), storage.to_string());
    }
    metadata.insert("platform".to_string(), info.platform.clone());

    let tower_id = config.tower_id.clone();
    let registration = ServiceRegistration {
        service_id: config.node_id.clone(),
        service_name: config.service_name.clone(),
        service_type: config.service_type.clone(),
        tower_id: tower_id.clone(),
        tower_name: tower_id, // Could be enhanced with hostname
        endpoint: format!("http://{}:{}", config.host, config.port),
        capabilities: config.capabilities.clone(),
        metadata,
        health_status: "healthy".to_string(),
        registered_at: Utc::now().to_rfc3339(),
        last_seen: Utc::now().to_rfc3339(),
    };

    let url = format!("{songbird_endpoint}/api/federation/services");
    debug!("📡 Registering with Songbird: POST {}", url);

    let response = state.http_client.post(&url).await.json(&registration)?.send().await?;

    if response.is_success() {
        info!("✅ Successfully registered with Songbird");
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("Registration failed ({status}): {body}").into())
    }
}

/// Heartbeat loop - keeps registration alive
async fn heartbeat_loop(state: BridgeState, songbird_endpoint: String) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        match register_with_songbird(&state, &songbird_endpoint).await {
            Ok(()) => debug!("💓 Heartbeat sent to Songbird"),
            Err(e) => warn!("⚠️  Heartbeat failed: {}", e),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// HTTP Handlers
// ═══════════════════════════════════════════════════════════════

async fn health_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

async fn info_handler(State(state): State<BridgeState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "node_id": state.config.node_id,
        "service_name": state.config.service_name,
        "service_type": state.config.service_type,
        "capabilities": state.config.capabilities,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn capabilities_handler(State(state): State<BridgeState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "capabilities": state.config.capabilities,
    }))
}

async fn resources_handler(State(state): State<BridgeState>) -> Json<ServiceInfo> {
    Json((*state.service_info).clone())
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkloadRequest {
    #[allow(
        dead_code,
        reason = "deserialized from API request JSON; not read until routing is wired"
    )]
    name: String,
    #[allow(
        dead_code,
        reason = "deserialized from API request JSON; not read until routing is wired"
    )]
    payload: serde_json::Value,
}

async fn submit_workload_handler(
    State(state): State<BridgeState>,
    Json(request): Json<WorkloadRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // If backend_url is configured, proxy to it
    if let Some(ref backend_url) = state.config.backend_url {
        let request_result =
            state.http_client.post(&format!("{backend_url}/api/v1/workloads")).await.json(&request);

        match request_result {
            Ok(request) => match request.send().await {
                Ok(response) => {
                    let status_code = StatusCode::from_u16(response.status())
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    response.json::<serde_json::Value>().await.map_or_else(
                        |_| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(
                                    serde_json::json!({"error": "Backend response parsing failed"}),
                                ),
                            )
                        },
                        |body| (status_code, Json(body)),
                    )
                }
                Err(e) => {
                    error!("Backend request failed: {e}");
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(serde_json::json!({"error": format!("Backend unavailable: {e}")})),
                    )
                }
            },
            Err(e) => {
                error!("Failed to build request: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Request build failed: {e}")})),
                )
            }
        }
    } else {
        // No backend - return mock response
        let workload_id = Uuid::new_v4().to_string();
        (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "workload_id": workload_id,
                "status": "accepted",
                "message": "Workload accepted (no backend configured - would be queued)"
            })),
        )
    }
}

async fn get_workload_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": "Workload status not implemented"})),
    )
}
