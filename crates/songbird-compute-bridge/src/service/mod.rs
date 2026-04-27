// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🌉 Songbird Compute Bridge (Capability-Based, Agnostic)
//!
//! **Philosophy:** Zero hardcoding, works with ANY compute service
//!
//! This bridge enables any compute service (Kubernetes, Lambda, custom providers, etc.) to
//! participate in Songbird federation by exposing a standard API and registering
//! its capabilities dynamically.
//!
//! ## Features
//! - ✅ **Agnostic:** Works with any backend compute service
//! - ✅ **Auto-Discovery:** Detects capabilities from environment/backend
//! - ✅ **Federation Integration:** Registers with Songbird automatically
//! - ✅ **Zero Hardcoding:** All configuration via environment variables
//! - ✅ **Health Monitoring:** Automatic heartbeats and health checks
//!
//! ## Environment Variables
//! ```bash
//! # Required
//! export COMPUTE_SERVICE_TYPE="compute"          # Service type
//! export COMPUTE_SERVICE_NAME="My Compute Node"  # Human-readable name
//! export COMPUTE_HOST="192.0.2.10"           # This node's address
//! export COMPUTE_PORT="9000"                     # This node's port
//! export SONGBIRD_FEDERATION_ENDPOINT="http://192.0.2.10:8080"
//!
//! # Optional - Auto-detected if not provided
//! export COMPUTE_CAPABILITIES="compute,cpu,gpu,batch-processing"
//! export COMPUTE_NODE_ID="auto-generated-uuid"
//! export COMPUTE_CPU_CORES="16"
//! export COMPUTE_MEMORY_GB="64"
//! export COMPUTE_GPU_COUNT="1"
//! export COMPUTE_GPU_MODEL="NVIDIA RTX 2070 SUPER"
//! ```

mod detection;
mod federation;
mod handlers;
mod types;

#[cfg(test)]
mod service_tests;

pub use types::Args;

use handlers::bridge_router;
use songbird_http_client::IpcHttpClient;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

fn normalize_capabilities_csv(raw: &str) -> Vec<String> {
    raw.split(',').map(|s| s.trim().to_string()).collect()
}

fn resolve_tower_id(
    explicit: Option<String>,
    service_id_env: Option<String>,
    hostname: &str,
) -> String {
    explicit.or(service_id_env).unwrap_or_else(|| format!("tower-{hostname}"))
}

fn init_tracing() {
    let filter = songbird_process_env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,songbird_compute_bridge=debug".to_string());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Run the compute bridge until shutdown (standalone binary or `songbird compute-bridge`).
///
/// # Errors
///
/// Returns an error if server binding, provider registration, or the main event loop fails.
pub async fn run(args: Args) -> anyhow::Result<()> {
    init_tracing();

    info!("🌉 Starting Songbird Compute Bridge (Agnostic)");
    info!("================================================");

    // Detect system resources
    let service_info = detection::detect_resources().await;
    info!("📊 Detected Resources:");
    info!("   CPU Cores: {}", service_info.cpu_cores);
    info!("   Memory: {}GB", service_info.memory_gb);
    info!("   GPUs: {}", service_info.gpu_count);
    if let Some(ref gpu_model) = service_info.gpu_model {
        info!("   GPU Model: {}", gpu_model);
    }

    // Build configuration
    let node_id = args.node_id.unwrap_or_else(|| format!("compute-{}", Uuid::new_v4()));

    let tower_id = resolve_tower_id(
        args.tower_id.clone(),
        songbird_process_env::var("SERVICE_ID").ok(),
        &gethostname::gethostname().to_string_lossy(),
    );

    let capabilities: Vec<String> = normalize_capabilities_csv(
        &args.capabilities.unwrap_or_else(|| detection::detect_capabilities(&service_info)),
    );

    let config = Arc::new(types::BridgeConfig {
        host: args.host.clone(),
        port: args.port,
        service_name: args.service_name,
        service_type: args.service_type,
        node_id: node_id.clone(),
        tower_id: tower_id.clone(),
        songbird_endpoint: args.songbird_endpoint,
        capabilities,
        backend_url: args.backend_url,
    });

    info!("🔧 Configuration:");
    info!("   Node ID: {}", config.node_id);
    info!("   Tower ID: {}", config.tower_id);
    info!("   Service: {} ({})", config.service_name, config.service_type);
    info!("   Endpoint: {}:{}", args.host, args.port);
    info!("   Capabilities: {:?}", config.capabilities);
    if let Some(ref songbird) = config.songbird_endpoint {
        info!("   Songbird: {}", songbird);
    }

    // Create state
    let http_client = IpcHttpClient::new().await.map_err(|e| {
        error!("❌ Failed to create IPC HTTP client: {}", e);
        error!("   This is required for compute-bridge operation.");
        error!("   Ensure Songbird IPC socket is available.");
        anyhow::anyhow!("IPC HTTP client is required for compute-bridge operation: {e}")
    })?;

    let state = types::BridgeState {
        config: Arc::clone(&config),
        http_client,
        service_info: Arc::new(service_info),
    };

    // Register with Songbird (if configured)
    if let Some(ref songbird_endpoint) = config.songbird_endpoint {
        match federation::register_with_songbird(&state, songbird_endpoint).await {
            Ok(()) => info!("✅ Registered with Songbird federation"),
            Err(e) => warn!("⚠️  Failed to register with Songbird: {}", e),
        }

        // Start heartbeat task
        let heartbeat_state = state.clone();
        let songbird_url = songbird_endpoint.clone();
        tokio::spawn(async move {
            federation::heartbeat_loop(heartbeat_state, songbird_url).await;
        });
    }

    let app = bridge_router(state);

    // Start server
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    info!("🚀 Compute Bridge listening on {}", addr);
    info!("================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{normalize_capabilities_csv, resolve_tower_id};

    #[test]
    fn normalize_capabilities_csv_trims_segments() {
        assert_eq!(normalize_capabilities_csv("compute, cpu , gpu"), vec!["compute", "cpu", "gpu"]);
        assert_eq!(normalize_capabilities_csv("single"), vec!["single"]);
        assert_eq!(normalize_capabilities_csv(""), vec![""]);
    }

    #[test]
    fn resolve_tower_id_prefers_explicit_then_env_then_hostname() {
        assert_eq!(
            resolve_tower_id(Some("t-explicit".into()), Some("svc".into()), "host"),
            "t-explicit"
        );
        assert_eq!(resolve_tower_id(None, Some("from-env".into()), "ignored"), "from-env");
        assert_eq!(resolve_tower_id(None, None, "mybox"), "tower-mybox");
    }
}
