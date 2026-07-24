// SPDX-License-Identifier: AGPL-3.0-or-later
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
    #[expect(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    accepted_method: String,
    chunk_size_mb: u32,
    total_chunks: usize,
    // Future: dynamic endpoint routing
    #[expect(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    chunk_upload_path: String,
    #[expect(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
    finalize_path: String,
    #[expect(dead_code, reason = "deserialized from negotiation JSON; not yet used by upload path")]
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
        let error_text = response.text().await.unwrap_or_else(|_| String::from("Unknown error"));
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
        let error_text = response.text().await.unwrap_or_else(|_| String::from("Unknown error"));
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
        let error_text = response.text().await.unwrap_or_else(|_| String::from("Unknown error"));
        return Err(anyhow!("Finalize failed with status {status}: {error_text}"));
    }

    let deployment: DeploymentResponse =
        response.json().await.map_err(|e| anyhow!("Failed to parse deployment response: {e}"))?;

    Ok(deployment)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{FinalizeRequest, NegotiationRequest, NegotiationResponse};

    #[test]
    fn negotiation_request_serializes_expected_keys() {
        let req = NegotiationRequest {
            binary_size_mb: 12.5,
            service_name: "svc-a".into(),
            compression: None,
        };
        let v = serde_json::to_value(&req).expect("NegotiationRequest is Serialize");
        assert_eq!(v["binary_size_mb"], 12.5);
        assert_eq!(v["service_name"], "svc-a");
        assert!(v["compression"].is_null());
    }

    #[test]
    fn negotiation_response_deserializes_sample() {
        let json = r#"{
            "negotiation_id": "neg-1",
            "accepted_method": "chunked",
            "chunk_size_mb": 4,
            "total_chunks": 3,
            "chunk_upload_path": "/chunk",
            "finalize_path": "/fin",
            "timeout_seconds": 120
        }"#;
        let n: NegotiationResponse = serde_json::from_str(json).expect("NegotiationResponse");
        assert_eq!(n.negotiation_id, "neg-1");
        assert_eq!(n.chunk_size_mb, 4);
        assert_eq!(n.total_chunks, 3);
    }

    #[test]
    fn finalize_request_serializes_env_map() {
        let mut env = std::collections::HashMap::new();
        env.insert("A".into(), "1".into());
        let req = FinalizeRequest {
            service_name: "s".into(),
            env_vars: env,
            auto_start: false,
        };
        let v = serde_json::to_value(&req).expect("FinalizeRequest");
        assert_eq!(v["service_name"], "s");
        assert_eq!(v["auto_start"], false);
        assert!(v["env_vars"].is_object());
    }
}
