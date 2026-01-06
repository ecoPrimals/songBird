//! Connection Manager for Progressive Trust
//!
//! Manages peer connections with trust-based capability enforcement.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use songbird_types::TrustLevel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::connections::{Connection, FederatedConnection, FullTrustConnection, LimitedConnection};
use crate::trust::peer_trust::PeerTrustDecision;

/// Metadata about a peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMetadata {
    pub peer_id: String,
    pub endpoint: String,
    pub trust_level: TrustLevel,
    pub discovery_method: String,
    pub capabilities: Vec<String>,
    #[serde(with = "systemtime_as_secs")]
    pub established_at: std::time::SystemTime,
}

// Helper module for serializing SystemTime as seconds since UNIX_EPOCH
mod systemtime_as_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

/// Manages connections to discovered peers with progressive trust
pub struct ConnectionManager {
    /// Active connections by peer_id
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    
    /// Metadata about each peer
    peer_metadata: Arc<RwLock<HashMap<String, PeerMetadata>>>,
    
    /// Rejected peers (for audit trail)
    rejected_peers: Arc<RwLock<HashMap<String, String>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            peer_metadata: Arc::new(RwLock::new(HashMap::new())),
            rejected_peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Handle a trust decision and establish appropriate connection
    pub async fn handle_trust_decision(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        trust_decision: &PeerTrustDecision,
        discovery_method: String,
    ) -> Result<()> {
        match trust_decision {
            PeerTrustDecision::AutoAccept { reason, confidence, .. } => {
                // Determine trust level from decision (currently binary, will use levels later)
                let trust_level = if reason.contains("same_genetic_family") || reason.contains("same_family") {
                    TrustLevel::Limited
                } else if confidence >= &0.9 {
                    TrustLevel::Elevated
                } else {
                    TrustLevel::Limited
                };
                
                self.establish_connection(
                    peer_id,
                    endpoint,
                    capabilities,
                    trust_level,
                    discovery_method,
                ).await
            }
            
            PeerTrustDecision::PromptUser { reason, .. } => {
                // For now, treat as limited connection
                // TODO: Implement user prompt in Phase 6
                warn!("⏳ User prompt needed for peer '{}': {}", peer_id, reason);
                self.establish_connection(
                    peer_id,
                    endpoint,
                    capabilities,
                    TrustLevel::Limited,
                    discovery_method,
                ).await
            }
            
            PeerTrustDecision::Reject { reason, .. } => {
                info!("❌ Rejecting peer '{}': {}", peer_id, reason);
                let mut rejected = self.rejected_peers.write().await;
                rejected.insert(peer_id, reason.clone());
                Ok(())
            }
        }
    }
    
    /// Establish a connection at a specific trust level
    pub async fn establish_connection(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        trust_level: TrustLevel,
        discovery_method: String,
    ) -> Result<()> {
        info!(
            "🔗 Establishing connection to '{}' at trust level {} ({})",
            peer_id,
            trust_level.as_u8(),
            trust_level.name()
        );
        
        // Create appropriate connection type
        let connection = match trust_level {
            TrustLevel::None => {
                warn!("Cannot establish connection at trust level 0 (None)");
                return Err(anyhow!("Trust level None - connection rejected"));
            }
            
            TrustLevel::Limited => {
                info!("🎵 Creating Limited connection (BirdSong only)");
                let conn = LimitedConnection::with_defaults(peer_id.clone(), endpoint.clone())?;
                Connection::Limited(conn)
            }
            
            TrustLevel::Elevated => {
                info!("✅ Creating Federated connection (full federation)");
                let conn = FederatedConnection::with_defaults(peer_id.clone(), endpoint.clone())?;
                Connection::Federated(conn)
            }
            
            TrustLevel::Highest => {
                info!("🔓 Creating Full Trust connection (all operations)");
                let conn = FullTrustConnection::new(peer_id.clone(), endpoint.clone())?;
                Connection::FullTrust(conn)
            }
        };
        
        // Store metadata
        let metadata = PeerMetadata {
            peer_id: peer_id.clone(),
            endpoint: endpoint.clone(),
            trust_level,
            discovery_method,
            capabilities,
            established_at: std::time::SystemTime::now(),
        };
        
        // Store connection and metadata
        let mut connections = self.connections.write().await;
        let mut peer_metadata = self.peer_metadata.write().await;
        
        connections.insert(peer_id.clone(), connection);
        peer_metadata.insert(peer_id.clone(), metadata);
        
        info!("✅ Connection established with '{}'", peer_id);
        Ok(())
    }
    
