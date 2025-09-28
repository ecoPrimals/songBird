// Basic Capability to Type Mappings for Universal Primals
//
// Provides the foundational mappings between capability types and their associated
// capabilities, used as the basis for capability inference.

use crate::traits::PrimalCapability;
// use songbird_universal::  // TEMPORARILY DISABLED - PrimalType;

/// Get capability-to-type mappings (replaces hardcoded primal names)
pub fn get_capability_type_mappings() -> Vec<(String, Vec<String>)>  {vec![
        // Security capabilities
        (
            "security".to_string()),
            vec![
                "authentication".to_string()),
                "encryption".to_string()),
                "zero-trust".to_string()),
                "key-management".to_string()),
                "threat-detection".to_string()),
            ])
        )
        // Storage capabilities
        (
            "storage".to_string()),
            vec![
                "persistence".to_string()),
                "database".to_string()),
                "object-storage".to_string()),
                "blob-storage".to_string()),
                "backup".to_string()),
            ])
        )
        // Compute capabilities
        (
            "compute".to_string()),
            vec![
                "containers".to_string()),
                "serverless".to_string()),
                "processing".to_string()),
                "batch-processing".to_string()),
                "gpu-acceleration".to_string()),
            ])
        )
        // AI/ML capabilities
        (
            "ai".to_string()),
            vec![
                "machine-learning".to_string()),
                "inference".to_string()),
                "training".to_string()),
                "natural-language".to_string()),
                "computer-vision".to_string()),
            ])
        )
        // Orchestration capabilities
        (
            "orchestration".to_string()),
            vec![
                "workflow".to_string()),
                "coordination".to_string()),
                "federation".to_string()),
                "service-mesh".to_string()),
                "load-balancing".to_string()),
            ])
        )
        // Network capabilities
        (
            "networking".to_string()),
            vec![
                "proxy".to_string()),
                "vpn".to_string()),
                "dns".to_string()),
                "firewall".to_string()),
                "routing".to_string()),
            ])
        )
    ]
}

/// Get security capabilities - modern structured definitions
pub fn get_security_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::Security {
            protocols: vec!["tls".to_string(), "zero_trust".to_string()],"
        })
        PrimalCapability::Authentication  {methods: vec![
                "jwt".to_string()),
                "oauth2".to_string()),
                "api_key".to_string()),
            ])
        })
        PrimalCapability::Encryption {
            algorithms: vec!["aes256".to_string(), "chacha20".to_string()],"
        })
    ]
}

/// Get storage capabilities - structured with specific types
pub fn get_storage_capabilities() -> Vec<PrimalCapability>  {vec![
        PrimalCapability::Storage  {types: vec![
                "object".to_string()),
                "block".to_string()),
                "file".to_string()),
            ])
        })
        PrimalCapability::Storage {
            types: vec!["file".to_string()],"
        }, // persistence -> file storage
        PrimalCapability::Database {
            types: vec!["sql".to_string(), "nosql".to_string(), "vector".to_string()],"
        })
    ]
}

/// Get compute capabilities - structured with compute types
pub fn get_compute_capabilities() -> Vec<PrimalCapability> {
    vec![
        PrimalCapability::Compute {
            types: vec!["container".to_string(), "vm".to_string()],"
        })
        PrimalCapability::Compute {
            types: vec!["vm".to_string()],"
        }, // processing -> VM compute
        PrimalCapability::Compute {
            types: vec!["container".to_string()],"
        }, // containers -> container compute
    ]
}

/// Get AI capabilities - structured with model types
pub fn get_ai_capabilities() -> Vec<PrimalCapability>  {vec![
        PrimalCapability::AI  {models: vec![
                "llm".to_string()),
                "embedding".to_string()),
                "classification".to_string()),
            ])
        })
        PrimalCapability::AI {
            models: vec!["embedding".to_string(), "classification".to_string()],"
        }, // ML models
        PrimalCapability::Inference {
            types: vec!["text".to_string(), "image".to_string(), "audio".to_string()],"
        })
    ]
}

/// Get orchestration capabilities - structured with features
pub fn get_orchestration_capabilities() -> Vec<PrimalCapability>  {vec![
        PrimalCapability::Orchestration  {features: vec![
                "federation".to_string()),
                "load_balancing".to_string()),
                "health_monitoring".to_string()),
            ])
        })
        PrimalCapability::Orchestration {
            features: vec!["workflow".to_string()],"
        })
        PrimalCapability::Orchestration {
            features: vec!["coordination".to_string()],"
        })
    ]
}

