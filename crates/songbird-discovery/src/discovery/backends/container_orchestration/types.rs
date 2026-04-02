// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Core types for the universal container orchestration adapter.

/// Universal container orchestration adapter
#[derive(Debug)]
pub struct UniversalContainerOrchestration {
    /// Detected orchestration endpoints
    pub(crate) orchestration_endpoints: Vec<String>,
    /// Container runtime information
    pub(crate) runtime_info: ContainerRuntimeInfo,
    /// Auto-detected orchestration methods
    pub(crate) orchestration_methods: Vec<OrchestrationMethod>,
    /// Discovered container information (for process-based detection)
    pub(crate) discovered_containers: std::collections::HashMap<String, ContainerInfo>,
}

/// Container runtime information with full API integration
#[derive(Debug, Clone)]
pub struct ContainerRuntimeInfo {
    /// Runtime type (kubernetes, docker, podman, etc.)
    pub(crate) runtime_type: String,
    /// API endpoint with connection details
    pub(crate) api_endpoint: Option<ApiEndpoint>,
    /// Authentication method configuration
    pub(crate) auth_method: Option<AuthenticationMethod>,
    /// Namespace or context configuration
    pub(crate) namespace: Option<NamespaceConfig>,
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
pub enum OrchestrationMethod {
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
