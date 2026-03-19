//! HTTP-based deployment client with capability negotiation
//!
//! Deploy services via Songbird's HTTP deployment API with intelligent
//! method selection based on node capabilities.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use songbird_http_client::{Form, IpcHttpClient, Part};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub service_url: Option<String>,
}

/// Deployment information from server API
/// Note: Currently unused - reserved for future status queries
#[allow(dead_code)]
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
    pub network: NetworkCapabilities,
    pub deployment_methods: DeploymentMethods,
    pub resources: ResourceInfo,
}

#[derive(Debug, Deserialize)]
pub struct NetworkCapabilities {
    #[serde(rename = "type")]
    pub network_type: String,
    pub bandwidth_estimate: BandwidthEstimate,
}

#[derive(Debug, Deserialize)]
pub struct BandwidthEstimate {
    #[allow(dead_code)] // Deserialized from API response
    pub download_mbps: u32,
    pub upload_mbps: u32,
    #[allow(dead_code)] // Deserialized from API response
    pub latency_ms: u32,
    #[allow(dead_code)] // Deserialized from API response
    pub confidence: String,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentMethods {
    pub single: SingleUploadMethod,
    pub chunked: ChunkedUploadMethod,
    pub streaming: StreamingUploadMethod,
}

/// Single upload method details
#[derive(Debug, Deserialize)]
pub struct SingleUploadMethod {
    pub enabled: bool,
    pub max_size_mb: u32,
    // Future: compression negotiation
    #[allow(dead_code)]
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    #[allow(dead_code)]
    pub recommended_for: String,
}

/// Chunked upload method details
#[derive(Debug, Deserialize)]
pub struct ChunkedUploadMethod {
    pub enabled: bool,
    pub max_total_size_mb: u32,
    pub chunk_size_mb: u32,
    // Future: adaptive chunking
    #[allow(dead_code)]
    pub max_chunks: u32,
    // Future: compression negotiation
    #[allow(dead_code)]
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    #[allow(dead_code)]
    pub recommended_for: String,
}

/// Streaming upload method details
#[derive(Debug, Deserialize)]
pub struct StreamingUploadMethod {
    pub enabled: bool,
    // Future: size restrictions
    #[allow(dead_code)]
    pub unlimited: bool,
    // Future: compression support
    #[allow(dead_code)]
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    #[allow(dead_code)]
    pub recommended_for: String,
}

/// Resource info from remote tower
#[derive(Debug, Deserialize)]
pub struct ResourceInfo {
    pub available_storage_gb: u64,
    pub available_memory_gb: u64,
    pub cpu_cores: usize,
    // Future: load-based selection
    #[allow(dead_code)]
    pub cpu_load_percent: f32,
    // Future: queue management
    #[allow(dead_code)]
    pub max_concurrent_deployments: usize,
    #[allow(dead_code)]
    pub current_deployments: usize,
}

/// Deployment preferences from server
/// Note: Reserved for future intelligent method selection
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DeploymentPreferences {
    pub preferred_compression: String,
    pub preferred_method: String,
    pub encryption_required: bool,
}

/// Selected deployment method
#[derive(Debug, Clone)]
pub enum SelectedMethod {
    Single {
        // Future: size validation
        #[allow(dead_code)]
        max_size_mb: u32,
    },
    Chunked {
        chunk_size_mb: u32,
    },
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

/// Negotiation response from tower
/// Future: implement full negotiation protocol
#[derive(Debug, Deserialize)]
struct NegotiationResponse {
    negotiation_id: String,
    // Future: method validation
    #[allow(dead_code)]
    accepted_method: String,
    chunk_size_mb: u32,
    total_chunks: usize,
    // Future: dynamic endpoint routing
    #[allow(dead_code)]
    chunk_upload_path: String,
    #[allow(dead_code)]
    finalize_path: String,
    #[allow(dead_code)]
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

    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;
    let url = format!("{tower_endpoint}/api/deployment/capabilities");

