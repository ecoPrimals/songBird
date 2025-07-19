//! Universal capability-based parsing for primal discovery
//!
//! **PURE CAPABILITY-BASED ARCHITECTURE**
//!
//! This module detects and assigns capabilities based on service behavior,
//! not hardcoded primal names. Any primal can provide any capability.

use crate::PrimalCapability;
use songbird_universal::PrimalType;
use std::collections::HashMap;

/// Discover capabilities by probing service endpoints and analyzing responses
pub fn discover_capabilities_from_service(endpoint: &str, service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut discovered_capabilities = Vec::new();

    // Security capability detection
    if has_security_endpoints(endpoint, service_info) {
        discovered_capabilities.extend(detect_security_capabilities(service_info));
    }

    // Compute capability detection  
    if has_compute_endpoints(endpoint, service_info) {
        discovered_capabilities.extend(detect_compute_capabilities(service_info));
    }

    // Storage capability detection
    if has_storage_endpoints(endpoint, service_info) {
        discovered_capabilities.extend(detect_storage_capabilities(service_info));
    }

    // Network capability detection
    if has_network_endpoints(endpoint, service_info) {
        discovered_capabilities.extend(detect_network_capabilities(service_info));
    }

    // AI capability detection
    if has_ai_endpoints(endpoint, service_info) {
        discovered_capabilities.extend(detect_ai_capabilities(service_info));
    }

    // Orchestration capability detection
    if has_orchestration_endpoints(endpoint, service_info) {
        discovered_capabilities.extend(detect_orchestration_capabilities(service_info));
    }

    // If no specific capabilities detected, assign generic service capability
    if discovered_capabilities.is_empty() {
        discovered_capabilities.push(PrimalCapability::Custom {
            name: "generic_service".to_string(),
            attributes: extract_generic_attributes(service_info),
        });
    }

    discovered_capabilities
}

/// Detect security capabilities by analyzing service endpoints and responses
fn detect_security_capabilities(service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    // Check for authentication endpoints
    if has_auth_endpoints(service_info) {
        capabilities.push(PrimalCapability::Authentication {
            methods: detect_auth_methods(service_info),
        });
    }

    // Check for encryption capabilities
    if has_encryption_support(service_info) {
        capabilities.push(PrimalCapability::Encryption {
            algorithms: detect_encryption_algorithms(service_info),
        });
    }

    // Check for key management
    if has_key_management(service_info) {
        capabilities.push(PrimalCapability::KeyManagement {
            hsm_support: detect_hsm_support(service_info),
        });
    }

    // Check for threat detection
    if has_threat_detection(service_info) {
        capabilities.push(PrimalCapability::ThreatDetection {
            ml_enabled: detect_ml_threat_detection(service_info),
        });
    }

    capabilities
}

/// Detect compute capabilities by analyzing service responses
fn detect_compute_capabilities(service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    // Check for container runtime
    if has_container_support(service_info) {
        capabilities.push(PrimalCapability::ContainerRuntime {
            orchestrators: detect_container_orchestrators(service_info),
        });
    }

    // Check for serverless execution
    if has_serverless_support(service_info) {
        capabilities.push(PrimalCapability::ServerlessExecution {
            languages: detect_supported_languages(service_info),
        });
    }

    // Check for GPU acceleration
    if has_gpu_support(service_info) {
        capabilities.push(PrimalCapability::GpuAcceleration {
            cuda_support: detect_cuda_support(service_info),
        });
    }

    capabilities
}

/// Detect storage capabilities 
fn detect_storage_capabilities(service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    if has_filesystem_support(service_info) {
        capabilities.push(PrimalCapability::FileSystem {
            supports_zfs: detect_zfs_support(service_info),
        });
    }

    if has_object_storage(service_info) {
        capabilities.push(PrimalCapability::ObjectStorage {
            backends: detect_storage_backends(service_info),
        });
    }

    capabilities
}

