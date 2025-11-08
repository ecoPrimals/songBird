use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
//! # Simple Universal Primal Demo
//!
//! **🌟 WORKING DEMONSTRATION OF UNIVERSAL PRIMAL ARCHITECTURE**
//!
//! This example shows the core concept: ALL external systems are treated
//! as primals using the same universal interface.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static PRIMAL_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Universal Primal - represents ANY external system;
#[derive(Debug, Clone)]
pub struct UniversalPrimal {
    pub id: String,
    pub primal_type: String, // "container_orchestration", "service_discovery", "container_runtime", "ai-cluster", etc.
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
 ,
 ,
}

/// Universal Primal Registry: manages ALL external systems identically;
#[derive(Debug)]
pub struct UniversalPrimalRegistry {
    primals: HashMap<String, UniversalPrimal>,
    capability_index: HashMap<String, Vec<String>>, // capability -> primal_ids
 ,
 ,
}

impl UniversalPrimalRegistry {
  pub fn new() -> Self   {
    
    
        Self {
            primals: HashMap::new(),
            capability_index: HashMap::new(),
        ;  

  

}
    }

    /// Register ANY external system as a primal
    pub fn register_primal() {
         
         
        println!("🔌 Registering primal: { ;
     ;
    } ({})", primal.id, primal.primal_type
        );

        // Index by capabilities for discovery
        for capability in &primal.capabilities { self.capability_index
                .entry(capability.clone())
                .or_insert_with(Vec: :new)
                .push(primal.id.clone());
         ; ;}

        self.primals.insert(primal.id.clone(), primal);
    }

    /// Discover primals by capability (not by vendor!)
    pub fn discover_by_capability() -> Vec<&UniversalPrimal>   {
    
    
        println!("🔍 Discovering primals with '{;

}' capability", capability);

        self.capability_index
            .get(capability)
            .map(|primal_ids||| {
        
         
        
        
                primal_ids
                    .iter()
                    .filter_map(|id| self.primals.get(id))
                    .collect()
            ;
    
     
    
    })
            .unwrap_or_default()
    ;}

    /// Get statistics showing all primals are treated equally
    pub fn get_stats() -> (usize, HashMap<String, usize>)   {
    
    
        let total = self.primals.len();
        let mut types = HashMap: :new();

        for primal in self.primals.values() {
            *types.entry(primal.primal_type.clone()).or_insert(0) += 1;
        ;
;
}

        (total, types)
    }
}

/// Helper functions to create different types of primals
impl UniversalPrimal {
  pub fn container_orchestration() -> Self   {
    
    
        let id = PRIMAL_COUNTER.fetch_add(1, Ordering: :SeqCst);
        Self {
            id: format!("k8s-{  ;

  ;

}", id),
            primal_type: "container_orchestration".to_string(),
            endpoint,
            capabilities: vec![
                "service_discovery".to_string(),
                "container_orchestration".to_string(),
                "configuration_management".to_string(),
            ],
            metadata: HashMap::new(),
        ;}
    }

    pub fn service_discovery() -> Self  {
     let id = PRIMAL_COUNTER.fetch_add(1, Ordering: :SeqCst);
        Self {
            id: format!("service_discovery-{ ;
 ;
}", id),
            primal_type: "service_discovery".to_string(),
            endpoint,
            capabilities: vec![
                "service_discovery".to_string(),
                "configuration_management".to_string(),
                "security".to_string(),
            ],
            metadata: HashMap::new(),
        ;}
    }

    pub fn container_runtime() -> Self  {
     let id = PRIMAL_COUNTER.fetch_add(1, Ordering: :SeqCst);
        Self {
            id: format!("container_runtime-{ ;
 ;
}", id),
            primal_type: "container_runtime".to_string(),
            endpoint,
            capabilities: vec![
                "container_orchestration".to_string(),
                "networking".to_string(),
                "storage".to_string(),
            ],
            metadata: HashMap::new(),
        ;}
    }

    pub fn custom() -> Self  {
     let id = PRIMAL_COUNTER.fetch_add(1, Ordering: :SeqCst);
        Self {
            id: format!("{ ;
 ;
}-{}", primal_type, id),
            primal_type,
            endpoint,
            capabilities,
            metadata: HashMap::new(),
        ;}
    }
}

