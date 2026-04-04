// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP-based deployment client with capability negotiation
//!
//! Deploy services via Songbird's HTTP deployment API with intelligent
//! method selection based on node capabilities.

mod capabilities;
mod chunked;
mod types;

pub use capabilities::{query_capabilities, select_deployment_method};
pub use types::{
    BandwidthEstimate, ChunkedUploadMethod, DeploymentCapabilities, DeploymentMethods,
    DeploymentResponse, NetworkCapabilities, ResourceInfo, SelectedMethod, SingleUploadMethod,
    StreamingUploadMethod,
};

use anyhow::{Result, anyhow};
use chunked::deploy_via_http_chunked;
use songbird_http_client::{Form, IpcHttpClient, Part};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// Deploys a binary using negotiated upload strategy (queries capabilities when possible).
///
/// # Errors
///
/// Propagates I/O, HTTP, and JSON errors from the underlying client and tower responses.
pub async fn deploy_via_http_adaptive<S: BuildHasher>(
    tower_endpoint: &str,
    binary_path: &str,
    service_name: &str,
    env_vars: HashMap<String, String, S>,
) -> Result<DeploymentResponse> {
    info!("📤 Adaptive deployment to {}", tower_endpoint);

    // Get binary size
    let metadata = tokio::fs::metadata(binary_path).await?;
    let binary_size_bytes = metadata.len();
    let binary_size_mb = f64::from(
        u32::try_from((binary_size_bytes / 1024 / 1024).min(u64::from(u32::MAX)))
            .unwrap_or(u32::MAX),
    );

    let binary_name =
        Path::new(binary_path).file_name().and_then(|n| n.to_str()).unwrap_or("unknown-binary");

    info!("   Binary: {} ({:.2} MB)", binary_name, binary_size_mb);

    // Query capabilities
    let capabilities = match query_capabilities(tower_endpoint).await {
        Ok(caps) => Some(caps),
        Err(e) => {
            warn!("⚠️  Failed to query capabilities: {}", e);
            warn!("   Falling back to direct deployment");
            None
        }
    };

    // Select method
    let method = select_deployment_method(capabilities.as_ref(), binary_size_mb);

    // Execute deployment based on selected method
    match method {
        SelectedMethod::Single | SelectedMethod::Fallback => {
            // Use existing single upload
            deploy_via_http(tower_endpoint, binary_path, service_name, env_vars).await
        }
        SelectedMethod::Chunked {
            chunk_size_mb,
        } => {
            // Phase 3: Use chunked upload
            deploy_via_http_chunked(
                tower_endpoint,
                binary_path,
                service_name,
                env_vars,
                chunk_size_mb,
            )
            .await
        }
        SelectedMethod::Streaming => {
            // PHASE 4 FEATURE: Streaming upload implementation planned
            // Will support: Real-time progress, resume on failure, adaptive chunking
            // Priority: Low (Phase 3 chunked upload is sufficient for most use cases)
            warn!("⚠️  Streaming upload not yet implemented, falling back to single");
            deploy_via_http(tower_endpoint, binary_path, service_name, env_vars).await
        }
    }
}

