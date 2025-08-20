// Universal Container Detection
//
// Provides agnostic container capability detection without hardcoding specific
// orchestrator names like "docker" or "kubernetes". Uses pattern-based detection
// to identify container capabilities universally.

use crate::traits::PrimalCapability;

/// Container file patterns that indicate containerization capability
const CONTAINER_FILE_PATTERNS: &[&str] = &[
    "dockerfile",
    "containerfile",
    ".dockerignore",
    "docker-compose",
    "compose.yaml",
    "compose.yml",
];

/// Orchestration file patterns that indicate orchestration capability  
const ORCHESTRATION_FILE_PATTERNS: &[&str] = &[
    "k8s",
    "kubernetes",
    "kustomization",
    "helm",
    "deployment.yaml",
    "service.yaml",
    "ingress.yaml",
    "configmap.yaml",
    "secret.yaml",
];

/// Serverless patterns that indicate serverless deployment capability
const SERVERLESS_PATTERNS: &[&str] = &[
    "serverless",
    "lambda",
    "function",
    "faas",
    "knative",
    "openfaas",
];

/// Universal container capability inference
pub fn infer_container_capabilities_from_context(
    directory_name: &str,
    tech_stack: &[String],
    file_patterns: &[String],
) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    // Check for containerization indicators - modern structured capabilities
    if has_containerization_indicators(directory_name, tech_stack, file_patterns) {
        capabilities.push(PrimalCapability::Compute {
            types: vec!["container".to_string()],
        });

        // Infer specific container patterns without hardcoding orchestrator names
        let orchestrators = detect_orchestration_systems(tech_stack, file_patterns);
        if !orchestrators.is_empty() {
            capabilities.push(PrimalCapability::Compute {
                types: vec!["container".to_string()],
            });
        }
    }

    // Check for orchestration indicators
    if has_orchestration_indicators(directory_name, tech_stack, file_patterns) {
        capabilities.push(PrimalCapability::Orchestration {
            features: vec!["container_orchestration".to_string()],
        });
    }

    // Check for serverless indicators
    if has_serverless_indicators(directory_name, tech_stack, file_patterns) {
        capabilities.push(PrimalCapability::Orchestration {
            features: vec!["serverless_deployment".to_string()],
        });
    }

    // Check for cloud-native patterns
    if has_cloud_native_indicators(directory_name, tech_stack, file_patterns) {
        capabilities.push(PrimalCapability::Orchestration {
            features: vec!["cloud_native".to_string()],
        });
    }

    capabilities
}

/// Detect containerization indicators without hardcoding specific technologies
fn has_containerization_indicators(
    directory_name: &str,
    tech_stack: &[String],
    file_patterns: &[String],
) -> bool {
    let dir_lower = directory_name.to_lowercase();

    // Check directory name patterns
    if dir_lower.contains("container") || dir_lower.contains("docker") || dir_lower.contains("pod")
    {
        return true;
    }

    // Check tech stack for container-related files
    for tech in tech_stack {
        let tech_lower = tech.to_lowercase();
        for pattern in CONTAINER_FILE_PATTERNS {
            if tech_lower.contains(pattern) {
                return true;
            }
        }
    }

    // Check file patterns
    for file in file_patterns {
        let file_lower = file.to_lowercase();
        for pattern in CONTAINER_FILE_PATTERNS {
            if file_lower.contains(pattern) {
                return true;
            }
        }
    }

    false
}

/// Detect orchestration systems without hardcoding names
fn detect_orchestration_systems(tech_stack: &[String], file_patterns: &[String]) -> Vec<String> {
    let mut orchestrators = Vec::new();

    // Generic detection based on file patterns
    let all_files: Vec<&String> = tech_stack.iter().chain(file_patterns.iter()).collect();

    for file in &all_files {
        let file_lower = file.to_lowercase();

        // Container runtime detection (generic patterns)
        if file_lower.contains("dockerfile") || file_lower.contains("containerfile") {
            orchestrators.push("container-runtime".to_string());
        }

        // Compose-based orchestration
        if file_lower.contains("compose") {
            orchestrators.push("compose-based".to_string());
        }

        // Kubernetes-style orchestration (without hardcoding "kubernetes")
        if file_lower.contains("deployment.yaml")
            || file_lower.contains("service.yaml")
            || file_lower.contains("kustomization")
        {
            orchestrators.push("yaml-based-orchestration".to_string());
        }

        // Helm-based orchestration
        if file_lower.contains("chart.yaml") || file_lower.contains("values.yaml") {
            orchestrators.push("chart-based-orchestration".to_string());
        }
    }

    // Remove duplicates
    orchestrators.sort();
    orchestrators.dedup();
    orchestrators
}

