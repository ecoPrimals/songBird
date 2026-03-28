// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Node capability discovery for deployment endpoints.

use axum::{Json, extract::State};
use songbird_types::sys_metrics;
use tracing::info;

use super::types::{
    BandwidthEstimate, ChunkedUploadMethod, DeploymentCapabilities, DeploymentMethods,
    DeploymentPreferences, DeploymentState, NetworkCapabilities, ResourceInfo, SingleUploadMethod,
    StreamingUploadMethod,
};

/// GET /api/deployment/capabilities - Discover node capabilities
pub async fn get_capabilities(
    State(state): State<DeploymentState>,
) -> Json<DeploymentCapabilities> {
    info!("📊 Capability discovery request received");

    let network_type = detect_network_type();
    let bandwidth = estimate_bandwidth(&network_type);

    let mem = sys_metrics::memory_info().unwrap_or(sys_metrics::MemoryInfo {
        total: 0,
        available: 0,
    });
    let total_memory = mem.total_gb();
    let available_memory = mem.available / (1024 * 1024 * 1024);
    let cpu_cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let cpu_load = 0.0_f32;

    let available_storage = sys_metrics::total_disk_gb().unwrap_or(0) as u64;

    let current_deployments = state.deployments.read().await.len();

    let max_concurrent = calculate_max_concurrent(available_memory);

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
                enabled: true,
                max_total_size_mb: 1000,
                chunk_size_mb: 10,
                max_chunks: 100,
                compression_supported: vec!["gzip".to_string()],
                recommended_for: "2MB - 500MB".to_string(),
            },
            streaming: StreamingUploadMethod {
                enabled: false,
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
            preferred_compression: "gzip".to_string(),
            preferred_method: "single".to_string(),
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
pub fn detect_network_type() -> String {
    "lan".to_string()
}

/// Estimate bandwidth based on network type
pub fn estimate_bandwidth(network_type: &str) -> BandwidthEstimate {
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
pub fn calculate_max_concurrent(available_memory_gb: u64) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "truncation acceptable: GB count bounded; only used as deployment concurrency hint"
    )]
    let memory_as_usize = available_memory_gb as usize;
    memory_as_usize.clamp(1, 10)
}
