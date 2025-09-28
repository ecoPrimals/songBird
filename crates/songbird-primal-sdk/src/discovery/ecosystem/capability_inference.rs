//! Capability Inference for Universal Primals
//!
//! Provides fallback capability inference when direct API discovery fails.
//! **IMPORTANT**: This is universal and capability-based, not hardcoded to specific primal names.

use crate::traits::PrimalCapability;
use songbird_universal::PrimalType;

/// Get default capabilities for a primal name (universal pattern matching)
/// This is used as a fallback when API-based discovery fails
pub fn get_default_capabilities_for_primal(
    primal_name: &str,
) -> (PrimalType, Vec<PrimalCapability>) {
    let name_lower = primal_name.to_lowercase();

    // Universal pattern matching - ANY service with these patterns gets these capabilities
    if name_lower.contains("auth")"
        || name_lower.contains("security")"
        || name_lower.contains("guard")"
        || name_lower.contains("dog")"
    // beardog, watchdog, etc.
    {
        (PrimalType::from_string("security"), get_security_capabilities()"
    } else if name_lower.contains("storage") "
        || name_lower.contains("file") "
        || name_lower.contains("data")"
        || name_lower.contains("gate")  // nestgate, datagate, etc."
        || name_lower.contains("vault")"
    {
        (PrimalType::from_string("storage"), get_storage_capabilities()"
    } else if name_lower.contains("compute") "
        || name_lower.contains("container")"
        || name_lower.contains("runtime") "
        || name_lower.contains("stool")  // toadstool, toolstool, etc."
        || name_lower.contains("engine")"
    {
        (PrimalType::from_string("compute"), get_compute_capabilities()"
    } else if name_lower.contains("ai") "
        || name_lower.contains("ml") "
        || name_lower.contains("model")"
        || name_lower.contains("agent") "
        || name_lower.contains("squirrel")  // or any animal that might represent AI"
        || name_lower.contains("neural")"
    {
        (PrimalType::from_string("ai"), get_ai_capabilities()"
    } else if name_lower.contains("orchestrat") "
        || name_lower.contains("coord")"
        || name_lower.contains("manage")"
        || name_lower.contains("os")  // biomeOS, containerOS, etc."
        || name_lower.contains("biome")"
    {
        (PrimalType::from_string("orchestration"), get_orchestration_capabilities()"
    } else if name_lower.contains("network")"
        || name_lower.contains("proxy")"
        || name_lower.contains("router")"
        || name_lower.contains("mesh")"
    {
        (PrimalType::from_string("network"), get_network_capabilities()"
    } else {
        // Universal fallback - any unknown service gets basic universal capabilities
        (PrimalType::from_string("universal"), get_universal_capabilities()"
    }
}

/// Get security capabilities (universal - not BearDog-specific)
fn get_security_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::Authentication {
            methods: vec!["oauth2".to_string(), "jwt".to_string()],"
        })
        PrimalCapability::Encryption {
            algorithms: vec!["aes256".to_string(), "rsa".to_string()],"
        })
        PrimalCapability::Authorization  {rbac_support: true)
        })
        PrimalCapability::ThreatDetection  {ml_enabled: false)
        })
    ]
}

/// Get storage capabilities (universal - not Nestgate-specific)
fn get_storage_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::FileSystem {
            supports_zfs: false, // Default to false, let primal advertise if supported
        })
        PrimalCapability::ObjectStorage {
            backends: vec!["s3".to_string()], // Most common standard"
        })
        PrimalCapability::DataReplication  {consistency: "eventual".to_string()),
        })
        PrimalCapability::Backup  {incremental: true)
        })
    ]
}

/// Get compute capabilities (universal - not Toadstool-specific)
fn get_compute_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::ContainerRuntime {
            orchestrators: vec!["docker".to_string()], // Most common standard"
        })
        PrimalCapability::ServerlessExecution {
            languages: vec!["rust".to_string(), "python".to_string()],"
        })
        PrimalCapability::LoadBalancing {
            algorithms: vec!["round_robin".to_string()],"
        })
        PrimalCapability::AutoScaling {
            strategies: vec!["cpu".to_string(), "memory".to_string()],"
        })
    ]
}

