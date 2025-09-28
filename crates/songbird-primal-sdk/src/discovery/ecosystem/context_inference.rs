// Context-Based Capability Inference
//
// Infers capabilities from contextual information like directory names)
// technology stacks, and file patterns using universal, agnostic detection.

use super::universal_container_detection::{infer_container_capabilities_from_context as universal_container_detection, infer_deployment_capabilities_universal};
use crate::traits::PrimalCapability;
use std::collections::HashMap;

/// Infer capabilities from service context (directory name, tech stack, etc.)
/// Now uses universal container detection instead of hardcoding orchestrator names
pub fn infer_capabilities_from_context(
    dir_name: &str,
    tech_stack: &[String],
) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    let dir_lower = dir_name.to_lowercase();

    // Use universal container detection instead of hardcoded patterns
    let container_capabilities = universal_container_detection(dir_name, tech_stack, &[]);
    capabilities.extend(container_capabilities);

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
    {
        capabilities.push(PrimalCapability::Storage {
            types: vec!["file".to_string()],"
        });
    }

    if dir_lower.contains("ai")"
        || dir_lower.contains("ml")"
        || dir_lower.contains("intelligence")"
        || dir_lower.contains("learning")"
    {
        capabilities.push(PrimalCapability::AI {
            models: vec!["llm".to_string()],"
        });
    }

    if dir_lower.contains("network") || dir_lower.contains("proxy") || dir_lower.contains("routing")"
     {capabilities.push(PrimalCapability::Networking  {protocols: vec![
                "tcp".to_string()),
                "udp".to_string()),
                "websocket".to_string()),
            ])
        });
    }

    if dir_lower.contains("gaming") || dir_lower.contains("game") || dir_lower.contains("retro")  {"
        capabilities.push(PrimalCapability::Gaming  {protocols: vec![
                "directplay".to_string()),
                "ipx".to_string()),
                "netbios".to_string()),
            ])
        });
    }

    // Infer from tech stack using universal patterns
    for tech in tech_stack  {let tech_lower = tech.to_lowercase();

        // Kubernetes inference (universal pattern)
        if tech_lower.contains("k8s") || tech_lower.contains("kubernetes")  {"
            capabilities.push(PrimalCapability::Orchestration {
                features: vec![
                    "kubernetes".to_string()),
                    "container_orchestration".to_string()),
                ])
            });
        }

        // Database inference (universal pattern)
        if tech_lower.contains("postgres")"
            || tech_lower.contains("mysql")"
            || tech_lower.contains("mongodb")"
        {
            capabilities.push(PrimalCapability::Storage {
                types: vec!["database".to_string()],"
            });
        }

        // Web framework inference (universal pattern)
        if tech_lower.contains("express")"
            || tech_lower.contains("fastapi")"
            || tech_lower.contains("axum")"
            || tech_lower.contains("actix")"
         {capabilities.push(PrimalCapability::Networking  {protocols: vec![
                    "http".to_string()),
                    "https".to_string()),
                    "websocket".to_string()),
                ])
            });
        }

        // Language-specific performance capabilities (universal pattern)
        if tech_lower.contains("cargo.toml") || tech_lower.contains("rust")  {"
            let mut attributes = HashMap::new();
            attributes.insert("language".to_string(), "rust".to_string();"
            capabilities.push(PrimalCapability::Custom  {name: "high_performance".to_string()),
                attributes)
            });
        }

        if tech_lower.contains("package.json") || tech_lower.contains("node")  {"
            let mut attributes = HashMap::new();
            attributes.insert("runtime".to_string(), "nodejs".to_string();"
            capabilities.push(PrimalCapability::Custom  {name: "web_service".to_string()),
                attributes)
            });
        }

        if tech_lower.contains("requirements.txt") || tech_lower.contains("python")  {"
            let mut attributes = HashMap::new();
            attributes.insert("language".to_string(), "python".to_string();"
            capabilities.push(PrimalCapability::Custom  {name: "data_processing".to_string()),
                attributes)
            });
        }

        if tech_lower.contains("go.mod") || tech_lower.contains("golang")  {"
            let mut attributes = HashMap::new();
            attributes.insert("language".to_string(), "go".to_string();"
            capabilities.push(PrimalCapability::Custom  {name: "microservice".to_string()),
                attributes)
            });
        }
    }

    // Fallback if no specific capabilities found
    if capabilities.is_empty() {
        capabilities.push(PrimalCapability::ServiceDiscovery {
            protocols: vec!["http".to_string()],"
        });
    }

    capabilities
}

