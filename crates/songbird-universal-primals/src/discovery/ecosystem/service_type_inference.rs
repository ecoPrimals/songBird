// Service Type to Capability Inference
//
// Provides capability inference based on service type patterns and naming conventions.

use crate::traits::PrimalCapability;

/// Get default capabilities for a service type (universal pattern matching)
pub fn get_default_capabilities_for_service_type(service_type: &str) -> Vec<PrimalCapability> {
    match service_type.to_lowercase().to_string().as_str() {
        // Security service patterns - modern structured capabilities
        "security" | "auth" | "authentication" => vec![
            PrimalCapability::Security {
                protocols: vec!["tls".to_string(), "zero_trust".to_string()],
            },
            PrimalCapability::Authentication {
                methods: vec![
                    "jwt".to_string(),
                    "oauth2".to_string(),
                    "api_key".to_string(),
                ],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["aes256".to_string(), "chacha20".to_string()],
            },
        ],

        // Storage service patterns - structured with specific types
        "storage" | "database" | "persistence" => vec![
            PrimalCapability::Storage {
                types: vec![
                    "object".to_string(),
                    "block".to_string(),
                    "file".to_string(),
                ],
            },
            PrimalCapability::Storage {
                types: vec!["file".to_string()],
            }, // persistence -> file storage
            PrimalCapability::Database {
                types: vec!["sql".to_string(), "nosql".to_string(), "vector".to_string()],
            },
        ],

        // Compute service patterns - structured with compute types
        "compute" | "processing" | "containers" => vec![
            PrimalCapability::Compute {
                types: vec!["container".to_string(), "vm".to_string()],
            },
            PrimalCapability::Compute {
                types: vec!["vm".to_string()],
            }, // processing -> VM compute
            PrimalCapability::Compute {
                types: vec!["container".to_string()],
            }, // containers -> container compute
        ],

        // AI/ML service patterns - structured with model types
        "ai" | "ml" | "machine-learning" | "inference" => vec![
            PrimalCapability::AI {
                models: vec![
                    "llm".to_string(),
                    "embedding".to_string(),
                    "classification".to_string(),
                ],
            },
            PrimalCapability::AI {
                models: vec!["embedding".to_string(), "classification".to_string()],
            }, // ML models
            PrimalCapability::Inference {
                types: vec!["text".to_string(), "image".to_string(), "audio".to_string()],
            },
        ],

        // Orchestration service patterns - structured with features
        "orchestration" | "workflow" | "coordination" => vec![
            PrimalCapability::Orchestration {
                features: vec![
                    "federation".to_string(),
                    "load_balancing".to_string(),
                    "health_monitoring".to_string(),
                ],
            },
            PrimalCapability::Orchestration {
                features: vec!["workflow".to_string()],
            },
            PrimalCapability::Orchestration {
                features: vec!["coordination".to_string()],
            },
        ],

        // Network service patterns - structured with protocols
        "networking" | "proxy" | "routing" | "vpn" => vec![
            PrimalCapability::Networking {
                protocols: vec![
                    "tcp".to_string(),
                    "udp".to_string(),
                    "websocket".to_string(),
                ],
            },
            PrimalCapability::Networking {
                protocols: vec!["http".to_string(), "https".to_string()],
            }, // proxy protocols
            PrimalCapability::Networking {
                protocols: vec!["tcp".to_string(), "udp".to_string()],
            }, // routing protocols
        ],

        // Gaming service patterns - structured with protocols
        "gaming" | "retro-gaming" | "multiplayer" => vec![
            PrimalCapability::Gaming {
                protocols: vec![
                    "directplay".to_string(),
                    "ipx".to_string(),
                    "netbios".to_string(),
                ],
            },
            PrimalCapability::Gaming {
                protocols: vec!["ipx".to_string(), "netbios".to_string()],
            }, // retro protocols
            PrimalCapability::Gaming {
                protocols: vec!["directplay".to_string()],
            }, // multiplayer protocols
        ],

        // Web service patterns - structured with protocols
        "web" | "http" | "api" | "rest" => vec![
            PrimalCapability::Networking {
                protocols: vec![
                    "http".to_string(),
                    "https".to_string(),
                    "websocket".to_string(),
                ],
            },
            PrimalCapability::Networking {
                protocols: vec!["http".to_string(), "https".to_string()],
            },
            PrimalCapability::DataAccess {
                patterns: vec!["rest".to_string(), "graphql".to_string()],
            },
        ],

        // Data processing patterns - structured with access patterns
        "data" | "analytics" | "batch-processing" => vec![
            PrimalCapability::DataAccess {
                patterns: vec!["rest".to_string(), "graphql".to_string(), "sql".to_string()],
            },
            PrimalCapability::AI {
                models: vec!["embedding".to_string(), "classification".to_string()],
            }, // analytics -> AI models
            PrimalCapability::Compute {
                types: vec!["vm".to_string()],
            }, // batch processing -> VM compute
        ],

        // Monitoring patterns - structured with features
        "monitoring" | "observability" | "metrics" => vec![
            PrimalCapability::Monitoring {
                features: vec![
                    "metrics".to_string(),
                    "logs".to_string(),
                    "traces".to_string(),
                ],
            },
            PrimalCapability::Monitoring {
                features: vec![
                    "metrics".to_string(),
                    "logs".to_string(),
                    "traces".to_string(),
                ],
            }, // observability
            PrimalCapability::Monitoring {
                features: vec!["metrics".to_string()],
            },
        ],

        // Communication patterns - structured with protocols and features
        "communication" | "messaging" | "chat" => vec![
            PrimalCapability::Messaging {
                protocols: vec![
                    "mqtt".to_string(),
                    "amqp".to_string(),
                    "websocket".to_string(),
                ],
            },
            PrimalCapability::Messaging {
                protocols: vec!["mqtt".to_string(), "amqp".to_string()],
            },
            PrimalCapability::Chat {
                features: vec!["text".to_string(), "voice".to_string(), "video".to_string()],
            },
        ],

        // Fallback for unknown service types - use Custom for extensibility
        _ => vec![
            PrimalCapability::Custom {
                name: "generic".to_string(),
                attributes: std::collections::HashMap::new(),
            },
            PrimalCapability::Custom {
                name: "service".to_string(),
                attributes: std::collections::HashMap::new(),
            },
        ],
    }
}

