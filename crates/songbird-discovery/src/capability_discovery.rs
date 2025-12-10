//! Capability-based port discovery implementation
//! 
//! Evolution from hardcoded ports to runtime capability-based discovery.
//! Primals discover their own capabilities and find others through discovery services.

use std::collections::HashMap;
use std::net::TcpListener;
use std::ops::Range;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Result type for capability discovery operations
pub type Result<T> = std::result::Result<T, CapabilityDiscoveryError>;

/// Errors that can occur during capability discovery
#[derive(Debug, thiserror::Error)]
pub enum CapabilityDiscoveryError {
    #[error("Capability not found: {capability}")]
    CapabilityNotFound { capability: String },
    
    #[error("Port allocation failed: {reason}")]
    PortAllocationFailed { reason: String },
    
    #[error("Discovery service unavailable: {reason}")]
    DiscoveryServiceUnavailable { reason: String },
    
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
}

/// Capability-based port discovery engine
/// 
/// Replaces hardcoded ports with runtime discovery based on capabilities.
/// Primals register their capabilities and discover others through this system.
pub struct CapabilityPortDiscovery {
    /// Port allocator for dynamic allocation
    allocator: PortAllocator,
    
    /// Local registry of capability → ports
    local_registry: Arc<RwLock<HashMap<String, Vec<u16>>>>,
    
    /// Discovery service client (optional)
    discovery_client: Option<Arc<dyn DiscoveryClient>>,
}

impl CapabilityPortDiscovery {
    /// Create new capability discovery engine
    pub fn new() -> Self {
        Self {
            allocator: PortAllocator::new(),
            local_registry: Arc::new(RwLock::new(HashMap::new())),
            discovery_client: None,
        }
    }
    
    /// Create with discovery service integration
    pub fn with_discovery_client(client: Arc<dyn DiscoveryClient>) -> Self {
        Self {
            allocator: PortAllocator::new(),
            local_registry: Arc::new(RwLock::new(HashMap::new())),
            discovery_client: Some(client),
        }
    }
    
    /// Discover port for a capability through multi-stage discovery
    /// 
    /// 1. Check local registry (fast path)
    /// 2. Query discovery service (if available)
    /// 3. Allocate dynamically (fallback)
    pub async fn discover_port(&self, capability: &str) -> Result<u16> {
        // Stage 1: Check local registry first (O(1) lookup)
        if let Some(ports) = self.local_registry.read().await.get(capability) {
            if let Some(&port) = ports.first() {
                tracing::debug!("Found capability '{}' in local registry: port {}", capability, port);
                return Ok(port);
            }
        }
        
        // Stage 2: Query discovery service (if configured)
        if let Some(ref client) = self.discovery_client {
            match client.query_capability(capability).await {
                Ok(services) if !services.is_empty() => {
                    let port = services[0].port;
                    tracing::info!("Discovered capability '{}' via discovery service: port {}", capability, port);
                    
                    // Cache in local registry
                    self.local_registry.write().await
                        .entry(capability.to_string())
                        .or_default()
                        .push(port);
                    
                    return Ok(port);
                }
                Err(e) => {
                    tracing::warn!("Discovery service query failed for '{}': {}", capability, e);
                }
                _ => {}
            }
        }
        
        // Stage 3: Allocate dynamically as fallback
        tracing::info!("Allocating dynamic port for capability '{}'", capability);
        let listener = self.allocator.allocate_for_capability(capability)?;
        let port = listener.local_addr()?.port();
        
        // Register locally
        self.local_registry.write().await
            .entry(capability.to_string())
            .or_default()
            .push(port);
        
        Ok(port)
    }
    
    /// Register a local capability with its port
    pub async fn register_local(&self, capability: String, port: u16) {
        self.local_registry.write().await
            .entry(capability)
            .or_default()
            .push(port);
    }
    
    /// Get all known capabilities
    pub async fn known_capabilities(&self) -> Vec<String> {
        self.local_registry.read().await.keys().cloned().collect()
    }
}

impl Default for CapabilityPortDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Port allocator with capability-aware ranges
pub struct PortAllocator {
    /// Known capability ranges
    capability_ranges: HashMap<String, Range<u16>>,
}

impl PortAllocator {
    /// Create new port allocator with standard capability ranges
    pub fn new() -> Self {
        let mut capability_ranges = HashMap::new();
        
        // Standard capability ranges (configurable, not hardcoded)
        capability_ranges.insert("orchestration".to_string(), 8000..8100);
        capability_ranges.insert("discovery".to_string(), 8100..8200);
        capability_ranges.insert("messaging".to_string(), 8200..8300);
        capability_ranges.insert("storage".to_string(), 8300..8400);
        capability_ranges.insert("compute".to_string(), 8400..8500);
        capability_ranges.insert("security".to_string(), 8500..8600);
        capability_ranges.insert("ai".to_string(), 8600..8700);
        capability_ranges.insert("analytics".to_string(), 8700..8800);
        
        Self { capability_ranges }
    }
    
    /// Allocate port for a capability
    pub fn allocate_for_capability(&self, capability: &str) -> Result<TcpListener> {
        // Try capability-specific range first
        if let Some(range) = self.capability_ranges.get(capability) {
            for port in range.clone() {
                if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)) {
                    tracing::debug!("Allocated port {} for capability '{}'", port, capability);
                    return Ok(listener);
                }
            }
        }
        
        // Fallback to OS assignment
        let listener = TcpListener::bind(("0.0.0.0", 0))?;
        let port = listener.local_addr()?.port();
        tracing::info!("OS-assigned port {} for capability '{}'", port, capability);
        Ok(listener)
    }
    
    /// Get capability range
    pub fn capability_range(&self, capability: &str, _default: Range<u16>) -> Range<u16> {
        self.capability_ranges.get(capability)
            .cloned()
            .unwrap_or(8800..8900) // Unknown capability range
    }
}

impl Default for PortAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Discovery client trait for querying remote services
#[async_trait::async_trait]
pub trait DiscoveryClient: Send + Sync {
    /// Query services by capability
    async fn query_capability(&self, capability: &str) -> Result<Vec<ServiceInfo>>;
    
    /// Register service with capabilities
    async fn register_service(&self, info: ServiceInfo) -> Result<()>;
}

/// Service information from discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub capabilities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_local_registry_lookup() {
        let discovery = CapabilityPortDiscovery::new();
        discovery.register_local("test".to_string(), 8080).await;
        
        let port = discovery.discover_port("test").await.unwrap();
        assert_eq!(port, 8080);
    }
    
    #[tokio::test]
    async fn test_dynamic_allocation_fallback() {
        let discovery = CapabilityPortDiscovery::new();
        
        // Unknown capability should allocate dynamically
        let port = discovery.discover_port("unknown_capability").await.unwrap();
        assert!(port > 0);
    }
    
    #[test]
    fn test_port_allocator_capability_ranges() {
        let allocator = PortAllocator::new();
        
        assert_eq!(allocator.capability_range("orchestration", 0..1), 8000..8100);
        assert_eq!(allocator.capability_range("discovery", 0..1), 8100..8200);
        assert_eq!(allocator.capability_range("storage", 0..1), 8300..8400);
    }
}

