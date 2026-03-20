// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP-based deployment client with capability negotiation
//!
//! Deploy services via Songbird's HTTP deployment API with intelligent
//! method selection based on node capabilities.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use songbird_http_client::{Form, IpcHttpClient, Part};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};

/// JSON body returned by the tower after accepting a deployment.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentResponse {
    /// Server-assigned deployment identifier.
    pub deployment_id: String,
    /// High-level outcome (`running`, `failed`, etc., as defined by the API).
    pub status: String,
    /// Human-readable status or error detail.
    pub message: String,
    /// Reachable URL for the deployed service, if the tower provides one.
    pub service_url: Option<String>,
}

/// Deployment information from server API
/// Note: Currently unused - reserved for future status queries
#[allow(dead_code, reason = "reserved for future deployment status API")]
#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Deployment identifier.
    pub deployment_id: String,
    /// Logical service name on the tower.
    pub service_name: String,
    /// Path to the deployed binary on the remote host.
    pub binary_path: String,
    /// Environment variables passed at deploy time.
    pub env_vars: HashMap<String, String>,
    /// Current lifecycle status string from the tower.
    pub status: String,
    /// RFC3339 or server-specific deployment timestamp.
    pub deployed_at: String,
    /// Remote process ID when running under process supervision.
    pub pid: Option<u32>,
    /// Listening port when exposed by the tower.
    pub port: Option<u16>,
}

// ============================================================================
// PHASE 2.2: CAPABILITY NEGOTIATION
// ============================================================================

/// Deployment-related limits and methods advertised by a tower (`/api/deployment/capabilities`).
#[derive(Debug, Deserialize)]
pub struct DeploymentCapabilities {
    /// Tower node reporting these capabilities.
    pub node_id: String,
    /// Observed network classification and throughput hints.
    pub network: NetworkCapabilities,
    /// Which upload strategies the tower supports.
    pub deployment_methods: DeploymentMethods,
    /// Free resources relevant to accepting a deployment.
    pub resources: ResourceInfo,
}

/// Network metadata included in [`DeploymentCapabilities`].
#[derive(Debug, Deserialize)]
pub struct NetworkCapabilities {
    /// Network class label from the tower (e.g. `lan`, `wan`).
    #[serde(rename = "type")]
    pub network_type: String,
    /// Rough bandwidth and latency estimate for method selection.
    pub bandwidth_estimate: BandwidthEstimate,
}

/// Throughput and latency hints returned with [`DeploymentCapabilities`].
#[derive(Debug, Deserialize)]
pub struct BandwidthEstimate {
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Estimated download throughput in Mbps.
    pub download_mbps: u32,
    /// Estimated upload throughput in Mbps.
    pub upload_mbps: u32,
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Estimated round-trip latency in milliseconds.
    pub latency_ms: u32,
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Qualitative confidence label for the estimate (tower-defined).
    pub confidence: String,
}

/// Per-method upload constraints returned by the tower.
#[derive(Debug, Deserialize)]
pub struct DeploymentMethods {
    /// One-shot multipart upload limits.
    pub single: SingleUploadMethod,
    /// Chunked upload limits and chunk sizing.
    pub chunked: ChunkedUploadMethod,
    /// Streaming upload availability (may be disabled server-side).
    pub streaming: StreamingUploadMethod,
}

/// Single upload method details
#[derive(Debug, Deserialize)]
pub struct SingleUploadMethod {
    /// Whether single-request upload is allowed.
    pub enabled: bool,
    /// Maximum artifact size for a single upload, in MiB.
    pub max_size_mb: u32,
    // Future: compression negotiation
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Compression codecs advertised for single-request uploads.
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Tower hint for when single upload is preferred (e.g. small artifacts).
    pub recommended_for: String,
}

/// Chunked upload method details
#[derive(Debug, Deserialize)]
pub struct ChunkedUploadMethod {
    /// Whether chunked upload is allowed.
    pub enabled: bool,
    /// Maximum total artifact size across chunks, in MiB.
    pub max_total_size_mb: u32,
    /// Preferred chunk size, in MiB.
    pub chunk_size_mb: u32,
    // Future: adaptive chunking
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Maximum number of chunks allowed for one deployment.
    pub max_chunks: u32,
    // Future: compression negotiation
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Compression codecs advertised for chunked uploads.
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Tower hint for when chunked upload is preferred.
    pub recommended_for: String,
}

/// Streaming upload method details
#[derive(Debug, Deserialize)]
pub struct StreamingUploadMethod {
    /// Whether streaming upload is implemented on the tower.
    pub enabled: bool,
    // Future: size restrictions
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Whether the tower treats streaming uploads as unbounded in size.
    pub unlimited: bool,
    // Future: compression support
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Compression codecs advertised for streaming uploads.
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Tower hint for when streaming upload is preferred.
    pub recommended_for: String,
}

/// Resource snapshot used to decide whether a deployment can proceed.
#[derive(Debug, Deserialize)]
pub struct ResourceInfo {
    /// Free disk space available for staging artifacts.
    pub available_storage_gb: u64,
    /// Free RAM available for the deployment pipeline.
    pub available_memory_gb: u64,
    /// Logical CPU cores available to new workloads.
    pub cpu_cores: usize,
    // Future: load-based selection
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Recent CPU utilization on the tower (0.0–1.0 or percent, tower-defined).
    pub cpu_load_percent: f32,
    // Future: queue management
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Maximum deployments the tower will run concurrently.
    pub max_concurrent_deployments: usize,
    #[allow(dead_code, reason = "deserialized from API response")]
    /// Deployments currently active on the tower.
    pub current_deployments: usize,
}