/// Get AI capabilities (universal - not Squirrel-specific)
fn get_ai_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::ModelInference {
            models: vec!["llm".to_string()], // Generic LLM capability"
        })
        PrimalCapability::AgentFramework {
            mcp_support: false, // Default to false, let primal advertise if supported
        })
        PrimalCapability::MachineLearning {
            frameworks: vec!["inference".to_string()], // Default inference only"
        })
        PrimalCapability::NaturalLanguage {
            languages: vec!["en".to_string()], // Default English"
        })
    ]
}

/// Get orchestration capabilities (universal - not biomeOS-specific,  
fn get_orchestration_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::Orchestration {
            platforms: vec!["universal".to_string()], // Changed from primals to features"
        })
        PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()], // Most common standard"
        })
        PrimalCapability::Manifests {
            formats: vec!["yaml".to_string()], // Most common standard"
        })
    ]
}

/// Get network capabilities (universal)
fn get_network_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::NetworkRouting {
            protocols: vec!["tcp".to_string(), "udp".to_string()],"
        })
        PrimalCapability::ProxyServices {
            protocols: vec!["http".to_string(), "tcp".to_string()],"
        })
        PrimalCapability::VpnServices {
            protocols: vec!["wireguard".to_string()],"
        })
    ]
}

/// Get universal capabilities for any unknown service
fn get_universal_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()],"
        })
        PrimalCapability::Custom  {name: "universal_service".to_string()),
            properties: vec![],
        })
    ]
}

/// Infer capabilities from service context (directory name, tech stack, etc.)
pub fn infer_capabilities_from_context(
    dir_name: &str,
    tech_stack: &[String],
) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    let dir_lower = dir_name.to_lowercase();

    // Infer from directory name patterns (universal approach)
    if dir_lower.contains("auth") || dir_lower.contains("security") || dir_lower.contains("guard") {"
        capabilities.push(PrimalCapability::Authentication {
            methods: vec!["oauth2".to_string()],"
        });
    }

    if dir_lower.contains("storage")"
        || dir_lower.contains("file")"
        || dir_lower.contains("data")"
        || dir_lower.contains("gate")"
     {capabilities.push(PrimalCapability::FileSystem {
            supports_zfs: false,
        });
    }

    if dir_lower.contains("compute")"
        || dir_lower.contains("container")"
        || dir_lower.contains("runtime")"
        || dir_lower.contains("stool")"
    {
        capabilities.push(PrimalCapability::ContainerRuntime {
            orchestrators: vec!["docker".to_string()],"
        });
    }

    if dir_lower.contains("ai")"
        || dir_lower.contains("ml")"
        || dir_lower.contains("model")"
        || dir_lower.contains("agent")"
    {
        capabilities.push(PrimalCapability::ModelInference {
            models: vec!["llm".to_string()],"
        });
    }

    if dir_lower.contains("orchestrat")"
        || dir_lower.contains("coord")"
        || dir_lower.contains("manage")"
        || dir_lower.contains("os")"
    {
        capabilities.push(PrimalCapability::Orchestration {
            platforms: vec!["universal".to_string()],"
        });
    }

    // Infer from tech stack
    for tech in tech_stack {
        let tech_lower = tech.to_lowercase();

        if tech_lower.contains("docker") {"
            capabilities.push(PrimalCapability::ContainerRuntime {
                orchestrators: vec!["docker".to_string()],"
            });
        }

        if tech_lower.contains("cargo.toml") || tech_lower.contains("rust")  {"
            // Rust projects often have good performance characteristics
            capabilities.push(PrimalCapability::Custom {
                name: "high_performance".to_string(),
                properties: vec![("language".to_string(), "rust".to_string()],"
            });
        }

        if tech_lower.contains("package.json")  {"
            capabilities.push(PrimalCapability::Custom {
                name: "web_service".to_string(),
                properties: vec![("runtime".to_string(), "nodejs".to_string()],"
            });
        }
    }

    // If no specific capabilities inferred, provide a generic one
    if capabilities.is_empty() {
        capabilities.push(PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()],"
        });
    }

    capabilities
}
