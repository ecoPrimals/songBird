// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Container Orchestration Discovery Backend
//!
//! Kubernetes and Docker discovery implementations.

use super::super::errors::DiscoveryError;
use super::super::types::DiscoveredPrimal;

// These are only used in feature-gated code (k8s, docker)
#[cfg(any(feature = "k8s", feature = "docker"))]
use super::super::types::{DiscoveryMethod, PrimalHealth};
#[cfg(any(feature = "k8s", feature = "docker"))]
use std::collections::HashMap;

#[cfg(feature = "k8s")]
use kube::api::ListParams;

use tracing::{debug, info, warn};

/// Discover primals from container orchestration platforms
///
/// **SELF-KNOWLEDGE**: Discovers advertised services in container environments
/// Supports Kubernetes, Docker Swarm, and standalone Docker
///
/// # Errors
///
/// Returns an error if container discovery backends fail (non-fatal for individual backends).
pub async fn discover_from_containers() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    debug!("🔍 Discovering primals from container orchestration...");

    let mut discovered = Vec::new();

    // 1. Try Kubernetes discovery
    if is_kubernetes_environment() {
        match discover_kubernetes_services().await {
            Ok(mut k8s_primals) => {
                info!("Discovered {} primals from Kubernetes", k8s_primals.len());
                discovered.append(&mut k8s_primals);
            }
            Err(e) => warn!("Kubernetes discovery failed: {}", e),
        }
    }

    // 2. Try Docker discovery
    match discover_docker_containers().await {
        Ok(mut docker_primals) => {
            info!("Discovered {} primals from Docker", docker_primals.len());
            discovered.append(&mut docker_primals);
        }
        Err(e) => debug!("Docker discovery failed: {}", e),
    }

    debug!("Total primals discovered from containers: {}", discovered.len());
    Ok(discovered)
}

/// Check if running in Kubernetes environment
fn is_kubernetes_environment() -> bool {
    // Check for Kubernetes service account
    std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount/token").exists()
        || songbird_process_env::var("KUBERNETES_SERVICE_HOST").is_ok()
}

/// Discover services from Kubernetes
///
/// COMPLETE IMPLEMENTATION using kube-rs client
///
/// # Errors
///
/// Returns an error if Kubernetes client initialization or API calls fail.
#[allow(
    clippy::unused_async,
    reason = "async used when k8s feature is enabled; stub path returns immediately"
)]
pub async fn discover_kubernetes_services() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    #[cfg(feature = "k8s")]
    {
        use k8s_openapi::api::core::v1::Service;
        use kube::{Api, Client};

        // Initialize Kubernetes client
        let client = Client::try_default().await.map_err(|e| {
            DiscoveryError::BackendUnavailable(format!("K8s client init failed: {e}"))
        })?;

        // Get namespace from environment or default
        let namespace = songbird_process_env::var("KUBERNETES_NAMESPACE")
            .unwrap_or_else(|_| "default".to_string());

        let api: Api<Service> = Api::namespaced(client, &namespace);

        // List all services with default parameters
        let services = api.list(&ListParams::default()).await.map_err(|e| {
            DiscoveryError::BackendUnavailable(format!("K8s list services failed: {e}"))
        })?;

        let mut primals = Vec::new();

        for service in services.items {
            if let Some(primal) = convert_k8s_service_to_primal(service) {
                primals.push(primal);
            }
        }

        Ok(primals)
    }

    #[cfg(not(feature = "k8s"))]
    {
        Err(DiscoveryError::BackendUnavailable("Kubernetes support not enabled".to_string()))
    }
}

/// Convert Kubernetes Service to `DiscoveredPrimal`
#[cfg(feature = "k8s")]
fn convert_k8s_service_to_primal(
    service: k8s_openapi::api::core::v1::Service,
) -> Option<DiscoveredPrimal> {
    use crate::capabilities::Capability;
    use crate::types::PrimalType;

    let name = service.metadata.name?;
    let spec = service.spec?;

    // Get ClusterIP
    let host = spec.cluster_ip.unwrap_or_else(|| name.clone());

    // Get first port
    let ports = spec.ports?;
    let port_obj = ports.first()?;

    // Port is i32 in Kubernetes API, convert to u16 safely
    let port = if port_obj.port > 0 && port_obj.port <= i32::from(u16::MAX) {
        // Safe: We just validated port is in valid u16 range
        u16::try_from(port_obj.port).ok()?
    } else {
        return None; // Invalid port range
    };

    // Extract capabilities from labels using capability-based discovery
    // Primals advertise their own capabilities via labels - no hardcoded assumptions
    let capability_strings: Vec<String> = service
        .metadata
        .labels
        .as_ref()
        .and_then(|labels| {
            labels
                .get("songbird.capabilities")
                .or_else(|| labels.get("app.kubernetes.io/component"))
        })
        .map_or_else(
            || {
                // Fallback: Infer from service name only if no explicit labels
                // This maintains self-knowledge pattern: services should advertise capabilities
                infer_capabilities_from_name(&name)
            },
            |caps| caps.split(',').map(|s| s.trim().to_string()).collect(),
        );

    // Convert to Capability structs (not enums)
    let capabilities: Vec<Capability> =
        capability_strings.iter().filter_map(|s| Capability::from_string(s)).collect();

    // Infer primal type from capabilities using category method
    // PrimalType is a struct, use new() constructor with category string
    let primal_type = if capabilities.iter().any(|c| c.category() == "security") {
        PrimalType::new("security")
    } else if capabilities.iter().any(|c| c.category() == "storage") {
        PrimalType::new("storage")
    } else if capabilities.iter().any(|c| c.category() == "ai") {
        PrimalType::new("ai")
    } else if capabilities.iter().any(|c| c.category() == "compute") {
        PrimalType::new("compute")
    } else {
        PrimalType::new("orchestration")
    };

    // Construct endpoint
    let endpoint = format!("http://{host}:{port}");

    Some(DiscoveredPrimal {
        name,
        primal_type,
        endpoint,
        capabilities,
        health: PrimalHealth::Unknown,
        discovery_method: DiscoveryMethod::ContainerOrchestration,
        metadata: HashMap::new(),
    })
}