/// Deployment preferences from server
/// Note: Reserved for future intelligent method selection
#[allow(dead_code, reason = "reserved for API response handling")]
#[derive(Debug, Deserialize)]
pub struct DeploymentPreferences {
    /// Preferred compression codec name from the tower.
    pub preferred_compression: String,
    /// Preferred upload method name when multiple apply.
    pub preferred_method: String,
    /// Whether the tower requires TLS or payload encryption.
    pub encryption_required: bool,
}

/// Upload strategy chosen from [`select_deployment_method`].
#[derive(Debug, Clone)]
pub enum SelectedMethod {
    /// Single HTTP request within `max_size_mb`.
    Single {
        // Future: size validation
        /// Maximum artifact size for this single upload (MiB), from tower limits.
        #[allow(dead_code, reason = "reserved for future size validation")]
        max_size_mb: u32,
    },
    /// Chunked upload using the given chunk size in MiB.
    Chunked {
        /// Chunk size to use for each part (MiB).
        chunk_size_mb: u32,
    },
    /// Streaming upload (falls back if not implemented client-side).
    Streaming,
    /// Capabilities missing or unusable; use a best-effort simple upload.
    Fallback,
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
    #[expect(dead_code, reason = "reserved for API response handling")]
    accepted_method: String,
    chunk_size_mb: u32,
    total_chunks: usize,
    // Future: dynamic endpoint routing
    #[expect(dead_code, reason = "reserved for API response handling")]
    chunk_upload_path: String,
    #[expect(dead_code, reason = "reserved for API response handling")]
    finalize_path: String,
    #[expect(dead_code, reason = "reserved for API response handling")]
    timeout_seconds: u64,
}

/// Finalize request
#[derive(Debug, Serialize)]
struct FinalizeRequest {
    service_name: String,
    env_vars: HashMap<String, String>,
    auto_start: bool,
}

/// Fetches [`DeploymentCapabilities`] from `GET {tower}/api/deployment/capabilities`.
///
/// # Errors
///
/// Returns an error if the HTTP client cannot be created, the request fails, or JSON parsing fails.
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

/// Picks single, chunked, streaming, or fallback upload based on size and tower limits.
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

/// Deploys a binary using negotiated upload strategy (queries capabilities when possible).
///
/// # Errors
///
/// Propagates I/O, HTTP, and JSON errors from the underlying client and tower responses.
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

/// Single-request multipart deploy to `POST {tower}/api/deployment/binary`.
///
/// # Errors
///
/// Returns an error if the binary cannot be read, the multipart request fails, or the response is not success JSON.
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

/// Fetches deployment status from `GET .../api/deployment/status/{id}`.
///
/// # Errors
///
/// Returns an error on HTTP failure or JSON parse errors.
#[allow(dead_code, reason = "reserved for future deployment lifecycle API")]
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

/// Stops a deployment via `DELETE .../api/deployment/{id}`.
///
/// # Errors
///
/// Returns an error if the HTTP request fails or the tower returns a non-success status.
#[allow(dead_code, reason = "reserved for future deployment lifecycle API")]
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

/// Lists deployments from `GET .../api/deployment/list`.
///
/// # Errors
///
/// Returns an error if the HTTP request fails or JSON parsing fails.
#[allow(dead_code, reason = "reserved for future deployment lifecycle API")]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_capabilities_json() -> &'static str {
        r#"{
            "node_id": "node-1",
            "network": {
                "type": "lan",
                "bandwidth_estimate": {
                    "download_mbps": 100,
                    "upload_mbps": 100,
                    "latency_ms": 5,
                    "confidence": "high"
                }
            },
            "deployment_methods": {
                "single": {
                    "enabled": true,
                    "max_size_mb": 50,
                    "compression_supported": [],
                    "recommended_for": "small"
                },
                "chunked": {
                    "enabled": true,
                    "max_total_size_mb": 500,
                    "chunk_size_mb": 10,
                    "max_chunks": 100,
                    "compression_supported": [],
                    "recommended_for": "large"
                },
                "streaming": {
                    "enabled": false,
                    "unlimited": false,
                    "compression_supported": [],
                    "recommended_for": "huge"
                }
            },
            "resources": {
                "available_storage_gb": 100,
                "available_memory_gb": 16,
                "cpu_cores": 8,
                "cpu_load_percent": 0.1,
                "max_concurrent_deployments": 4,
                "current_deployments": 0
            }
        }"#
    }

    #[test]
    fn deployment_response_json_roundtrip() {
        let original = DeploymentResponse {
            deployment_id: "dep-1".into(),
            status: "ok".into(),
            message: "deployed".into(),
            service_url: Some("http://localhost:8080".into()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: DeploymentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.deployment_id, original.deployment_id);
        assert_eq!(parsed.status, original.status);
        assert_eq!(parsed.message, original.message);
        assert_eq!(parsed.service_url, original.service_url);
    }

    #[test]
    fn select_deployment_method_fallback_when_no_capabilities() {
        assert!(matches!(select_deployment_method(None, 1.0), SelectedMethod::Fallback));
    }

    #[test]
    fn select_deployment_method_single_when_under_limit() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).unwrap();
        let m = select_deployment_method(Some(&caps), 10.0);
        assert!(matches!(m, SelectedMethod::Single { .. }));
    }

    #[test]
    fn select_deployment_method_chunked_when_over_single_limit() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).unwrap();
        let m = select_deployment_method(Some(&caps), 100.0);
        assert!(matches!(m, SelectedMethod::Chunked { .. }));
    }

    #[test]
    fn deployment_capabilities_deserialize() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).unwrap();
        assert_eq!(caps.node_id, "node-1");
        assert_eq!(caps.network.network_type, "lan");
        assert!(caps.deployment_methods.single.enabled);
    }
}
