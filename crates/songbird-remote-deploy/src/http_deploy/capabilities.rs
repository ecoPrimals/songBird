// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability query and upload-method selection.

use super::types::{DeploymentCapabilities, SelectedMethod};
use anyhow::{Result, anyhow};
use songbird_http_client::IpcHttpClient;
use tracing::{debug, info, warn};

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
        return SelectedMethod::Single;
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
