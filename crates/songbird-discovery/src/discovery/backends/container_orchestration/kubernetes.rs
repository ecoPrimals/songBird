// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Kubernetes-compatible API detection and service discovery.

use super::types::{
    ApiEndpoint, AuthenticationMethod, ContainerRuntimeInfo, NamespaceConfig, OrchestrationMethod,
    UniversalContainerOrchestration,
};
use crate::traits::ServiceQuery;
use crate::traits::service::ServiceInfo;
use songbird_http_client::IpcHttpClient;
use songbird_types::errors::SongbirdResult;
use tracing::debug;

impl UniversalContainerOrchestration {
    /// Detect Kubernetes-compatible APIs (K8s, K3s, `OpenShift`, etc.)
    pub(super) async fn detect_kubernetes_apis(&mut self) {
        use songbird_config::canonical::constants;

        // Get configurable defaults
        let k8s_protocol =
            songbird_process_env::var("K8S_PROTOCOL").unwrap_or_else(|_| "https".to_string());
        let k8s_default_port =
            songbird_process_env::var("K8S_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(443); // Standard HTTPS port for Kubernetes
        let k8s_api_port = songbird_process_env::var("K8S_API_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6443); // Standard Kubernetes API port

        let potential_endpoints = vec![
            songbird_process_env::var("KUBERNETES_SERVICE_HOST")
                .ok()
                .map(|host| {
                    let port = songbird_process_env::var("KUBERNETES_SERVICE_PORT")
                        .unwrap_or_else(|_| k8s_default_port.to_string());
                    format!("{k8s_protocol}://{host}:{port}")
                })
                .unwrap_or_default(),
            songbird_process_env::var("KUBECONFIG").unwrap_or_default(),
            songbird_process_env::var("K8S_CLUSTER_ENDPOINT").unwrap_or_else(|_| {
                format!("{k8s_protocol}://kubernetes.default.svc.cluster.local")
            }),
            songbird_process_env::var("K8S_LOCAL_ENDPOINT").unwrap_or_else(|_| {
                format!("{k8s_protocol}://{}:{k8s_api_port}", constants::network::default_host())
            }),
        ];

        for endpoint in potential_endpoints {
            if !endpoint.is_empty() && self.test_kubernetes_endpoint(&endpoint).await {
                let namespace = songbird_process_env::var("KUBERNETES_NAMESPACE").ok();
                debug!("Detected Kubernetes API: {} (namespace: {:?})", endpoint, namespace);

                self.orchestration_methods.push(OrchestrationMethod::KubernetesApi {
                    endpoint: endpoint.clone(),
                    namespace: namespace.clone(),
                });
                self.orchestration_endpoints.push(endpoint.clone());

                // Get API version from environment or default
                let k8s_api_version = songbird_process_env::var("K8S_API_VERSION")
                    .unwrap_or_else(|_| "v1".to_string());
                let k8s_timeout_secs = songbird_process_env::var("K8S_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                let k8s_verify_tls = songbird_process_env::var("K8S_VERIFY_TLS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true);

                self.runtime_info = ContainerRuntimeInfo {
                    runtime_type: "kubernetes".to_string(),
                    api_endpoint: Some(ApiEndpoint {
                        url: endpoint.clone(),
                        version: k8s_api_version,
                        timeout: std::time::Duration::from_secs(k8s_timeout_secs),
                        verify_tls: k8s_verify_tls,
                    }),
                    auth_method: Some(AuthenticationMethod::ServiceAccount {
                        token_path: songbird_process_env::var("KUBECONFIG")
                            .or_else(|_| songbird_process_env::var("K8S_TOKEN_PATH"))
                            .unwrap_or_else(|_| {
                                format!(
                                    "{}/.kube/config",
                                    songbird_process_env::var("HOME")
                                        .unwrap_or_else(|_| "/root".to_string())
                                )
                            }),
                    }),
                    namespace: Some(NamespaceConfig {
                        default_namespace: "default".to_string(),
                        accessible_namespaces: vec!["default".to_string()],
                        auto_discover: false,
                    }),
                };
            }
        }
    }

    /// Test if an endpoint is a valid Kubernetes API
    async fn test_kubernetes_endpoint(&self, endpoint: &str) -> bool {
        // Try common Kubernetes API endpoints
        let test_paths = vec!["/api/v1", "/apis", "/version"];

        let client = match IpcHttpClient::new().await {
            Ok(c) => c,
            Err(_) => return false,
        };

        for path in test_paths {
            let url = format!("{}{}", endpoint.trim_end_matches('/'), path);
            // In a real implementation, this would handle authentication
            if client.get(&url).await.is_ok() {
                return true;
            }
        }

        // Also check for kubeconfig file
        if endpoint.contains("kubeconfig") && std::path::Path::new(&endpoint).exists() {
            return true;
        }

        false
    }

    /// Discover services from Kubernetes API
    #[expect(
        clippy::ref_option,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    pub(super) async fn discover_from_kubernetes_api(
        &self,
        endpoint: &str,
        namespace: &Option<String>,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering services from Kubernetes API: {}", endpoint);

        // In a real implementation, this would:
        // 1. Use kubectl or Kubernetes client library
        // 2. Query services, pods, deployments
        // 3. Parse Kubernetes service definitions
        // 4. Handle authentication and authorization

        let mut services = Vec::new();

        // For now, create example services based on common Kubernetes patterns
        if namespace.is_some() {
            services.push(self.create_service_info("kubernetes-service-example", "kubernetes"));
        }

        Ok(services)
    }
}
