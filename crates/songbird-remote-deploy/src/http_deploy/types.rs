// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Deployment API DTOs and capability negotiation types.

use serde::{Deserialize, Serialize};

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
    /// Estimated download throughput in Mbps.
    pub download_mbps: u32,
    /// Estimated upload throughput in Mbps.
    pub upload_mbps: u32,
    /// Estimated round-trip latency in milliseconds.
    pub latency_ms: u32,
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
    /// Compression codecs advertised for single-request uploads.
    pub compression_supported: Vec<String>,
    // Future: method recommendations
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
    /// Maximum number of chunks allowed for one deployment.
    pub max_chunks: u32,
    // Future: compression negotiation
    /// Compression codecs advertised for chunked uploads.
    pub compression_supported: Vec<String>,
    // Future: method recommendations
    /// Tower hint for when chunked upload is preferred.
    pub recommended_for: String,
}

/// Streaming upload method details
#[derive(Debug, Deserialize)]
pub struct StreamingUploadMethod {
    /// Whether streaming upload is implemented on the tower.
    pub enabled: bool,
    // Future: size restrictions
    /// Whether the tower treats streaming uploads as unbounded in size.
    pub unlimited: bool,
    // Future: compression support
    /// Compression codecs advertised for streaming uploads.
    pub compression_supported: Vec<String>,
    // Future: method recommendations
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
    /// Recent CPU utilization on the tower (0.0–1.0 or percent, tower-defined).
    pub cpu_load_percent: f32,
    // Future: queue management
    /// Maximum deployments the tower will run concurrently.
    pub max_concurrent_deployments: usize,
    /// Deployments currently active on the tower.
    pub current_deployments: usize,
}

/// Upload strategy chosen from capability negotiation (`select_deployment_method`).
#[derive(Debug, Clone)]
pub enum SelectedMethod {
    /// Single HTTP request within tower `single.max_size_mb` (validated in selection).
    Single,
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
