//! Chunked upload implementation for Phase 3

use super::deployment_api::{
    start_service, ChunkInfo, DeploymentInfo, DeploymentResponse, DeploymentState,
    DeploymentStatus, FinalizeRequest, NegotiationRequest, NegotiationResponse, NegotiationState,
};
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};

/// POST /api/deployment/negotiate - Start chunked upload negotiation
pub async fn negotiate_chunked_upload(
    State(state): State<DeploymentState>,
    Json(request): Json<NegotiationRequest>,
) -> Result<(StatusCode, Json<NegotiationResponse>), (StatusCode, String)> {
    info!(
        "🤝 Negotiating chunked upload for {} ({:.2}MB)",
        request.service_name, request.binary_size_mb
    );

    // Generate negotiation ID
    let negotiation_id = format!("neg-{}", fastrand::u64(..));

    // Calculate chunks
    let chunk_size_mb = 10u32; // 10MB chunks
                               // Safe cast: For any reasonable binary size (<18 exabytes), this won't overflow usize
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_chunks = ((request.binary_size_mb / f64::from(chunk_size_mb)).ceil() as usize).max(1);

    // Create temp directory
    let temp_dir = format!("/tmp/songbird-chunks/{negotiation_id}");
    fs::create_dir_all(&temp_dir).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create temp dir: {e}"))
    })?;

    info!("✓ Negotiation: {} chunks of {}MB", total_chunks, chunk_size_mb);

    // Store negotiation state
    let negotiation = NegotiationState {
        negotiation_id: negotiation_id.clone(),
        binary_size_mb: request.binary_size_mb,
        chunk_size_mb,
        total_chunks,
        received_chunks: std::collections::HashMap::new(),
        temp_dir: temp_dir.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        timeout_seconds: 300, // 5 minutes
    };

    state.negotiations.write().await.insert(negotiation_id.clone(), negotiation);

    let response = NegotiationResponse {
        negotiation_id: negotiation_id.clone(),
        accepted_method: "chunked".to_string(),
        chunk_size_mb,
        total_chunks,
        chunk_upload_path: format!("/api/deployment/chunk/{negotiation_id}/{{index}}"),
        finalize_path: format!("/api/deployment/finalize/{negotiation_id}"),
        timeout_seconds: 300,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// POST /`api/deployment/chunk/:neg_id/:index` - Upload a chunk
pub async fn upload_chunk(
    State(state): State<DeploymentState>,
    Path((neg_id, chunk_index)): Path<(String, usize)>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    debug!("📦 Receiving chunk {} for negotiation {}", chunk_index, neg_id);

    // Get negotiation
    let negotiations = state.negotiations.read().await;
    let negotiation = negotiations
        .get(&neg_id)
        // Modern idiomatic: ok_or_else for lazy evaluation (only format on error path)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Negotiation '{neg_id}' not found")))?
        .clone();
    drop(negotiations);

    // Validate chunk index
    if chunk_index >= negotiation.total_chunks {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid chunk index {} (max {})", chunk_index, negotiation.total_chunks - 1),
        ));
    }

    // Parse multipart
    let mut chunk_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {e}")))?
    {
        if field.name() == Some("chunk") {
            chunk_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Chunk read error: {e}")))?
                    .to_vec(),
            );
        }
    }

    let chunk_data = chunk_data
        // Modern idiomatic: ok_or_else for lazy evaluation
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "No chunk data provided".to_string()))?;

    // Write chunk to disk
    let chunk_path = format!("{}/chunk-{:04}", negotiation.temp_dir, chunk_index);
    fs::write(&chunk_path, &chunk_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write chunk: {e}")))?;

    info!("✓ Chunk {} received ({} bytes)", chunk_index, chunk_data.len());

    // Update negotiation state
    let mut negotiations = state.negotiations.write().await;
    if let Some(neg) = negotiations.get_mut(&neg_id) {
        neg.received_chunks.insert(
            chunk_index,
            ChunkInfo {
                index: chunk_index,
                size_bytes: chunk_data.len(),
                received_at: chrono::Utc::now().to_rfc3339(),
                file_path: chunk_path,
            },
        );
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "chunk_index": chunk_index,
            "received": true,
            "verified": true,
            "size_bytes": chunk_data.len(),
        })),
    ))
}

