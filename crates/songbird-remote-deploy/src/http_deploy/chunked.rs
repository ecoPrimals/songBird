// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Chunked upload: negotiate, per-chunk multipart posts, finalize.

use super::types::DeploymentResponse;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use songbird_http_client::{Form, IpcHttpClient, Part};
use std::collections::HashMap;
use std::hash::BuildHasher;
use tokio::fs;
use tracing::info;

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
    #[allow(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    accepted_method: String,
    chunk_size_mb: u32,
    total_chunks: usize,
    // Future: dynamic endpoint routing
    #[allow(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    chunk_upload_path: String,
    #[allow(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    finalize_path: String,
    #[allow(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    timeout_seconds: u64,
}

/// Finalize request
#[derive(Debug, Serialize)]
struct FinalizeRequest {
    service_name: String,
    env_vars: HashMap<String, String>,
    auto_start: bool,
}

/// Deploy a binary via chunked upload
pub(super) async fn deploy_via_http_chunked<S: BuildHasher>(
    tower_endpoint: &str,
    binary_path: &str,
    service_name: &str,
    env_vars: HashMap<String, String, S>,
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
async fn finalize_chunked_upload<S: BuildHasher>(
    client: &IpcHttpClient,
    tower_endpoint: &str,
    negotiation_id: &str,
    service_name: &str,
    env_vars: HashMap<String, String, S>,
) -> Result<DeploymentResponse> {
    let url = format!("{tower_endpoint}/api/deployment/finalize/{negotiation_id}");

    let request = FinalizeRequest {
        service_name: service_name.to_string(),
        env_vars: env_vars.into_iter().collect(),
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
