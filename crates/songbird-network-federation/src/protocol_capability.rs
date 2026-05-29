// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    /// `security provider` Secure Tunnel Protocol
    Btsp,
    /// WebSocket
    WebSocket,
    /// WebSocket with TLS
    WebSocketSecure,
}

impl Protocol {
    /// Get protocol name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Http => "HTTP",
            Self::Https => "HTTPS",
            Self::JsonRpc => "JSON-RPC",
            Self::Tarpc => "tarpc",
            Self::Btsp => "BTSP",
            Self::WebSocket => "WebSocket",
            Self::WebSocketSecure => "WebSocket/TLS",
        }
    }

    /// Get protocol performance tier (higher is better)
    #[must_use]
    pub const fn performance_tier(&self) -> u8 {
        match self {
            Self::Tarpc => 5, // Highest performance
            Self::Btsp => 4,  // High performance + security
            Self::WebSocketSecure => 3,
            Self::JsonRpc => 2,
            Self::Https => 2,
            Self::WebSocket => 1,
            Self::Http => 1, // Lowest (unencrypted)
        }
    }

    /// Check if protocol is encrypted
    #[must_use]
    pub const fn is_encrypted(&self) -> bool {
        matches!(
            self,
            Self::Https | Self::Btsp | Self::WebSocketSecure | Self::Tarpc // tarpc can use TLS
        )
    }

    /// Check if protocol is recommended for production
    #[must_use]
    pub const fn is_production_ready(&self) -> bool {
        !matches!(self, Self::Http | Self::WebSocket)
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
    #[must_use]
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
    #[must_use]
    pub fn best_protocol(&self) -> Option<&ProtocolCapability> {
        self.protocols
            .iter()
            .filter(|p| p.status == ProtocolStatus::Active)
            .max_by_key(|p| p.protocol.performance_tier())
    }

    /// Get best encrypted protocol
    #[must_use]
    pub fn best_encrypted_protocol(&self) -> Option<&ProtocolCapability> {
        self.protocols
            .iter()
            .filter(|p| p.status == ProtocolStatus::Active && p.protocol.is_encrypted())
            .max_by_key(|p| p.protocol.performance_tier())
    }

    /// Check if protocol is supported
    #[must_use]
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
    #[must_use]
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
        let peer = self.peer_capabilities.read().await.get(peer_id).cloned()?;
        let local_protocols = self.local_capabilities.read().await.protocols.clone();

        // Find protocols supported by both
        let mutual_protocols: Vec<_> = local_protocols
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

    #[test]
    fn best_encrypted_protocol_returns_highest_tier_encrypted() {
        let mut caps = TowerCapabilities::new("t".into(), "e".into());
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Http,
            port: 80,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Https,
            port: 443,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Btsp,
            port: 9090,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        let best = caps.best_encrypted_protocol().unwrap();
        assert_eq!(best.protocol, Protocol::Btsp);
    }

    #[test]
    fn best_encrypted_protocol_none_when_only_unencrypted() {
        let mut caps = TowerCapabilities::new("t".into(), "e".into());
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Http,
            port: 80,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::JsonRpc,
            port: 3000,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        assert!(caps.best_encrypted_protocol().is_none());
    }

    #[test]
    fn best_encrypted_protocol_ignores_inactive() {
        let mut caps = TowerCapabilities::new("t".into(), "e".into());
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Btsp,
            port: 9090,
            path: None,
            status: ProtocolStatus::Deprecated,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Https,
            port: 443,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        let best = caps.best_encrypted_protocol().unwrap();
        assert_eq!(best.protocol, Protocol::Https);
    }

    #[test]
    fn supports_protocol_checks_active_status() {
        let mut caps = TowerCapabilities::new("t".into(), "e".into());
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Tarpc,
            port: 8081,
            path: None,
            status: ProtocolStatus::Planned,
            metadata: HashMap::new(),
        });
        assert!(!caps.supports_protocol(&Protocol::Tarpc));

        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Https,
            port: 443,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        assert!(caps.supports_protocol(&Protocol::Https));
        assert!(!caps.supports_protocol(&Protocol::Http));
    }

    #[test]
    fn best_protocol_prefers_highest_performance_tier() {
        let mut caps = TowerCapabilities::new("t".into(), "e".into());
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::JsonRpc,
            port: 3000,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::WebSocketSecure,
            port: 8443,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Tarpc,
            port: 8081,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        let best = caps.best_protocol().unwrap();
        assert_eq!(best.protocol, Protocol::Tarpc);
    }

    #[test]
    fn best_protocol_ignores_inactive() {
        let mut caps = TowerCapabilities::new("t".into(), "e".into());
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Tarpc,
            port: 8081,
            path: None,
            status: ProtocolStatus::Deprecated,
            metadata: HashMap::new(),
        });
        caps.add_protocol(ProtocolCapability {
            protocol: Protocol::Http,
            port: 80,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        let best = caps.best_protocol().unwrap();
        assert_eq!(best.protocol, Protocol::Http);
    }

    #[tokio::test]
    async fn negotiate_protocol_no_mutual_returns_none() {
        let manager =
            ProtocolCapabilityManager::new("tower-1".into(), "http://localhost:8080".into());
        manager
            .register_protocol(ProtocolCapability {
                protocol: Protocol::Tarpc,
                port: 8081,
                path: None,
                status: ProtocolStatus::Active,
                metadata: HashMap::new(),
            })
            .await;

        let mut peer = TowerCapabilities::new("tower-2".into(), "http://peer:80".into());
        peer.add_protocol(ProtocolCapability {
            protocol: Protocol::Http,
            port: 80,
            path: None,
            status: ProtocolStatus::Active,
            metadata: HashMap::new(),
        });
        manager.store_peer_capabilities(peer).await;

        assert_eq!(manager.negotiate_protocol("tower-2").await, None);
    }

    #[tokio::test]
    async fn negotiate_protocol_unknown_peer_returns_none() {
        let manager =
            ProtocolCapabilityManager::new("tower-1".into(), "http://localhost:8080".into());
        assert_eq!(manager.negotiate_protocol("nonexistent").await, None);
    }

    #[tokio::test]
    async fn get_active_peers_returns_stored_ids() {
        let manager =
            ProtocolCapabilityManager::new("tower-1".into(), "http://localhost:8080".into());
        let p1 = TowerCapabilities::new("peer-a".into(), "http://a:1".into());
        let p2 = TowerCapabilities::new("peer-b".into(), "http://b:1".into());
        manager.store_peer_capabilities(p1).await;
        manager.store_peer_capabilities(p2).await;
        let peers = manager.get_active_peers().await;
        assert_eq!(peers.len(), 2);
        assert!(peers.contains(&"peer-a".to_string()));
        assert!(peers.contains(&"peer-b".to_string()));
    }

    #[tokio::test]
    async fn register_feature_stored_in_capabilities() {
        let manager =
            ProtocolCapabilityManager::new("tower-1".into(), "http://localhost:8080".into());
        manager.register_feature("gpu-compute".into()).await;
        manager.register_feature("arm64".into()).await;
        let caps = manager.get_local_capabilities().await;
        assert_eq!(caps.features.len(), 2);
        assert!(caps.features.contains(&"gpu-compute".to_string()));
    }

    #[test]
    fn protocol_encryption_classification() {
        assert!(!Protocol::Http.is_encrypted());
        assert!(Protocol::Https.is_encrypted());
        assert!(Protocol::Btsp.is_encrypted());
        assert!(Protocol::Tarpc.is_encrypted());
        assert!(!Protocol::WebSocket.is_encrypted());
        assert!(Protocol::WebSocketSecure.is_encrypted());
        assert!(!Protocol::JsonRpc.is_encrypted());
    }

    #[test]
    fn performance_tiers_ordered() {
        assert!(Protocol::Tarpc.performance_tier() > Protocol::Btsp.performance_tier());
        assert!(Protocol::Btsp.performance_tier() > Protocol::Https.performance_tier());
        assert!(Protocol::Https.performance_tier() >= Protocol::Http.performance_tier());
        assert!(
            Protocol::WebSocketSecure.performance_tier() > Protocol::WebSocket.performance_tier()
        );
    }
}
