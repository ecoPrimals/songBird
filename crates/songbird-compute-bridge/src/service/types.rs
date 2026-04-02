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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::{Args, BridgeConfig, ServiceInfo, ServiceRegistration, WorkloadRequest};
    use clap::Parser;
    use std::collections::HashMap;

    #[test]
    fn bridge_config_clone_and_debug_roundtrip() {
        let c = BridgeConfig {
            host: "0.0.0.0".into(),
            port: 9000,
            service_name: "S".into(),
            service_type: "compute".into(),
            node_id: "n".into(),
            tower_id: "t".into(),
            songbird_endpoint: Some("http://sb".into()),
            capabilities: vec!["a".into()],
            backend_url: None,
        };
        let d = c.clone();
        assert_eq!(format!("{c:?}"), format!("{d:?}"));
    }

    #[test]
    fn service_info_serializes_expected_fields() {
        let i = ServiceInfo {
            cpu_cores: 2,
            memory_gb: 4,
            gpu_count: 0,
            gpu_model: None,
            storage_gb: None,
            platform: "linux-x86_64".into(),
        };
        let v = serde_json::to_value(&i).expect("serialize ServiceInfo");
        assert_eq!(v["cpu_cores"], 2);
        assert_eq!(v["platform"], "linux-x86_64");
    }

    #[test]
    fn args_accepts_empty_optional_strings() {
        let args = Args::try_parse_from(["songbird-compute-bridge"]).expect("default argv");
        assert!(args.songbird_endpoint.is_none());
        assert!(args.node_id.is_none());
        assert!(args.capabilities.is_none());
    }

    #[test]
    fn service_registration_includes_metadata_roundtrip() {
        let mut m = HashMap::new();
        m.insert("k".into(), "v".into());
        let r = ServiceRegistration {
            service_id: "id".into(),
            service_name: "sn".into(),
            service_type: "st".into(),
            tower_id: "tid".into(),
            tower_name: "tn".into(),
            endpoint: "http://e".into(),
            capabilities: vec!["c".into()],
            metadata: m,
            health_status: "ok".into(),
            registered_at: "rfc3339".into(),
            last_seen: "rfc3339b".into(),
        };
        let json = serde_json::to_string(&r).expect("to json");
        let back: ServiceRegistration = serde_json::from_str(&json).expect("from json");
        assert_eq!(back.metadata.get("k"), Some(&"v".into()));
    }

    #[test]
    fn workload_request_preserves_null_payload() {
        let w = WorkloadRequest {
            name: "n".into(),
            payload: serde_json::Value::Null,
        };
        let v = serde_json::to_value(&w).expect("serde");
        assert!(v["payload"].is_null());
    }
}
