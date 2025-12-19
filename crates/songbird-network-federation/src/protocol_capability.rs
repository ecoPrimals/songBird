//! Protocol Capability Advertisement System
//!
//! Allows Songbird towers to advertise their supported protocols and
//! discover peer capabilities for intelligent protocol selection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Protocol that a tower supports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// HTTP (unencrypted)
    Http,
    /// HTTPS with TLS
    Https,
    /// JSON-RPC 2.0 over HTTP/HTTPS
    JsonRpc,
    /// High-performance binary RPC
    Tarpc,
    /// BearDog Secure Tunnel Protocol
    Btsp,
    /// WebSocket
    WebSocket,
    /// WebSocket with TLS
    WebSocketSecure,
}

impl Protocol {
    /// Get protocol name
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::Http => "HTTP",
            Protocol::Https => "HTTPS",
            Protocol::JsonRpc => "JSON-RPC",
            Protocol::Tarpc => "tarpc",
            Protocol::Btsp => "BTSP",
            Protocol::WebSocket => "WebSocket",
            Protocol::WebSocketSecure => "WebSocket/TLS",
        }
    }

    /// Get protocol performance tier (higher is better)
    pub fn performance_tier(&self) -> u8 {
        match self {
            Protocol::Tarpc => 5, // Highest performance
            Protocol::Btsp => 4,  // High performance + security
            Protocol::WebSocketSecure => 3,
            Protocol::JsonRpc => 2,
            Protocol::Https => 2,
            Protocol::WebSocket => 1,
            Protocol::Http => 1, // Lowest (unencrypted)
        }
    }

    /// Check if protocol is encrypted
    pub fn is_encrypted(&self) -> bool {
        matches!(
            self,
            Protocol::Https | Protocol::Btsp | Protocol::WebSocketSecure | Protocol::Tarpc // tarpc can use TLS
        )
    }

    /// Check if protocol is recommended for production
    pub fn is_production_ready(&self) -> bool {
        !matches!(self, Protocol::Http | Protocol::WebSocket)
    }
}

/// Protocol capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapability {
    /// Protocol type
    pub protocol: Protocol,

    /// Port the protocol listens on
    pub port: u16,

    /// Optional path (e.g., "/jsonrpc")
    pub path: Option<String>,

    /// Status (active, planned, deprecated)
    pub status: ProtocolStatus,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Protocol status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolStatus {
    /// Protocol is active and ready
    Active,

    /// Protocol is planned but not yet implemented
    Planned,

    /// Protocol is deprecated, will be removed
    Deprecated,

    /// Protocol is under development
    Development,
}

/// Tower protocol capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerCapabilities {
    /// Tower ID
    pub tower_id: String,

    /// Tower endpoint (base URL)
    pub endpoint: String,

    /// Supported protocols
    pub protocols: Vec<ProtocolCapability>,

    /// Tower version
    pub version: String,

    /// Additional capabilities
    pub features: Vec<String>,
}

impl TowerCapabilities {
    /// Create new tower capabilities
    pub fn new(tower_id: String, endpoint: String) -> Self {
        Self {
            tower_id,
            endpoint,
            protocols: Vec::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            features: Vec::new(),
        }
    }

    /// Add a protocol capability
    pub fn add_protocol(&mut self, capability: ProtocolCapability) {
        self.protocols.push(capability);
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: String) {
        self.features.push(feature);
    }

    /// Get best protocol for communication
    pub fn best_protocol(&self) -> Option<&ProtocolCapability> {
        self.protocols
            .iter()
            .filter(|p| p.status == ProtocolStatus::Active)
            .max_by_key(|p| p.protocol.performance_tier())
    }

    /// Get best encrypted protocol
    pub fn best_encrypted_protocol(&self) -> Option<&ProtocolCapability> {
        self.protocols
            .iter()
            .filter(|p| p.status == ProtocolStatus::Active && p.protocol.is_encrypted())
            .max_by_key(|p| p.protocol.performance_tier())
    }

    /// Check if protocol is supported
    pub fn supports_protocol(&self, protocol: &Protocol) -> bool {
        self.protocols.iter().any(|p| &p.protocol == protocol && p.status == ProtocolStatus::Active)
    }
}

/// Protocol capability manager
pub struct ProtocolCapabilityManager {
    /// Local tower capabilities
    local_capabilities: Arc<RwLock<TowerCapabilities>>,

    /// Known peer capabilities
    peer_capabilities: Arc<RwLock<HashMap<String, TowerCapabilities>>>,
}

