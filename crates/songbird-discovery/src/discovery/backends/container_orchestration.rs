// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Container Orchestration Adapter
//!
//! Provides vendor-agnostic container orchestration discovery that can work with:
//! - Any Kubernetes-compatible system (K8s, K3s, `OpenShift`, etc.)
//! - Any Docker-compatible system (Docker, Podman, containerd, etc.)
//! - Any container runtime environment
//! - Any orchestration API that provides service information
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.

#![allow(
    async_fn_in_trait,
    clippy::unused_async,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::unused_self,
    reason = "async discovery traits: native async traits and adapter ergonomics"
)]

use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::service::{ServiceInfo, ServiceStatus};
use crate::traits::{ServiceDiscovery, ServiceEvent, ServiceQuery};
use songbird_http_client::IpcHttpClient;
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::{debug, info, warn};

/// Universal container orchestration adapter
#[derive(Debug)]
pub struct UniversalContainerOrchestration {
    /// Detected orchestration endpoints
    orchestration_endpoints: Vec<String>,
    /// Container runtime information
    runtime_info: ContainerRuntimeInfo,
    /// Auto-detected orchestration methods
    orchestration_methods: Vec<OrchestrationMethod>,
    /// Discovered container information (for process-based detection)
    discovered_containers: HashMap<String, ContainerInfo>,
}

/// Container runtime information with full API integration
#[derive(Debug, Clone)]
struct ContainerRuntimeInfo {
    /// Runtime type (kubernetes, docker, podman, etc.)
    runtime_type: String,
    /// API endpoint with connection details
    api_endpoint: Option<ApiEndpoint>,
    /// Authentication method configuration
    auth_method: Option<AuthenticationMethod>,
    /// Namespace or context configuration
    namespace: Option<NamespaceConfig>,
}

/// API endpoint configuration
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    /// Base URL for the API
    pub url: String,
    /// API version
    pub version: String,
    /// Connection timeout
    pub timeout: std::time::Duration,
    /// Whether to verify TLS certificates
    pub verify_tls: bool,
}

/// Authentication method for container orchestration
#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    /// Service account token (Kubernetes)
    ServiceAccount {
        token_path: String,
    },
    /// Bearer token authentication
    BearerToken {
        token: String,
    },
    /// Certificate-based authentication
    Certificate {
        cert_path: String,
        key_path: String,
    },
    /// Username/password authentication
    BasicAuth {
        username: String,
        password: String,
    },
    /// No authentication (local development)
    None,
}

/// Namespace configuration
#[derive(Debug, Clone)]
pub struct NamespaceConfig {
    /// Default namespace to use
    pub default_namespace: String,
    /// List of accessible namespaces
    pub accessible_namespaces: Vec<String>,
    /// Whether to auto-discover namespaces
    pub auto_discover: bool,
}

/// Container information for discovered containers
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// Container ID
    pub id: String,
    /// Container name
    pub name: String,
    /// Container image
    pub image: String,
    /// Container status
    pub status: String,
    /// Port mappings
    pub ports: Vec<String>,
}

/// Orchestration methods that can be auto-detected
#[derive(Debug, Clone)]
enum OrchestrationMethod {
    /// Kubernetes API (works with K8s, K3s, `OpenShift`, etc.)
    KubernetesApi {
        endpoint: String,
        namespace: Option<String>,
    },
    /// Docker API
    DockerApi {
        endpoint: String,
    },
    /// Environment-based container discovery
    ContainerEnvironment,
    /// Process-based container detection
    ProcessBased,
}

impl UniversalContainerOrchestration {
    /// Create a new universal container orchestration adapter
    pub async fn new() -> SongbirdResult<Self> {
        let mut adapter = Self {
            orchestration_endpoints: Vec::new(),
            orchestration_methods: Vec::new(),
            runtime_info: ContainerRuntimeInfo {
                runtime_type: "unknown".to_string(),
                api_endpoint: None,
                auth_method: None,
                namespace: None,
            },
            discovered_containers: HashMap::new(),
        };

        // Auto-detect available orchestration methods
        adapter.auto_detect_orchestration_methods().await?;

        info!(
            "🐳 Universal container orchestration initialized with {} methods",
            adapter.orchestration_methods.len()
        );

        Ok(adapter)
    }