    /// Call a peer operation with capability enforcement
    pub async fn call_peer(
        &self,
        peer_id: &str,
        operation: &str,
        request: Value,
    ) -> Result<Value> {
        // Get connection
        let connections = self.connections.read().await;
        let connection = connections.get(peer_id)
            .ok_or_else(|| anyhow!("Peer '{}' not connected", peer_id))?;
        
        // Check if operation is allowed
        if !connection.is_operation_allowed(operation) {
            let metadata = self.peer_metadata.read().await;
            let peer_meta = metadata.get(peer_id);
            
            warn!(
                "🔒 Operation '{}' denied for peer '{}' at trust level {}",
                operation,
                peer_id,
                connection.trust_level().name()
            );
            
            return Err(anyhow!(
                "Operation '{}' not allowed for peer '{}' at trust level {} ({}). \
                 Allowed capabilities: {:?}. \
                 To enable this operation, elevate trust level via user approval.",
                operation,
                peer_id,
                connection.trust_level().as_u8(),
                connection.trust_level().name(),
                connection.as_peer_connection().allowed_capabilities()
            ));
        }
        
        debug!(
            "📞 Calling operation '{}' on peer '{}' (trust level {})",
            operation,
            peer_id,
            connection.trust_level().name()
        );
        
        // Make the call
        connection.call(operation, request).await
    }
    
    /// Get connection for a peer
    pub async fn get_connection(&self, peer_id: &str) -> Option<TrustLevel> {
        let connections = self.connections.read().await;
        connections.get(peer_id).map(|conn| conn.trust_level())
    }
    
    /// Get all connected peers
    pub async fn list_peers(&self) -> Vec<(String, TrustLevel)> {
        let connections = self.connections.read().await;
        connections.iter()
            .map(|(id, conn)| (id.clone(), conn.trust_level()))
            .collect()
    }
    
    /// Get metadata for a peer
    pub async fn get_peer_metadata(&self, peer_id: &str) -> Option<PeerMetadata> {
        let metadata = self.peer_metadata.read().await;
        metadata.get(peer_id).cloned()
    }
    
    /// Get all discovered peers (for discovery.list_peers API)
    pub async fn get_all_peers(&self) -> Vec<PeerMetadata> {
        let metadata = self.peer_metadata.read().await;
        metadata.values().cloned().collect()
    }
    
    /// Get peer count (for discovery.peer_count API)
    pub async fn get_peer_count(&self) -> usize {
        let metadata = self.peer_metadata.read().await;
        metadata.len()
    }
    
    /// Get rejected peers (for diagnostics)
    pub async fn get_rejected_peers(&self) -> HashMap<String, String> {
        let rejected = self.rejected_peers.read().await;
        rejected.clone()
    }
    
    /// Close connection to a peer
    pub async fn close_connection(&self, peer_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(peer_id) {
            info!("🔌 Closing connection to peer '{}'", peer_id);
            connection.as_peer_connection().close().await?;
        }
        Ok(())
    }
    
