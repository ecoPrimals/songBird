// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::{Context, Result};
use serde::Deserialize;
use songbird_http_client::IpcHttpClient;
use tracing::{debug, info};

#[derive(Debug, Deserialize)]
pub(super) struct NodeInfo {
    pub(super) node_id: String,
    pub(super) node_name: String,
    pub(super) node_address: String,
    pub(super) capabilities: Vec<String>,
    pub(super) cpu_cores: usize,
    pub(super) memory_gb: usize,
}

pub(super) fn parse_tower_address(address: &str) -> String {
    let parts: Vec<&str> = address.split(':').collect();
    parts[0].to_string()
}

pub(super) async fn get_tower_info(songbird_endpoint: &str, tower_id: &str) -> Result<NodeInfo> {
    let url = format!("{songbird_endpoint}/api/federation/nodes");
    debug!("Fetching tower info from: {}", url);

    let client = IpcHttpClient::new().await.context("Failed to create HTTP client")?;
    let nodes: Vec<NodeInfo> = client
        .get(&url)
        .await
        .context("Failed to query Songbird federation")?
        .json()
        .await
        .context("Failed to parse tower list")?;

    nodes
        .into_iter()
        .find(|n| {
            n.node_id == tower_id || n.node_name.to_lowercase().contains(&tower_id.to_lowercase())
        })
        .ok_or_else(|| anyhow::anyhow!("Tower '{tower_id}' not found in federation"))
}

pub(super) async fn verify_service_health(host: &str, port: u16) -> Result<()> {
    let url = format!("http://{host}:{port}/health");
    debug!("Health check: {}", url);

    let client = IpcHttpClient::new().await.context("Failed to create HTTP client")?;
    let response = client.get(&url).await.context("Health check request failed")?;

    if response.is_success() {
        Ok(())
    } else {
        let status = response.status();
        anyhow::bail!("Health check failed with status: {status}")
    }
}

pub(super) async fn list_towers(songbird_endpoint: &str, detailed: bool) -> Result<()> {
    let url = format!("{songbird_endpoint}/api/federation/nodes");

    let client = IpcHttpClient::new().await.context("Failed to create HTTP client")?;
    let nodes: Vec<NodeInfo> = client
        .get(&url)
        .await
        .context("Failed to query Songbird federation")?
        .json()
        .await
        .context("Failed to parse tower list")?;

    info!("📡 Available Towers in Federation");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    for node in nodes {
        info!("🏗️  {} ({})", node.node_name, node.node_id);
        info!("   Address: {}", node.node_address);

        if detailed {
            info!("   CPU Cores: {}", node.cpu_cores);
            info!("   Memory: {}GB", node.memory_gb);
            info!("   Capabilities: {}", node.capabilities.join(", "));
        }

        info!("");
    }

    Ok(())
}

pub(super) async fn check_status(
    songbird_endpoint: &str,
    tower_id: &str,
    port: Option<u16>,
) -> Result<()> {
    let tower_info = get_tower_info(songbird_endpoint, tower_id).await?;
    let tower_address = parse_tower_address(&tower_info.node_address);

    info!("🔍 Checking status on: {} ({})", tower_info.node_name, tower_address);

    if let Some(port) = port {
        match verify_service_health(&tower_address, port).await {
            Ok(()) => info!("✅ Service on port {} is healthy", port),
            Err(e) => info!("❌ Service on port {} is not responding: {}", port, e),
        }
    } else {
        #[derive(Deserialize)]
        struct ServiceInfo {
            service_name: String,
            service_type: String,
            endpoint: String,
            health_status: String,
        }

        let url = format!("{songbird_endpoint}/api/federation/services");
        let client = IpcHttpClient::new().await.context("Failed to create HTTP client")?;
        let services: Vec<ServiceInfo> = client.get(&url).await?.json().await?;

        let tower_services: Vec<_> =
            services.into_iter().filter(|s| s.endpoint.contains(&tower_address)).collect();

        if tower_services.is_empty() {
            info!("ℹ️  No services registered for this tower");
        } else {
            info!("📊 Registered Services:");
            for svc in tower_services {
                info!("   • {} ({})", svc.service_name, svc.service_type);
                info!("     Endpoint: {}", svc.endpoint);
                info!("     Status: {}", svc.health_status);
            }
        }
    }

    Ok(())
}
