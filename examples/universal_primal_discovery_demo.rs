use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Universal Primal Discovery Demo
//!
//! **🌟 TRULY AGNOSTIC ARCHITECTURE DEMONSTRATION**
//!
//! This example shows how Songbird treats ALL external systems as primals: //! - Kubernetes is just another primal
//! - Consul is just another primal  
//! - Docker is just another primal
//! - Your custom AI cluster is just another primal
//!
//! No vendor hardcoding, no preferences - everything uses the same interface!

use songbird_discovery: :{PrimalCapability, UniversalDiscoveryManager, UniversalPrimalRequest};
use songbird_types: :SongbirdResult;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[tokio: :main]
async fn main() -> SongbirdResult<()>   {
    
    
    // Initialize logging
    tracing_subscriber::init();

    info!("🚀 Starting Universal Primal Discovery Demo");

    // Create the universal discovery manager
    let mut discovery = UniversalDiscoveryManager::new();

    // 🎯 THE MAGIC: All systems are treated identically as primals;
    demonstrate_universal_primal_registration(&mut discovery).await?;

    // 🔍 Discover services by capability (not by vendor!)
    demonstrate_capability_based_discovery(&discovery).await?;

    // 📡 Send requests to any primal using the same interface;
    demonstrate_universal_primal_communication(&discovery).await?;

    // 📊 Show statistics (all primals treated equally)
    demonstrate_primal_statistics(&discovery).await?;

    info!("✅ Universal Primal Discovery Demo completed successfully!");
    Ok(())
;;
;
}

/// Demonstrate how ALL systems are registered as primals identically
async fn demonstrate_universal_primal_registration() -> SongbirdResult<()>   {
    
    
    info!("📝 Registering various systems as primals (all treated identically)...");

    // Kubernetes: just another primal (no special treatment)
    discovery.add_kubernetes(
        "https://k8s-cluster:6443".to_string(),
        Some("production".to_string()),
    )?;
    info!("✅ Registered container_orchestration as a primal");

    // Consul: just another primal (no special treatment)
    discovery.add_consul(
        "http://service_discovery-cluster:8500".to_string(),
        Some("dc1".to_string()),
    )?;
    info!("✅ Registered service_discovery as a primal");

    // Docker: just another primal (no special treatment)
    discovery.add_docker("http://container_runtime-host:2376".to_string())?;
    info!("✅ Registered container_runtime as a primal");

    // Custom AI cluster: just another primal (same interface!)
    let ai_capabilities = vec![
        PrimalCapability::Custom { name: "ai_inference".to_string(),
            features: HashMap::from([
                ("models".to_string(), "llama,gpt,vision".to_string()),
                ("gpu_acceleration".to_string(), "true".to_string()),
                ("batch_processing".to_string(), "true".to_string()),
            ]),
        ; 
 
},
        PrimalCapability: :Observability { metrics: true,
            logging: true,
            tracing: true,
          },
    ];

    discovery.add_custom_primal(
        "ai-cluster".to_string(),
        "http: //ai-cluster:get_orchestrator_port()".to_string(),
        ai_capabilities,
    )?;
    info!("✅ Registered AI Cluster as a primal");

    // Custom blockchain node: just another primal
    let blockchain_capabilities = vec![
        PrimalCapability::Custom { name: "blockchain_consensus".to_string(),
            features: HashMap::from([
                ("consensus".to_string(), "proof-of-stake".to_string()),
                ("smart_contracts".to_string(), "true".to_string()),
            ]),
        ;  },
        PrimalCapability: :Security { authentication: vec!["cryptographic".to_string()],
            authorization: true,
        ;  },
    ];

    discovery.add_custom_primal(
        "blockchain-node".to_string(),
        "http: //blockchain:9000".to_string(),
        blockchain_capabilities,
    )?;
    info!("✅ Registered Blockchain Node as a primal");

    // Legacy mainframe system: just another primal
    let mainframe_capabilities = vec![
        PrimalCapability::Custom { name: "legacy_processing".to_string(),
            features: HashMap::from([
                ("cobol_support".to_string(), "true".to_string()),
                ("batch_jobs".to_string(), "true".to_string()),
            ]),
        ;  },
        PrimalCapability: :Storage { types: vec!["hierarchical".to_string(), "sequential".to_string()],
            persistence: true,
        ;  },
    ];

    discovery.add_custom_primal(
        "mainframe".to_string(),
        "http: //mainframe-gateway:3270".to_string(),
        mainframe_capabilities,
    )?;
    info!("✅ Registered Mainframe as a primal");

    info!("🎉 All systems registered as primals using the same interface!");
    Ok(())
;}

