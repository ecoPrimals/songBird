//! HTTP-based deployment client
//!
//! Deploy services via Songbird's HTTP deployment API

use anyhow::{anyhow, Result};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub service_url: Option<String>,
}

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

/// Deploy a binary via HTTP to a remote tower
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
        .map_err(|e| anyhow!("Failed to read binary '{}': {}", binary_path, e))?;

    let binary_filename = Path::new(binary_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("service");

    info!("   Binary: {} ({} bytes)", binary_filename, binary_data.len());
    info!("   Service name: {}", service_name);
    info!("   Environment vars: {}", env_vars.len());

    // Build multipart form
    let mut form = multipart::Form::new()
        .text("service_name", service_name.to_string())
        .text("env_vars", serde_json::to_string(&env_vars)?)
        .text("auto_start", "true")
        .part(
            "binary",
            multipart::Part::bytes(binary_data).file_name(binary_filename.to_string()),
        );

    // Send deployment request
    let client = Client::new();
    let url = format!("{}/api/deployment/binary", tower_endpoint);

    info!("📡 Sending deployment request to {}", url);

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Deployment failed with status {}: {}", status, error_text));
    }

    let deployment_response: DeploymentResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    info!("✅ Deployment successful: {}", deployment_response.deployment_id);
    if let Some(ref url) = deployment_response.service_url {
        info!("   Service URL: {}", url);
    }

    Ok(deployment_response)
}

/// Get deployment status
pub async fn get_deployment_status(
    tower_endpoint: &str,
    deployment_id: &str,
) -> Result<DeploymentInfo> {
    let client = Client::new();
    let url = format!("{}/api/deployment/status/{}", tower_endpoint, deployment_id);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to get deployment status: {}", response.status()));
    }

    let deployment_info: DeploymentInfo = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(deployment_info)
}

/// Stop a deployment
pub async fn stop_deployment(tower_endpoint: &str, deployment_id: &str) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/api/deployment/{}", tower_endpoint, deployment_id);

    let response = client
        .delete(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to stop deployment: {}", response.status()));
    }

    info!("✅ Deployment {} stopped", deployment_id);

    Ok(())
}

/// List all deployments on a tower
pub async fn list_deployments(tower_endpoint: &str) -> Result<Vec<DeploymentInfo>> {
    let client = Client::new();
    let url = format!("{}/api/deployment/list", tower_endpoint);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to list deployments: {}", response.status()));
    }

    let deployments: Vec<DeploymentInfo> = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

    Ok(deployments)
}