    let response =
        client.get(&url).await.map_err(|e| anyhow!("Failed to query capabilities: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        return Err(anyhow!("Capabilities query failed with status {status}"));
    }

    let capabilities: DeploymentCapabilities =
        response.json().await.map_err(|e| anyhow!("Failed to parse capabilities: {e}"))?;

    info!("✅ Capabilities received from {}", capabilities.node_id);
    debug!(
        "   Network: {} ({} Mbps up/down)",
        capabilities.network.network_type, capabilities.network.bandwidth_estimate.upload_mbps
    );
    debug!(
        "   Resources: {} cores, {}GB RAM, {}GB storage",
        capabilities.resources.cpu_cores,
        capabilities.resources.available_memory_gb,
        capabilities.resources.available_storage_gb
    );

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
    if binary_size_mb < f64::from(caps.deployment_methods.single.max_size_mb)
        && caps.deployment_methods.single.enabled
    {
        info!(
            "✓ Selected: Single upload ({:.2}MB < {}MB limit)",
            binary_size_mb, caps.deployment_methods.single.max_size_mb
        );
        return SelectedMethod::Single {
            max_size_mb: caps.deployment_methods.single.max_size_mb,
        };
    }

    // Check if chunked is available
    if binary_size_mb < f64::from(caps.deployment_methods.chunked.max_total_size_mb)
        && caps.deployment_methods.chunked.enabled
    {
        info!(
            "✓ Selected: Chunked upload ({:.2}MB, chunks of {}MB)",
            binary_size_mb, caps.deployment_methods.chunked.chunk_size_mb
        );
        return SelectedMethod::Chunked {
            chunk_size_mb: caps.deployment_methods.chunked.chunk_size_mb,
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
    let binary_size_mb = f64::from(
        u32::try_from((binary_size_bytes / 1024 / 1024).min(u64::from(u32::MAX)))
            .unwrap_or(u32::MAX),
    );

    let binary_name =
        Path::new(binary_path).file_name().and_then(|n| n.to_str()).unwrap_or("unknown-binary");

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
        SelectedMethod::Single {
            ..
        }
        | SelectedMethod::Fallback => {
            // Use existing single upload
            deploy_via_http(tower_endpoint, binary_path, service_name, env_vars).await
        }
        SelectedMethod::Chunked {
            chunk_size_mb,
        } => {
            // Phase 3: Use chunked upload
            deploy_via_http_chunked(
                tower_endpoint,
                binary_path,
                service_name,
                env_vars,
                chunk_size_mb,
            )
            .await
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
    let binary_size_mb = f64::from(
        u32::try_from((binary_data.len() / 1024 / 1024).min(u32::MAX as usize)).unwrap_or(u32::MAX),
    );

    info!("   Binary size: {:.2}MB", binary_size_mb);

    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;

    // Step 1: Negotiate
    info!("🤝 Step 1: Negotiating chunked upload...");
    let negotiation =
        negotiate_chunked_upload(&client, tower_endpoint, binary_size_mb, service_name).await?;

    info!(
        "✅ Negotiation complete: {} chunks of {}MB",
        negotiation.total_chunks, negotiation.chunk_size_mb
    );

    // Step 2: Split into chunks
    let chunk_size_bytes = (chunk_size_mb as usize) * 1024 * 1024;
    let chunks: Vec<&[u8]> = binary_data.chunks(chunk_size_bytes).collect();

    info!("📦 Step 2: Uploading {} chunks...", chunks.len());

    // Step 3: Upload chunks
    for (index, chunk) in chunks.iter().enumerate() {
        upload_chunk(&client, tower_endpoint, &negotiation.negotiation_id, index, chunk).await?;
        info!("   ✓ Chunk {}/{} uploaded ({} bytes)", index + 1, chunks.len(), chunk.len());
    }

    info!("✅ All chunks uploaded");

    // Step 4: Finalize
    info!("🎯 Step 3: Finalizing deployment...");
    let deployment = finalize_chunked_upload(
        &client,
        tower_endpoint,
        &negotiation.negotiation_id,
        service_name,
        env_vars,
    )
    .await?;

    info!("🎉 Chunked deployment complete: {}", deployment.deployment_id);

    Ok(deployment)
}

/// Negotiate chunked upload with server
async fn negotiate_chunked_upload(
    client: &IpcHttpClient,
    tower_endpoint: &str,
    binary_size_mb: f64,
    service_name: &str,
) -> Result<NegotiationResponse> {
    let url = format!("{tower_endpoint}/api/deployment/negotiate");

    let request = NegotiationRequest {
        binary_size_mb,
        service_name: service_name.to_string(),
        compression: None,
    };

    let response = client
        .post(&url)
        .await
        .json(&request)?
        .send()
        .await
        .map_err(|e| anyhow!("Negotiation request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Negotiation failed with status {status}: {error_text}"));
    }

    let negotiation: NegotiationResponse =
        response.json().await.map_err(|e| anyhow!("Failed to parse negotiation response: {e}"))?;

    Ok(negotiation)
}

/// Upload a single chunk
async fn upload_chunk(
    client: &IpcHttpClient,
    tower_endpoint: &str,
    negotiation_id: &str,
    chunk_index: usize,
    chunk_data: &[u8],
) -> Result<()> {
    let url = format!("{tower_endpoint}/api/deployment/chunk/{negotiation_id}/{chunk_index}");

    let form = Form::new().part(
        "chunk",
        Part::bytes(chunk_data.to_vec()).file_name(format!("chunk-{chunk_index:04}")),
    );

    let response = client
        .post(&url)
        .await
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("Chunk upload failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!(
            "Chunk {chunk_index} upload failed with status {status}: {error_text}"
        ));
    }

    Ok(())
}

/// Finalize chunked upload
async fn finalize_chunked_upload(
    client: &IpcHttpClient,
    tower_endpoint: &str,
    negotiation_id: &str,
    service_name: &str,
    env_vars: HashMap<String, String>,
) -> Result<DeploymentResponse> {
    let url = format!("{tower_endpoint}/api/deployment/finalize/{negotiation_id}");

    let request = FinalizeRequest {
        service_name: service_name.to_string(),
        env_vars,
        auto_start: true,
    };

    let response = client
        .post(&url)
        .await
        .json(&request)?
        .send()
        .await
        .map_err(|e| anyhow!("Finalize request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Finalize failed with status {status}: {error_text}"));
    }