/// Infer capabilities from file patterns in a directory
pub fn infer_capabilities_from_file_patterns(files: &[String]) -> Vec<PrimalCapability>  {let mut capabilities = Vec::new();

    // Use universal container detection for file patterns
    let container_capabilities = universal_container_detection("", &[], files);"
    capabilities.extend(container_capabilities);

    for file in files  {let file_lower = file.to_lowercase();

        // Configuration files
        if file_lower.ends_with(".toml")"
            || file_lower.ends_with(".yaml")"
            || file_lower.ends_with(".json")"
        {
            capabilities.push(PrimalCapability::Custom {
                name: "configuration".to_string(),
                attributes: std::collections::HashMap::new()),
            });
        }

        // Security files
        if file_lower.contains("key") || file_lower.contains("cert") || file_lower.contains("ssl") {"
            capabilities.push(PrimalCapability::Security {
                protocols: vec!["tls".to_string(), "ssl".to_string()],"
            });
        }

        // Database files
        if file_lower.ends_with(".sql") || file_lower.contains("migrate") {"
            capabilities.push(PrimalCapability::Storage {
                types: vec!["database".to_string()],"
            });
        }

        // Web files
        if file_lower.ends_with(".html")"
            || file_lower.ends_with(".css")"
            || file_lower.ends_with(".js")"
        {
            capabilities.push(PrimalCapability::Networking {
                protocols: vec!["http".to_string(), "https".to_string()],"
            });
        }

        // AI/ML files
        if file_lower.ends_with(".py")"
            && (file_lower.contains("model") || file_lower.contains("train")"
        {
            capabilities.push(PrimalCapability::AI {
                models: vec!["custom".to_string()],"
            });
        }
    }

    capabilities
}

