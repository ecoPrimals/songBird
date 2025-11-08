//! Federated Service Registry
//!
//! Enables service discovery across multiple towers in a federation.
//! Services registered on one tower become discoverable on all connected towers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Federated service registry
#[derive(Debug, Clone)]
pub struct FederatedServiceRegistry {
    /// Local services (registered on this tower)
    local_services: Arc<RwLock<HashMap<String, ServiceRegistration>>>,
    
    /// Remote services (discovered from other towers)
    remote_services: Arc<RwLock<HashMap<String, ServiceRegistration>>>,
}

impl FederatedServiceRegistry {
    /// Create a new federated service registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            local_services: Arc::new(RwLock::new(HashMap::new())),
            remote_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a local service
    pub async fn register_local(&self, service: ServiceRegistration) {
        info!(
            "📝 Registering local service: {} ({})",
            service.service_name, service.service_type
        );
        
        let mut local = self.local_services.write().await;
        local.insert(service.service_id.clone(), service);
    }
    
    /// Deregister a local service
    pub async fn deregister_local(&self, service_id: &str) {
        info!("🗑️  Deregistering local service: {}", service_id);
        
        let mut local = self.local_services.write().await;
        local.remove(service_id);
    }
    
    /// Register a remote service (from another tower)
    pub async fn register_remote(&self, service: ServiceRegistration) {
        debug!(
            "📡 Registering remote service: {} from {}",
            service.service_name, service.tower_id
        );
        
        let mut remote = self.remote_services.write().await;
        remote.insert(service.service_id.clone(), service);
    }
    
    /// Update remote services from a tower
    pub async fn sync_remote_services(&self, tower_id: &str, services: Vec<ServiceRegistration>) {
        debug!("🔄 Syncing {} services from tower {}", services.len(), tower_id);
        
        let mut remote = self.remote_services.write().await;
        
        // Remove old services from this tower
        remote.retain(|_, svc| svc.tower_id != tower_id);
        
        // Add new services
        for service in services {
            remote.insert(service.service_id.clone(), service);
        }
    }
    
    /// Get all services (local + remote)
    pub async fn get_all_services(&self) -> Vec<ServiceRegistration> {
        let local = self.local_services.read().await;
        let remote = self.remote_services.read().await;
        
        local
            .values()
            .chain(remote.values())
            .cloned()
            .collect()
    }
    
    /// Get all local services
    pub async fn get_local_services(&self) -> Vec<ServiceRegistration> {
        let local = self.local_services.read().await;
        local.values().cloned().collect()
    }
    
    /// Get all remote services
    pub async fn get_remote_services(&self) -> Vec<ServiceRegistration> {
        let remote = self.remote_services.read().await;
        remote.values().cloned().collect()
    }
    
    /// Find services by type
    pub async fn find_by_type(&self, service_type: &str) -> Vec<ServiceRegistration> {
        let all_services = self.get_all_services().await;
        all_services
            .into_iter()
            .filter(|svc| svc.service_type == service_type)
            .collect()
    }
    
    /// Find services by capability
    pub async fn find_by_capability(&self, capability: &str) -> Vec<ServiceRegistration> {
        let all_services = self.get_all_services().await;
        all_services
            .into_iter()
            .filter(|svc| svc.capabilities.contains(&capability.to_string()))
            .collect()
    }
    
    /// Find service by ID
    pub async fn find_by_id(&self, service_id: &str) -> Option<ServiceRegistration> {
        // Check local first
        {
            let local = self.local_services.read().await;
            if let Some(service) = local.get(service_id) {
                return Some(service.clone());
            }
        }
        
        // Then check remote
        let remote = self.remote_services.read().await;
        remote.get(service_id).cloned()
    }
    
    /// Get service statistics
    pub async fn get_stats(&self) -> ServiceRegistryStats {
        let local = self.local_services.read().await;
        let remote = self.remote_services.read().await;
        
        ServiceRegistryStats {
            total_services: local.len() + remote.len(),
            local_services: local.len(),
            remote_services: remote.len(),
            service_types: {
                let mut types = std::collections::HashSet::new();
                for svc in local.values().chain(remote.values()) {
                    types.insert(svc.service_type.clone());
                }
                types.into_iter().collect()
            },
        }
    }
    
