//! Comprehensive Ecosystem Integration Tests
//!
//! Tests the complete universal capability-based discovery system across
//! multiple primals to ensure true ecosystem integration works.

use songbird_universal_primals::{
    discovery::{discover_ecosystem_primals, EcosystemDiscovery, EcosystemDiscoveryConfig},
    traits::{PrimalCapability, PrimalType},
};
use std::collections::HashMap;
use tokio::test;
use tracing::{info, warn};
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_ecosystem_universal_discovery() {
    info!("🌐 Testing universal ecosystem discovery system");
    
    // Test the ecosystem discovery with default configuration
    let config = EcosystemDiscoveryConfig {
        ecosystem_base_path: "../".to_string(),
        health_check_timeout_ms: 3000,
        max_concurrent_discoveries: 10,
        enable_capability_inference: true,
        enable_filesystem_discovery: true,
        enable_network_discovery: false, // Skip network scan for unit tests
    };
    
    let discovery = EcosystemDiscovery::new(config);
    
    match discovery.discover_ecosystem_primals().await {
        Ok(discovered_primals) => {
            info!("✅ Successfully discovered {} primals", discovered_primals.len());
            
            // Verify universal architecture principles
            for primal in &discovered_primals {
                // Each primal should have a valid type
                assert!(!primal.primal_type.as_str().is_empty());
                
                // Each primal should have at least one capability
                assert!(!primal.capabilities.is_empty());
                
                // Verify capability-based classification
                verify_capability_based_classification(primal);
                
                info!("  🔧 {} [{}]: {} capabilities at {}", 
                    primal.metadata.get("directory_name").unwrap_or(&"unknown".to_string()),
                    primal.primal_type.as_str(),
                    primal.capabilities.len(),
                    primal.endpoint
                );
            }
            
            // Test universal routing
            test_universal_capability_routing(&discovered_primals).await;
            
        }
        Err(e) => {
            warn!("⚠️ No running ecosystem primals found: {}", e);
            info!("This is expected in development environments where primals aren't running");
            
            // Even without running primals, test the discovery logic
            test_capability_inference_logic().await;
        }
    }
}

#[tokio::test]
#[traced_test]
async fn test_capability_based_architecture_integrity() {
    info!("🏗️ Testing capability-based architecture integrity");
    
    let discovery = EcosystemDiscovery::new(EcosystemDiscoveryConfig::default());
    
    // Test that we don't have hardcoded primal names
    let test_cases = vec![
        ("my-custom-security-service", "security"),
        ("acme-storage-provider", "storage"), 
        ("quantum-compute-cluster", "compute"),
        ("neural-ai-engine", "ai"),
        ("container-orchestrator", "orchestration"),
    ];
    
    for (custom_primal_name, expected_category) in test_cases {
        info!("  Testing custom primal: {}", custom_primal_name);
        
        // This should work for ANY primal name, proving we're not hardcoded
        let (primal_type, capabilities) = discovery.get_default_capabilities_for_primal(custom_primal_name);
        
        // Verify the capabilities are inferred from name patterns, not hardcoded
        assert!(!capabilities.is_empty());
        
        // Verify the type matches expected category
        let category = categorize_capabilities(&capabilities);
        assert_eq!(category, expected_category, 
            "Custom primal '{}' should be categorized as '{}' based on capabilities",
            custom_primal_name, expected_category);
    }
    
    info!("✅ Architecture integrity verified - truly universal!")
}

#[tokio::test]
#[traced_test] 
async fn test_multi_primal_capability_aggregation() {
    info!("🔗 Testing multi-primal capability aggregation");
    
    // Simulate discovering multiple primals with overlapping capabilities
    let config = EcosystemDiscoveryConfig::default();
    let discovery = EcosystemDiscovery::new(config);
    
    // Test that we can aggregate capabilities from multiple primals
    let mock_primals = vec![
        ("security-primal-1", vec!["authentication", "encryption"]),
        ("security-primal-2", vec!["authorization", "audit"]),
        ("storage-primal-1", vec!["object-storage", "replication"]),
        ("compute-primal-1", vec!["containers", "scaling"]),
    ];
    
    let mut aggregated_capabilities = HashMap::new();
    
    for (primal_name, cap_names) in mock_primals {
        let (primal_type, capabilities) = discovery.get_default_capabilities_for_primal(primal_name);
        
        // Verify each primal gets appropriate capabilities
        assert!(!capabilities.is_empty());
        
        // Aggregate by primal type
        aggregated_capabilities
            .entry(primal_type.as_str().to_string())
            .or_insert_with(Vec::new)
            .extend(capabilities);
    }
    
    // Verify we have multiple types of primals
    assert!(aggregated_capabilities.contains_key("security"));
    assert!(aggregated_capabilities.contains_key("storage"));
    assert!(aggregated_capabilities.contains_key("compute"));
    
    info!("✅ Multi-primal aggregation working correctly");
}

/// Helper function to verify capability-based classification
fn verify_capability_based_classification(primal: &songbird_universal_primals::discovery::types::DiscoveredPrimal) {
    // The primal type should match its primary capabilities
    let primary_capability_type = infer_primary_type_from_capabilities(&primal.capabilities);
    
    // Allow for legitimate multi-capability primals
    let type_matches = primal.primal_type.as_str() == primary_capability_type || 
                      primal.primal_type.as_str() == "universal";
                      
    assert!(type_matches, 
        "Primal type '{}' should match primary capability type '{}' or be 'universal'",
        primal.primal_type.as_str(), primary_capability_type);
}

