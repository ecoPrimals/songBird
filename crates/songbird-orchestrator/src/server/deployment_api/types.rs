// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Deployment API types: state, DTOs, and capability discovery payloads.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
#[derive(Debug, Serialize, Deserialize)]
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