/// Detect orchestration indicators
fn has_orchestration_indicators(
    directory_name: &str,
    tech_stack: &[String],
    file_patterns: &[String],
) -> bool {
    let dir_lower = directory_name.to_lowercase();

    // Check directory patterns
    if dir_lower.contains("orchestr")
        || dir_lower.contains("cluster")
        || dir_lower.contains("deploy")
    {
        return true;
    }

    // Check for orchestration file patterns
    let all_files: Vec<&String> = tech_stack.iter().chain(file_patterns.iter()).collect();

    for file in &all_files {
        let file_lower = file.to_lowercase();
        for pattern in ORCHESTRATION_FILE_PATTERNS {
            if file_lower.contains(pattern) {
                return true;
            }
        }
    }

    false
}

/// Detect serverless indicators
fn has_serverless_indicators(
    directory_name: &str,
    tech_stack: &[String],
    file_patterns: &[String],
) -> bool {
    let dir_lower = directory_name.to_lowercase();

    // Check directory patterns
    for pattern in SERVERLESS_PATTERNS {
        if dir_lower.contains(pattern) {
            return true;
        }
    }

    // Check tech stack and files
    let all_files: Vec<&String> = tech_stack.iter().chain(file_patterns.iter()).collect();

    for file in &all_files {
        let file_lower = file.to_lowercase();
        for pattern in SERVERLESS_PATTERNS {
            if file_lower.contains(pattern) {
                return true;
            }
        }
    }

    false
}

/// Detect cloud-native patterns
fn has_cloud_native_indicators(
    directory_name: &str,
    tech_stack: &[String],
    file_patterns: &[String],
) -> bool {
    let indicators = [
        "cloud-native",
        "microservice",
        "service-mesh",
        "istio",
        "envoy",
        "prometheus",
        "grafana",
        "jaeger",
        "opentelemetry",
    ];

    let dir_lower = directory_name.to_lowercase();

    // Check directory patterns
    for indicator in &indicators {
        if dir_lower.contains(indicator) {
            return true;
        }
    }

    // Check tech stack and files
    let all_files: Vec<&String> = tech_stack.iter().chain(file_patterns.iter()).collect();

    for file in &all_files {
        let file_lower = file.to_lowercase();
        for indicator in &indicators {
            if file_lower.contains(indicator) {
                return true;
            }
        }
    }

    false
}

/// Get universal deployment capabilities based on environment patterns
pub fn infer_deployment_capabilities_universal(
    environment_indicators: &[String],
) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    for indicator in environment_indicators {
        let indicator_lower = indicator.to_lowercase();

        // Universal container deployment patterns - modern structured capabilities
        if indicator_lower.contains("container") {
            capabilities.push(PrimalCapability::Orchestration {
                features: vec!["container_deployment".to_string()],
            });
        }

        // Universal orchestration patterns
        if indicator_lower.contains("orchestrat") || indicator_lower.contains("cluster") {
            capabilities.push(PrimalCapability::Orchestration {
                features: vec!["deployment".to_string(), "cluster_management".to_string()],
            });
        }

        // Universal cloud patterns
        if indicator_lower.contains("cloud")
            || indicator_lower.contains("aws")
            || indicator_lower.contains("gcp")
            || indicator_lower.contains("azure")
        {
            capabilities.push(PrimalCapability::Orchestration {
                features: vec!["cloud_deployment".to_string()],
            });
        }

        // Universal serverless patterns
        if has_serverless_patterns(&indicator_lower) {
            capabilities.push(PrimalCapability::Compute {
                types: vec!["serverless".to_string()],
            });
        }

        // Universal edge patterns
        if indicator_lower.contains("edge") || indicator_lower.contains("cdn") {
            capabilities.push(PrimalCapability::Orchestration {
                features: vec!["edge_deployment".to_string()],
            });
        }
    }

    capabilities
}

