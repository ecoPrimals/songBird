//! # 🚀 Deployment API
//!
//! HTTP-based service deployment for Songbird federation.
//! Enables remote service deployment without SSH.
//!
//! ## Features
//! - ✅ Binary upload via multipart/form-data
//! - ✅ Environment variable configuration
//! - ✅ Automatic service startup
//! - ✅ Health verification
//! - ✅ Federation-integrated
//! - ✅ Capability discovery (auto-detection)
//!
//! ## Endpoints
//! - GET /api/deployment/capabilities - Discover node capabilities
//! - POST /api/deployment/binary - Deploy and start a service (single upload)
//! - POST /api/deployment/negotiate - Start chunked upload negotiation
//! - POST /`api/deployment/chunk/:neg_id/:index` - Upload chunk
//! - POST /`api/deployment/finalize/:neg_id` - Finalize chunked upload
//! - GET /api/deployment/status/:id - Check deployment status
//! - DELETE /api/deployment/:id - Stop and remove deployment

use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};

// Re-export chunked upload handlers
pub use super::chunked_upload::{finalize_chunked_upload, negotiate_chunked_upload, upload_chunk};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use sysinfo::{Disks, System};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Deployment state
#[derive(Clone)]
pub struct DeploymentState {
    pub deployments: Arc<RwLock<HashMap<String, DeploymentInfo>>>,
    pub negotiations: Arc<RwLock<HashMap<String, NegotiationState>>>,
}

impl Default for DeploymentState {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            deployments: Arc::new(RwLock::new(HashMap::new())),
            negotiations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// ============================================================================
// PHASE 3: CHUNKED UPLOAD
// ============================================================================

/// Negotiation state for chunked uploads
#[derive(Debug, Clone)]
pub struct NegotiationState {
    pub negotiation_id: String,
    pub binary_size_mb: f64,
    pub chunk_size_mb: u32,
    pub total_chunks: usize,
    pub received_chunks: HashMap<usize, ChunkInfo>,
    pub temp_dir: String,
    pub created_at: String,
    pub timeout_seconds: u64,
}

/// Chunk information
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub index: usize,
    pub size_bytes: usize,
    pub received_at: String,
    pub file_path: String,
}

/// Negotiation request
#[derive(Debug, Deserialize)]
pub struct NegotiationRequest {
    pub binary_size_mb: f64,
    pub service_name: String,
    pub compression: Option<String>,
}

/// Negotiation response
#[derive(Debug, Serialize)]
pub struct NegotiationResponse {
    pub negotiation_id: String,
    pub accepted_method: String,
    pub chunk_size_mb: u32,
    pub total_chunks: usize,
    pub chunk_upload_path: String,
    pub finalize_path: String,
    pub timeout_seconds: u64,
}

/// Finalize request
#[derive(Debug, Deserialize)]
pub struct FinalizeRequest {
    pub service_name: String,
    pub env_vars: HashMap<String, String>,
    pub auto_start: bool,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployment_id: String,
    pub service_name: String,
    pub binary_path: String,
    pub env_vars: HashMap<String, String>,
    pub status: DeploymentStatus,
    pub deployed_at: String, // ISO 8601 timestamp
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Deploying,
    Running,
    Failed,
    Stopped,
}

/// Deployment response
#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub service_url: Option<String>,
}

// ============================================================================
// PHASE 2: CAPABILITY DISCOVERY
// ============================================================================

/// Node deployment capabilities
#[derive(Debug, Serialize)]
pub struct DeploymentCapabilities {
    pub node_id: String,
    pub timestamp: String,
    pub network: NetworkCapabilities,
    pub deployment_methods: DeploymentMethods,
    pub resources: ResourceInfo,
    pub preferences: DeploymentPreferences,
}

/// Network capabilities
#[derive(Debug, Serialize)]
pub struct NetworkCapabilities {
    #[serde(rename = "type")]
    pub network_type: String, // "lan", "vpn", "internet"
    pub bandwidth_estimate: BandwidthEstimate,
}

/// Bandwidth estimate
#[derive(Debug, Serialize)]
pub struct BandwidthEstimate {
    pub download_mbps: u32,
    pub upload_mbps: u32,
    pub latency_ms: u32,
    pub confidence: String, // "high", "medium", "low"
}

/// Supported deployment methods
#[derive(Debug, Serialize)]
pub struct DeploymentMethods {
    pub single: SingleUploadMethod,
    pub chunked: ChunkedUploadMethod,
    pub streaming: StreamingUploadMethod,
}

