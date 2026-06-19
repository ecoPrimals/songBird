// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Deployment API types: state, DTOs, and capability discovery payloads.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Deployment state
#[derive(Clone)]
pub struct DeploymentState {
    pub deployments: Arc<RwLock<HashMap<String, DeploymentInfo>>>,
    pub negotiations: Arc<RwLock<HashMap<String, NegotiationState>>>,
}

impl Default for DeploymentState {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            deployments: Arc::new(RwLock::new(HashMap::new())),
            negotiations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Negotiation state for chunked uploads
#[derive(Debug, Clone)]
pub struct NegotiationState {
    pub negotiation_id: String,
    pub binary_size_mb: f64,
    pub chunk_size_mb: u32,
    pub total_chunks: usize,
    pub received_chunks: HashMap<usize, ChunkInfo>,
    pub temp_dir: String,
    pub created_at: String,
    pub timeout_seconds: u64,
}

/// Chunk information
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub index: usize,
    pub size_bytes: usize,
    pub received_at: String,
    pub file_path: String,
}

/// Negotiation request
#[derive(Debug, Deserialize)]
pub struct NegotiationRequest {
    pub binary_size_mb: f64,
    pub service_name: String,
    pub compression: Option<String>,
}

/// Negotiation response
#[derive(Debug, Serialize)]
pub struct NegotiationResponse {
    pub negotiation_id: String,
    pub accepted_method: String,
    pub chunk_size_mb: u32,
    pub total_chunks: usize,
    pub chunk_upload_path: String,
    pub finalize_path: String,
    pub timeout_seconds: u64,
}

/// Finalize request
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalizeRequest {
    pub service_name: String,
    pub env_vars: HashMap<String, String>,
    pub auto_start: bool,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub deployment_id: String,
    pub service_name: String,
    pub binary_path: String,
    pub env_vars: HashMap<String, String>,
    pub status: DeploymentStatus,
    pub deployed_at: String, // ISO 8601 timestamp
    pub pid: Option<u32>,
    pub port: Option<u16>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Deploying,
    Running,
    Failed,
    Stopped,
}

/// Deployment response
#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub deployment_id: String,
    pub status: String,
    pub message: String,
    pub service_url: Option<String>,
}

/// Node deployment capabilities
#[derive(Debug, Serialize)]
pub struct DeploymentCapabilities {
    pub node_id: String,
    pub timestamp: String,
    pub network: NetworkCapabilities,
    pub deployment_methods: DeploymentMethods,
    pub resources: ResourceInfo,
    pub preferences: DeploymentPreferences,
}

/// Network capabilities
#[derive(Debug, Serialize)]
pub struct NetworkCapabilities {
    #[serde(rename = "type")]
    pub network_type: String, // "lan", "vpn", "internet"
    pub bandwidth_estimate: BandwidthEstimate,
}

/// Bandwidth estimate
#[derive(Debug, Serialize)]
pub struct BandwidthEstimate {
    pub download_mbps: u32,
    pub upload_mbps: u32,
    pub latency_ms: u32,
    pub confidence: String, // "high", "medium", "low"
}

/// Supported deployment methods
#[derive(Debug, Serialize)]
pub struct DeploymentMethods {
    pub single: SingleUploadMethod,
    pub chunked: ChunkedUploadMethod,
    pub streaming: StreamingUploadMethod,
}