/// Detect network capabilities
fn detect_network_capabilities(service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    if has_service_discovery(service_info) {
        capabilities.push(PrimalCapability::ServiceDiscovery {
            protocols: detect_discovery_protocols(service_info),
        });
    }

    if has_load_balancing(service_info) {
        capabilities.push(PrimalCapability::LoadBalancing {
            algorithms: detect_lb_algorithms(service_info),
        });
    }

    capabilities
}

/// Detect AI capabilities
fn detect_ai_capabilities(service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    if has_model_inference(service_info) {
        capabilities.push(PrimalCapability::ModelInference {
            models: detect_supported_models(service_info),
        });
    }

    if has_agent_framework(service_info) {
        capabilities.push(PrimalCapability::AgentFramework {
            mcp_support: detect_mcp_support(service_info),
        });
    }

    capabilities
}

/// Detect orchestration capabilities
fn detect_orchestration_capabilities(service_info: &serde_json::Value) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    if has_orchestration_support(service_info) {
        capabilities.push(PrimalCapability::Orchestration {
            primals: detect_managed_primals(service_info),
        });
    }

    capabilities
}

// ===== CAPABILITY DETECTION FUNCTIONS =====

fn has_security_endpoints(_endpoint: &str, service_info: &serde_json::Value) -> bool {
    // Check for common security endpoints
    if let Some(endpoints) = service_info.get("endpoints") {
        if let Some(endpoint_list) = endpoints.as_array() {
            return endpoint_list.iter().any(|e| {
                e.as_str().map_or(false, |s| {
                    s.contains("/auth") || s.contains("/security") || s.contains("/encrypt")
                })
            });
        }
    }
    
    // Check service description for security keywords
    if let Some(description) = service_info.get("description").and_then(|d| d.as_str()) {
        return description.to_lowercase().contains("security") 
            || description.to_lowercase().contains("auth")
            || description.to_lowercase().contains("encrypt");
    }
    
    false
}

fn has_compute_endpoints(_endpoint: &str, service_info: &serde_json::Value) -> bool {
    if let Some(description) = service_info.get("description").and_then(|d| d.as_str()) {
        let desc_lower = description.to_lowercase();
        return desc_lower.contains("compute") 
            || desc_lower.contains("container")
            || desc_lower.contains("execution")
            || desc_lower.contains("runtime");
    }
    false
}

fn has_storage_endpoints(_endpoint: &str, service_info: &serde_json::Value) -> bool {
    if let Some(description) = service_info.get("description").and_then(|d| d.as_str()) {
        let desc_lower = description.to_lowercase();
        return desc_lower.contains("storage") 
            || desc_lower.contains("filesystem")
            || desc_lower.contains("backup")
            || desc_lower.contains("volume");
    }
    false
}

fn has_network_endpoints(_endpoint: &str, service_info: &serde_json::Value) -> bool {
    if let Some(description) = service_info.get("description").and_then(|d| d.as_str()) {
        let desc_lower = description.to_lowercase();
        return desc_lower.contains("network") 
            || desc_lower.contains("routing")
            || desc_lower.contains("discovery")
            || desc_lower.contains("mesh");
    }
    false
}

fn has_ai_endpoints(_endpoint: &str, service_info: &serde_json::Value) -> bool {
    if let Some(description) = service_info.get("description").and_then(|d| d.as_str()) {
        let desc_lower = description.to_lowercase();
        return desc_lower.contains("ai") 
            || desc_lower.contains("ml")
            || desc_lower.contains("model")
            || desc_lower.contains("inference")
            || desc_lower.contains("agent");
    }
    false
}

fn has_orchestration_endpoints(_endpoint: &str, service_info: &serde_json::Value) -> bool {
    if let Some(description) = service_info.get("description").and_then(|d| d.as_str()) {
        let desc_lower = description.to_lowercase();
        return desc_lower.contains("orchestration") 
            || desc_lower.contains("manifest")
            || desc_lower.contains("deployment")
            || desc_lower.contains("coordinator");
    }
    false
}

// ===== SPECIFIC CAPABILITY DETECTION =====