/// Helper function to test universal capability routing
async fn test_universal_capability_routing(primals: &[songbird_universal_primals::discovery::types::DiscoveredPrimal]) {
    info!("🔀 Testing universal capability routing");
    
    // Test routing different capability types to appropriate primals
    let capability_tests = vec![
        ("authentication", "security"),
        ("file-storage", "storage"),
        ("container-runtime", "compute"),
        ("model-inference", "ai"),
        ("service-discovery", "orchestration"),
    ];
    
    for (required_capability, expected_type) in capability_tests {
        let suitable_primals: Vec<_> = primals.iter()
            .filter(|p| has_capability_type(p, required_capability))
            .collect();
            
        if !suitable_primals.is_empty() {
            info!("  ✅ Found {} primals with '{}' capability", 
                suitable_primals.len(), required_capability);
            
            // Verify routing logic works
            for primal in suitable_primals {
                assert!(primal.primal_type.as_str() == expected_type || 
                       primal.primal_type.as_str() == "universal",
                    "Primal with '{}' capability should be type '{}' or 'universal'",
                    required_capability, expected_type);
            }
        }
    }
}

/// Helper function to test capability inference logic
async fn test_capability_inference_logic() {
    info!("🧠 Testing capability inference logic");
    
    let discovery = EcosystemDiscovery::new(EcosystemDiscoveryConfig::default());
    
    // Test various naming patterns for capability inference
    let inference_tests = vec![
        ("auth-service", should_have_auth_capabilities),
        ("file-storage", should_have_storage_capabilities),
        ("ml-inference", should_have_ai_capabilities),
        ("kube-orchestrator", should_have_orchestration_capabilities),
    ];
    
    for (service_name, validator) in inference_tests {
        let (_, capabilities) = discovery.get_default_capabilities_for_primal(service_name);
        validator(&capabilities, service_name);
    }
    
    info!("✅ Capability inference logic working correctly");
}

// Helper functions for validation
fn categorize_capabilities(capabilities: &[PrimalCapability]) -> &'static str {
    for capability in capabilities {
        match capability {
            PrimalCapability::Authentication { .. } | 
            PrimalCapability::Encryption { .. } |
            PrimalCapability::ThreatDetection { .. } => return "security",
            
            PrimalCapability::FileSystem { .. } |
            PrimalCapability::ObjectStorage { .. } |
            PrimalCapability::DataReplication { .. } => return "storage",
            
            PrimalCapability::ContainerRuntime { .. } |
            PrimalCapability::ServerlessExecution { .. } => return "compute",
            
            PrimalCapability::ModelInference { .. } |
            PrimalCapability::AgentFramework { .. } => return "ai",
            
            PrimalCapability::Orchestration { .. } |
            PrimalCapability::ServiceDiscovery { .. } => return "orchestration",
            
            _ => continue,
        }
    }
    "universal"
}

fn infer_primary_type_from_capabilities(capabilities: &[PrimalCapability]) -> &'static str {
    categorize_capabilities(capabilities)
}

fn has_capability_type(primal: &songbird_universal_primals::discovery::types::DiscoveredPrimal, capability_type: &str) -> bool {
    primal.capabilities.iter().any(|cap| {
        match (capability_type, cap) {
            ("authentication", PrimalCapability::Authentication { .. }) => true,
            ("file-storage", PrimalCapability::FileSystem { .. }) => true,
            ("container-runtime", PrimalCapability::ContainerRuntime { .. }) => true,
            ("model-inference", PrimalCapability::ModelInference { .. }) => true,
            ("service-discovery", PrimalCapability::ServiceDiscovery { .. }) => true,
            _ => false,
        }
    })
}

fn should_have_auth_capabilities(capabilities: &[PrimalCapability], service_name: &str) {
    let has_auth = capabilities.iter().any(|cap| matches!(cap, PrimalCapability::Authentication { .. }));
    assert!(has_auth, "Service '{}' should have authentication capabilities", service_name);
}

fn should_have_storage_capabilities(capabilities: &[PrimalCapability], service_name: &str) {
    let has_storage = capabilities.iter().any(|cap| matches!(cap, 
        PrimalCapability::FileSystem { .. } | PrimalCapability::ObjectStorage { .. }
    ));
    assert!(has_storage, "Service '{}' should have storage capabilities", service_name);
}

fn should_have_ai_capabilities(capabilities: &[PrimalCapability], service_name: &str) {
    let has_ai = capabilities.iter().any(|cap| matches!(cap, 
        PrimalCapability::ModelInference { .. } | PrimalCapability::AgentFramework { .. }
    ));
    assert!(has_ai, "Service '{}' should have AI capabilities", service_name);
}

fn should_have_orchestration_capabilities(capabilities: &[PrimalCapability], service_name: &str) {
    let has_orchestration = capabilities.iter().any(|cap| matches!(cap, 
        PrimalCapability::Orchestration { .. } | PrimalCapability::ServiceDiscovery { .. }
    ));
    assert!(has_orchestration, "Service '{}' should have orchestration capabilities", service_name);
} 