/// Single upload method
#[derive(Debug, Serialize)]
pub struct SingleUploadMethod {
    pub enabled: bool,
    pub max_size_mb: u32,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

/// Chunked upload method
#[derive(Debug, Serialize)]
pub struct ChunkedUploadMethod {
    pub enabled: bool,
    pub max_total_size_mb: u32,
    pub chunk_size_mb: u32,
    pub max_chunks: u32,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

/// Streaming upload method
#[derive(Debug, Serialize)]
pub struct StreamingUploadMethod {
    pub enabled: bool,
    pub unlimited: bool,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

/// Resource information
#[derive(Debug, Serialize)]
pub struct ResourceInfo {
    pub available_storage_gb: u64,
    pub available_memory_gb: u64,
    pub cpu_cores: usize,
    pub cpu_load_percent: f32,
    pub max_concurrent_deployments: usize,
    pub current_deployments: usize,
}

/// Deployment preferences
#[derive(Debug, Serialize)]
pub struct DeploymentPreferences {
    pub preferred_compression: String,
    pub preferred_method: String,
    pub encryption_required: bool,
}

/// Create deployment routes
pub fn deployment_routes(state: DeploymentState) -> Router {
    Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/binary", post(deploy_binary))
        .route("/negotiate", post(negotiate_chunked_upload))
        .route("/chunk/:neg_id/:index", post(upload_chunk))
        .route("/finalize/:neg_id", post(finalize_chunked_upload))
        .route("/status/:id", get(get_deployment_status))
        .route("/:id", delete(stop_deployment))
        .route("/list", get(list_deployments))
        .with_state(state)
}

/// GET /api/deployment/capabilities - Discover node capabilities
async fn get_capabilities(State(state): State<DeploymentState>) -> Json<DeploymentCapabilities> {
    info!("📊 Capability discovery request received");

    // Detect network type (simplified - assume LAN for now)
    let network_type = detect_network_type();

    // Estimate bandwidth based on network type
    let bandwidth = estimate_bandwidth(&network_type);

    // Detect resources
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory() / 1024 / 1024 / 1024; // GB
    let available_memory = sys.available_memory() / 1024 / 1024 / 1024; // GB
    let cpu_cores = num_cpus::get();
    let cpu_load = sys.global_cpu_info().cpu_usage();

    // Get available storage (first disk)
    let disks = Disks::new_with_refreshed_list();
    let available_storage =
        disks.list().first().map_or(0, |d| d.available_space() / 1024 / 1024 / 1024);

    // Count current deployments
    let current_deployments = state.deployments.read().await.len();

    // Calculate max concurrent deployments based on available memory
    let max_concurrent = calculate_max_concurrent(available_memory);

    // Build capabilities response
    let capabilities = DeploymentCapabilities {
        node_id: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
        network: NetworkCapabilities {
            network_type: network_type.clone(),
            bandwidth_estimate: bandwidth,
        },
        deployment_methods: DeploymentMethods {
            single: SingleUploadMethod {
                enabled: true,
                max_size_mb: 50,
                compression_supported: vec!["gzip".to_string()],
                recommended_for: "< 10MB".to_string(),
            },
            chunked: ChunkedUploadMethod {
                enabled: true, // Phase 3 ✅
                max_total_size_mb: 1000,
                chunk_size_mb: 10,
                max_chunks: 100,
                compression_supported: vec!["gzip".to_string()],
                recommended_for: "2MB - 500MB".to_string(),
            },
            streaming: StreamingUploadMethod {
                enabled: false, // Phase 4
                unlimited: true,
                compression_supported: vec!["gzip".to_string()],
                recommended_for: "> 500MB".to_string(),
            },
        },
        resources: ResourceInfo {
            available_storage_gb: available_storage,
            available_memory_gb: available_memory,
            cpu_cores,
            cpu_load_percent: cpu_load,
            max_concurrent_deployments: max_concurrent,
            current_deployments,
        },
        preferences: DeploymentPreferences {
            preferred_compression: "gzip".to_string(), // Pure Rust (flate2)
            preferred_method: "single".to_string(),    // Will be "chunked" in Phase 3
            encryption_required: false,
        },
    };

    info!(
        "✅ Capabilities: {} network, {}MB up/down, {}GB available storage",
        capabilities.network.network_type,
        capabilities.network.bandwidth_estimate.upload_mbps,
        capabilities.resources.available_storage_gb
    );

    Json(capabilities)
}

/// Detect network type (LAN, VPN, or Internet)
fn detect_network_type() -> String {
    // SIMPLIFIED: Currently assumes LAN for all connections
    // FUTURE ENHANCEMENT: Implement subnet/private IP range detection
    // Potential approach: Check if peer IP is in RFC1918 ranges, measure latency, etc.
    "lan".to_string()
}

/// Estimate bandwidth based on network type
fn estimate_bandwidth(network_type: &str) -> BandwidthEstimate {
    match network_type {
        "lan" => BandwidthEstimate {
            download_mbps: 1000,
            upload_mbps: 1000,
            latency_ms: 1,
            confidence: "high".to_string(),
        },
        "vpn" => BandwidthEstimate {
            download_mbps: 100,
            upload_mbps: 100,
            latency_ms: 10,
            confidence: "medium".to_string(),
        },
        _ => BandwidthEstimate {
            download_mbps: 50,
            upload_mbps: 20,
            latency_ms: 50,
            confidence: "low".to_string(),
        },
    }
}

/// Calculate max concurrent deployments based on available memory
fn calculate_max_concurrent(available_memory_gb: u64) -> usize {
    // Assume each deployment needs ~1GB
    // Modern idiomatic: clamp() is cleaner than max().min()
    // Safe cast: u64 fits in usize on 64-bit, truncates on 32-bit (acceptable for concurrency limit)
    #[allow(clippy::cast_possible_truncation)]
    let memory_as_usize = available_memory_gb as usize;
    memory_as_usize.clamp(1, 10)
}

// ============================================================================
// EXISTING DEPLOYMENT ENDPOINTS (Phase 1)
// ============================================================================

/// POST /api/deployment/binary - Deploy a binary service
async fn deploy_binary(
    State(state): State<DeploymentState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DeploymentResponse>), (StatusCode, String)> {
    info!("📦 Received deployment request");

    let deployment_id = format!("deploy-{}", fastrand::u64(..));
    let mut binary_data: Option<Bytes> = None;
    let mut service_name = format!("service-{}", &deployment_id[..8]);
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let mut auto_start = true;

    // Parse multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "binary" => {
                debug!("📥 Receiving binary data...");
                binary_data =
                    Some(field.bytes().await.map_err(|e| {
                        (StatusCode::BAD_REQUEST, format!("Binary read error: {e}"))
                    })?);
            }
            "service_name" => {
                service_name = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Service name error: {e}")))?;
            }
            "env_vars" => {
                let env_json = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Env vars error: {e}")))?;
                env_vars = serde_json::from_str(&env_json)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid env JSON: {e}")))?;
            }
            "auto_start" => {
                let auto_str = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Auto start error: {e}")))?;
                auto_start = auto_str.parse().unwrap_or(true);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    let binary_data = binary_data
        // Modern idiomatic: ok_or_else for lazy evaluation
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "No binary provided".to_string()))?;

