// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird federation registration and heartbeats.

use super::types::{BridgeState, ServiceRegistration};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Derive health from actual bridge state: backend reachable or bridge-only operational.
fn derive_health_status(config: &super::types::BridgeConfig) -> &'static str {
    match config.backend_url {
        Some(_) => "healthy",
        None => "degraded",
    }
}

/// Register with Songbird federation
pub async fn register_with_songbird(
    state: &BridgeState,
    songbird_endpoint: &str,
) -> anyhow::Result<()> {
    let config = &state.config;
    let info = &state.service_info;

    let mut metadata = HashMap::new();
    metadata.insert("cpu_cores".to_string(), info.cpu_cores.to_string());
    metadata.insert("memory_gb".to_string(), info.memory_gb.to_string());
    metadata.insert("gpu_count".to_string(), info.gpu_count.to_string());
    if let Some(ref gpu_model) = info.gpu_model {
        metadata.insert("gpu_model".to_string(), gpu_model.clone());
    }
    if let Some(storage) = info.storage_gb {
        metadata.insert("storage_gb".to_string(), storage.to_string());
    }
    metadata.insert("platform".to_string(), info.platform.clone());

    let health = derive_health_status(config);
    let tower_id = config.tower_id.clone();
    let registration = ServiceRegistration {
        service_id: config.node_id.clone(),
        service_name: config.service_name.clone(),
        service_type: config.service_type.clone(),
        tower_id: tower_id.clone(),
        tower_name: tower_id,
        endpoint: format!("http://{}:{}", config.host, config.port),
        capabilities: config.capabilities.clone(),
        metadata,
        health_status: health.to_string(),
        registered_at: Utc::now().to_rfc3339(),
        last_seen: Utc::now().to_rfc3339(),
    };

    let url = format!("{songbird_endpoint}/api/federation/services");
    debug!("📡 Registering with Songbird: POST {}", url);

    let response = state.http_client.post(&url).await.json(&registration)?.send().await?;

    if response.is_success() {
        info!("✅ Successfully registered with Songbird");
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Registration failed ({status}): {body}")
    }
}

/// Heartbeat loop - keeps registration alive
pub async fn heartbeat_loop(state: BridgeState, songbird_endpoint: String) {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        match register_with_songbird(&state, &songbird_endpoint).await {
            Ok(()) => debug!("💓 Heartbeat sent to Songbird"),
            Err(e) => warn!("⚠️  Heartbeat failed: {}", e),
        }
    }
}