/// Single-request multipart deploy to `POST {tower}/api/deployment/binary`.
///
/// # Errors
///
/// Returns an error if the binary cannot be read, the multipart request fails, or the response is not success JSON.
pub async fn deploy_via_http<S: BuildHasher>(
    tower_endpoint: &str,
    binary_path: &str,
    service_name: &str,
    env_vars: HashMap<String, String, S>,
) -> Result<DeploymentResponse> {
    info!("📤 Deploying '{}' to {} via HTTP", service_name, tower_endpoint);

    // Read binary file
    let binary_data = fs::read(binary_path)
        .await
        .map_err(|e| anyhow!("Failed to read binary '{binary_path}': {e}"))?;

    let binary_filename =
        Path::new(binary_path).file_name().and_then(|n| n.to_str()).unwrap_or("service");

    info!("   Binary: {} ({} bytes)", binary_filename, binary_data.len());
    info!("   Service name: {}", service_name);
    info!("   Environment vars: {}", env_vars.len());

    // Build multipart form
    let form = Form::new()
        .text("service_name", service_name.to_string())
        .text("env_vars", serde_json::to_string(&env_vars)?)
        .text("auto_start", "true")
        .part("binary", Part::bytes(binary_data).file_name(binary_filename.to_string()));

    // Send deployment request
    let client =
        IpcHttpClient::new().await.map_err(|e| anyhow!("Failed to create HTTP client: {e}"))?;
    let url = format!("{tower_endpoint}/api/deployment/binary");

    info!("📡 Sending deployment request to {}", url);

    let response = client
        .post(&url)
        .await
        .multipart(form)
        .send()
        .await
        .map_err(|e| anyhow!("HTTP request failed: {e}"))?;

    if !response.is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow!("Deployment failed with status {status}: {error_text}"));
    }

    let deployment_response: DeploymentResponse =
        response.json().await.map_err(|e| anyhow!("Failed to parse response: {e}"))?;

    info!("✅ Deployment successful: {}", deployment_response.deployment_id);
    if let Some(ref url) = deployment_response.service_url {
        info!("   Service URL: {}", url);
    }

    Ok(deployment_response)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    fn sample_capabilities_json() -> &'static str {
        r#"{
            "node_id": "node-1",
            "network": {
                "type": "lan",
                "bandwidth_estimate": {
                    "download_mbps": 100,
                    "upload_mbps": 100,
                    "latency_ms": 5,
                    "confidence": "high"
                }
            },
            "deployment_methods": {
                "single": {
                    "enabled": true,
                    "max_size_mb": 50,
                    "compression_supported": [],
                    "recommended_for": "small"
                },
                "chunked": {
                    "enabled": true,
                    "max_total_size_mb": 500,
                    "chunk_size_mb": 10,
                    "max_chunks": 100,
                    "compression_supported": [],
                    "recommended_for": "large"
                },
                "streaming": {
                    "enabled": false,
                    "unlimited": false,
                    "compression_supported": [],
                    "recommended_for": "huge"
                }
            },
            "resources": {
                "available_storage_gb": 100,
                "available_memory_gb": 16,
                "cpu_cores": 8,
                "cpu_load_percent": 0.1,
                "max_concurrent_deployments": 4,
                "current_deployments": 0
            }
        }"#
    }

    #[test]
    fn deployment_response_json_roundtrip() {
        let original = DeploymentResponse {
            deployment_id: "dep-1".into(),
            status: "ok".into(),
            message: "deployed".into(),
            service_url: Some("http://localhost:8080".into()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: DeploymentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.deployment_id, original.deployment_id);
        assert_eq!(parsed.status, original.status);
        assert_eq!(parsed.message, original.message);
        assert_eq!(parsed.service_url, original.service_url);
    }

    #[test]
    fn select_deployment_method_fallback_when_no_capabilities() {
        assert!(matches!(select_deployment_method(None, 1.0), SelectedMethod::Fallback));
    }

    #[test]
    fn select_deployment_method_single_when_under_limit() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).unwrap();
        let m = select_deployment_method(Some(&caps), 10.0);
        assert!(matches!(m, SelectedMethod::Single));
    }

    #[test]
    fn select_deployment_method_chunked_when_over_single_limit() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).unwrap();
        let m = select_deployment_method(Some(&caps), 100.0);
        assert!(matches!(m, SelectedMethod::Chunked { .. }));
    }

    #[test]
    fn deployment_capabilities_deserialize() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).unwrap();
        assert_eq!(caps.node_id, "node-1");
        assert_eq!(caps.network.network_type, "lan");
        assert!(caps.deployment_methods.single.enabled);
    }

    #[test]
    fn deployment_response_serializes_none_service_url() {
        let r = DeploymentResponse {
            deployment_id: "d".into(),
            status: "pending".into(),
            message: "m".into(),
            service_url: None,
        };
        let json = serde_json::to_string(&r).expect("serialize DeploymentResponse");
        let back: DeploymentResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.service_url.is_none(), "optional service_url should round-trip as None");
    }

    #[test]
    fn select_deployment_method_exact_single_limit_uses_chunked() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).expect("sample caps");
        // Selection uses strict `<` for single max, so size == max uses chunked path
        let max = f64::from(caps.deployment_methods.single.max_size_mb);
        let m = select_deployment_method(Some(&caps), max);
        assert!(
            matches!(m, SelectedMethod::Chunked { .. }),
            "binary size equal to single max should not use Single (strict <): got {m:?}"
        );
    }

    #[test]
    fn select_deployment_method_single_disabled_uses_chunked() {
        let mut caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).expect("sample caps");
        caps.deployment_methods.single.enabled = false;
        let m = select_deployment_method(Some(&caps), 5.0);
        assert!(
            matches!(m, SelectedMethod::Chunked { .. }),
            "when single upload disabled, small binaries should use chunked: got {m:?}"
        );
    }

    #[test]
    fn select_deployment_method_streams_when_chunked_too_small() {
        let json = r#"{
            "node_id": "n",
            "network": {
                "type": "wan",
                "bandwidth_estimate": {
                    "download_mbps": 10,
                    "upload_mbps": 10,
                    "latency_ms": 100,
                    "confidence": "low"
                }
            },
            "deployment_methods": {
                "single": {
                    "enabled": true,
                    "max_size_mb": 10,
                    "compression_supported": [],
                    "recommended_for": "small"
                },
                "chunked": {
                    "enabled": true,
                    "max_total_size_mb": 100,
                    "chunk_size_mb": 10,
                    "max_chunks": 10,
                    "compression_supported": [],
                    "recommended_for": "large"
                },
                "streaming": {
                    "enabled": true,
                    "unlimited": true,
                    "compression_supported": [],
                    "recommended_for": "huge"
                }
            },
            "resources": {
                "available_storage_gb": 1,
                "available_memory_gb": 1,
                "cpu_cores": 1,
                "cpu_load_percent": 0.5,
                "max_concurrent_deployments": 1,
                "current_deployments": 0
            }
        }"#;
        let caps: DeploymentCapabilities = serde_json::from_str(json).expect("caps");
        let m = select_deployment_method(Some(&caps), 200.0);
        assert!(
            matches!(m, SelectedMethod::Streaming),
            "over chunked max with streaming enabled should select Streaming: got {m:?}"
        );
    }

    #[test]
    fn select_deployment_method_fallback_when_all_methods_unusable() {
        let json = r#"{
            "node_id": "n",
            "network": {
                "type": "wan",
                "bandwidth_estimate": {
                    "download_mbps": 1,
                    "upload_mbps": 1,
                    "latency_ms": 500,
                    "confidence": "low"
                }
            },
            "deployment_methods": {
                "single": {
                    "enabled": false,
                    "max_size_mb": 100,
                    "compression_supported": [],
                    "recommended_for": ""
                },
                "chunked": {
                    "enabled": false,
                    "max_total_size_mb": 100,
                    "chunk_size_mb": 10,
                    "max_chunks": 10,
                    "compression_supported": [],
                    "recommended_for": ""
                },
                "streaming": {
                    "enabled": false,
                    "unlimited": false,
                    "compression_supported": [],
                    "recommended_for": ""
                }
            },
            "resources": {
                "available_storage_gb": 1,
                "available_memory_gb": 1,
                "cpu_cores": 1,
                "cpu_load_percent": 0.9,
                "max_concurrent_deployments": 1,
                "current_deployments": 1
            }
        }"#;
        let caps: DeploymentCapabilities = serde_json::from_str(json).expect("caps");
        let m = select_deployment_method(Some(&caps), 1.0);
        assert!(
            matches!(m, SelectedMethod::Fallback),
            "when no method applies, expect Fallback: got {m:?}"
        );
    }

    #[test]
    fn selected_method_clone_and_debug() {
        let a = SelectedMethod::Single;
        let b = a.clone();
        assert!(matches!(b, SelectedMethod::Single));
        let dbg = format!("{a:?}");
        assert!(dbg.contains("Single"), "SelectedMethod should implement Debug: {dbg}");
    }

    #[test]
    fn select_deployment_method_zero_mb_uses_single_when_under_limit() {
        let caps: DeploymentCapabilities =
            serde_json::from_str(sample_capabilities_json()).expect("sample caps");
        let m = select_deployment_method(Some(&caps), 0.0);
        assert!(
            matches!(m, SelectedMethod::Single),
            "0MB should be strictly under single max and select Single: got {m:?}"
        );
    }

    #[test]
    fn select_deployment_method_oversized_when_chunked_and_streaming_disabled_uses_fallback() {
        let json = r#"{
            "node_id": "n",
            "network": {"type": "wan", "bandwidth_estimate": {"download_mbps":1,"upload_mbps":1,"latency_ms":1,"confidence":"low"}},
            "deployment_methods": {
                "single": {"enabled": true, "max_size_mb": 10, "compression_supported": [], "recommended_for": ""},
                "chunked": {"enabled": true, "max_total_size_mb": 20, "chunk_size_mb": 5, "max_chunks": 10, "compression_supported": [], "recommended_for": ""},
                "streaming": {"enabled": false, "unlimited": false, "compression_supported": [], "recommended_for": ""}
            },
            "resources": {"available_storage_gb": 1, "available_memory_gb": 1, "cpu_cores": 1, "cpu_load_percent": 0.0, "max_concurrent_deployments": 1, "current_deployments": 0}
        }"#;
        let caps: DeploymentCapabilities = serde_json::from_str(json).expect("caps");
        let m = select_deployment_method(Some(&caps), 100.0);
        assert!(
            matches!(m, SelectedMethod::Fallback),
            "over chunked max with streaming off should Fallback: got {m:?}"
        );
    }

    #[test]
    fn deployment_capabilities_rejects_invalid_json() {
        let err = serde_json::from_str::<DeploymentCapabilities>("not json").unwrap_err();
        assert!(
            err.to_string().contains("expected") || err.to_string().contains("EOF"),
            "deserialize error should mention parse failure: {err}"
        );
    }

    #[test]
    fn resource_info_accepts_zero_cpu_and_storage() {
        let json = r#"{
            "node_id": "n",
            "network": {"type": "x", "bandwidth_estimate": {"download_mbps":0,"upload_mbps":0,"latency_ms":0,"confidence":"low"}},
            "deployment_methods": {
                "single": {"enabled": true, "max_size_mb": 1, "compression_supported": [], "recommended_for": ""},
                "chunked": {"enabled": false, "max_total_size_mb": 0, "chunk_size_mb": 1, "max_chunks": 0, "compression_supported": [], "recommended_for": ""},
                "streaming": {"enabled": false, "unlimited": false, "compression_supported": [], "recommended_for": ""}
            },
            "resources": {
                "available_storage_gb": 0,
                "available_memory_gb": 0,
                "cpu_cores": 0,
                "cpu_load_percent": 0.0,
                "max_concurrent_deployments": 0,
                "current_deployments": 0
            }
        }"#;
        let caps: DeploymentCapabilities = serde_json::from_str(json).expect("zero resources");
        assert_eq!(caps.resources.cpu_cores, 0);
        assert_eq!(caps.resources.available_storage_gb, 0);
    }
}