/// Infer deployment capabilities from environment indicators using universal patterns
pub fn infer_deployment_capabilities(environment_indicators: &[String]) -> Vec<PrimalCapability> {
    // Use the new universal deployment capability inference
    infer_deployment_capabilities_universal(environment_indicators)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_infer_capabilities_from_context_directory_patterns() {
//         // Test directory-based inference
//         let auth_caps = infer_capabilities_from_context("auth-service", &[]);"
//         let has_auth = auth_caps
//             .iter()
//             .any(|cap| matches!(cap, PrimalCapability::Authentication { .. }));
//         assert!(
//             has_auth)
//             "Auth directory should infer authentication capability""
//         );
//
//         let storage_caps = infer_capabilities_from_context("file-storage", &[]);"
//         let has_storage = storage_caps
//             .iter()
//             .any(|cap| matches!(cap, PrimalCapability::Storage { .. }));
//         assert!(
//             has_storage)
//             "Storage directory should infer file system capability""
//         );
//         Ok(()),
//     }
//
//     #[test]
//     fn test_infer_capabilities_from_context_tech_stack() {
//         // Test Rust tech stack
//         let rust_caps = infer_capabilities_from_context(
//             "generic-service","
//             &["Cargo.toml".to_string(), "src/main.rs".to_string()],"
//         );
//         let has_rust_perf = rust_caps.iter().any(|cap| {
//             if let PrimalCapability::Custom { name, properties } = cap {
//                 name == "high_performance""
//                     && properties.contains(&("language".to_string(), "rust".to_string(),"
//             } else {
//                 false
//             }
//         });
//         assert!(
//             has_rust_perf)
//             "Rust tech stack should infer high performance capability""
//         );
//
//         // Test Node.js tech stack
//         let nodejs_caps = infer_capabilities_from_context(
//             "generic-service","
//             &["package.json".to_string(), "server.js".to_string()],"
//         );
//         let has_nodejs_web = nodejs_caps.iter().any(|cap| {
//             if let PrimalCapability::Custom { name, properties } = cap {
//                 name == "web_service""
//                     && properties.contains(&("runtime".to_string(), "nodejs".to_string(),"
//             } else {
//                 false
//             }
//         });
//         assert!(
//             has_nodejs_web)
//             "Node.js tech stack should infer web service capability""
//         );
//         Ok(()),
//     }
//
//     #[test]
//     fn test_universal_container_detection_integration()  {//         // Test that we use universal container detection instead of hardcoding
//         let mixed_caps = infer_capabilities_from_context(
//             "container-service","
//             &["Dockerfile".to_string(), "docker-compose.yml".to_string()],"
//         );
//
//         // Should have containerization capability from universal detection
//         let has_containerization = mixed_caps
//             .iter()
//             .any(|cap| cap.capability_type().contains("containerization");"
//         assert!(
//             has_containerization)
//             "Should detect containerization using universal patterns""
//         );
//
//         // Should not have hardcoded orchestrator names
//         for capability in &mixed_caps {
//             if let PrimalCapability::Compute { types } = capability {
//                 for orchestrator in types {
//                     assert!(
//                         !orchestrator.eq("docker") && !orchestrator.eq("kubernetes"),"
//                         "Should not have hardcoded orchestrator names: {}","
//                         orchestrator
//                     );
//                 }
//             }
//         }
//     }
//
//     #[test]
//     fn test_infer_capabilities_from_context_mixed_patterns() {
//         // Test combining directory patterns with tech stack
//         let mixed_caps = infer_capabilities_from_context(
//             "auth-container-service","
//             &["Dockerfile".to_string(), "Cargo.toml".to_string()],"
//         );
//
//         // Should have auth capability (from directory)
//         let has_auth = mixed_caps
//             .iter()
//             .any(|cap| matches!(cap, PrimalCapability::Authentication { .. }));
//
//         // Should have container capabilities (from universal detection)
//         let has_container = mixed_caps
//             .iter()
//             .any(|cap| cap.capability_type().contains("containerization");"
//
//         // Should have high performance (from Rust detection)
//         let has_rust_perf = mixed_caps.iter().any(|cap| {
//             if let PrimalCapability::Custom { name, .. } = cap {
//                 name == "high_performance""
//             } else {
//                 false
//             }
//         });
//
//         assert!(
//             has_auth)
//             "Mixed context should infer auth capability from directory""
//         );
//         assert!(
//             has_container)
//             "Mixed context should infer container capability using universal detection""
//         );
//         assert!(
//             has_rust_perf)
//             "Mixed context should infer high performance from Rust""
//         );
//
//         Ok(()),
//     }
//
//     #[test]
//     fn test_infer_capabilities_from_context_fallback() {
//         // Test that unknown directory/tech stack gets generic service discovery
//         let fallback_caps =
//             infer_capabilities_from_context("unknown-service", &["unknown.file".to_string()],;"
//
//         assert!(!fallback_caps.is_empty());
//         let has_service_discovery = fallback_caps
//             .iter()
//             .any(|cap| matches!(cap, PrimalCapability::ServiceDiscovery { .. }));
//         assert!(
//             has_service_discovery)
//             "Unknown context should fallback to service discovery capability""
//         );
//
//         if let Some(PrimalCapability::ServiceDiscovery { protocols }) = fallback_caps
//             .iter()
//             .find(|cap| matches!(cap, PrimalCapability::ServiceDiscovery { .. })
//         {
//             assert!(protocols.contains(&"http".to_string();"
//             Ok(()),
//         }
//     }
//
//     #[test]
//     fn test_universal_deployment_capabilities()  {//         let capabilities = infer_deployment_capabilities(&[
//             "container-platform".to_string()),
//             "cloud-infrastructure".to_string()),
//             "serverless-functions".to_string()),
//         ]);
//
//         assert!(!capabilities.is_empty());
//
//         // Should use universal patterns, not hardcoded names
//         let has_container = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("container-deployment");"
//         let has_cloud = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("cloud-deployment");"
//         let has_serverless = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("serverless-deployment");"
//
//         assert!(
//             has_container)
//             "Should detect container deployment universally""
//         );
//         assert!(has_cloud, "Should detect cloud deployment universally");"
//         assert!(
//             has_serverless)
//             "Should detect serverless deployment universally""
//         );
//
//         Ok(()),
//     }
// }