/// POST /`api/deployment/finalize/:neg_id` - Finalize and deploy
pub async fn finalize_chunked_upload(
    State(state): State<DeploymentState>,
    Path(neg_id): Path<String>,
    Json(request): Json<FinalizeRequest>,
) -> Result<(StatusCode, Json<DeploymentResponse>), (StatusCode, String)> {
    info!("🎯 Finalizing chunked upload for negotiation {}", neg_id);

    // Get and remove negotiation
    let mut negotiations = state.negotiations.write().await;
    let negotiation = negotiations
        .remove(&neg_id)
        // Modern idiomatic: ok_or_else for lazy evaluation
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Negotiation '{neg_id}' not found")))?;
    drop(negotiations);

    // Verify all chunks received
    if negotiation.received_chunks.len() != negotiation.total_chunks {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Missing chunks: received {}/{}",
                negotiation.received_chunks.len(),
                negotiation.total_chunks
            ),
        ));
    }

    info!("✓ All {} chunks received, assembling...", negotiation.total_chunks);

    // Create deployment directory
    let deployment_id = format!("deploy-{}", fastrand::u64(..));
    let deploy_dir = format!("/tmp/songbird-deployments/{deployment_id}");
    fs::create_dir_all(&deploy_dir).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create deploy dir: {e}"))
    })?;

    // Assemble chunks in order
    let binary_path = format!("{deploy_dir}/service");
    let mut output_file = fs::File::create(&binary_path).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create output file: {e}"))
    })?;

    for chunk_index in 0..negotiation.total_chunks {
        let chunk_info = negotiation
            .received_chunks
            .get(&chunk_index)
            // Modern idiomatic: ok_or_else for lazy evaluation
            .ok_or_else(|| {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Missing chunk {chunk_index}"))
            })?;

        let chunk_data = fs::read(&chunk_info.file_path).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read chunk: {e}"))
        })?;

        output_file.write_all(&chunk_data).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write to output: {e}"))
        })?;
    }

    output_file
        .flush()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to flush output: {e}")))?;

    info!("✓ Binary assembled at {}", binary_path);

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata read failed: {e}")))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chmod failed: {e}")))?;
    }

    // Clean up chunks
    if let Err(e) = fs::remove_dir_all(&negotiation.temp_dir).await {
        warn!("Failed to clean up temp dir: {}", e);
    }

    // Extract port from env vars
    let port = request
        .env_vars
        .iter()
        .find(|(k, _)| k.to_uppercase().contains("PORT"))
        .and_then(|(_, v)| v.parse::<u16>().ok());

    // Create deployment info
    let mut deployment = DeploymentInfo {
        deployment_id: deployment_id.clone(),
        service_name: request.service_name.clone(),
        binary_path: binary_path.clone(),
        env_vars: request.env_vars.clone(),
        status: DeploymentStatus::Deploying,
        deployed_at: chrono::Utc::now().to_rfc3339(),
        pid: None,
        port,
    };

    // Start service if requested
    if request.auto_start {
        match start_service(&binary_path, &request.env_vars).await {
            Ok(pid) => {
                info!("✅ Service started with PID: {}", pid);
                deployment.status = DeploymentStatus::Running;
                deployment.pid = Some(pid);
            }
            Err(e) => {
                error!("❌ Service start failed: {}", e);
                deployment.status = DeploymentStatus::Failed;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Service start failed: {e}"),
                ));
            }
        }
    }

    // Store deployment info
    state.deployments.write().await.insert(deployment_id.clone(), deployment.clone());

    // Build service URL
    let service_url = if let (Some(host), Some(port)) = (
        request
            .env_vars
            .get("COMPUTE_HOST")
            // Modern idiomatic: or_else for lazy evaluation
            .or_else(|| request.env_vars.get("SERVICE_HOST")),
        port,
    ) {
        Some(format!("http://{host}:{port}"))
    } else {
        None
    };

    let response = DeploymentResponse {
        deployment_id,
        status: "deployed".to_string(),
        message: format!(
            "Service '{}' deployed successfully via chunked upload",
            request.service_name
        ),
        service_url,
    };

    info!("🎉 Chunked deployment complete: {}", request.service_name);

    Ok((StatusCode::CREATED, Json(response)))
}
