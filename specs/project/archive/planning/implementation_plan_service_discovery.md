# Service Discovery Implementation Plan

## Overview

This document outlines the implementation plan for the **Songbird Discovery Service** - a custom Rust-native service discovery system optimized for scientific computing federation.

## Architecture

### Core Components

```rust
pub struct SongbirdDiscovery {
    local_node: LocalNode,
    known_nodes: Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
    federation_manager: FederationManager,
    trust_verifier: TrustVerifier,
    resource_tracker: ResourceTracker,
    dataset_locator: DatasetLocator,
    algorithm_registry: AlgorithmRegistry,
    network_topology: NetworkTopology,
    placement_optimizer: PlacementOptimizer,
    orchestrator_client: OrchestratorClient,
    attribution_system: AttributionSystem,
}
```

### Discovery Factory

```rust
pub fn create_discovery_service(config: &DiscoveryConfig) -> Result<Box<dyn ServiceDiscovery>, DiscoveryError> {
    match &config.backend {
        DiscoveryBackend::Songbird { 
            federation_enabled, 
            trust_verification, 
            attribution_tracking 
        } => {
            Ok(Box::new(SongbirdDiscovery::new(
                *federation_enabled,
                *trust_verification, 
                *attribution_tracking
            )?))
        },
        DiscoveryBackend::Static => {
            Ok(Box::new(StaticServiceDiscovery::new(config)?))
        },
        DiscoveryBackend::Etcd { endpoints, username, password } => {
            Ok(Box::new(EtcdServiceDiscovery::new(
                endpoints.clone(),
                username.clone(),
                password.clone()
            )?))
        },
        DiscoveryBackend::Kubernetes { namespace, in_cluster, kubeconfig_path } => {
            Ok(Box::new(KubernetesServiceDiscovery::new(
                namespace.clone(),
                *in_cluster,
                kubeconfig_path.clone()
            )?))
        },
    }
}
```

## Implementation Details

### Songbird Discovery Service

**File:** `src/discovery/songbird.rs`

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::traits::discovery::{ServiceDiscovery, ServiceInfo, ServiceQuery, ServiceEvent, HealthStatus, DiscoveryError};

pub struct SongbirdDiscovery {
    nodes: Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
    services: Arc<RwLock<HashMap<ServiceId, ServiceInfo>>>,
    federation_enabled: bool,
    trust_verification: bool,
    attribution_tracking: bool,
    event_bus: EventBus,
}

impl SongbirdDiscovery {
    pub fn new(
        federation_enabled: bool,
        trust_verification: bool,
        attribution_tracking: bool,
    ) -> Result<Self, DiscoveryError> {
        Ok(Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            services: Arc::new(RwLock::new(HashMap::new())),
            federation_enabled,
            trust_verification,
            attribution_tracking,
            event_bus: EventBus::new(),
        })
    }

    // Scientific computing specific methods
    pub async fn find_optimal_nodes(&self, query: ResourceQuery) -> Result<Vec<NodeInfo>, DiscoveryError> {
        // Implementation for resource-aware node selection
        todo!("Implement resource-aware discovery")
    }

    pub async fn register_dataset(&self, dataset: DatasetInfo) -> Result<(), DiscoveryError> {
        // Implementation for dataset registration
        todo!("Implement dataset registration")
    }

    pub async fn find_datasets(&self, query: DatasetQuery) -> Result<Vec<DatasetInfo>, DiscoveryError> {
        // Implementation for dataset discovery
        todo!("Implement dataset discovery")
    }
}