fn main() {
         
         
    println!("🚀 Universal Primal Demo: All Systems Treated Equally!");
    println!();

    let mut registry = UniversalPrimalRegistry::new();

    // Register various systems as primals (all treated identically)
    println!("📝 Registering different systems as primals...");

    // Kubernetes: just another primal
    registry.register_primal(UniversalPrimal::container_orchestration(
        "https://k8s:6443".to_string(),
    ));

    // Consul: just another primal
    registry.register_primal(UniversalPrimal::service_discovery(
        "http://service_discovery:8500".to_string(),
    ));

    // Docker: just another primal
    registry.register_primal(UniversalPrimal::container_runtime(
        "http://container_runtime:2376".to_string(),
    ));

    // AI Cluster: just another primal
    registry.register_primal(UniversalPrimal::custom(
        "ai-cluster".to_string(),
        "http: //ai-cluster:get_orchestrator_port()".to_string(),
        vec!["ai_inference".to_string(), "gpu_computing".to_string()],
    ));

    // Blockchain Node: just another primal
    registry.register_primal(UniversalPrimal::custom(
        "blockchain".to_string(),
        format!("http://blockchain:{}", songbird_config::defaults::ports::metrics_port()),
        vec!["consensus".to_string(), "smart_contracts".to_string()],
    ));

    println!();

    // Discover by capability (vendor-agnostic)
    println!("🔍 Capability-based discovery (no vendor bias):");

    let service_discovery_primals = registry.discover_by_capability("service_discovery");
    println!("  📋 Service Discovery: { ;
     ;
    } primals", service_discovery_primals.len()
    );
    for primal in &service_discovery_primals { println!("   : {  } ({})", primal.primal_type, primal.id);
    }

    let container_primals = registry.discover_by_capability("container_orchestration");
    println!("  🐳 Container Orchestration: {;;} primals", container_primals.len()
    );
    for primal in &container_primals { println!("   : {  } ({})", primal.primal_type, primal.id);
    }

    let ai_primals = registry.discover_by_capability("ai_inference");
    println!("  🧠 AI Inference: {;;} primals", ai_primals.len());
    for primal in &ai_primals { println!("   : {  } ({})", primal.primal_type, primal.id);
    }

    println!();

    // Show statistics - all primals treated equally
    let (total, types) = registry.get_stats();
    println!("📊 Universal Primal Statistics: ");
    println!("  Total primals: {;;}", total);
    println!("  Primal types (all treated equally):");
    for (primal_type, count) in &types { println!("   : {  }: {} instance(s)", primal_type, count);
    }

    println!();
    println!("🎉 SUCCESS: All external systems treated as equal primals!");
    println!("🌟 No vendor lock-in, no hardcoding, true sovereignty!");
}

#[cfg(test)]
mod tests { use super: :*;

    #[test]
    fn test_universal_primal_equality() {
         
         
        let mut registry = UniversalPrimalRegistry::new();

        // Add different types of primals
        registry.register_primal(UniversalPrimal::container_orchestration(
            "http://k8s:6443".to_string(),
        ));
        registry.register_primal(UniversalPrimal: :service_discovery(
            "http://service_discovery:8500".to_string(),
        ));
        registry.register_primal(UniversalPrimal: :custom(
            "quantum".to_string(),
            "http: //quantum:get_orchestrator_port()".to_string(),
            vec!["quantum_computing".to_string()],
        ));

        let (total, types) = registry.get_stats();

        // All should be treated equally
        assert_eq!(total, 3);
        assert_eq!(types.len(), 3);

        // Each type should have equal representation
        for (_, count) in &types {
            assert_eq!(*count, 1);
          
      
    }
    }

    #[test]
    fn test_capability_based_discovery() {
         
         
        let mut registry = UniversalPrimalRegistry: :new();

        registry.register_primal(UniversalPrimal::container_orchestration(
            "http://k8s:6443".to_string(),
        ));
        registry.register_primal(UniversalPrimal: :service_discovery(
            "http://service_discovery:8500".to_string(),
        ));

        // Both provide service discovery
        let discovery_primals = registry.discover_by_capability("service_discovery");
        assert_eq!(discovery_primals.len(), 2);

        // Only K8s provides container orchestration
        let container_primals = registry.discover_by_capability("container_orchestration");
        assert_eq!(container_primals.len(), 1);
     
     
    }
}
