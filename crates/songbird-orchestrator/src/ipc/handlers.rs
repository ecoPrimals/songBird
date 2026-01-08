//! JSON-RPC API handlers for Unix socket IPC
//!
//! v3.19.1: Modern idiomatic Rust handlers for biomeOS integration

use anyhow::Result;
use jsonrpsee::types::{ErrorObject, Params};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::*;
use crate::app::core::SongbirdOrchestrator;
use songbird_types::TrustLevel;

/// API handlers for Unix socket JSON-RPC methods
pub struct IpcHandlers {
    /// Reference to the orchestrator (for accessing discovery, connections, etc.)
    orchestrator: Arc<RwLock<SongbirdOrchestrator>>,
}

impl IpcHandlers {
    /// Create new API handlers
    pub fn new(orchestrator: Arc<RwLock<SongbirdOrchestrator>>) -> Self {
        Self { orchestrator }
    }
    
    /// Handle `discover_by_family` RPC call
    ///
    /// Filters discovered peers by genetic family tags.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "discover_by_family",
    ///   "params": {
    ///     "family_tags": ["nat0", "lan0"],
    ///     "timeout_ms": 5000
    ///   },
    ///   "id": 1
    /// }
    /// ```
    pub async fn discover_by_family(
        &self,
        params: Params<'_>,
    ) -> Result<DiscoverByFamilyResponse, ErrorObject<'static>> {
        debug!("📡 IPC: discover_by_family called");
        
        // Parse request parameters
        let request: DiscoverByFamilyRequest = params
            .parse()
            .map_err(|e| ErrorObject::owned(
                -32602,
                "Invalid params",
                Some(format!("{}", e)),
            ))?;
        
        info!("🔍 Discovering peers in families: {:?}", request.family_tags);
        
        // Get orchestrator (read lock)
        let orch = self.orchestrator.read().await;
        
        // Get all discovered peers from discovery listener
        let all_peers = orch
            .get_discovered_peers()
            .await
            .map_err(|e| ErrorObject::owned(
                -32603,
                "Failed to get discovered peers",
                Some(format!("{}", e)),
            ))?;
        
        debug!("   Total discovered peers: {}", all_peers.len());
        
        // Filter peers by family tags
        let filtered_peers: Vec<_> = all_peers
            .into_iter()
            .filter(|peer| {
                // Check if peer has any of the requested family tags
                peer.tags.as_ref()
                    .map(|tags| {
                        request.family_tags.iter().any(|family_tag| {
                            tags.iter().any(|tag| {
                                // Match format: "beardog:family:FAMILY_ID" or "family:FAMILY_ID"
                                tag.contains(&format!(":family:{}:", family_tag))
                                    || tag.contains(&format!("family_{}", family_tag))
                                    || tag.contains(&format!(":{}", family_tag))
                            })
                        })
                    })
                    .unwrap_or(false)
            })
            .collect();
        
        info!("   Filtered to {} peers in requested families", filtered_peers.len());
        
        // Convert to response format
        let nodes: Vec<DiscoveredNode> = filtered_peers
            .into_iter()
            .map(|peer| {
                // Extract genetic families from tags
                let genetic_families = peer.tags.as_ref()
                    .map(|tags| Self::extract_families_from_tags(tags))
                    .unwrap_or_default();
                
                // Extract sub-federations (if present)
                let sub_federations = peer.tags.as_ref()
                    .map(|tags| Self::extract_subfederations_from_tags(tags))
                    .unwrap_or_default();
                
                // Check if peer supports BTSP
                let btsp_endpoint = if peer.tags.as_ref()
                    .map(|tags| tags.iter().any(|t| t == "btsp_enabled"))
                    .unwrap_or(false)
                {
                    Some(format!("udp://{}", peer.address))
                } else {
                    None
                };
                
                DiscoveredNode {
                    node_id: peer.node_id.unwrap_or_else(|| peer.session_id.clone()),
                    node_name: peer.node_name,
                    genetic_families,
                    sub_federations,
                    capabilities: peer.capabilities.clone(),
                    btsp_endpoint,
                    https_endpoint: format!("https://{}:{}", peer.address.ip(), peer.port),
                    last_seen: system_time_to_iso8601(peer.last_seen),
                }
            })
            .collect();
        