/// Single upload method
#[derive(Debug, Serialize)]
pub struct SingleUploadMethod {
    pub enabled: bool,
    pub max_size_mb: u32,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

/// Chunked upload method
#[derive(Debug, Serialize)]
pub struct ChunkedUploadMethod {
    pub enabled: bool,
    pub max_total_size_mb: u32,
    pub chunk_size_mb: u32,
    pub max_chunks: u32,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

/// Streaming upload method
#[derive(Debug, Serialize)]
pub struct StreamingUploadMethod {
    pub enabled: bool,
    pub unlimited: bool,
    pub compression_supported: Vec<String>,
    pub recommended_for: String,
}

/// Resource information
#[derive(Debug, Serialize)]
pub struct ResourceInfo {
    pub available_storage_gb: u64,
    pub available_memory_gb: u64,
    pub cpu_cores: usize,
    pub cpu_load_percent: f32,
    pub max_concurrent_deployments: usize,
    pub current_deployments: usize,
}

/// Deployment preferences
#[derive(Debug, Serialize)]
pub struct DeploymentPreferences {
    pub preferred_compression: String,
    pub preferred_method: String,
    pub encryption_required: bool,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let s = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&s).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), s);
    }

    #[test]
    fn deployment_state_new_and_default_empty() {
        let s = DeploymentState::new();
        assert!(s.deployments.try_read().unwrap().is_empty());
        assert!(s.negotiations.try_read().unwrap().is_empty());
        let d = DeploymentState::default();
        assert!(d.deployments.try_read().unwrap().is_empty());
    }

    #[test]
    fn negotiation_state_construction() {
        let mut chunks = HashMap::new();
        chunks.insert(
            0,
            ChunkInfo {
                index: 0,
                size_bytes: 10,
                received_at: String::from("t"),
                file_path: String::from("/tmp/x"),
            },
        );
        let n = NegotiationState {
            negotiation_id: String::from("n1"),
            binary_size_mb: 1.0,
            chunk_size_mb: 1,
            total_chunks: 1,
            received_chunks: chunks,
            temp_dir: String::from("/tmp"),
            created_at: String::from("now"),
            timeout_seconds: 60,
        };
        assert_eq!(n.received_chunks.len(), 1);
    }

    #[test]
    fn negotiation_request_deserializes() {
        let v = json!({
            "binary_size_mb": 4.5,
            "service_name": "alpha",
            "compression": "gzip"
        });
        let r: NegotiationRequest = serde_json::from_value(v).unwrap();
        assert!((r.binary_size_mb - 4.5).abs() < f64::EPSILON);
        assert_eq!(r.service_name, "alpha");
        assert_eq!(r.compression.as_deref(), Some("gzip"));
    }

    #[test]
    fn negotiation_response_serializes() {
        let r = NegotiationResponse {
            negotiation_id: String::from("id"),
            accepted_method: String::from("chunked"),
            chunk_size_mb: 2,
            total_chunks: 5,
            chunk_upload_path: String::from("/upload"),
            finalize_path: String::from("/done"),
            timeout_seconds: 120,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["negotiation_id"], json!("id"));
        assert_eq!(v["total_chunks"], json!(5));
    }

    #[test]
    fn finalize_request_roundtrip() {
        let f = FinalizeRequest {
            service_name: String::from("svc"),
            env_vars: HashMap::from([(String::from("A"), String::from("b"))]),
            auto_start: true,
        };
        assert_json_roundtrip(&f);
    }

    #[test]
    fn deployment_info_roundtrip() {
        let d = DeploymentInfo {
            deployment_id: String::from("d1"),
            service_name: String::from("svc"),
            binary_path: String::from("/bin/app"),
            env_vars: HashMap::new(),
            status: DeploymentStatus::Deploying,
            deployed_at: String::from("2026-01-01T00:00:00Z"),
            pid: Some(42),
            port: Some(8080),
        };
        assert_json_roundtrip(&d);
    }

    #[test]
    fn deployment_status_serde_lowercase_and_roundtrip() {
        assert_eq!(serde_json::to_string(&DeploymentStatus::Running).unwrap(), "\"running\"");
        for st in [
            DeploymentStatus::Deploying,
            DeploymentStatus::Running,
            DeploymentStatus::Failed,
            DeploymentStatus::Stopped,
        ] {
            assert_json_roundtrip(&st);
        }
    }

    #[test]
    fn deployment_response_serializes() {
        let r = DeploymentResponse {
            deployment_id: String::from("d"),
            status: String::from("ok"),
            message: String::from("deployed"),
            service_url: Some(String::from("http://localhost:1")),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["status"], json!("ok"));
    }

    #[test]
    fn deployment_capabilities_serializes() {
        let c = DeploymentCapabilities {
            node_id: String::from("node"),
            timestamp: String::from("t"),
            network: NetworkCapabilities {
                network_type: String::from("lan"),
                bandwidth_estimate: BandwidthEstimate {
                    download_mbps: 100,
                    upload_mbps: 50,
                    latency_ms: 5,
                    confidence: String::from("high"),
                },
            },
            deployment_methods: DeploymentMethods {
                single: SingleUploadMethod {
                    enabled: true,
                    max_size_mb: 50,
                    compression_supported: vec![String::from("gzip")],
                    recommended_for: String::from("small"),
                },
                chunked: ChunkedUploadMethod {
                    enabled: true,
                    max_total_size_mb: 500,
                    chunk_size_mb: 5,
                    max_chunks: 100,
                    compression_supported: vec![],
                    recommended_for: String::from("large"),
                },
                streaming: StreamingUploadMethod {
                    enabled: false,
                    unlimited: false,
                    compression_supported: vec![],
                    recommended_for: String::from("huge"),
                },
            },
            resources: ResourceInfo {
                available_storage_gb: 100,
                available_memory_gb: 8,
                cpu_cores: 4,
                cpu_load_percent: 0.1,
                max_concurrent_deployments: 2,
                current_deployments: 0,
            },
            preferences: DeploymentPreferences {
                preferred_compression: String::from("gzip"),
                preferred_method: String::from("chunked"),
                encryption_required: true,
            },
        };
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["node_id"], json!("node"));
        assert_eq!(v["network"]["type"], json!("lan"));
    }
}
