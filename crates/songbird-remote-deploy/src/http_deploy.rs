//! HTTP-based deployment client with capability negotiation
//!
//! Deploy services via Songbird's HTTP deployment API with intelligent
//! method selection based on node capabilities.

use anyhow::{anyhow, Result};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, debug, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub service_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployment_id: String,
    pub service_name: String,
    pub binary_path: String,
    pub env_vars: HashMap<String, String>,
    pub status: String,
    pub deployed_at: String,
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

// ============================================================================
// PHASE 2.2: CAPABILITY NEGOTIATION
// ============================================================================

/// Node deployment capabilities (from server)
#[derive(Debug, Deserialize)]
pub struct DeploymentCapabilities {
    pub node_id: String,
    pub timestamp: String,
    pub network: NetworkCapabilities,
    pub deployment_methods: DeploymentMethods,
    pub resources: ResourceInfo,
    pub preferences: DeploymentPreferences,
}

#[derive(Debug, Deserialize)]
pub struct NetworkCapabilities {
    #[serde(rename = "type")]
    pub network_type: String,
    pub bandwidth_estimate: BandwidthEstimate,
}

#[derive(Debug, Deserialize)]
pub struct BandwidthEstimate {
    pub download_mbps: u32,
    pub upload_mbps: u32,
    pub latency_ms: u32,
    pub confidence: String,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentMethods {
    pub single: SingleUploadMethod,
    pub chunked: ChunkedUploadMethod,
    pub streaming: StreamingUploadMethod,
}

#[derive(Debug, Deserialize)]
pub struct SingleUploadMethod {
    pub enabled: bool,
    pub max_size_mb: u32,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

#[derive(Debug, Deserialize)]
pub struct ChunkedUploadMethod {
    pub enabled: bool,
    pub max_total_size_mb: u32,
    pub chunk_size_mb: u32,
    pub max_chunks: u32,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamingUploadMethod {
    pub enabled: bool,
    pub unlimited: bool,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

#[derive(Debug, Deserialize)]
pub struct ResourceInfo {
    pub available_storage_gb: u64,
    pub available_memory_gb: u64,
    pub cpu_cores: usize,
    pub cpu_load_percent: f32,
    pub max_concurrent_deployments: usize,
    pub current_deployments: usize,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentPreferences {
    pub preferred_compression: String,
    pub preferred_method: String,
    pub encryption_required: bool,
}

/// Selected deployment method
#[derive(Debug, Clone)]
pub enum SelectedMethod {
    Single { max_size_mb: u32 },
    Chunked { chunk_size_mb: u32 },
    Streaming,
    Fallback, // Use if capabilities unavailable
}

// ============================================================================
// PHASE 3: CHUNKED UPLOAD
// ============================================================================

/// Negotiation request
#[derive(Debug, Serialize)]
struct NegotiationRequest {
    binary_size_mb: f64,
    service_name: String,
    compression: Option<String>,
}

/// Negotiation response
#[derive(Debug, Deserialize)]
struct NegotiationResponse {
    negotiation_id: String,
    accepted_method: String,
    chunk_size_mb: u32,
    total_chunks: usize,
    chunk_upload_path: String,
    finalize_path: String,
    timeout_seconds: u64,
}

/// Finalize request
#[derive(Debug, Serialize)]
struct FinalizeRequest {
    service_name: String,
    env_vars: HashMap<String, String>,
    auto_start: bool,
}

/// Query deployment capabilities from a tower
pub async fn query_capabilities(tower_endpoint: &str) -> Result<DeploymentCapabilities> {
    debug!("📊 Querying capabilities from {}", tower_endpoint);
    
    let client = Client::new();
    let url = format!("{}/api/deployment/capabilities", tower_endpoint);
    
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| anyhow!("Failed to query capabilities: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Capabilities query failed with status {}", response.status()));
    }
    
    let capabilities: DeploymentCapabilities = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse capabilities: {}", e))?;
    
    info!("✅ Capabilities received from {}", capabilities.node_id);
    debug!("   Network: {} ({} Mbps up/down)", 
        capabilities.network.network_type,
        capabilities.network.bandwidth_estimate.upload_mbps);
    debug!("   Resources: {} cores, {}GB RAM, {}GB storage",
        capabilities.resources.cpu_cores,
        capabilities.resources.available_memory_gb,
        capabilities.resources.available_storage_gb);
    
    Ok(capabilities)
}

/// Select optimal deployment method based on capabilities and binary size
pub fn select_deployment_method(
    capabilities: Option<&DeploymentCapabilities>,
    binary_size_mb: f64,
) -> SelectedMethod {
    // If capabilities unavailable, use fallback
    let Some(caps) = capabilities else {
        warn!("⚠️  Capabilities unavailable, using fallback method");
        return SelectedMethod::Fallback;
    };
    
    // Check if binary fits in single upload
    if binary_size_mb < (caps.deployment_methods.single.max_size_mb as f64) && 
       caps.deployment_methods.single.enabled {
        info!("✓ Selected: Single upload ({:.2}MB < {}MB limit)", 
            binary_size_mb, caps.deployment_methods.single.max_size_mb);
        return SelectedMethod::Single { 
            max_size_mb: caps.deployment_methods.single.max_size_mb 
        };
    }
    
    // Check if chunked is available
    if binary_size_mb < (caps.deployment_methods.chunked.max_total_size_mb as f64) && 
       caps.deployment_methods.chunked.enabled {
        info!("✓ Selected: Chunked upload ({:.2}MB, chunks of {}MB)", 
            binary_size_mb, caps.deployment_methods.chunked.chunk_size_mb);
        return SelectedMethod::Chunked { 
            chunk_size_mb: caps.deployment_methods.chunked.chunk_size_mb 
        };
    }
    
    // Check if streaming is available
    if caps.deployment_methods.streaming.enabled {
        info!("✓ Selected: Streaming upload ({:.2}MB)", binary_size_mb);
        return SelectedMethod::Streaming;
    }
    
    // Fallback
    warn!("⚠️  No suitable method found, using fallback");
    SelectedMethod::Fallback
}

/// Deploy a binary via HTTP to a remote tower (with capability negotiation)
pub async fn deploy_via_http_adaptive(
    tower_endpoint: &str,
    binary_path: &str,
    service_name: &str,
    env_vars: HashMap<String, String>,
) -> Result<DeploymentResponse> {
    info!("📤 Adaptive deployment to {}", tower_endpoint);
    
    // Get binary size
    let metadata = tokio::fs::metadata(binary_path).await?;
    let binary_size_bytes = metadata.len();
    let binary_size_mb = binary_size_bytes as f64 / 1024.0 / 1024.0;
    
    let binary_name = Path::new(binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-binary");
    
    info!("   Binary: {} ({:.2} MB)", binary_name, binary_size_mb);
    
    // Query capabilities
    let capabilities = match query_capabilities(tower_endpoint).await {
        Ok(caps) => Some(caps),
        Err(e) => {
            warn!("⚠️  Failed to query capabilities: {}", e);
            warn!("   Falling back to direct deployment");
            None
        }
    };
    
    // Select method
    let method = select_deployment_method(capabilities.as_ref(), binary_size_mb);
    
    // Execute deployment based on selected method
    match method {
        SelectedMethod::Single { .. } | SelectedMethod::Fallback => {
            // Use existing single upload
            deploy_via_http(tower_endpoint, binary_path, service_name, env_vars).await
        }
        SelectedMethod::Chunked { chunk_size_mb } => {
            // Phase 3: Use chunked upload
            deploy_via_http_chunked(tower_endpoint, binary_path, service_name, env_vars, chunk_size_mb).await
        }
        SelectedMethod::Streaming => {
            // PHASE 4 FEATURE: Streaming upload implementation planned
            // Will support: Real-time progress, resume on failure, adaptive chunking
            // Priority: Low (Phase 3 chunked upload is sufficient for most use cases)
            warn!("⚠️  Streaming upload not yet implemented, falling back to single");
            deploy_via_http(tower_endpoint, binary_path, service_name, env_vars).await
        }
    }
}

/// Deploy a binary via chunked upload
async fn deploy_via_http_chunked(
    tower_endpoint: &str,
    binary_path: &str,
    service_name: &str,
    env_vars: HashMap<String, String>,
    chunk_size_mb: u32,
) -> Result<DeploymentResponse> {
    info!("🧩 Deploying '{}' via chunked upload ({}MB chunks)", service_name, chunk_size_mb);
    
    // Read binary
    let binary_data = fs::read(binary_path).await?;
    let binary_size_mb = binary_data.len() as f64 / 1024.0 / 1024.0;
    
    info!("   Binary size: {:.2}MB", binary_size_mb);
    
    let client = Client::new();
    
    // Step 1: Negotiate
    info!("🤝 Step 1: Negotiating chunked upload...");
    let negotiation = negotiate_chunked_upload(&client, tower_endpoint, binary_size_mb, service_name).await?;
    
    info!("✅ Negotiation complete: {} chunks of {}MB", 
        negotiation.total_chunks, negotiation.chunk_size_mb);
    
    // Step 2: Split into chunks
    let chunk_size_bytes = (chunk_size_mb as usize) * 1024 * 1024;
    let chunks: Vec<&[u8]> = binary_data.chunks(chunk_size_bytes).collect();
    
    info!("📦 Step 2: Uploading {} chunks...", chunks.len());
    
    // Step 3: Upload chunks
    for (index, chunk) in chunks.iter().enumerate() {
        upload_chunk(&client, tower_endpoint, &negotiation.negotiation_id, index, chunk).await?;
        info!("   ✓ Chunk {}/{} uploaded ({} bytes)", 
            index + 1, chunks.len(), chunk.len());
    }
    
    info!("✅ All chunks uploaded");
    
    // Step 4: Finalize
    info!("🎯 Step 3: Finalizing deployment...");
    let deployment = finalize_chunked_upload(&client, tower_endpoint, &negotiation.negotiation_id, 
        service_name, env_vars).await?;
    
    info!("🎉 Chunked deployment complete: {}", deployment.deployment_id);
    
    Ok(deployment)
}

/// Negotiate chunked upload with server
async fn negotiate_chunked_upload(
    client: &Client,
    tower_endpoint: &str,
    binary_size_mb: f64,
    service_name: &str,
) -> Result<NegotiationResponse> {
    let url = format!("{}/api/deployment/negotiate", tower_endpoint);
    
    let request = NegotiationRequest {
        binary_size_mb,
        service_name: service_name.to_string(),
        compression: None,
    };
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Negotiation request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Negotiation failed with status {}: {}", status, error_text));
    }
    
    let negotiation: NegotiationResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse negotiation response: {}", e))?;
    
    Ok(negotiation)
}

/// Upload a single chunk
async fn upload_chunk(
    client: &Client,
    tower_endpoint: &str,
    negotiation_id: &str,
    chunk_index: usize,
    chunk_data: &[u8],
) -> Result<()> {
    let url = format!("{}/api/deployment/chunk/{}/{}", 
        tower_endpoint, negotiation_id, chunk_index);
    
    let form = multipart::Form::new()
        .part(
            "chunk",
            multipart::Part::bytes(chunk_data.to_vec())
                .file_name(format!("chunk-{:04}", chunk_index)),
        );
    
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("Chunk upload failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Chunk {} upload failed with status {}: {}", chunk_index, status, error_text));
    }
    