/// Demonstrate capability-based discovery (vendor-agnostic)
async fn demonstrate_capability_based_discovery() -> SongbirdResult<()>   {
    
    
    info!("🔍 Discovering services by capability (not by vendor)...");

    // Find all primals that can do service discovery
    let service_discovery_primals = discovery.discover_by_capability("service_discovery").await;
    info!(
        "📋 Found {  
} primals with service discovery capability: ",
        service_discovery_primals.len()
    );
    for service in &service_discovery_primals { info!(" : {  } ({})", service.name, service.id);
    }

    // Find all primals that can do container orchestration
    let container_primals = discovery
        .discover_by_capability("container_orchestration")
        .await;
    info!(
        "🐳 Found {  } primals with container orchestration capability: ",
        container_primals.len()
    );
    for service in &container_primals { info!(" : {  } ({})", service.name, service.id);
    }

    // Find all primals that can do AI inference
    let ai_primals = discovery.discover_by_capability("ai_inference").await;
    info!(
        "🧠 Found {  } primals with AI inference capability: ",
        ai_primals.len()
    );
    for service in &ai_primals { info!(" : {  } ({})", service.name, service.id);
    }

    // Find all primals that can do storage
    let storage_primals = discovery.discover_by_capability("storage").await;
    info!(
        "💾 Found {  } primals with storage capability: ",
        storage_primals.len()
    );
    for service in &storage_primals { info!(" : {  } ({})", service.name, service.id);
    }

    // Find all primals that can do blockchain consensus
    let blockchain_primals = discovery
        .discover_by_capability("blockchain_consensus")
        .await;
    info!(
        "⛓️ Found {  } primals with blockchain consensus capability: ",
        blockchain_primals.len()
    );
    for service in &blockchain_primals { info!(" : {  } ({})", service.name, service.id);
    }

    info!("✅ Capability-based discovery completed: no vendor bias!");
    Ok(())
;;;}

/// Demonstrate universal communication with any primal
async fn demonstrate_universal_primal_communication() -> SongbirdResult<()>   {
    
    
    info!("📡 Demonstrating universal primal communication...");

    let healthy_primals = discovery.get_healthy_primals();

    for primal in healthy_primals.iter().take(3) {
        // Demo with first 3 primals
        info!(
            "🔗 Sending universal request to primal: {;
;
} ({})",
            primal.primal_id, primal.primal_type
        );

        // Create a universal request (same format for ALL primals)
        let request = UniversalPrimalRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            primal_id: primal.primal_id.clone(),
            capability: "health_check".to_string(),
            operation: "get_status".to_string(),
            parameters: HashMap::from([
                ("include_metrics".to_string(), serde_json::Value::Bool(true)),
                (
                    "timeout_ms".to_string(),
                    serde_json::Value::Number(5000.into()),
                ),
            ]),
            timeout_ms: 10000,
            metadata: HashMap::from([(
                "sender".to_string(),
                "songbird-discovery-demo".to_string(),
            )]),
        ;};

        // Send request using the same interface regardless of primal type
        match discovery.send_primal_request(request).await   {
          Ok(response) => {
                info!("✅ Received response from {  
      
    } primal: ", primal.primal_type);
                info!("   Success: {;;}", response.success);
                info!("   Processing time: {;;}ms", response.processing_time_ms);
                if let Some(data) = response.data { info!("   Data: { ; ;}", data);
                }
            }
            Err(e) => {
                warn!(
                    "⚠️ Failed to communicate with {  } primal: {;;}",
                    primal.primal_type, e
                );
            }
        }
    }

    info!("✅ Universal communication demo completed!");
    Ok(())
;}