#[async_trait]
impl ServiceDiscovery for SongbirdDiscovery {
    async fn register(&self, service: ServiceInfo) -> Result<(), DiscoveryError> {
        let mut services = self.services.write().await;
        services.insert(service.id.clone(), service.clone());
        
        // Emit registration event
        self.event_bus.emit(ServiceEvent::Registered(service)).await;
        
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<(), DiscoveryError> {
        let mut services = self.services.write().await;
        if let Some(service) = services.remove(service_id) {
            // Emit unregistration event
            self.event_bus.emit(ServiceEvent::Unregistered(service)).await;
        }

        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>, DiscoveryError> {
        let services = self.services.read().await;
        let mut results = Vec::new();
        
        for service in services.values() {
            if self.matches_query(service, &query) {
                results.push(service.clone());
            }
        }
        
        Ok(results)
    }

    async fn watch(&self, _query: ServiceQuery) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>, DiscoveryError> {
        // Return event stream
        Ok(self.event_bus.subscribe())
    }

    async fn update_health(&self, service_id: &str, health_status: HealthStatus) -> Result<(), DiscoveryError> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.health_status = health_status;
            
            // Emit health update event
            self.event_bus.emit(ServiceEvent::HealthUpdated {
                service_id: service_id.to_string(),
                health_status,
            }).await;
        }

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>, DiscoveryError> {
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }

    async fn exists(&self, service_id: &str) -> Result<bool, DiscoveryError> {
        let services = self.services.read().await;
        Ok(services.contains_key(service_id))
    }

    async fn update_metadata(&self, service_id: &str, metadata: HashMap<String, serde_json::Value>) -> Result<(), DiscoveryError> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.metadata = metadata;
            
            // Emit metadata update event
            self.event_bus.emit(ServiceEvent::MetadataUpdated {
                service_id: service_id.to_string(),
            }).await;
        }

        Ok(())
    }
}

impl SongbirdDiscovery {
    fn matches_query(&self, service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Service name matching
        if let Some(name) = &query.name {
            if service.name != *name {
                return false;
            }
        }

        // Tag matching
        if !query.tags.is_empty() {
            let service_tags: HashSet<_> = service.tags.iter().collect();
            let query_tags: HashSet<_> = query.tags.iter().collect();
            if !query_tags.is_subset(&service_tags) {
                return false;
            }
        }

        // Health status matching
        if let Some(health) = &query.health_status {
            if service.health_status != *health {
                return false;
            }
        }

        true
    }
}
```

## Configuration

### Songbird Configuration

```toml
[discovery]
backend = "songbird"

[discovery.songbird]
federation_enabled = true
trust_verification = true
attribution_tracking = true
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_songbird_discovery_registration() {
        let discovery = SongbirdDiscovery::new(false, false, false).unwrap();
        
        let service = ServiceInfo {
            id: "test-service".to_string(),
            name: "test".to_string(),
            // ... other fields
        };
        
        discovery.register(service.clone()).await.unwrap();
        
        let services = discovery.list_all().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "test-service");
    }
    
    #[tokio::test]
    async fn test_resource_aware_discovery() {
        let discovery = SongbirdDiscovery::new(true, false, false).unwrap();
        
        let query = ResourceQuery {
            min_cpu_cores: Some(4),
            min_memory_gb: Some(8),
            required_gpu_types: vec![GpuType::V100],
            // ... other fields
        };
        
        let nodes = discovery.find_optimal_nodes(query).await.unwrap();
        // Assert nodes meet requirements
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_songbird_discovery_integration() {
    let config = DiscoveryConfig {
        backend: DiscoveryBackend::Songbird {
            federation_enabled: true,
            trust_verification: false,
            attribution_tracking: false,
        },
        // ... other config
    };
    
    let discovery = create_discovery_service(&config).unwrap();
    
    // Test full workflow
    let service = create_test_service();
    discovery.register(service.clone()).await.unwrap();
    
    let query = ServiceQuery {
        name: Some(service.name.clone()),
        ..Default::default()
    };

    let results = discovery.discover(query).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, service.id);
}
```

## Migration Path

1. **Remove Consul dependencies** from Cargo.toml ✅
2. **Update DiscoveryBackend enum** to include Songbird ✅
3. **Implement SongbirdDiscovery service**
4. **Update configuration examples**
5. **Update documentation**
6. **Run comprehensive tests**

## Benefits

- **Zero external dependencies** - No Consul installation required
- **Scientific computing optimized** - Resource-aware discovery
- **Federation ready** - Multi-institution support
- **Attribution integrated** - Provenance tracking
- **High performance** - Rust-native implementation

## Next Steps

1. Implement `SongbirdDiscovery` struct and basic methods
2. Add scientific computing extensions (resource discovery, dataset location)
3. Implement federation support (multi-institution discovery)
4. Add attribution system integration
5. Comprehensive testing and documentation 