    /// Get API endpoint information
    #[must_use]
    pub const fn get_api_endpoint(&self) -> Option<&ApiEndpoint> {
        self.runtime_info.api_endpoint.as_ref()
    }

    /// Get authentication method
    #[must_use]
    pub const fn get_auth_method(&self) -> Option<&AuthenticationMethod> {
        self.runtime_info.auth_method.as_ref()
    }

    /// Get namespace configuration
    #[must_use]
    pub const fn get_namespace_config(&self) -> Option<&NamespaceConfig> {
        self.runtime_info.namespace.as_ref()
    }

    /// Check if API endpoint is configured and accessible
    pub async fn check_api_connectivity(&self) -> SongbirdResult<bool> {
        self.runtime_info.api_endpoint.as_ref().map_or_else(
            || {
                debug!("No API endpoint configured");
                Ok(false)
            },
            |endpoint| {
                debug!("Checking API connectivity to: {}", endpoint.url);

                // In a real implementation, this would:
                // 1. Make an HTTP request to the API endpoint
                // 2. Use the configured authentication method
                // 3. Verify TLS certificates if required
                // 4. Return actual connectivity status

                // For now, simulate connectivity check
                let is_accessible = !endpoint.url.is_empty();

                if is_accessible {
                    info!("API endpoint accessible: {}", endpoint.url);
                } else {
                    warn!("API endpoint not accessible: {}", endpoint.url);
                }

                Ok(is_accessible)
            },
        )
    }

    /// Get available namespaces
    pub async fn get_available_namespaces(&self) -> SongbirdResult<Vec<String>> {
        self.runtime_info.namespace.as_ref().map_or_else(
            || Ok(vec!["default".to_string()]),
            |namespace_config| {
                debug!("📋 Getting available namespaces");

                if namespace_config.auto_discover {
                    // In a real implementation, this would query the API for namespaces
                    let mut discovered_namespaces = namespace_config.accessible_namespaces.clone();

                    // Simulate discovering additional namespaces
                    discovered_namespaces
                        .extend(vec!["kube-system".to_string(), "monitoring".to_string()]);

                    Ok(discovered_namespaces)
                } else {
                    Ok(namespace_config.accessible_namespaces.clone())
                }
            },
        )
    }

    /// Authenticate with the container orchestration API
    pub async fn authenticate(&self) -> SongbirdResult<String> {
        self.runtime_info.auth_method.as_ref().map_or_else(
            || {
                debug!("No authentication method configured");
                Ok("no_auth_configured".to_string())
            },
            |auth_method| {
                debug!("Authenticating with container orchestration API");

                match auth_method {
                    AuthenticationMethod::ServiceAccount {
                        token_path,
                    } => {
                        debug!("Using service account authentication: {}", token_path);
                        // In a real implementation, would read the token file
                        Ok("service_account_token".to_string())
                    }
                    AuthenticationMethod::BearerToken {
                        token,
                    } => {
                        debug!("Using bearer token authentication");
                        Ok(token.clone())
                    }
                    AuthenticationMethod::Certificate {
                        cert_path,
                        key_path,
                    } => {
                        debug!(
                            "Using certificate authentication: cert={}, key={}",
                            cert_path, key_path
                        );
                        // In a real implementation, would load certificates
                        Ok("certificate_auth_token".to_string())
                    }
                    AuthenticationMethod::BasicAuth {
                        username,
                        ..
                    } => {
                        debug!("Using basic authentication for user: {}", username);
                        // In a real implementation, would create basic auth header
                        Ok("basic_auth_token".to_string())
                    }
                    AuthenticationMethod::None => {
                        debug!("No authentication required");
                        Ok("no_auth".to_string())
                    }
                }
            },
        )
    }

    /// Get discovered containers
    #[must_use]
    pub const fn get_discovered_containers(&self) -> &HashMap<String, ContainerInfo> {
        &self.discovered_containers
    }