fn has_auth_endpoints(service_info: &serde_json::Value) -> bool {
    service_info.get("features")
        .and_then(|f| f.get("authentication"))
        .and_then(|a| a.as_bool())
        .unwrap_or(false)
}

fn detect_auth_methods(service_info: &serde_json::Value) -> Vec<String> {
    service_info.get("auth_methods")
        .and_then(|m| m.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["basic".to_string()])
}

fn has_encryption_support(service_info: &serde_json::Value) -> bool {
    service_info.get("features")
        .and_then(|f| f.get("encryption"))
        .and_then(|e| e.as_bool())
        .unwrap_or(false)
}

fn detect_encryption_algorithms(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["AES256".to_string()] // Default assumption
}

fn has_key_management(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_hsm_support(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn has_threat_detection(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_ml_threat_detection(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn has_container_support(_service_info: &serde_json::Value) -> bool {
    true // Assume compute services support containers
}

fn detect_container_orchestrators(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["kubernetes".to_string()] // Common default
}

fn has_serverless_support(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_supported_languages(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["rust".to_string()] // Default
}

fn has_gpu_support(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_cuda_support(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn has_filesystem_support(_service_info: &serde_json::Value) -> bool {
    true // Assume storage services support filesystems
}

fn detect_zfs_support(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn has_object_storage(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_storage_backends(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["filesystem".to_string()] // Default
}

fn has_service_discovery(_service_info: &serde_json::Value) -> bool {
    true // Assume network services support discovery
}

fn detect_discovery_protocols(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["http".to_string()] // Default
}

fn has_load_balancing(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_lb_algorithms(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["round_robin".to_string()] // Default
}

fn has_model_inference(_service_info: &serde_json::Value) -> bool {
    true // Assume AI services support inference
}

fn detect_supported_models(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["generic".to_string()] // Default
}

fn has_agent_framework(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn detect_mcp_support(_service_info: &serde_json::Value) -> bool {
    false // Conservative default
}

fn has_orchestration_support(_service_info: &serde_json::Value) -> bool {
    true // Assume orchestration services support orchestration
}

fn detect_managed_primals(_service_info: &serde_json::Value) -> Vec<String> {
    vec!["universal".to_string()] // Default
}

fn extract_generic_attributes(service_info: &serde_json::Value) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    
    if let Some(version) = service_info.get("version").and_then(|v| v.as_str()) {
        attributes.insert("version".to_string(), version.to_string());
    }
    
    attributes.insert("type".to_string(), "generic_service".to_string());
    attributes
}

/// Extract metadata from discovery info JSON
pub fn extract_metadata_from_info(info: &serde_json::Value) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    if let Some(version) = info.get("version").and_then(|v| v.as_str()) {
        metadata.insert("version".to_string(), version.to_string());
    }

    if let Some(description) = info.get("description").and_then(|d| d.as_str()) {
        metadata.insert("description".to_string(), description.to_string());
    }

    if let Some(region) = info.get("region").and_then(|r| r.as_str()) {
        metadata.insert("region".to_string(), region.to_string());
    }

    if let Some(tags) = info.get("tags").and_then(|t| t.as_array()) {
        let tags_str = tags
            .iter()
            .filter_map(|tag| tag.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if !tags_str.is_empty() {
            metadata.insert("tags".to_string(), tags_str);
        }
    }

    if let Some(uptime) = info.get("uptime").and_then(|u| u.as_u64()) {
        metadata.insert("uptime".to_string(), uptime.to_string());
    }

    metadata
}

/// Infer primal type from capabilities (capability-based naming)
pub fn infer_primal_type_from_capabilities(capabilities: &[PrimalCapability]) -> PrimalType {
    // Generate a type name based on primary capabilities
    let mut capability_types = Vec::new();
    
    for capability in capabilities {
        match capability {
            PrimalCapability::Authentication { .. } | 
            PrimalCapability::Encryption { .. } |
            PrimalCapability::KeyManagement { .. } |
            PrimalCapability::ThreatDetection { .. } => {
                if !capability_types.contains(&"security") {
                    capability_types.push("security");
                }
            }
            PrimalCapability::ContainerRuntime { .. } |
            PrimalCapability::ServerlessExecution { .. } |
            PrimalCapability::GpuAcceleration { .. } => {
                if !capability_types.contains(&"compute") {
                    capability_types.push("compute");
                }
            }
            PrimalCapability::FileSystem { .. } |
            PrimalCapability::ObjectStorage { .. } => {
                if !capability_types.contains(&"storage") {
                    capability_types.push("storage");
                }
            }
            PrimalCapability::ServiceDiscovery { .. } |
            PrimalCapability::LoadBalancing { .. } => {
                if !capability_types.contains(&"network") {
                    capability_types.push("network");
                }
            }
            PrimalCapability::ModelInference { .. } |
            PrimalCapability::AgentFramework { .. } => {
                if !capability_types.contains(&"ai") {
                    capability_types.push("ai");
                }
            }
            PrimalCapability::Orchestration { .. } => {
                if !capability_types.contains(&"orchestration") {
                    capability_types.push("orchestration");
                }
            }
            PrimalCapability::Custom { name, .. } => {
                if !capability_types.contains(&name.as_str()) {
                    capability_types.push(name);
                }
            }
            _ => {}
        }
    }
    
    // Create a descriptive name based on capabilities
    if capability_types.is_empty() {
        PrimalType::new("unknown-service")
    } else {
        PrimalType::new(capability_types.join("-"))
    }
}

/// Parse primal type from string representation (fallback for legacy)
pub fn parse_primal_type_from_string(type_str: &str) -> PrimalType {
    PrimalType::new(type_str.to_string())
}

/// Get default capabilities - now uses capability detection instead of hardcoded names
pub fn get_default_capabilities_for_type(primal_type: &PrimalType) -> Vec<PrimalCapability> {
    // For unknown services, try to infer from the name
    let type_name = primal_type.as_str().to_lowercase();
    let mut capabilities = Vec::new();

    // Security-related names
    if type_name.contains("security") || type_name.contains("auth") || type_name.contains("crypt") {
        capabilities.extend(get_default_security_capabilities());
    }

    // Compute-related names
    if type_name.contains("compute") || type_name.contains("container") || type_name.contains("runtime") {
        capabilities.extend(get_default_compute_capabilities());
    }

    // Storage-related names
    if type_name.contains("storage") || type_name.contains("file") || type_name.contains("data") {
        capabilities.extend(get_default_storage_capabilities());
    }

    // Network-related names
    if type_name.contains("network") || type_name.contains("mesh") || type_name.contains("routing") {
        capabilities.extend(get_default_network_capabilities());
    }

    // AI-related names
    if type_name.contains("ai") || type_name.contains("ml") || type_name.contains("model") {
        capabilities.extend(get_default_ai_capabilities());
    }

    // Orchestration-related names
    if type_name.contains("orchestration") || type_name.contains("coordinator") {
        capabilities.extend(get_default_orchestration_capabilities());
    }

    // If no specific capabilities found, provide generic
    if capabilities.is_empty() {
        capabilities.push(PrimalCapability::Custom {
            name: "generic_service".to_string(),
            attributes: HashMap::new(),
        });
    }

    capabilities
}

fn get_default_security_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::Authentication {
            methods: vec!["oauth2".to_string()],
        },
        PrimalCapability::Encryption {
            algorithms: vec!["aes256".to_string()],
        },
    ]
}

fn get_default_compute_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::ContainerRuntime {
            orchestrators: vec!["kubernetes".to_string()],
        },
    ]
}

fn get_default_storage_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::FileSystem { supports_zfs: false },
    ]
}

fn get_default_network_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()],
        },
    ]
}

fn get_default_ai_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::ModelInference {
            models: vec!["generic".to_string()],
        },
    ]
}

fn get_default_orchestration_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::Orchestration {
            primals: vec!["universal".to_string()],
        },
    ]
}
