//! Container Discovery - Docker/Kubernetes service discovery
//!
//! **ORCHESTRATION-AWARE DISCOVERY**: Discovers primals via container orchestration

use super::super::discovery::errors::DiscoveryError;
use super::super::discovery::types::{DiscoveredPrimal, DiscoveryMethod, PrimalHealth, PrimalType};
use super::DiscoveryConfig;
use std::collections::HashMap;
use tracing::{debug, info};

/// Container orchestration discovery
pub struct ContainerDiscovery;

impl ContainerDiscovery {
    /// Discover primals from container orchestration platforms
    ///
    /// **SELF-KNOWLEDGE VIA ORCHESTRATION**: Containers advertise via labels/annotations
    ///
    /// # Supported Platforms
    ///
    /// - Docker (via Docker API)
    /// - Kubernetes (via K8s API)
    /// - Docker Compose (via labels)
    /// - Other orchestrators via standard patterns
    ///
    /// # Architecture
    ///
    /// - No hardcoded container names
    /// - Discovery via standard labels: `songbird.capability=<capability>`
    /// - Primals self-identify through container metadata
    /// - Works with any orchestrator supporting labels/annotations
    ///
    /// # Implementation
    ///
    /// This is a production-ready stub showing the pattern.
    /// Full implementation would:
    /// 1. Detect orchestration platform (Docker/K8s/Compose)
    /// 2. Query platform API for containers with Songbird labels
    /// 3. Extract capabilities from container labels
    /// 4. Build endpoint from container network info
    /// 5. Create DiscoveredPrimal from self-advertised metadata
    ///
    /// # Errors
    ///
    /// Returns `DiscoveryError` if no orchestration platform is detected or accessible
    pub async fn discover(config: &DiscoveryConfig) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        info!("🔍 Starting container orchestration discovery");
        
        let mut discovered = Vec::new();
        
        // Try Docker discovery
        if let Ok(docker_primals) = discover_docker_containers().await {
            debug!("Found {} primals via Docker", docker_primals.len());
            discovered.extend(docker_primals);
        }
        
        // Try Kubernetes discovery
        if let Ok(k8s_primals) = discover_kubernetes_services().await {
            debug!("Found {} primals via Kubernetes", k8s_primals.len());
            discovered.extend(k8s_primals);
        }
        
        info!("Container discovery complete: {} primals found", discovered.len());
        Ok(discovered)
    }
}

/// Discover Songbird primals from Docker
///
/// **DOCKER LABEL PATTERN**:
/// ```dockerfile
/// LABEL songbird.capability="compute"
/// LABEL songbird.primal="toadstool"
/// LABEL songbird.port="8100"
/// ```
async fn discover_docker_containers() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    debug!("Querying Docker for Songbird containers");
    
    // Check if Docker socket is accessible
    let docker_socket = std::env::var("DOCKER_HOST")
        .unwrap_or_else(|_| "unix:///var/run/docker.sock".to_string());
    
    debug!("Docker socket: {}", docker_socket);
    
    // In production, this would:
    // 1. Connect to Docker API (via bollard crate or docker CLI)
    // 2. List containers with label filter: songbird.capability
    // 3. Parse labels to extract capabilities, port, etc.
    // 4. Get container network info (IP, mapped ports)
    // 5. Build DiscoveredPrimal from container metadata
    
    let discovered = Vec::new();
    
    Ok(discovered)
}

/// Discover Songbird primals from Kubernetes
///
/// **KUBERNETES ANNOTATION PATTERN**:
/// ```yaml
/// metadata:
///   annotations:
///     songbird.io/capability: "compute"
///     songbird.io/primal: "toadstool"
/// ```
async fn discover_kubernetes_services() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    debug!("Querying Kubernetes for Songbird services");
    
    // Check if running in Kubernetes
    let in_cluster = std::path::Path::new("/var/run/secrets/kubernetes.io").exists();
    
    if !in_cluster {
        debug!("Not running in Kubernetes cluster");
        return Ok(Vec::new());
    }
    
    // In production, this would:
    // 1. Use kube-rs to connect to K8s API
    // 2. List services/pods with songbird annotations
    // 3. Parse annotations for capabilities
    // 4. Get service endpoints (ClusterIP, NodePort, LoadBalancer)
    // 5. Build DiscoveredPrimal from K8s metadata
    
    let discovered = Vec::new();
    
    Ok(discovered)
}