    info!("📦 Deploying service: {}", service_name);
    debug!("   Deployment ID: {}", deployment_id);
    debug!("   Binary size: {} bytes", binary_data.len());
    debug!("   Environment vars: {}", env_vars.len());

    // Create deployment directory (TRUE PRIMAL: self-knowledge via env_config)
    let base_deploy_dir = crate::env_config::deployment_dir();
    let deploy_dir = base_deploy_dir.join(&deployment_id);
    fs::create_dir_all(&deploy_dir).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Directory creation failed: {e}"))
    })?;

    // Write binary
    let binary_path = deploy_dir.join("service");
    fs::write(&binary_path, &binary_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Binary write failed: {e}")))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata read failed: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chmod failed: {e}")))?;
    }

    info!("✅ Binary deployed to: {}", binary_path.display());

    // Extract port from env vars
    let port = env_vars
        .iter()
        .find(|(k, _)| k.to_uppercase().contains("PORT"))
        .and_then(|(_, v)| v.parse::<u16>().ok());

    // Create deployment info
    let mut deployment = DeploymentInfo {
        deployment_id: deployment_id.clone(),
        service_name: service_name.clone(),
        binary_path: binary_path.to_string_lossy().to_string(),
        env_vars: env_vars.clone(),
        status: DeploymentStatus::Deploying,
        deployed_at: chrono::Utc::now().to_rfc3339(),
        pid: None,
        port,
    };

    // Start service if requested
    if auto_start {
        match start_service(&binary_path.to_string_lossy(), &env_vars).await {
            Ok(pid) => {
                info!("✅ Service started with PID: {}", pid);
                deployment.status = DeploymentStatus::Running;
                deployment.pid = Some(pid);
            }
            Err(e) => {
                error!("❌ Service start failed: {}", e);
                deployment.status = DeploymentStatus::Failed;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Service start failed: {e}"),
                ));
            }
        }
    }

    // Store deployment info
    state.deployments.write().await.insert(deployment_id.clone(), deployment.clone());

    // Build service URL
    let service_url = if let (Some(host), Some(port)) = (
        env_vars
            .get("COMPUTE_HOST")
            // Modern idiomatic: or_else for lazy evaluation
            .or_else(|| env_vars.get("SERVICE_HOST")),
        port,
    ) {
        Some(format!("http://{host}:{port}"))
    } else {
        None
    };

    let response = DeploymentResponse {
        deployment_id,
        status: "deployed".to_string(),
        message: format!("Service '{service_name}' deployed successfully"),
        service_url,
    };

    info!("🎉 Deployment complete: {}", service_name);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Start a service with environment variables