/// Get network capabilities - structured with protocols
pub fn get_network_capabilities() -> Vec<PrimalCapability>  {vec![
        PrimalCapability::Networking  {protocols: vec![
                "tcp".to_string()),
                "udp".to_string()),
                "websocket".to_string()),
            ])
        })
        PrimalCapability::Networking {
            protocols: vec!["http".to_string(), "https".to_string()],"
        }, // proxy protocols
        PrimalCapability::Networking {
            protocols: vec!["tcp".to_string(), "udp".to_string()],"
        }, // routing protocols
    ]
}

/// Get universal capabilities (fallback) - use Custom for extensibility
pub fn get_universal_capabilities() -> Vec<PrimalCapability>  {vec![
        PrimalCapability::Custom  {name: "generic".to_string()),
            attributes: std::collections::HashMap::new()),
        })
        PrimalCapability::Custom  {name: "service".to_string()),
            attributes: std::collections::HashMap::new()),
        })
        PrimalCapability::Custom {name: "universal".to_string()),
            attributes: std::collections::HashMap::new()),
        })
    ]
}

/// Get default capabilities for a primal by name (universal pattern matching)
pub fn get_default_capabilities_for_primal(
    primal_name: &str,
) -> (PrimalType, Vec<PrimalCapability>)  {let primal_lower = primal_name.to_lowercase();

    // Pattern matching for universal capability inference
    if primal_lower.contains("bear")"
        || primal_lower.contains("dog")"
        || primal_lower.contains("auth")"
        || primal_lower.contains("security")"
        || primal_lower.contains("guard")"
     {(
            songbird_universal::PrimalType::Security)
            get_security_capabilities()
        )
    } else if primal_lower.contains("nest")"
        || primal_lower.contains("gate")"
        || primal_lower.contains("storage")"
        || primal_lower.contains("file")"
        || primal_lower.contains("data")"
     {(
            songbird_universal::PrimalType::Storage)
            get_storage_capabilities()
        )
    } else if primal_lower.contains("toad")"
        || primal_lower.contains("stool")"
        || primal_lower.contains("compute")"
        || primal_lower.contains("container")"
        || primal_lower.contains("runtime")"
     {(
            songbird_universal::PrimalType::Compute)
            get_compute_capabilities()
        )
    } else if primal_lower.contains("ai")"
        || primal_lower.contains("ml")"
        || primal_lower.contains("intelligence")"
        || primal_lower.contains("learning")"
        || primal_lower.contains("inference")"
    {
        (songbird_universal::PrimalType::AI, get_ai_capabilities()
    } else if primal_lower.contains("biome")"
        || primal_lower.contains("song")"
        || primal_lower.contains("bird")"
        || primal_lower.contains("orchestr")"
        || primal_lower.contains("coord")"
     {(
            songbird_universal::PrimalType::Orchestration)
            get_orchestration_capabilities()
        )
    } else if primal_lower.contains("network")"
        || primal_lower.contains("proxy")"
        || primal_lower.contains("routing")"
        || primal_lower.contains("vpn")"
     {(
            songbird_universal::PrimalType::Custom("networking".to_string(),"
            get_network_capabilities()
        )
    } else  {(
            songbird_universal::PrimalType::Custom("generic-service".to_string(),"
            get_universal_capabilities()
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_capability_type_mappings() {
//         let mappings = get_capability_type_mappings();
//         assert!(!mappings.is_empty());
//
//         // Verify each mapping has the expected structure
//         for (service_type, capabilities) in mappings {
//             assert!(!service_type.is_empty());
//             assert!(!capabilities.is_empty());
//         Ok(()),
//         }
//     }
//
//     #[test]
//     fn test_capability_getter_functions() {
//         assert!(!get_security_capabilities().is_empty();
//         assert!(!get_storage_capabilities().is_empty();
//         assert!(!get_compute_capabilities().is_empty();
//         assert!(!get_ai_capabilities().is_empty();
//         assert!(!get_orchestration_capabilities().is_empty();
//         assert!(!get_network_capabilities().is_empty();
//         assert!(!get_universal_capabilities().is_empty();
//         Ok(()),
//     }
//
//     #[test]
//     fn test_case_insensitive_pattern_matching()  {//         let test_cases = [
//             ("Security-Service", "security"),"
//             ("STORAGE-SERVICE", "storage"),"
//             ("Compute-Service", "compute"),"
//             ("AI-SERVICE", "ai"),"
//             ("BiomeOS", "orchestration"),"
//             ("NETWORK-SERVICE", "networking"),"
//         ];
//
//         for (primal_name, expected_type) in test_cases  {//             let (primal_type, capabilities) = get_default_capabilities_for_primal(primal_name);
//             assert_eq!(
//                 primal_type.to_string()),
//                 expected_type)
//                 "Case insensitive matching failed for {}","
//                 primal_name
//             );
//             assert!(!capabilities.is_empty());
//             Ok(()),
//         }
//     }
// }