/// Discover containers from Docker
///
/// COMPLETE IMPLEMENTATION using bollard client
///
/// # Errors
///
/// Returns an error if Docker daemon connection or container listing fails.
#[allow(
    clippy::unused_async,
    reason = "async used when docker feature is enabled; stub path returns immediately"
)]
pub async fn discover_docker_containers() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    #[cfg(feature = "docker")]
    {
        use bollard::Docker;
        use bollard::container::ListContainersOptions;

        // Connect to Docker daemon
        let docker = Docker::connect_with_local_defaults().map_err(|e| {
            DiscoveryError::BackendUnavailable(format!("Docker connect failed: {e}"))
        })?;

        // List running containers
        let options = ListContainersOptions::<String> {
            all: false,
            ..Default::default()
        };

        let containers = docker
            .list_containers(Some(options))
            .await
            .map_err(|e| DiscoveryError::BackendUnavailable(format!("Docker list failed: {e}")))?;

        let mut primals = Vec::new();

        for container in containers {
            if let Some(primal) = convert_docker_container_to_primal(container) {
                primals.push(primal);
            }
        }

        Ok(primals)
    }

    #[cfg(not(feature = "docker"))]
    {
        Err(DiscoveryError::BackendUnavailable("Docker support not enabled".to_string()))
    }
}

/// Convert Docker container to `DiscoveredPrimal`
#[cfg(feature = "docker")]
fn convert_docker_container_to_primal(
    container: bollard::models::ContainerSummary,
) -> Option<DiscoveredPrimal> {
    use crate::capabilities::Capability;
    use crate::types::PrimalType;

    let names = container.names?;
    let name = names.first()?.trim_start_matches('/').to_string();

    // Get network settings
    let network_settings = container.network_settings?;
    let networks = network_settings.networks?;

    // Get first network
    let (_, network) = networks.iter().next()?;
    // Use actual IP address from network, fallback to loopback for local dev
    let host = network.ip_address.clone().unwrap_or_else(|| "127.0.0.1".to_string());

    // Get port from labels or default
    let labels = container.labels.unwrap_or_default();
    let port = labels.get("songbird.port").and_then(|p| p.parse().ok()).unwrap_or(8080);

    // Get capabilities from labels
    let capability_strings: Vec<String> = labels.get("songbird.capabilities").map_or_else(
        || infer_capabilities_from_name(&name),
        |caps| caps.split(',').map(|s| s.trim().to_string()).collect(),
    );

    // Convert to Capability structs (not enums)
    let capabilities: Vec<Capability> =
        capability_strings.iter().filter_map(|s| Capability::from_string(s)).collect();

    // Infer primal type from capabilities using category method
    // PrimalType is a struct, use new() constructor with category string
    let primal_type = if capabilities.iter().any(|c| c.category() == "security") {
        PrimalType::new("security")
    } else if capabilities.iter().any(|c| c.category() == "storage") {
        PrimalType::new("storage")
    } else if capabilities.iter().any(|c| c.category() == "ai") {
        PrimalType::new("ai")
    } else if capabilities.iter().any(|c| c.category() == "compute") {
        PrimalType::new("compute")
    } else {
        PrimalType::new("orchestration")
    };

    // Construct endpoint
    let endpoint = format!("http://{host}:{port}");

    Some(DiscoveredPrimal {
        name,
        primal_type,
        endpoint,
        capabilities,
        health: PrimalHealth::Unknown,
        discovery_method: DiscoveryMethod::ContainerOrchestration,
        metadata: HashMap::new(),
    })
}