    let deployment: DeploymentResponse =
        response.json().await.map_err(|e| anyhow!("Failed to parse deployment response: {e}"))?;

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
        .map_err(|e| anyhow!("Failed to read binary '{binary_path}': {e}"))?;

    let binary_filename =
        Path::new(binary_path).file_name().and_then(|n| n.to_str()).unwrap_or("service");

    info!("   Binary: {} ({} bytes)", binary_filename, binary_data.len());
    info!("   Service name: {}", service_name);
    info!("   Environment vars: {}", env_vars.len());

    // Build multipart form
    let form = Form::new()
        .text("service_name", service_name.to_string())
        .text("env_vars", serde_json::to_string(&env_vars)?)
        .text("auto_start", "true")
        .part("binary", Part::bytes(binary_data).file_name(binary_filename.to_string()));

    // Send deployment request
    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;
    let url = format!("{tower_endpoint}/api/deployment/binary");

    info!("📡 Sending deployment request to {}", url);

    let response = client
        .post(&url)
        .await
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Deployment failed with status {status}: {error_text}"));
    }

    let deployment_response: DeploymentResponse =
        response.json().await.map_err(|e| anyhow!("Failed to parse response: {e}"))?;

    info!("✅ Deployment successful: {}", deployment_response.deployment_id);
    if let Some(ref url) = deployment_response.service_url {
        info!("   Service URL: {}", url);
    }

    Ok(deployment_response)
}

/// Get deployment status
/// Future: implement full deployment monitoring
#[allow(dead_code)]
pub async fn get_deployment_status(
    tower_endpoint: &str,
    deployment_id: &str,
) -> Result<DeploymentInfo> {
    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;
    let url = format!("{tower_endpoint}/api/deployment/status/{deployment_id}");

    let response = client.get(&url).await.map_err(|e| anyhow!("HTTP request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        return Err(anyhow!("Failed to get deployment status: {status}"));
    }

    let deployment_info: DeploymentInfo =
        response.json().await.map_err(|e| anyhow!("Failed to parse response: {e}"))?;

    Ok(deployment_info)
}

/// Stop a deployment
/// Stop a running deployment
/// Future: implement deployment lifecycle management
#[allow(dead_code)]
pub async fn stop_deployment(tower_endpoint: &str, deployment_id: &str) -> Result<()> {
    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;
    let url = format!("{tower_endpoint}/api/deployment/{deployment_id}");

    let response = client.delete(&url).await.map_err(|e| anyhow!("HTTP request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        return Err(anyhow!("Failed to stop deployment: {status}"));
    }

    info!("✅ Deployment {} stopped", deployment_id);

    Ok(())
}

/// List all deployments on a tower
/// List all deployments on a tower
/// Future: implement deployment inventory
#[allow(dead_code)]
pub async fn list_deployments(tower_endpoint: &str) -> Result<Vec<DeploymentInfo>> {
    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;
    let url = format!("{tower_endpoint}/api/deployment/list");

    let response = client.get(&url).await.map_err(|e| anyhow!("HTTP request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        return Err(anyhow!("Failed to list deployments: {status}"));
    }

    let deployments: Vec<DeploymentInfo> =
        response.json().await.map_err(|e| anyhow!("Failed to parse response: {e}"))?;

    Ok(deployments)
}