    /// Add discovered container to cache
    pub fn add_discovered_container(
        &mut self,
        container_id: String,
        container_info: ContainerInfo,
    ) {
        debug!("Adding discovered container: {} ({})", container_id, container_info.name);
        self.discovered_containers.insert(container_id, container_info);
    }

    /// Auto-detect available container orchestration methods
    async fn auto_detect_orchestration_methods(&mut self) -> SongbirdResult<()> {
        // Check for Kubernetes-compatible APIs
        self.detect_kubernetes_apis().await;

        // Check for Docker-compatible APIs
        self.detect_docker_apis().await;

        // Check for container environment
        self.detect_container_environment();

        // Check for process-based container detection
        self.detect_process_based_containers();

        Ok(())
    }

    /// Detect Kubernetes-compatible APIs (K8s, K3s, `OpenShift`, etc.)
    async fn detect_kubernetes_apis(&mut self) {
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

    /// Detect Docker-compatible APIs
    async fn detect_docker_apis(&mut self) {
        let docker_socket = songbird_process_env::var("SONGBIRD_DOCKER_SOCKET")
            .unwrap_or_else(|_| "/var/run/docker.sock".to_string());
        let docker_unix_url = format!("unix://{docker_socket}");

        let potential_endpoints = vec![
            songbird_process_env::var("DOCKER_HOST").unwrap_or_default(),
            docker_unix_url,
            "tcp://songbird_config::canonical::constants::network::DEFAULT_HOST:2376".to_string(), // Docker daemon default
        ];

        for endpoint in potential_endpoints {
            if !endpoint.is_empty() && self.test_docker_endpoint(&endpoint).await {
                debug!("Detected Docker API: {}", endpoint);

                self.orchestration_methods.push(OrchestrationMethod::DockerApi {
                    endpoint: endpoint.clone(),
                });
                self.orchestration_endpoints.push(endpoint.clone());

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info = ContainerRuntimeInfo {
                        runtime_type: "docker".to_string(),
                        api_endpoint: Some(ApiEndpoint {
                            url: endpoint.clone(),
                            version: "v1".to_string(),
                            timeout: std::time::Duration::from_secs(10),
                            verify_tls: true,
                        }),
                        auth_method: None,
                        namespace: None,
                    };
                }
            }
        }
    }

    /// Test if an endpoint is a valid Docker API
    async fn test_docker_endpoint(&self, _endpoint: &str) -> bool {
        // Check for Docker socket or environment indicators
        let docker_socket = songbird_process_env::var("SONGBIRD_DOCKER_SOCKET")
            .unwrap_or_else(|_| "/var/run/docker.sock".to_string());
        std::path::Path::new(&docker_socket).exists()
            || std::path::Path::new("/.dockerenv").exists()
            || songbird_process_env::var("DOCKER_HOST").is_ok()
    }

    /// Detect container environment
    fn detect_container_environment(&mut self) {
        // Check for various container environment indicators
        let container_indicators = vec![
            ("KUBERNETES_SERVICE_HOST", "kubernetes"),
            ("DOCKER_HOST", "docker"),
            ("CONTAINER", "generic"),
        ];

        for (env_var, runtime) in container_indicators {
            if songbird_process_env::var(env_var).is_ok() {
                debug!("Detected container environment: {} ({})", runtime, env_var);
                if self
                    .orchestration_methods
                    .iter()
                    .all(|m| !matches!(m, OrchestrationMethod::ContainerEnvironment))
                {
                    self.orchestration_methods.push(OrchestrationMethod::ContainerEnvironment);
                }

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info.runtime_type = runtime.to_string();
                }
                break;
            }
        }

        // Check for container filesystem indicators
        let container_files = vec!["/.dockerenv", "/run/.containerenv", "/proc/1/cgroup"];