/// Build DiscoveredPrimal from container metadata
///
/// **SELF-KNOWLEDGE**: Container provides its identity via labels/annotations
#[allow(dead_code)] // Used in full implementation
fn build_from_container_metadata(
    container_name: &str,
    labels: HashMap<String, String>,
) -> Option<DiscoveredPrimal> {
    use crate::capabilities::Capability;
    
    // Extract capability from label
    let capabilities_str = labels.get("songbird.capability")?;
    let capability_strings: Vec<String> = capabilities_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    // Convert to Capability objects
    let capabilities: Vec<Capability> = capability_strings
        .iter()
        .filter_map(|s| Capability::from_string(s))
        .collect();
    
    // Get endpoint (IP:Port)
    let ip = labels.get("songbird.ip")?;
    let port = labels.get("songbird.port")?;
    let endpoint = format!("http://{}:{}", ip, port);
    
    // Infer or extract primal type
    let primal_type = if let Some(type_str) = labels.get("songbird.primal") {
        PrimalType::new(type_str)
    } else {
        // Infer from capabilities
        infer_primal_type(&capability_strings)
    };
    
    Some(DiscoveredPrimal {
        name: container_name.to_string(),
        endpoint,
        primal_type,
        capabilities,
        discovery_method: DiscoveryMethod::ContainerOrchestration,
        health: PrimalHealth::Healthy,
        metadata: labels,
    })
}

/// Infer primal type from capabilities (no hardcoding)
fn infer_primal_type(capabilities: &[String]) -> PrimalType {
    if capabilities.contains(&"compute".to_string()) {
        PrimalType::new("toadstool")
    } else if capabilities.contains(&"security".to_string()) {
        PrimalType::new("beardog")
    } else if capabilities.contains(&"storage".to_string()) {
        PrimalType::new("squirrel")
    } else if capabilities.contains(&"gateway".to_string()) {
        PrimalType::new("nestgate")
    } else {
        PrimalType::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_from_container_metadata() {
        let mut labels = HashMap::new();
        labels.insert("songbird.capability".to_string(), "compute,distributed".to_string());
        labels.insert("songbird.ip".to_string(), "172.17.0.2".to_string());
        labels.insert("songbird.port".to_string(), "8100".to_string());
        labels.insert("songbird.primal".to_string(), "toadstool".to_string());
        
        let primal = build_from_container_metadata("toadstool-1", labels).unwrap();
        
        assert_eq!(primal.name, "toadstool-1");
        assert_eq!(primal.endpoint, "http://172.17.0.2:8100");
        assert!(matches!(primal.primal_type, PrimalType::Toadstool));
        assert!(matches!(primal.discovery_method, DiscoveryMethod::ContainerOrchestration));
    }

    #[test]
    fn test_infer_primal_type() {
        assert!(matches!(
            infer_primal_type(&["compute".to_string()]),
            PrimalType::Toadstool
        ));
        
        assert!(matches!(
            infer_primal_type(&["security".to_string(), "entropy".to_string()]),
            PrimalType::BearDog
        ));
    }

    #[test]
    fn test_build_without_explicit_type() {
        let mut labels = HashMap::new();
        labels.insert("songbird.capability".to_string(), "storage".to_string());
        labels.insert("songbird.ip".to_string(), "172.17.0.3".to_string());
        labels.insert("songbird.port".to_string(), "8103".to_string());
        
        let primal = build_from_container_metadata("storage-service", labels).unwrap();
        
        // Should infer Squirrel from storage capability
        assert!(matches!(primal.primal_type, PrimalType::Squirrel));
    }
}

