//! Primal Bridge - abstraction for connecting to any primal
//!
//! **ZERO HARDCODING**: No primal names, only capability discovery

use crate::{error::Result, types::*};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Connection to a specific primal
///
/// This represents an active connection managed by Songbird's networking layer.
/// It's agnostic to which primal it's connected to - only knows capabilities.
#[derive(Debug, Clone)]
pub struct PrimalConnection {
    /// Connection identifier
    pub connection_id: String,
    
    /// Endpoint (discovered, not hardcoded)
    pub endpoint: String,
    
    /// Capabilities advertised by this primal
    pub capabilities: Arc<RwLock<PrimalCapabilities>>,
    
    /// Connection metadata
    pub metadata: Arc<RwLock<std::collections::HashMap<String, serde_json::Value>>>,
}

impl PrimalConnection {
    /// Create a new primal connection
    #[must_use]
    pub fn new(connection_id: String, endpoint: String, capabilities: PrimalCapabilities) -> Self {
        Self {
            connection_id,
            endpoint,
            capabilities: Arc::new(RwLock::new(capabilities)),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Send a request to the primal
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or the response is invalid
    pub async fn send_request(&self, request: PrimalRequest) -> Result<PrimalResponse> {
        tracing::debug!(
            "Sending request to primal at {} (connection: {})",
            self.endpoint,
            self.connection_id
        );
        
        // TODO: Implement actual network communication
        // For now, this is a placeholder that will be implemented with real P2P networking
        tracing::warn!("PrimalConnection::send_request is not yet fully implemented");
        
        // Placeholder response based on request type
        match request {
            PrimalRequest::DiscoverCapabilities => {
                let caps = self.capabilities.read().await.clone();
                Ok(PrimalResponse::Capabilities(caps))
            }
            PrimalRequest::Status => {
                Ok(PrimalResponse::StatusResponse(ServiceStatus {
                    healthy: true,
                    version: "0.1.0".to_string(),
                    capabilities: self.capabilities.read().await.services.clone(),
                    metrics: std::collections::HashMap::new(),
                }))
            }
            _ => Err(crate::error::PrimalCoordinationError::Internal(
                "Request type not yet implemented".to_string(),
            )),
        }
    }

    /// Check if this connection supports a capability
    pub async fn supports_capability(&self, capability: &CapabilityType) -> bool {
        self.capabilities.read().await.supports_capability(capability)
    }

    /// Get current capabilities
    pub async fn get_capabilities(&self) -> PrimalCapabilities {
        self.capabilities.read().await.clone()
    }

    /// Update capabilities (from periodic discovery)
    pub async fn update_capabilities(&self, new_capabilities: PrimalCapabilities) {
        *self.capabilities.write().await = new_capabilities;
    }
}

/// Primal Bridge trait - defines how to connect to and interact with a primal
///
/// **KEY PRINCIPLE**: This trait is capability-based, not primal-name-based.
/// Implementations discover and connect to primals by capability, not by hardcoded names.
#[async_trait]
pub trait PrimalBridge: Send + Sync {
    /// Connect to a primal that provides the requested capability
    ///
    /// # Errors
    ///
    /// Returns an error if no primal with the capability can be found or connection fails
    async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection>;

    /// Discover capabilities offered by primals in the environment
    ///
    /// # Errors
    ///
    /// Returns an error if discovery fails
    async fn discover_capabilities(&self, connection: &PrimalConnection) -> Result<PrimalCapabilities>;

    /// Get the capabilities this bridge can connect to
    ///
    /// Used for routing and capability matching
    fn supported_capabilities(&self) -> Vec<CapabilityType>;
}

/// Discovery-based primal bridge
///
/// Uses `songbird-discovery` to find primals by capability at runtime.
/// **ZERO HARDCODING** - discovers everything from environment/network.
pub struct DiscoveryBasedBridge {
    /// Discovery engine (from songbird-discovery)
    discovery: Arc<dyn PrimalDiscovery>,
}

impl DiscoveryBasedBridge {
    /// Create a new discovery-based bridge
    #[must_use]
    pub fn new(discovery: Arc<dyn PrimalDiscovery>) -> Self {
        Self { discovery }
    }
}

#[async_trait]
impl PrimalBridge for DiscoveryBasedBridge {
    async fn connect(&self, capability: CapabilityType) -> Result<PrimalConnection> {
        tracing::info!("Discovering primal for capability: {}", capability);
        
        // Use discovery engine to find a primal
        let discovered = self.discovery.discover_by_capability(&capability).await?;
        
        tracing::info!(
            "Found primal at {} for capability {}",
            discovered.endpoint,
            capability
        );
        
        // Create connection
        Ok(PrimalConnection::new(
            uuid::Uuid::new_v4().to_string(),
            discovered.endpoint,
            discovered.capabilities,
        ))
    }

    async fn discover_capabilities(&self, connection: &PrimalConnection) -> Result<PrimalCapabilities> {
        // Query the primal for its capabilities
        let response = connection.send_request(PrimalRequest::DiscoverCapabilities).await?;
        
        match response {
            PrimalResponse::Capabilities(caps) => Ok(caps),
            PrimalResponse::Error(e) => Err(crate::error::PrimalCoordinationError::PrimalError(e)),
            _ => Err(crate::error::PrimalCoordinationError::UnexpectedResponse(
                "Expected Capabilities response".to_string(),
            )),
        }
    }

    fn supported_capabilities(&self) -> Vec<CapabilityType> {
        // Discovery-based bridge supports all capabilities
        vec![
            CapabilityType::Security,
            CapabilityType::Compute,
            CapabilityType::Storage,
            CapabilityType::Ai,
            CapabilityType::Discovery,
            CapabilityType::Orchestration,
            CapabilityType::Networking,
        ]
    }
}

/// Trait for primal discovery (implemented by songbird-discovery)
#[async_trait]
pub trait PrimalDiscovery: Send + Sync {
    /// Discover a primal by capability
    ///
    /// # Errors
    ///
    /// Returns an error if no primal with the capability is found
    async fn discover_by_capability(&self, capability: &CapabilityType) -> Result<DiscoveredPrimal>;
}

/// Discovered primal information
#[derive(Debug, Clone)]
pub struct DiscoveredPrimal {
    pub endpoint: String,
    pub capabilities: PrimalCapabilities,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_primal_connection_creation() {
        let caps = PrimalCapabilities {
            services: vec!["security".to_string()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        
        let conn = PrimalConnection::new(
            "test-conn-1".to_string(),
            "http://localhost:8080".to_string(),
            caps,
        );
        
        assert_eq!(conn.connection_id, "test-conn-1");
        assert_eq!(conn.endpoint, "http://localhost:8080");
        assert!(conn.supports_capability(&CapabilityType::Security).await);
        assert!(!conn.supports_capability(&CapabilityType::Compute).await);
    }

    #[tokio::test]
    async fn test_capability_update() {
        let initial_caps = PrimalCapabilities {
            services: vec!["security".to_string()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        
        let conn = PrimalConnection::new(
            "test-conn-2".to_string(),
            "http://localhost:8080".to_string(),
            initial_caps,
        );
        
        // Initially only supports security
        assert!(conn.supports_capability(&CapabilityType::Security).await);
        assert!(!conn.supports_capability(&CapabilityType::Compute).await);
        
        // Update capabilities
        let new_caps = PrimalCapabilities {
            services: vec!["security".to_string(), "compute".to_string()],
            resources: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            quality: ServiceQuality::default(),
        };
        conn.update_capabilities(new_caps).await;
        
        // Now supports both
        assert!(conn.supports_capability(&CapabilityType::Security).await);
        assert!(conn.supports_capability(&CapabilityType::Compute).await);
    }
}