    /// Get count of connections by trust level
    pub async fn connection_stats(&self) -> HashMap<TrustLevel, usize> {
        let connections = self.connections.read().await;
        let mut stats = HashMap::new();
        
        for conn in connections.values() {
            *stats.entry(conn.trust_level()).or_insert(0) += 1;
        }
        
        stats
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust::peer_trust::PeerTrustDecision;

    #[tokio::test]
    async fn test_limited_connection_establishment() {
        let manager = ConnectionManager::new();
        
        let decision = PeerTrustDecision::AutoAccept {
            reason: "same_genetic_family".to_string(),
            confidence: 1.0,
            encryption_tag: Some("test_tag".to_string()),
        };
        
        manager.handle_trust_decision(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
            vec!["birdsong/*".to_string()],
            &decision,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let trust_level = manager.get_connection("test_peer").await;
        assert_eq!(trust_level, Some(TrustLevel::Limited));
    }

    #[tokio::test]
    async fn test_reject_decision() {
        let manager = ConnectionManager::new();
        
        let decision = PeerTrustDecision::Reject {
            reason: "different_family".to_string(),
            trust_level: "none".to_string(),
        };
        
        manager.handle_trust_decision(
            "rejected_peer".to_string(),
            "http://localhost:8080".to_string(),
            vec![],
            &decision,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let trust_level = manager.get_connection("rejected_peer").await;
        assert_eq!(trust_level, None);
        
        let rejected = manager.rejected_peers.read().await;
        assert_eq!(rejected.get("rejected_peer"), Some(&"different_family".to_string()));
    }
    
    // ========================================================================
    // Unit Tests for Peer Discovery API Methods (v3.8.0)
    // ========================================================================
    
    #[tokio::test]
    async fn test_get_all_peers_empty() {
        let manager = ConnectionManager::new();
        
        let peers = manager.get_all_peers().await;
        assert_eq!(peers.len(), 0, "Should start with no peers");
    }
    
    #[tokio::test]
    async fn test_get_all_peers_single() {
        let manager = ConnectionManager::new();
        
        // Establish a connection
        manager.establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let peers = manager.get_all_peers().await;
        assert_eq!(peers.len(), 1, "Should have 1 peer");
        assert_eq!(peers[0].peer_id, "tower1");
        assert_eq!(peers[0].endpoint, "https://192.168.1.100:8080");
        assert_eq!(peers[0].trust_level, TrustLevel::Limited);
        assert_eq!(peers[0].discovery_method, "udp_multicast");
        assert_eq!(peers[0].capabilities, vec!["orchestrator".to_string()]);
    }
    
    #[tokio::test]
    async fn test_get_all_peers_multiple() {
        let manager = ConnectionManager::new();
        
        // Establish multiple connections
        manager.establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        manager.establish_connection(
            "tower2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["orchestrator", "federation-member"].iter().map(|s| s.to_string()).collect(),
            TrustLevel::Elevated,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        manager.establish_connection(
            "tower3".to_string(),
            "https://192.168.1.102:8080".to_string(),
            vec!["orchestrator", "storage"].iter().map(|s| s.to_string()).collect(),
            TrustLevel::Highest,
            "mdns".to_string(),
        ).await.unwrap();
        
        let peers = manager.get_all_peers().await;
        assert_eq!(peers.len(), 3, "Should have 3 peers");
        
        // Verify all peers are present
        let peer_ids: Vec<_> = peers.iter().map(|p| p.peer_id.as_str()).collect();
        assert!(peer_ids.contains(&"tower1"));
        assert!(peer_ids.contains(&"tower2"));
        assert!(peer_ids.contains(&"tower3"));
        
        // Verify trust levels
        let tower2 = peers.iter().find(|p| p.peer_id == "tower2").unwrap();
        assert_eq!(tower2.trust_level, TrustLevel::Elevated);
        
        let tower3 = peers.iter().find(|p| p.peer_id == "tower3").unwrap();
        assert_eq!(tower3.trust_level, TrustLevel::Highest);
        assert_eq!(tower3.discovery_method, "mdns");
    }
    
    #[tokio::test]
    async fn test_get_peer_count_empty() {
        let manager = ConnectionManager::new();
        
        let count = manager.get_peer_count().await;
        assert_eq!(count, 0, "Should start with 0 peers");
    }
    
    #[tokio::test]
    async fn test_get_peer_count_incremental() {
        let manager = ConnectionManager::new();
        
        assert_eq!(manager.get_peer_count().await, 0);
        
        manager.establish_connection(
            "peer1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        assert_eq!(manager.get_peer_count().await, 1);
        
        manager.establish_connection(
            "peer2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        assert_eq!(manager.get_peer_count().await, 2);
        
        manager.establish_connection(
            "peer3".to_string(),
            "https://192.168.1.102:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        assert_eq!(manager.get_peer_count().await, 3);
    }
    
    #[tokio::test]
    async fn test_get_rejected_peers_empty() {
        let manager = ConnectionManager::new();
        
        let rejected = manager.get_rejected_peers().await;
        assert_eq!(rejected.len(), 0, "Should start with no rejected peers");
    }
    
    #[tokio::test]
    async fn test_get_rejected_peers_single() {
        let manager = ConnectionManager::new();
        
        let decision = PeerTrustDecision::Reject {
            reason: "no_genetic_lineage".to_string(),
            trust_level: "none".to_string(),
        };
        
        manager.handle_trust_decision(
            "rogue_device".to_string(),
            "https://192.168.1.200:8080".to_string(),
            vec![],
            &decision,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let rejected = manager.get_rejected_peers().await;
        assert_eq!(rejected.len(), 1, "Should have 1 rejected peer");
        assert_eq!(rejected.get("rogue_device"), Some(&"no_genetic_lineage".to_string()));
    }
    
    #[tokio::test]
    async fn test_get_rejected_peers_multiple() {
        let manager = ConnectionManager::new();
        
        // Reject multiple peers with different reasons
        let decision1 = PeerTrustDecision::Reject {
            reason: "no_genetic_lineage".to_string(),
            trust_level: "none".to_string(),
        };
        
        manager.handle_trust_decision(
            "rogue1".to_string(),
            "https://192.168.1.200:8080".to_string(),
            vec![],
            &decision1,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let decision2 = PeerTrustDecision::Reject {
            reason: "different_family".to_string(),
            trust_level: "none".to_string(),
        };
        
        manager.handle_trust_decision(
            "rogue2".to_string(),
            "https://192.168.1.201:8080".to_string(),
            vec![],
            &decision2,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let decision3 = PeerTrustDecision::Reject {
            reason: "failed_attestation".to_string(),
            trust_level: "none".to_string(),
        };
        
        manager.handle_trust_decision(
            "rogue3".to_string(),
            "https://192.168.1.202:8080".to_string(),
            vec![],
            &decision3,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let rejected = manager.get_rejected_peers().await;
        assert_eq!(rejected.len(), 3, "Should have 3 rejected peers");
        assert_eq!(rejected.get("rogue1"), Some(&"no_genetic_lineage".to_string()));
        assert_eq!(rejected.get("rogue2"), Some(&"different_family".to_string()));
        assert_eq!(rejected.get("rogue3"), Some(&"failed_attestation".to_string()));
    }
    
    #[tokio::test]
    async fn test_peer_metadata_get_specific() {
        let manager = ConnectionManager::new();
        
        manager.establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator", "storage"].iter().map(|s| s.to_string()).collect(),
            TrustLevel::Elevated,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let metadata = manager.get_peer_metadata("tower1").await;
        assert!(metadata.is_some(), "Should find peer metadata");
        
        let meta = metadata.unwrap();
        assert_eq!(meta.peer_id, "tower1");
        assert_eq!(meta.endpoint, "https://192.168.1.100:8080");
        assert_eq!(meta.trust_level, TrustLevel::Elevated);
        assert_eq!(meta.capabilities.len(), 2);
        assert!(meta.capabilities.contains(&"orchestrator".to_string()));
        assert!(meta.capabilities.contains(&"storage".to_string()));
    }
    
    #[tokio::test]
    async fn test_concurrent_peer_access() {
        use tokio::task;
        
        let manager = Arc::new(ConnectionManager::new());
        
        // Add some initial peers
        manager.establish_connection(
            "peer1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        manager.establish_connection(
            "peer2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        // Spawn multiple concurrent readers
        let mut handles = vec![];
        
        for _ in 0..10 {
            let mgr = Arc::clone(&manager);
            let handle = task::spawn(async move {
                let peers = mgr.get_all_peers().await;
                assert!(peers.len() >= 2, "Should see at least 2 peers");
                
                let count = mgr.get_peer_count().await;
                assert!(count >= 2, "Count should be at least 2");
            });
            handles.push(handle);
        }
        
        // Wait for all concurrent reads
        for handle in handles {
            handle.await.unwrap();
        }
    }
    
    #[tokio::test]
    async fn test_peer_metadata_serialization() {
        use serde_json;
        
        let manager = ConnectionManager::new();
        
        manager.establish_connection(
            "test_peer".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let peers = manager.get_all_peers().await;
        assert_eq!(peers.len(), 1);
        
        // Test that PeerMetadata can be serialized to JSON
        let json = serde_json::to_string(&peers[0]).unwrap();
        assert!(json.contains("test_peer"));
        assert!(json.contains("192.168.1.100"));
        assert!(json.contains("orchestrator"));
        
        // Test that it can be deserialized back
        let deserialized: PeerMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.peer_id, "test_peer");
        assert_eq!(deserialized.endpoint, "https://192.168.1.100:8080");
        assert_eq!(deserialized.trust_level, TrustLevel::Limited);
    }

    #[tokio::test]
    async fn test_connection_stats() {
        let manager = ConnectionManager::new();
        
        // Add limited connection
        manager.establish_connection(
            "peer1".to_string(),
            "http://localhost:8080".to_string(),
            vec![],
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        // Add federated connection
        manager.establish_connection(
            "peer2".to_string(),
            "http://localhost:8081".to_string(),
            vec![],
            TrustLevel::Elevated,
            "udp_multicast".to_string(),
        ).await.unwrap();
        
        let stats = manager.connection_stats().await;
        assert_eq!(stats.get(&TrustLevel::Limited), Some(&1));
        assert_eq!(stats.get(&TrustLevel::Elevated), Some(&1));
    }
}