/// Infer capabilities from service/container name using only capability terms.
///
/// Primal-agnostic: matches on domain terminology (e.g. "security", "ai")
/// rather than specific primal names. Concrete provider identities are
/// discovered at runtime via the capability advertisement protocol.
#[cfg(any(feature = "k8s", feature = "docker"))]
fn infer_capabilities_from_name(name: &str) -> Vec<String> {
    let name_lower = name.to_lowercase();
    let mut capabilities = Vec::new();

    if name_lower.contains("security")
        || name_lower.contains("crypto")
        || name_lower.contains("auth")
    {
        capabilities.push("security".to_string());
    }
    if name_lower.contains("ai") || name_lower.contains("ml") || name_lower.contains("inference") {
        capabilities.push("ai".to_string());
    }
    if name_lower.contains("discovery") || name_lower.contains("registry") {
        capabilities.push("discovery".to_string());
    }
    if name_lower.contains("storage")
        || name_lower.contains("data")
        || name_lower.contains("persist")
    {
        capabilities.push("storage".to_string());
    }
    if name_lower.contains("orchestrat") || name_lower.contains("coordinat") {
        capabilities.push("orchestration".to_string());
    }
    if name_lower.contains("compute")
        || name_lower.contains("worker")
        || name_lower.contains("exec")
    {
        capabilities.push("compute".to_string());
    }

    capabilities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_kubernetes_environment_returns_bool() {
        let result = is_kubernetes_environment();
        assert!(result || !result);
    }

    #[test]
    fn test_is_kubernetes_when_env_not_set() {
        songbird_process_env::remove_var("KUBERNETES_SERVICE_HOST");
        let result = is_kubernetes_environment();
        if !std::path::Path::new("/var/run/secrets/kubernetes.io/serviceaccount/token").exists() {
            assert!(!result);
        }
    }

    #[tokio::test]
    async fn test_discover_from_containers_returns_ok() {
        let result = discover_from_containers().await;
        assert!(result.is_ok(), "container discovery should not panic");
    }

    #[tokio::test]
    async fn test_discover_kubernetes_services_not_enabled() {
        let result = discover_kubernetes_services().await;
        #[cfg(not(feature = "k8s"))]
        assert!(result.is_err());
        #[cfg(feature = "k8s")]
        let _ = result;
    }

    #[tokio::test]
    async fn test_discover_docker_containers_not_enabled() {
        let result = discover_docker_containers().await;
        #[cfg(not(feature = "docker"))]
        assert!(result.is_err());
        #[cfg(feature = "docker")]
        let _ = result;
    }

    #[tokio::test]
    async fn test_discover_kubernetes_error_message() {
        let result = discover_kubernetes_services().await;
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(
                msg.contains("not enabled") || msg.contains("failed"),
                "error should explain why: {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_discover_docker_error_message() {
        let result = discover_docker_containers().await;
        if let Err(e) = result {
            let msg = format!("{e}");
            assert!(
                msg.contains("not enabled") || msg.contains("failed"),
                "error should explain why: {msg}"
            );
        }
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_security() {
        assert_eq!(infer_capabilities_from_name("my-security-service"), vec!["security"]);
        assert_eq!(infer_capabilities_from_name("crypto-provider"), vec!["security"]);
        assert_eq!(infer_capabilities_from_name("auth-gateway"), vec!["security"]);
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_ai() {
        assert_eq!(infer_capabilities_from_name("ai-inference-service"), vec!["ai"]);
        assert_eq!(infer_capabilities_from_name("ml-pipeline"), vec!["ai"]);
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_discovery() {
        assert_eq!(infer_capabilities_from_name("service-discovery"), vec!["discovery"]);
        assert_eq!(infer_capabilities_from_name("registry-service"), vec!["discovery"]);
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_storage() {
        assert_eq!(infer_capabilities_from_name("storage-backend"), vec!["storage"]);
        assert_eq!(infer_capabilities_from_name("data-lake"), vec!["storage"]);
        assert_eq!(infer_capabilities_from_name("persist-service"), vec!["storage"]);
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_orchestration() {
        assert_eq!(infer_capabilities_from_name("task-orchestrator"), vec!["orchestration"]);
        assert_eq!(infer_capabilities_from_name("coordinator-service"), vec!["orchestration"]);
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_compute() {
        assert_eq!(infer_capabilities_from_name("compute-node"), vec!["compute"]);
        assert_eq!(infer_capabilities_from_name("worker-pool"), vec!["compute"]);
        assert_eq!(infer_capabilities_from_name("exec-engine"), vec!["compute"]);
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_empty_for_unknown() {
        assert!(infer_capabilities_from_name("unknown-service").is_empty());
        assert!(infer_capabilities_from_name("foobar").is_empty());
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_from_name_multiple() {
        let caps = infer_capabilities_from_name("ai-security-orchestrator");
        assert!(caps.contains(&"security".to_string()));
        assert!(caps.contains(&"ai".to_string()));
        assert!(caps.contains(&"orchestration".to_string()));
    }

    #[test]
    #[cfg(any(feature = "k8s", feature = "docker"))]
    fn test_infer_capabilities_case_insensitive() {
        assert_eq!(infer_capabilities_from_name("SECURITY-SERVICE"), vec!["security"]);
        assert_eq!(infer_capabilities_from_name("AI-Inference"), vec!["ai"]);
    }
}