impl ProtocolCapabilityManager {
    /// Create new manager
    pub fn new(tower_id: String, endpoint: String) -> Self {
        Self {
            local_capabilities: Arc::new(RwLock::new(TowerCapabilities::new(tower_id, endpoint))),
            peer_capabilities: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a local protocol capability
    pub async fn register_protocol(&self, capability: ProtocolCapability) {
        let mut caps = self.local_capabilities.write().await;
        caps.add_protocol(capability);
    }

    /// Register a feature
    pub async fn register_feature(&self, feature: String) {
        let mut caps = self.local_capabilities.write().await;
        caps.add_feature(feature);
    }

    /// Get local capabilities
    pub async fn get_local_capabilities(&self) -> TowerCapabilities {
        self.local_capabilities.read().await.clone()
    }

    /// Store peer capabilities
    pub async fn store_peer_capabilities(&self, capabilities: TowerCapabilities) {
        let mut peers = self.peer_capabilities.write().await;
        peers.insert(capabilities.tower_id.clone(), capabilities);
    }

    /// Get peer capabilities
    pub async fn get_peer_capabilities(&self, tower_id: &str) -> Option<TowerCapabilities> {
        let peers = self.peer_capabilities.read().await;
        peers.get(tower_id).cloned()
    }

    /// Find best mutual protocol with peer
    pub async fn negotiate_protocol(&self, peer_id: &str) -> Option<Protocol> {
        let local = self.local_capabilities.read().await;
        let peers = self.peer_capabilities.read().await;

        let peer = peers.get(peer_id)?;

        // Find protocols supported by both
        let mutual_protocols: Vec<_> = local
            .protocols
            .iter()
            .filter(|lp| {
                lp.status == ProtocolStatus::Active
                    && peer
                        .protocols
                        .iter()
                        .any(|pp| pp.protocol == lp.protocol && pp.status == ProtocolStatus::Active)
            })
            .collect();

        // Select best mutual protocol
        mutual_protocols
            .iter()
            .max_by_key(|p| p.protocol.performance_tier())
            .map(|p| p.protocol.clone())
    }

    /// Get all active peers
    pub async fn get_active_peers(&self) -> Vec<String> {
        let peers = self.peer_capabilities.read().await;
        peers.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_performance_tiers() {
        assert!(Protocol::Tarpc.performance_tier() > Protocol::Https.performance_tier());
        assert!(Protocol::Btsp.performance_tier() > Protocol::JsonRpc.performance_tier());
    }

    #[test]
    fn test_protocol_encryption() {
        assert!(Protocol::Https.is_encrypted());
        assert!(Protocol::Btsp.is_encrypted());
        assert!(!Protocol::Http.is_encrypted());
    }

    #[test]
    fn test_tower_capabilities() {
        let mut caps =
            TowerCapabilities::new("tower-1".to_string(), "http://localhost:8080".to_string());

        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Http,
            port: 8080,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });

        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Https,
            port: 8443,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });

        let best = caps.best_protocol().unwrap();
        assert_eq!(best.protocol, Protocol::Https);
    }

    #[tokio::test]
    async fn test_capability_manager() {
        let manager = ProtocolCapabilityManager::new(
            "tower-1".to_string(),
            "http://localhost:8080".to_string(),
        );

        manager
            .register_protocol(ProtocolCapability {
                protocol: Protocol::Https,
                port: 8443,
                path: None,
                status: ProtocolStatus::Active,
                metadata: HashMap::new(),
            })
            .await;

        let caps = manager.get_local_capabilities().await;
        assert_eq!(caps.protocols.len(), 1);
        assert!(caps.supports_protocol(&Protocol::Https));
    }

    #[tokio::test]
    async fn test_protocol_negotiation() {
        let manager = ProtocolCapabilityManager::new(
            "tower-1".to_string(),
            "http://localhost:8080".to_string(),
        );

        // Register local protocols
        manager
            .register_protocol(ProtocolCapability {
                protocol: Protocol::Http,
                port: 8080,
                path: None,
                status: ProtocolStatus::Active,
                metadata: HashMap::new(),
            })
            .await;

        manager
            .register_protocol(ProtocolCapability {
                protocol: Protocol::Tarpc,
                port: 8081,
                path: None,
                status: ProtocolStatus::Active,
                metadata: HashMap::new(),
            })
            .await;

        // Store peer capabilities
        let mut peer_caps =
            TowerCapabilities::new("tower-2".to_string(), "http://peer:8080".to_string());
        peer_caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Http,
            port: 8080,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        peer_caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Tarpc,
            port: 8081,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });

        manager.store_peer_capabilities(peer_caps).await;

        // Negotiate protocol
        let protocol = manager.negotiate_protocol("tower-2").await;
        assert_eq!(protocol, Some(Protocol::Tarpc)); // Should choose highest performance
    }
}
