// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Bridge configuration, runtime state, and API payloads.

use clap::Parser;
use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use std::collections::HashMap;
use std::sync::Arc;

/// Command-line arguments for the compute bridge HTTP service.
#[derive(Parser, Debug)]
#[command(name = "songbird-compute-bridge")]
#[command(about = "Agnostic compute service bridge for Songbird federation")]
pub struct Args {
    /// Bind address
    #[arg(long, env = "COMPUTE_HOST", default_value = "0.0.0.0")]
    pub(crate) host: String,

    /// Bind port
    #[arg(long, env = "COMPUTE_PORT", default_value = "9000")]
    pub(crate) port: u16,

    /// Service name
    #[arg(long, env = "COMPUTE_SERVICE_NAME", default_value = "Compute Service")]
    pub(crate) service_name: String,

    /// Service type
    #[arg(long, env = "COMPUTE_SERVICE_TYPE", default_value = "compute")]
    pub(crate) service_type: String,

    /// Songbird federation endpoint
    #[arg(long, env = "SONGBIRD_FEDERATION_ENDPOINT")]
    pub(crate) songbird_endpoint: Option<String>,

    /// Node ID (auto-generated if not provided)
    #[arg(long, env = "COMPUTE_NODE_ID")]
    pub(crate) node_id: Option<String>,

    /// Tower ID (for federation)
    #[arg(long, env = "COMPUTE_TOWER_ID")]
    pub(crate) tower_id: Option<String>,

    /// Capabilities (comma-separated)
    #[arg(long, env = "COMPUTE_CAPABILITIES")]
    pub(crate) capabilities: Option<String>,

    /// Backend service URL (optional - for proxying to actual compute service)
    #[arg(long, env = "COMPUTE_BACKEND_URL")]
    pub(crate) backend_url: Option<String>,
}

/// Bridge state
#[derive(Clone)]
pub struct BridgeState {
    pub config: Arc<BridgeConfig>,
    pub http_client: IpcHttpClient,
    pub service_info: Arc<ServiceInfo>,
}

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub host: String,
    pub port: u16,
    pub service_name: String,
    pub service_type: String,
    pub node_id: String,
    pub tower_id: String,
    pub songbird_endpoint: Option<String>,
    pub capabilities: Vec<String>,
    pub backend_url: Option<String>,
}

/// Service information (auto-detected)
#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub cpu_cores: usize,
    pub memory_gb: usize,
    pub gpu_count: usize,
    pub gpu_model: Option<String>,
    pub storage_gb: Option<usize>,
    pub platform: String,
}

/// Service registration payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub service_name: String,
    pub service_type: String,
    pub tower_id: String,
    pub tower_name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_status: String,
    pub registered_at: String,
    pub last_seen: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkloadRequest {
    #[allow(
        dead_code,
        reason = "deserialized from API request JSON; not read until routing is wired"
    )]
    pub name: String,
    #[allow(
        dead_code,
        reason = "deserialized from API request JSON; not read until routing is wired"
    )]
    pub payload: serde_json::Value,
}