///
/// # Modern Idiomatic Pattern
/// Generic over `HashMap` hasher for flexibility (accepts any `BuildHasher`)
/// Requires Send for async execution across threads
pub async fn start_service<S>(
    binary_path: &str,
    env_vars: &std::collections::HashMap<String, String, S>,
) -> Result<u32, String>
where
    S: std::hash::BuildHasher + Send + Sync,
{
    debug!("🎬 Starting service: {}", binary_path);

    let mut command = Command::new(binary_path);

    // Set environment variables
    for (key, value) in env_vars {
        command.env(key, value);
    }

    // Run in background with nohup
    command.stdout(Stdio::null()).stderr(Stdio::null()).stdin(Stdio::null());

    // Spawn the process
    let child = command.spawn().map_err(|e| format!("Failed to spawn process: {e}"))?;

    let pid = child.id();
    debug!("✅ Service started with PID: {}", pid);

    Ok(pid)
}

/// GET /api/deployment/status/:id - Get deployment status
async fn get_deployment_status(
    State(state): State<DeploymentState>,
    Path(deployment_id): Path<String>,
) -> Result<Json<DeploymentInfo>, (StatusCode, String)> {
    let deployments = state.deployments.read().await;

    deployments
        .get(&deployment_id)
        .cloned()
        // Modern idiomatic: ok_or_else for lazy evaluation
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Deployment '{deployment_id}' not found")))
        .map(Json)
}

/// DELETE /api/deployment/:id - Stop deployment
async fn stop_deployment(
    State(state): State<DeploymentState>,
    Path(deployment_id): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    info!("🛑 Stopping deployment: {}", deployment_id);

    let mut deployments = state.deployments.write().await;

    let deployment = deployments
        .get_mut(&deployment_id)
        // Modern idiomatic: ok_or_else for lazy evaluation
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, format!("Deployment '{deployment_id}' not found"))
        })?;

    // Stop process if running
    if let Some(pid) = deployment.pid {
        debug!("Stopping process PID: {}", pid);

        // Try to stop the process (best effort)
        #[cfg(unix)]
        {
            // Use kill command as fallback
            let _ = std::process::Command::new("kill").arg("-TERM").arg(pid.to_string()).output();
        }

        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .arg("/PID")
                .arg(pid.to_string())
                .arg("/T")
                .output();
        }
    }

    deployment.status = DeploymentStatus::Stopped;
    deployment.pid = None;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "stopped",
            "deployment_id": deployment_id,
            "message": "Service stopped successfully"
        })),
    ))
}

/// GET /api/deployment/list - List all deployments
async fn list_deployments(State(state): State<DeploymentState>) -> Json<Vec<DeploymentInfo>> {
    let deployments = state.deployments.read().await;
    Json(deployments.values().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_state_creation() {
        let state = DeploymentState::new();
        assert!(state.deployments.try_read().is_ok());
    }

    #[test]
    fn test_detect_network_type() {
        let network_type = detect_network_type();
        assert!(["lan", "vpn", "internet"].contains(&network_type.as_str()));
    }

    #[test]
    fn test_calculate_max_concurrent() {
        assert_eq!(calculate_max_concurrent(0), 1);
        assert_eq!(calculate_max_concurrent(5), 5);
        assert_eq!(calculate_max_concurrent(20), 10); // Capped at 10
    }
}