        Ok(DiscoverByFamilyResponse { nodes })
    }
    
    /// Handle `create_genetic_tunnel` RPC call
    ///
    /// Establishes a BTSP tunnel to a peer using genetic proof.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "create_genetic_tunnel",
    ///   "params": {
    ///     "peer_node_id": "node-beta",
    ///     "genetic_proof": {
    ///       "family_id": "nat0",
    ///       "parent_seed_hash": "abc123",
    ///       "relationship": "sibling"
    ///     }
    ///   },
    ///   "id": 2
    /// }
    /// ```
    pub async fn create_genetic_tunnel(
        &self,
        params: Params<'_>,
    ) -> Result<CreateGeneticTunnelResponse, ErrorObject<'static>> {
        debug!("🔐 IPC: create_genetic_tunnel called");
        
        // Parse request parameters
        let request: CreateGeneticTunnelRequest = params
            .parse()
            .map_err(|e| ErrorObject::owned(
                -32602,
                "Invalid params",
                Some(format!("{}", e)),
            ))?;
        
        info!("🌐 Creating genetic tunnel to peer: {}", request.peer_node_id);
        
        // Get orchestrator (read lock for query, then write lock for mutation)
        let orch = self.orchestrator.read().await;
        
        // Get peer endpoint (from request or discovery)
        let peer_endpoint = if let Some(endpoint) = request.peer_endpoint {
            endpoint
        } else {
            // Look up peer in discovered peers
            let discovered_peers = orch
                .get_discovered_peers()
                .await
                .map_err(|e| ErrorObject::owned(
                    -32603,
                    "Failed to get discovered peers",
                    Some(format!("{}", e)),
                ))?;
            
            discovered_peers
                .iter()
                .find(|p| {
                    p.node_id.as_ref() == Some(&request.peer_node_id)
                        || p.session_id == request.peer_node_id
                })
                .map(|p| format!("https://{}:{}", p.address.ip(), p.port))
                .ok_or_else(|| ErrorObject::owned(
                    -32001,
                    "Peer not found",
                    Some(format!("No peer with node_id '{}'", request.peer_node_id)),
                ))?
        };
        
        // Get peer tags (for BTSP)
        let peer_tags = vec!["btsp_enabled".to_string()];  // Minimal tags for tunnel
        
        // Determine trust level from genetic proof
        let trust_level = if request.genetic_proof.is_some() {
            info!("   Using trust level Limited (genetic family member)");
            TrustLevel::Limited  // Same family = Limited trust
        } else {
            warn!("   No genetic proof provided - defaulting to Limited trust");
            TrustLevel::Limited
        };
        
        // Drop read lock before acquiring write lock
        drop(orch);
        
        // Establish connection (BTSP-first)
        let mut orch_mut = self.orchestrator.write().await;
        orch_mut
            .establish_connection(
                request.peer_node_id.clone(),
                peer_endpoint.clone(),
                vec![], // Capabilities can be empty for tunnel creation
                peer_tags,
                trust_level,
                "genetic_tunnel".to_string(),
            )
            .await
            .map_err(|e| ErrorObject::owned(
                -32603,
                "Failed to establish tunnel",
                Some(format!("{}", e)),
            ))?;
        
        info!("✅ Genetic tunnel established to {}", request.peer_node_id);
        
        // Create tunnel ID
        let tunnel_id = format!(
            "tunnel-{}-{}",
            request.genetic_proof
                .as_ref()
                .map(|p| p.family_id.as_str())
                .unwrap_or("unknown"),
            &request.peer_node_id
        );
        
        Ok(CreateGeneticTunnelResponse {
            tunnel_id,
            status: "established".to_string(),
            local_endpoint: None,  // TODO: Get from BTSP client
            peer_endpoint: Some(peer_endpoint),
            encryption: Some("BearDog-AES-256-GCM".to_string()),
            created_at: system_time_to_iso8601(SystemTime::now()),
        })
    }
    
    /// Handle `announce_capabilities` RPC call
    ///
    /// Updates the capabilities and tags that this node broadcasts.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "announce_capabilities",
    ///   "params": {
    ///     "capabilities": ["storage", "compute"],
    ///     "sub_federations": ["gaming"],
    ///     "genetic_families": ["nat0"]
    ///   },
    ///   "id": 3
    /// }
    /// ```
    pub async fn announce_capabilities(
        &self,
        params: Params<'_>,
    ) -> Result<AnnounceCapabilitiesResponse, ErrorObject<'static>> {
        debug!("📢 IPC: announce_capabilities called");
        
        // Parse request parameters
        let request: AnnounceCapabilitiesRequest = params
            .parse()
            .map_err(|e| ErrorObject::owned(
                -32602,
                "Invalid params",
                Some(format!("{}", e)),
            ))?;
        
        info!("📣 Updating capabilities: {:?}", request.capabilities);
        info!("   Genetic families: {:?}", request.genetic_families);
        info!("   Sub-federations: {:?}", request.sub_federations);
        
        // Get orchestrator (write lock for mutation)
        let orch = self.orchestrator.write().await;
        
        // Update capabilities in broadcaster
        // TODO: Implement orchestrator.update_capabilities() method
        // For now, log the update (implementation in next phase)
        
        warn!("⚠️  Capability update logged but not yet applied to broadcaster");
        warn!("   This requires broadcaster to be wrapped in Arc<RwLock<>>");
        warn!("   Tracked for v3.19.2 implementation");
        
        Ok(AnnounceCapabilitiesResponse {
            status: "pending".to_string(),  // "updated" when implemented
            broadcasting: true,
            updated_at: system_time_to_iso8601(SystemTime::now()),
        })
    }
    
    /// Extract genetic families from tags
    ///
    /// Format: `beardog:family:FAMILY_ID` or `family:FAMILY_ID`
    fn extract_families_from_tags(tags: &[String]) -> Vec<String> {
        tags.iter()
            .filter_map(|tag| {
                if tag.contains(":family:") {
                    // Format: "beardog:family:nat0" -> "nat0"
                    tag.split(":family:").nth(1)
                        .map(|s| s.split(':').next().unwrap_or(s).to_string())
                } else if tag.starts_with("family_") {
                    // Format: "family_nat0" -> "nat0"
                    Some(tag.trim_start_matches("family_").to_string())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Extract sub-federations from tags
    ///
    /// Format: `:subfed:SUBFED_NAME` or `subfed_NAME`
    fn extract_subfederations_from_tags(tags: &[String]) -> Vec<String> {
        tags.iter()
            .filter_map(|tag| {
                if let Some(pos) = tag.find(":subfed:") {
                    // Extract everything after ":subfed:" (e.g., "beardog:subfed:gaming" -> "gaming")
                    Some(tag[pos + 8..].to_string())
                } else if tag.starts_with("subfed_") {
                    Some(tag.trim_start_matches("subfed_").to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_families_from_tags() {
        let tags = vec![
            "beardog:family:nat0".to_string(),
            "beardog:family:lan0:node1".to_string(),
            "family_nat0".to_string(),
            "other_tag".to_string(),
        ];
        
        let families = IpcHandlers::extract_families_from_tags(&tags);
        
        assert_eq!(families.len(), 3);
        assert!(families.contains(&"nat0".to_string()));
        assert!(families.contains(&"lan0".to_string()));
    }
    
    #[test]
    fn test_extract_subfederations_from_tags() {
        let tags = vec![
            "beardog:subfed:gaming".to_string(),  // Full format
            "subfed_family".to_string(),           // Short format
            "other_tag".to_string(),               // Ignored
        ];
        
        let subfeds = IpcHandlers::extract_subfederations_from_tags(&tags);
        
        assert_eq!(subfeds.len(), 2);
        assert!(subfeds.contains(&"gaming".to_string()));
        assert!(subfeds.contains(&"family".to_string()));
    }
}

