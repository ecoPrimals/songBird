//! BTSP Provider Trait and Configuration
//! 
//! Defines the interface that all BTSP implementations must provide.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::btsp::tunnel::{Tunnel, TunnelHandle, TunnelStatus, SecurityContext};
use songbird_types::{SongbirdError, SongbirdResult};

/// Configuration for BTSP provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspConfig {
    /// Enable BTSP encryption
    pub enabled: bool,
    
    /// BearDog discovery method (capability-based, not hardcoded)
    pub discovery_method: DiscoveryMethod,
    
    /// Capability to discover BearDog service
    pub security_capability: String,
    
    /// Fallback to local implementation if BearDog unavailable
    pub local_fallback: bool,
    
    /// Genetic auth enabled (requires BearDog)
    pub genetic_auth: bool,
    
    /// Key lineage tracking
    pub key_lineage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Discover via capability system (sovereign)
    Capability,
    /// Discover via mDNS on LAN
    Mdns,
    /// Discover via registry
    Registry,
    /// Environment variable (for explicit config)
    Environment,
}

impl Default for BtspConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, enable via env
            discovery_method: DiscoveryMethod::Capability,
            security_capability: "enterprise-security".to_string(),
            local_fallback: true, // Graceful degradation
            genetic_auth: false, // Requires BearDog
            key_lineage: false, // Requires BearDog
        }
    }
}

/// Peer information for BTSP tunnel establishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer tower ID
    pub id: String,
    
    /// Peer endpoint
    pub endpoint: String,
    
    /// Peer public key (if available)
    pub public_key: Option<Vec<u8>>,
    
    /// Supported protocols
    pub protocols: Vec<String>,
}

/// BTSP Provider trait
/// 
/// This trait defines the interface for all BTSP implementations.
/// Implementations can be:
/// - Local (for testing)
/// - BearDog (real genetic crypto)
/// - Mock (for unit tests)
#[async_trait]
pub trait BtspProvider: Send + Sync {
    /// Establish a secure tunnel with peer
    async fn establish_tunnel(&self, peer: &PeerInfo) -> SongbirdResult<TunnelHandle>;
    
    /// Encrypt data for transmission through tunnel
    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>>;
    
    /// Decrypt data received through tunnel
    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> SongbirdResult<Vec<u8>>;
    
    /// Get tunnel status
    async fn tunnel_status(&self, handle: &TunnelHandle) -> SongbirdResult<TunnelStatus>;
    
    /// Close tunnel
    async fn close_tunnel(&self, handle: &TunnelHandle) -> SongbirdResult<()>;
    
    /// Get provider name (for logging/debugging)
    fn provider_name(&self) -> &str;
    
    /// Check if provider supports genetic auth
    fn supports_genetic_auth(&self) -> bool;
    
    /// Check if provider supports key lineage
    fn supports_key_lineage(&self) -> bool;
}

/// Factory for creating BTSP providers based on runtime discovery
pub struct BtspProviderFactory {
    config: BtspConfig,
}

impl BtspProviderFactory {
    /// Create a new factory with configuration
    pub fn new(config: BtspConfig) -> Self {
        Self { config }
    }
    
    /// Create BTSP provider based on runtime discovery
    /// 
    /// This method discovers BearDog via capability system at runtime.
    /// If BearDog is not available and local_fallback is enabled, returns
    /// local implementation.
    pub async fn create_provider(&self) -> SongbirdResult<Arc<dyn BtspProvider>> {
        if !self.config.enabled {
            return Err(SongbirdError::configuration("BTSP is not enabled"));
        }
        
        // Try to discover BearDog via capability system
        match self.discover_beardog().await {
            Ok(provider) => {
                tracing::info!("✅ BearDog BTSP provider discovered and connected");
                Ok(provider)
            }
            Err(e) => {
                if self.config.local_fallback {
                    tracing::warn!(
                        "⚠️ BearDog not available ({}), falling back to local BTSP implementation",
                        e
                    );
                    Ok(Arc::new(crate::btsp::local::LocalBtspProvider::new()))
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Discover BearDog service via capability system
    async fn discover_beardog(&self) -> SongbirdResult<Arc<dyn BtspProvider>> {
        // TODO: Implement actual capability-based discovery
        // For now, return error to trigger fallback
        Err(SongbirdError::service(
            "beardog",
            "BearDog service not yet discovered (capability discovery not wired)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_btsp_config_default() {
        let config = BtspConfig::default();
        assert!(!config.enabled);
        assert!(config.local_fallback);
        assert!(!config.genetic_auth);
    }

    #[tokio::test]
    async fn test_factory_creates_local_fallback() {
        let config = BtspConfig {
            enabled: true,
            local_fallback: true,
            ..Default::default()
        };
        
        let factory = BtspProviderFactory::new(config);
        let provider = factory.create_provider().await.unwrap();
        
        assert_eq!(provider.provider_name(), "Local");
        assert!(!provider.supports_genetic_auth());
    }
}