/// Infer service type from service name patterns
pub fn infer_service_type_from_name(service_name: &str) -> String {
    let name_lower = service_name.to_lowercase();

    // Security patterns
    if name_lower.contains("auth")
        || name_lower.contains("security")
        || name_lower.contains("bear")
        || name_lower.contains("dog")
    {
        return "security".to_string();
    }

    // Storage patterns
    if name_lower.contains("storage")
        || name_lower.contains("database")
        || name_lower.contains("nest")
        || name_lower.contains("gate")
    {
        return "storage".to_string();
    }

    // Compute patterns
    if name_lower.contains("compute")
        || name_lower.contains("container")
        || name_lower.contains("toad")
        || name_lower.contains("stool")
    {
        return "compute".to_string();
    }

    // AI patterns
    if name_lower.contains("ai")
        || name_lower.contains("ml")
        || name_lower.contains("intelligence")
        || name_lower.contains("inference")
        || name_lower.contains("learning")
    {
        return "ai".to_string();
    }

    // Orchestration patterns
    if name_lower.contains("orchestr")
        || name_lower.contains("coord")
        || name_lower.contains("song")
        || name_lower.contains("bird")
    {
        return "orchestration".to_string();
    }

    // Network patterns
    if name_lower.contains("network")
        || name_lower.contains("proxy")
        || name_lower.contains("routing")
    {
        return "networking".to_string();
    }

    // Gaming patterns
    if name_lower.contains("gaming") || name_lower.contains("game") || name_lower.contains("retro")
    {
        return "gaming".to_string();
    }

    // Web service patterns
    if name_lower.contains("web") || name_lower.contains("http") || name_lower.contains("api") {
        return "web".to_string();
    }

    // Default fallback
    "generic".to_string()
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_get_default_capabilities_for_service_type_universal_patterns() {
//         let test_cases = vec![
//             ("security-provider", vec!["security", "authentication"]),
//             ("storage-provider", vec!["storage", "persistence"]),
//             ("compute-provider", vec!["compute", "containers"]),
//             ("ai-provider", vec!["ai", "machine-learning"]),
//         ];
//
//         for (service_type, expected_capabilities) in test_cases {
//             let capabilities = get_default_capabilities_for_service_type(service_type);
//             assert!(!capabilities.is_empty());
//
//             // Should contain expected capability types
//             for expected_cap in expected_capabilities {
//                 assert!(
//                     capabilities
//                         .iter()
//                         .any(|cap| cap.capability_type().contains(expected_cap)),
//                     "Service type {} should have capability {}",
//                     service_type,
//                     expected_cap
//                 );
//             }
//         }
//     }
//
//     #[test]
//     fn test_infer_service_type_from_name() {
//         let test_cases = vec![
//             ("security-auth", "security"),
//             ("storage-service", "storage"),
//             ("compute-service", "compute"),
//             ("ai-service", "ai"),
//             ("songbird-orchestrator", "orchestration"),
//             ("network-proxy", "networking"),
//             ("retro-gaming", "gaming"),
//             ("web-api", "web"),
//             ("unknown-service", "generic"),
//         ];
//
//         for (service_name, expected_type) in test_cases {
//             let service_type = infer_service_type_from_name(service_name);
//             assert_eq!(
//                 service_type, expected_type,
//                 "Service name {} should infer type {}",
//                 service_name, expected_type
//             );
//         }
//     }
//
//     #[test]
//     fn test_case_insensitive_service_type_inference() {
//         let test_cases = vec![
//             ("SECURITY-AUTH", "security"),
//             ("Storage-Service", "storage"),
//             ("Compute-SERVICE", "compute"),
//             ("AI-service", "ai"),
//         ];
//
//         for (service_name, expected_type) in test_cases {
//             let service_type = infer_service_type_from_name(service_name);
//             assert_eq!(
//                 service_type, expected_type,
//                 "Case insensitive inference failed for {}",
//                 service_name
//             );
//         }
//     }
//
//     #[test]
//     fn test_service_type_capability_consistency() {
//         // Test that inferring service type and then getting capabilities works consistently
//         let service_names = vec![
//             "security-provider",
//             "storage-provider",
//             "compute-provider",
//             "ai-provider",
//         ];
//
//         for service_name in service_names {
//             let service_type = infer_service_type_from_name(service_name);
//             let capabilities = get_default_capabilities_for_service_type(&service_type);
//
//             assert!(
//                 !capabilities.is_empty(),
//                 "Service {} should have capabilities after type inference",
//                 service_name
//             );
//         }
//     }
// }