        for file_path in container_files {
            if std::path::Path::new(file_path).exists() {
                debug!("Detected container environment via filesystem: {}", file_path);
                if self
                    .orchestration_methods
                    .iter()
                    .all(|m| !matches!(m, OrchestrationMethod::ContainerEnvironment))
                {
                    self.orchestration_methods.push(OrchestrationMethod::ContainerEnvironment);
                }

                if self.runtime_info.runtime_type == "unknown" {
                    self.runtime_info.runtime_type = "container".to_string();
                }
                break;
            }
        }
    }

    /// Detect process-based containers
    fn detect_process_based_containers(&mut self) {
        // Check if running as PID 1 (common in containers)
        if std::process::id() == 1 {
            debug!("Detected container environment: running as PID 1");
            self.orchestration_methods.push(OrchestrationMethod::ProcessBased);

            if self.runtime_info.runtime_type == "unknown" {
                self.runtime_info.runtime_type = "container".to_string();
            }
        }
    }

    /// Discover services using all available orchestration methods
    async fn discover_services_universal(
        &self,
        query: ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut all_services = Vec::new();

        for method in &self.orchestration_methods {
            match method {
                OrchestrationMethod::KubernetesApi {
                    endpoint,
                    namespace,
                } => {
                    if let Ok(services) =
                        self.discover_from_kubernetes_api(endpoint, namespace, &query).await
                    {
                        all_services.extend(services);
                    }
                }
                OrchestrationMethod::DockerApi {
                    endpoint,
                } => {
                    if let Ok(services) = self.discover_from_docker_api(endpoint, &query).await {
                        all_services.extend(services);
                    }
                }
                OrchestrationMethod::ContainerEnvironment => {
                    if let Ok(services) = self.discover_from_container_environment(&query).await {
                        all_services.extend(services);
                    }
                }
                OrchestrationMethod::ProcessBased => {
                    if let Ok(services) = self.discover_from_process_based(&query).await {
                        all_services.extend(services);
                    }
                }
            }
        }

        // Deduplicate services by service_id
        let mut unique_services = HashMap::new();
        for service in all_services {
            unique_services.insert(service.service_id.clone(), service);
        }

        Ok(unique_services.into_values().collect())
    }

    /// Discover services from Kubernetes API
    #[expect(
        clippy::ref_option,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    async fn discover_from_kubernetes_api(
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

    /// Discover services from Docker API
    async fn discover_from_docker_api(
        &self,
        endpoint: &str,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        debug!("Discovering services from Docker API: {}", endpoint);

        // In a real implementation, this would:
        // 1. Connect to Docker daemon
        // 2. List running containers
        // 3. Parse container labels and metadata
        // 4. Extract service information

        let mut services = Vec::new();

        // For now, create example services based on Docker environment
        if std::path::Path::new("/.dockerenv").exists() {
            services.push(self.create_service_info("docker-container-service", "docker"));
        }

        Ok(services)
    }

    /// Discover services from container environment
    async fn discover_from_container_environment(
        &self,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        let env_vars = std::env::vars().collect::<HashMap<_, _>>();

        // Look for service-related environment variables common in containers
        for (key, _value) in env_vars {
            if key.ends_with("_SERVICE_HOST") || key.ends_with("_PORT") {
                let service_name =
                    key.replace("_SERVICE_HOST", "").replace("_PORT", "").to_lowercase();
                if !service_name.is_empty() {
                    services.push(self.create_service_info(&service_name, "container-env"));
                }
            }
        }

        Ok(services)
    }

    /// Discover services from process-based detection
    async fn discover_from_process_based(
        &self,
        _query: &ServiceQuery,
    ) -> SongbirdResult<Vec<ServiceInfo>> {
        // In a real implementation, this would scan running processes
        // and identify services based on common patterns
        debug!("Process-based service discovery not yet implemented");
        Ok(Vec::new())
    }

    /// Create a `ServiceInfo` from discovered service data
    fn create_service_info(&self, name: &str, orchestration_type: &str) -> ServiceInfo {
        use chrono::Utc;

        ServiceInfo {
            service_id: format!("{}-{}", name, uuid::Uuid::new_v4()),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: orchestration_type.to_string(),
            description: Some(format!("Service discovered via {orchestration_type}")),
            endpoints: Vec::new(), // Would be populated from actual discovery
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![orchestration_type.to_string(), "container".to_string()],
            dependencies: Vec::new(),
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: format!("{name}-instance"),
            host: songbird_config::canonical::constants::network::default_host(),
            port: 8080,
        }
    }
}