    Ok(())
}

/// Finalize chunked upload
async fn finalize_chunked_upload(
    client: &Client,
    tower_endpoint: &str,
    negotiation_id: &str,
    service_name: &str,
    env_vars: HashMap<String, String>,
) -> Result<DeploymentResponse> {
    let url = format!("{}/api/deployment/finalize/{}", tower_endpoint, negotiation_id);
    
    let request = FinalizeRequest {
        service_name: service_name.to_string(),
        env_vars,
        auto_start: true,
    };
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| anyhow!("Finalize request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Finalize failed with status {}: {}", status, error_text));
    }
    
    let deployment: DeploymentResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse deployment response: {}", e))?;
    
    Ok(deployment)
}

/// Deploy a binary via HTTP to a remote tower (legacy, direct method)
pub async fn deploy_via_http(
    tower_endpoint: &str,
    binary_path: &str,
    service_name: &str,
    env_vars: HashMap<String, String>,
) -> Result<DeploymentResponse> {
    info!("📤 Deploying '{}' to {} via HTTP", service_name, tower_endpoint);

    // Read binary file
    let binary_data = fs::read(binary_path)
        .await
        .map_err(|e| anyhow!("Failed to read binary '{}': {}", binary_path, e))?;

    let binary_filename = Path::new(binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("service");

    info!("   Binary: {} ({} bytes)", binary_filename, binary_data.len());
    info!("   Service name: {}", service_name);
    info!("   Environment vars: {}", env_vars.len());

    // Build multipart form
    let mut form = multipart::Form::new()
        .text("service_name", service_name.to_string())
        .text("env_vars", serde_json::to_string(&env_vars)?)
        .text("auto_start", "true")
        .part(
            "binary",
            multipart::Part::bytes(binary_data).file_name(binary_filename.to_string()),
        );

    // Send deployment request
    let client = Client::new();
    let url = format!("{}/api/deployment/binary", tower_endpoint);

    info!("📡 Sending deployment request to {}", url);

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Deployment failed with status {}: {}", status, error_text));
    }

    let deployment_response: DeploymentResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    info!("✅ Deployment successful: {}", deployment_response.deployment_id);
    if let Some(ref url) = deployment_response.service_url {
        info!("   Service URL: {}", url);
    }

    Ok(deployment_response)
}

/// Get deployment status
pub async fn get_deployment_status(
    tower_endpoint: &str,
    deployment_id: &str,
) -> Result<DeploymentInfo> {
    let client = Client::new();
    let url = format!("{}/api/deployment/status/{}", tower_endpoint, deployment_id);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to get deployment status: {}", response.status()));
    }

    let deployment_info: DeploymentInfo = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(deployment_info)
}

/// Stop a deployment
pub async fn stop_deployment(tower_endpoint: &str, deployment_id: &str) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/api/deployment/{}", tower_endpoint, deployment_id);

    let response = client
        .delete(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to stop deployment: {}", response.status()));
    }

    info!("✅ Deployment {} stopped", deployment_id);

    Ok(())
}

/// List all deployments on a tower
pub async fn list_deployments(tower_endpoint: &str) -> Result<Vec<DeploymentInfo>> {
    let client = Client::new();
    let url = format!("{}/api/deployment/list", tower_endpoint);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to list deployments: {}", response.status()));
    }

    let deployments: Vec<DeploymentInfo> = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(deployments)
}