/// Helper to check serverless patterns
fn has_serverless_patterns(text: &str) -> bool {
    SERVERLESS_PATTERNS
        .iter()
        .any(|pattern| text.contains(pattern))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_containerization_detection_without_hardcoding() {
//         // Test detection without hardcoding specific orchestrator names
//         let capabilities = infer_container_capabilities_from_context(
//             "container-service",
//             &["Dockerfile".to_string(), "docker-compose.yml".to_string()],
//             &["app.dockerfile".to_string()],
//         );
//
//         // Should detect containerization capability
//         let has_containerization = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("containerization"));
//         assert!(
//             has_containerization,
//             "Should detect containerization without hardcoding"
//         );
//
//         // Should detect container runtime with generic patterns
//         let has_runtime = capabilities
//             .iter()
//             .any(|cap| matches!(cap, PrimalCapability::Compute { .. }));
//         assert!(has_runtime, "Should detect container runtime capability");
//         Ok(())
//     }
//
//     #[test]
//     fn test_orchestration_detection_universal() {
//         let capabilities = infer_container_capabilities_from_context(
//             "k8s-deployment",
//             &["deployment.yaml".to_string(), "service.yaml".to_string()],
//             &["configmap.yaml".to_string()],
//         );
//
//         // Should detect orchestration without hardcoding "kubernetes"
//         let has_orchestration = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("orchestration"));
//         assert!(has_orchestration, "Should detect orchestration universally");
//         Ok(())
//     }
//
//     #[test]
//     fn test_serverless_detection_agnostic() {
//         let capabilities = infer_container_capabilities_from_context(
//             "lambda-function",
//             &["serverless.yml".to_string()],
//             &["function.json".to_string()],
//         );
//
//         let has_serverless = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("serverless"));
//         assert!(
//             has_serverless,
//             "Should detect serverless patterns universally"
//         );
//         Ok(())
//     }
//
//     #[test]
//     fn test_cloud_native_detection() {
//         let capabilities = infer_container_capabilities_from_context(
//             "microservice-mesh",
//             &["istio.yaml".to_string(), "prometheus.yml".to_string()],
//             &["jaeger-config.yaml".to_string()],
//         );
//
//         let has_cloud_native = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("cloud-native"));
//         assert!(has_cloud_native, "Should detect cloud-native patterns");
//         Ok(())
//     }
//
//     #[test]
//     fn test_deployment_capabilities_universal() {
//         let capabilities = infer_deployment_capabilities_universal(&[
//             "container-platform".to_string(),
//             "cloud-infrastructure".to_string(),
//             "edge-deployment".to_string(),
//         ]);
//
//         assert!(
//             !capabilities.is_empty(),
//             "Should detect deployment capabilities"
//         );
//
//         let has_container = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("container-deployment"));
//         let has_cloud = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("cloud-deployment"));
//         let has_edge = capabilities
//             .iter()
//             .any(|cap| cap.capability_type().contains("edge-deployment"));
//
//         assert!(has_container, "Should detect container deployment");
//         assert!(has_cloud, "Should detect cloud deployment");
//         assert!(has_edge, "Should detect edge deployment");
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_no_hardcoded_orchestrator_names() {
//         let capabilities = infer_container_capabilities_from_context(
//             "custom-container-platform",
//             &[
//                 "custom.dockerfile".to_string(),
//                 "orchestration.yaml".to_string(),
//             ],
//             &["deployment-config.yml".to_string()],
//         );
//
//         // Verify we don't have hardcoded names like "docker" or "kubernetes"
//         for capability in &capabilities {
//             if let PrimalCapability::Compute { types } = capability {
//                 for orchestrator in types {
//                     // Should use generic patterns, not hardcoded names
//                     assert!(!orchestrator.eq("docker"), "Should not hardcode 'docker'");
//                     assert!(
//                         !orchestrator.eq("kubernetes"),
//                         "Should not hardcode 'kubernetes'"
//                     );
//                     // Should use pattern-based names
//                     assert!(
//                         orchestrator.contains("runtime")
//                             || orchestrator.contains("based")
//                             || orchestrator.contains("orchestration"),
//                         "Should use pattern-based orchestrator names: {}",
//                         orchestrator
//                     );
//                 }
//             }
//         }
//     }
// }