    /// Clean up stale services (not updated in timeout period)
    pub async fn cleanup_stale_services(&self, timeout_secs: i64) {
        let now = Utc::now();
        let mut remote = self.remote_services.write().await;
        
        let before_count = remote.len();
        remote.retain(|_, svc| {
            let elapsed = (now - svc.last_seen).num_seconds();
            elapsed < timeout_secs
        });
        
        let removed = before_count - remote.len();
        if removed > 0 {
            info!("🧹 Cleaned up {} stale remote services", removed);
        }
    }
}

impl Default for FederatedServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    /// Unique service identifier
    pub service_id: String,
    
    /// Human-readable service name
    pub service_name: String,
    
    /// Service type (e.g., "beardog", "squirrel", "biome")
    pub service_type: String,
    
    /// Tower this service is running on
    pub tower_id: String,
    
    /// Tower name
    pub tower_name: String,
    
    /// Service endpoint URL
    pub endpoint: String,
    
    /// Service capabilities
    pub capabilities: Vec<String>,
    
    /// Service metadata
    pub metadata: HashMap<String, String>,
    
    /// Health status
    pub health_status: ServiceHealthStatus,
    
    /// When service was registered
    pub registered_at: DateTime<Utc>,
    
    /// Last time service was seen/updated
    pub last_seen: DateTime<Utc>,
}

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceHealthStatus {
    /// Service is healthy and operational
    Healthy,
    
    /// Service is experiencing degraded performance
    Degraded,
    
    /// Service is unhealthy
    Unhealthy,
    
    /// Service status is unknown
    Unknown,
}

impl std::fmt::Display for ServiceHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Service registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistryStats {
    /// Total number of services (local + remote)
    pub total_services: usize,
    
    /// Number of local services
    pub local_services: usize,
    
    /// Number of remote services
    pub remote_services: usize,
    
    /// List of service types
    pub service_types: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_register_local_service() {
        let registry = FederatedServiceRegistry::new();
        
        let service = ServiceRegistration {
            service_id: "test-service-1".to_string(),
            service_name: "Test Service".to_string(),
            service_type: "test".to_string(),
            tower_id: "tower-1".to_string(),
            tower_name: "Tower 1".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test-capability".to_string()],
            metadata: HashMap::new(),
            health_status: ServiceHealthStatus::Healthy,
            registered_at: Utc::now(),
            last_seen: Utc::now(),
        };
        
        registry.register_local(service.clone()).await;
        
        let found = registry.find_by_id("test-service-1").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().service_name, "Test Service");
    }
    
    #[tokio::test]
    async fn test_find_by_type() {
        let registry = FederatedServiceRegistry::new();
        
        let service1 = ServiceRegistration {
            service_id: "service-1".to_string(),
            service_name: "Service 1".to_string(),
            service_type: "beardog".to_string(),
            tower_id: "tower-1".to_string(),
            tower_name: "Tower 1".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            metadata: HashMap::new(),
            health_status: ServiceHealthStatus::Healthy,
            registered_at: Utc::now(),
            last_seen: Utc::now(),
        };
        
        let service2 = ServiceRegistration {
            service_id: "service-2".to_string(),
            service_name: "Service 2".to_string(),
            service_type: "squirrel".to_string(),
            tower_id: "tower-1".to_string(),
            tower_name: "Tower 1".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec![],
            metadata: HashMap::new(),
            health_status: ServiceHealthStatus::Healthy,
            registered_at: Utc::now(),
            last_seen: Utc::now(),
        };
        
        registry.register_local(service1).await;
        registry.register_local(service2).await;
        
        let beardog_services = registry.find_by_type("beardog").await;
        assert_eq!(beardog_services.len(), 1);
        assert_eq!(beardog_services[0].service_name, "Service 1");
    }
}