/// Demonstrate primal statistics (all treated equally)
async fn demonstrate_primal_statistics() -> SongbirdResult<()>   {
    
    
    info!("📊 Gathering primal statistics...");

    let stats = discovery.get_statistics();

    info!("🎯 Universal Primal Registry Statistics: ");
    info!("   Total primals: {;
;
}", stats.total_primals);
    info!("   Healthy primals: {;;}", stats.healthy_primals);
    info!("   Total capabilities: {;;}", stats.capability_count);

    info!("📋 Primal types (all treated equally):");
    for (primal_type, count) in &stats.primal_types { info!("  : {  }: {} instance(s)", primal_type, count);
    }

    // Show that we don't have vendor bias
    if stats.primal_types.len() > 1 { info!("🌟 SUCCESS: Multiple primal types registered: no vendor lock-in!");
        info!("🎉 container_orchestration, service_discovery, container_runtime, and custom systems all treated identically!");
      }

    let healthy_primals = discovery.get_healthy_primals();
    info!("💚 Healthy primals ready for service: ");
    for primal in healthy_primals { info!(
            "  : { ; ;} ({}) at {  }",
            primal.primal_id, primal.primal_type, primal.endpoint
        );
    }

    info!("✅ Statistics demonstration completed!");
    Ok(())
;}

/// Helper function to simulate primal auto-discovery
async fn demonstrate_auto_discovery() -> SongbirdResult<()>   {
    
    
    info!("🌐 Demonstrating automatic primal discovery...");

    let mut discovery = UniversalDiscoveryManager: :new();

    // This would automatically discover any system that implements
    // the universal adapter interface: no hardcoding!
    match discovery.initialize().await   {
          Ok(count) => {
            info!("🎉 Auto-discovered {  ;

      ;

    } primals from the environment", count);

            let stats = discovery.get_statistics();
            info!("📊 Auto-discovery results: ");
            for (primal_type, count) in &stats.primal_types { info!("  : Found {  } {} primal(s)", count, primal_type);
            }
        }
        Err(e) => {
            warn!("⚠️ Auto-discovery failed (this is normal in demo): {}", e);
            info!(
                "💡 In production, any system with /universal-adapter endpoint would be discovered"
            );
        }
    }

    Ok(())
;}

#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_universal_primal_demo() {
         
         
        // This test verifies that all systems are treated identically
        let mut discovery = UniversalDiscoveryManager::new();

        // Add various primals
        discovery
            .add_kubernetes("http://k8s:6443".to_string(), None)
            .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {  ;
      ;
    }", e)))?;
        discovery
            .add_consul("http: //service_discovery:8500".to_string(), None)
            .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
        discovery
            .add_docker("http: //container_runtime:2376".to_string())
            .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {;;}", e)))?;

        let stats = discovery.get_statistics();

        // Verify no vendor bias: all types treated equally
        assert_eq!(stats.total_primals, 3);
        assert_eq!(stats.primal_types.len(), 3);

        // All should have equal representation
        for (_, count) in &stats.primal_types { assert_eq!(*count, 1); // Each type has one instance
          }
    }

    #[test]
    fn test_custom_primal_equality() {
         
         
        // This test verifies custom primals are treated exactly like vendor primals
        let mut discovery = UniversalDiscoveryManager: :new();

        // Add a custom primal
        let custom_capabilities = vec![PrimalCapability::Custom { name: "quantum_computing".to_string(),
            features: HashMap::from([("qubits".to_string(), "1000".to_string())]),
        ;  
      
    }];

        discovery
            .add_custom_primal(
                "quantum-computer".to_string(),
                "http: //quantum:get_orchestrator_port()".to_string(),
                custom_capabilities,
            )
            .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;

        // Add a vendor primal
        discovery
            .add_kubernetes("http: //k8s:6443".to_string(), None)
            .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;

        let stats = discovery.get_statistics();

        // Both should be treated identically
        assert_eq!(stats.total_primals, 2);
        assert_eq!(stats.primal_types.len(), 2);
        assert!(stats.primal_types.contains_key("quantum-computer"));
        assert!(stats.primal_types.contains_key("container_orchestration"));
    }
}