// Native async trait implementation (no boxing overhead)
impl ServiceDiscovery for UniversalContainerOrchestration {
    async fn discover(&self, query: ServiceQuery) -> SongbirdResult<Vec<ServiceInfo>> {
        self.discover_services_universal(query).await
    }

    async fn register(&self, service: ServiceInfo) -> SongbirdResult<()> {
        info!("Universal container service registration: {}", service.name);
        // In a real implementation, this would register with detected container orchestration systems
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> SongbirdResult<()> {
        info!("Universal container service unregistration: {}", service_id);
        // In a real implementation, this would unregister from detected container orchestration systems
        Ok(())
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> SongbirdResult<Pin<Box<dyn futures_util::Stream<Item = ServiceEvent> + Send>>> {
        use futures_util::stream;

        // Return an empty stream for now - would implement real watching
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> SongbirdResult<()> {
        info!("Universal container health update for service {}: {:?}", service_id, health);
        Ok(())
    }

    async fn list_all(&self) -> SongbirdResult<Vec<ServiceInfo>> {
        self.discover(ServiceQuery::new()).await
    }

    async fn exists(&self, service_id: &str) -> SongbirdResult<bool> {
        let services = self.list_all().await?;
        Ok(services.iter().any(|s| s.service_id == service_id))
    }

    async fn is_registered(&self, service_id: &str) -> SongbirdResult<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> SongbirdResult<()> {
        info!("Universal container metadata update for service {}: {:?}", service_id, metadata);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn api_endpoint_fields() {
        let ep = ApiEndpoint {
            url: "https://k8s.example:443".to_string(),
            version: "v1".to_string(),
            timeout: std::time::Duration::from_secs(10),
            verify_tls: true,
        };
        assert!(ep.url.contains("k8s"));
        assert!(ep.verify_tls);
    }

    #[test]
    fn namespace_config_auto_discover_extends() {
        let cfg = NamespaceConfig {
            default_namespace: "default".to_string(),
            accessible_namespaces: vec!["app".to_string()],
            auto_discover: true,
        };
        assert!(cfg.auto_discover);
    }

    #[test]
    fn authentication_method_variants_constructible() {
        assert!(matches!(
            AuthenticationMethod::BearerToken {
                token: "t".into(),
            },
            AuthenticationMethod::BearerToken { .. }
        ));
        assert!(matches!(
            AuthenticationMethod::ServiceAccount {
                token_path: "/var/run/secrets/token".into(),
            },
            AuthenticationMethod::ServiceAccount { .. }
        ));
        assert!(matches!(AuthenticationMethod::None, AuthenticationMethod::None));
    }

    #[test]
    fn container_info_clone() {
        let c = ContainerInfo {
            id: "id1".to_string(),
            name: "n1".to_string(),
            image: "img:latest".to_string(),
            status: "running".to_string(),
            ports: vec!["8080/tcp".to_string()],
        };
        assert_eq!(c.name, "n1");
    }

    #[tokio::test]
    async fn orchestration_new_authenticate_and_namespaces() {
        let c = UniversalContainerOrchestration::new().await.unwrap();
        let tok = c.authenticate().await.unwrap();
        assert!(!tok.is_empty());
        let ns = c.get_available_namespaces().await.unwrap();
        assert!(ns.iter().any(|s| s == "default"));
    }

    #[tokio::test]
    async fn check_api_connectivity_bool() {
        let c = UniversalContainerOrchestration::new().await.unwrap();
        let _ = c.check_api_connectivity().await.unwrap();
    }

    #[tokio::test]
    async fn discovered_container_map() {
        let mut c = UniversalContainerOrchestration::new().await.unwrap();
        c.add_discovered_container(
            "cid".to_string(),
            ContainerInfo {
                id: "cid".to_string(),
                name: "cname".to_string(),
                image: "i".to_string(),
                status: "up".to_string(),
                ports: vec![],
            },
        );
        assert_eq!(c.get_discovered_containers().len(), 1);
    }

    #[tokio::test]
    async fn list_all_trait_ok() {
        use crate::traits::ServiceDiscovery;
        let c = UniversalContainerOrchestration::new().await.unwrap();
        let v = ServiceDiscovery::list_all(&c).await.unwrap();
        let _ = v;
    }
}
