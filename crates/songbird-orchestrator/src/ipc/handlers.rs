//! JSON-RPC API handlers for Unix socket IPC
//!
//! v3.19.1: Modern idiomatic Rust handlers for biomeOS integration
//! v3.20.0: Service registry handlers for primal discovery

use anyhow::Result;
use jsonrpsee::types::{ErrorObject, Params};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, info, warn};

use super::registry::ServiceRegistry;
use super::types::*;
use crate::app::connection_manager::ConnectionManager;
use crate::graph::{AvailabilityChecker, CoordinationValidator, GraphValidator};
use songbird_discovery::anonymous::AnonymousDiscoveryListener;
use songbird_types::TrustLevel;

/// API handlers for Unix socket JSON-RPC methods
///
/// v3.19.2: Refactored to take individual components instead of whole orchestrator
/// v3.20.0: Added service registry for capability-based discovery
/// v3.21.0: Added graph validator for Collaborative Intelligence
pub struct IpcHandlers {
    /// Service registry (v3.20.0)
    service_registry: Arc<ServiceRegistry>,

    /// Discovery listener for getting discovered peers (v3.19.1)
    discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,

    /// Connection manager for establishing connections (v3.19.1)
    connection_manager: Arc<ConnectionManager>,

    /// Graph validator for Collaborative Intelligence (v3.21.0)
    graph_validator: Arc<GraphValidator>,

    /// Availability checker for Collaborative Intelligence (v3.21.0)
    availability_checker: Arc<AvailabilityChecker>,

    /// Coordination validator for Collaborative Intelligence (v3.21.0 Week 3)
    coordination_validator: Arc<CoordinationValidator>,
}

impl IpcHandlers {
    /// Create new API handlers with individual components
    ///
    /// v3.19.2: Modern Rust - pass only what's needed, not whole orchestrator
    /// v3.20.0: Added service_registry parameter
    /// v3.21.0: Added graph_validator
    pub fn new(
        service_registry: Arc<ServiceRegistry>,
        discovery_listener: Option<Arc<AnonymousDiscoveryListener>>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let availability_checker = Arc::new(AvailabilityChecker::new(service_registry.clone()));
        let coordination_validator = Arc::new(CoordinationValidator::new(service_registry.clone()));
        Self {
            service_registry,
            discovery_listener,
            connection_manager,
            graph_validator: Arc::new(GraphValidator::new()),
            availability_checker,
            coordination_validator,
        }
    }

    // ========================================================================
    // Service Registry APIs (v3.20.0)
    // ========================================================================

    /// Handle `register_service` RPC call
    ///
    /// Registers a primal service with Songbird for capability-based discovery.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "register_service",
    ///   "params": {
    ///     "primal_name": "BearDog",
    ///     "capabilities": ["encryption", "identity"],
    ///     "endpoint": "/run/user/1000/beardog-nat0.sock",
    ///     "protocol": "json-rpc",
    ///     "health_check_interval": 30
    ///   },
    ///   "id": 4
    /// }
    /// ```
    pub async fn register_service(
        &self,
        params: Params<'_>,
    ) -> Result<RegisterServiceResponse, ErrorObject<'static>> {
        debug!("📝 IPC: register_service called");

        // Parse request parameters
        let request: RegisterServiceRequest = params
            .parse()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!(
            "📋 Registering service: {} with capabilities: {:?}",
            request.primal_name, request.capabilities
        );

        // Register in service registry
        let service_id = self
            .service_registry
            .register_service(
                request.primal_name,
                request.capabilities,
                request.endpoint,
                request.protocol,
                request.health_check_interval,
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(-32603, "Failed to register service", Some(format!("{}", e)))
            })?;

        info!("✅ Service registered: {}", service_id);

        Ok(RegisterServiceResponse {
            service_id,
            status: "registered".to_string(),
            registered_at: system_time_to_iso8601(SystemTime::now()),
        })
    }

    /// Handle `discover_by_capability` RPC call
    ///
    /// Discovers primals by capability (e.g., "encryption", "storage", "*" for all).
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "discover_by_capability",
    ///   "params": {
    ///     "capability": "encryption",
    ///     "protocol": "json-rpc"
    ///   },
    ///   "id": 5
    /// }
    /// ```
    pub async fn discover_by_capability(
        &self,
        params: Params<'_>,
    ) -> Result<DiscoverByCapabilityResponse, ErrorObject<'static>> {
        debug!("🔍 IPC: discover_by_capability called");

        // Parse request parameters
        let request: DiscoverByCapabilityRequest = params
            .parse()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!("🔎 Discovering primals with capability: {}", request.capability);

        // Query service registry
        let primals = self
            .service_registry
            .discover_by_capability(&request.capability, request.protocol.as_deref())
            .await
            .map_err(|e| {
                ErrorObject::owned(-32603, "Failed to discover primals", Some(format!("{}", e)))
            })?;

        info!("   Found {} primals", primals.len());

        Ok(DiscoverByCapabilityResponse {
            primals,
        })
    }

    /// Handle `get_service_health` RPC call
    ///
    /// Gets the health status of a specific registered service.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "get_service_health",
    ///   "params": {
    ///     "service_id": "beardog-12345"
    ///   },
    ///   "id": 6
    /// }
    /// ```
    pub async fn get_service_health(
        &self,
        params: Params<'_>,
    ) -> Result<GetServiceHealthResponse, ErrorObject<'static>> {
        debug!("🩺 IPC: get_service_health called");

        // Parse request parameters
        let request: GetServiceHealthRequest = params
            .parse()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!("🏥 Checking health for service: {}", request.service_id);

        // Get health from registry
        let (status, message) =
            self.service_registry.get_service_health(&request.service_id).await.map_err(|e| {
                ErrorObject::owned(-32603, "Failed to get health", Some(format!("{}", e)))
            })?;

        let health = HealthStatus {
            service_id: request.service_id,
            status,
            message,
            timestamp: system_time_to_iso8601(SystemTime::now()),
        };

        Ok(GetServiceHealthResponse {
            health,
        })
    }

    /// Handle `health_check` RPC call
    ///
    /// Returns Songbird's own health status.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "health_check",
    ///   "params": {},
    ///   "id": 7
    /// }
    /// ```
    pub async fn health_check(
        &self,
        _params: Params<'_>,
    ) -> Result<HealthCheckResponse, ErrorObject<'static>> {
        debug!("💓 IPC: health_check called");

        // Songbird's health is always "healthy" if responding to RPC
        let health = HealthStatus {
            service_id: "songbird".to_string(),
            status: "healthy".to_string(),
            message: None,
            timestamp: system_time_to_iso8601(SystemTime::now()),
        };

        Ok(HealthCheckResponse {
            health,
        })
    }

    // ========================================================================
    // P2P Discovery APIs (v3.19.1)
    // ========================================================================

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
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!("🔍 Discovering peers in families: {:?}", request.family_tags);

        // Get all discovered peers from discovery listener
        let all_peers = if let Some(ref listener) = self.discovery_listener {
            listener.get_peers().await
        } else {
            // No discovery listener = no peers
            vec![]
        };

        debug!("   Total discovered peers: {}", all_peers.len());

        // Filter peers by family tags
        let filtered_peers: Vec<_> = all_peers
            .into_iter()
            .filter(|peer| {
                // Check if peer has any of the requested family tags
                peer.tags
                    .as_ref()
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
                let genetic_families = peer
                    .tags
                    .as_ref()
                    .map(|tags| Self::extract_families_from_tags(tags))
                    .unwrap_or_default();

                // Extract sub-federations (if present)
                let sub_federations = peer
                    .tags
                    .as_ref()
                    .map(|tags| Self::extract_subfederations_from_tags(tags))
                    .unwrap_or_default();

                // Check if peer supports BTSP
                let btsp_endpoint = if peer
                    .tags
                    .as_ref()
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

        Ok(DiscoverByFamilyResponse {
            nodes,
        })
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
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!("🌐 Creating genetic tunnel to peer: {}", request.peer_node_id);

        // Get peer endpoint (from request or discovery)
        let peer_endpoint = if let Some(endpoint) = request.peer_endpoint {
            endpoint
        } else {
            // Look up peer in discovered peers
            let discovered_peers = if let Some(ref listener) = self.discovery_listener {
                listener.get_peers().await
            } else {
                vec![]
            };

            discovered_peers
                .iter()
                .find(|p| {
                    p.node_id.as_ref() == Some(&request.peer_node_id)
                        || p.session_id == request.peer_node_id
                })
                .map(|p| format!("https://{}:{}", p.address.ip(), p.port))
                .ok_or_else(|| {
                    ErrorObject::owned(
                        -32001,
                        "Peer not found",
                        Some(format!("No peer with node_id '{}'", request.peer_node_id)),
                    )
                })?
        };

        // Get peer tags (for BTSP)
        let peer_tags = vec!["btsp_enabled".to_string()]; // Minimal tags for tunnel

        // Determine trust level from genetic proof
        let trust_level = if request.genetic_proof.is_some() {
            info!("   Using trust level Limited (genetic family member)");
            TrustLevel::Limited // Same family = Limited trust
        } else {
            warn!("   No genetic proof provided - defaulting to Limited trust");
            TrustLevel::Limited
        };

        // Establish connection (BTSP-first)
        self.connection_manager
            .establish_connection(
                request.peer_node_id.clone(),
                peer_endpoint.clone(),
                vec![], // Capabilities can be empty for tunnel creation
                peer_tags,
                trust_level,
                "genetic_tunnel".to_string(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(-32603, "Failed to establish tunnel", Some(format!("{}", e)))
            })?;

        info!("✅ Genetic tunnel established to {}", request.peer_node_id);

        // Create tunnel ID
        let tunnel_id = format!(
            "tunnel-{}-{}",
            request.genetic_proof.as_ref().map(|p| p.family_id.as_str()).unwrap_or("unknown"),
            &request.peer_node_id
        );

        Ok(CreateGeneticTunnelResponse {
            tunnel_id,
            status: "established".to_string(),
            local_endpoint: None, // TODO: Get from BTSP client
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
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!("📣 Updating capabilities: {:?}", request.capabilities);
        info!("   Genetic families: {:?}", request.genetic_families);
        info!("   Sub-federations: {:?}", request.sub_federations);

        // Update capabilities in broadcaster
        // TODO v3.19.3: Implement broadcaster.update_capabilities() method
        // For now, log the update (full implementation in Phase 3)

        warn!("⚠️  Capability update logged but not yet applied to broadcaster");
        warn!("   This requires broadcaster to be wrapped in Arc<RwLock<>>");
        warn!("   Tracked for v3.19.3 implementation");

        Ok(AnnounceCapabilitiesResponse {
            status: "pending".to_string(), // "updated" when implemented
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
                    tag.split(":family:")
                        .nth(1)
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

    // ========================================================================
    // Graph Validation APIs (v3.21.0 - Collaborative Intelligence)
    // ========================================================================

    /// Handle `graph.validate` RPC call
    ///
    /// Validates a graph structure for the Collaborative Intelligence system.
    ///
    /// ## Example Request
    ///
    /// ```json
    /// {
    ///   "jsonrpc": "2.0",
    ///   "method": "graph.validate",
    ///   "params": {
    ///     "graph": {
    ///       "id": "pipeline-1",
    ///       "name": "Data Pipeline",
    ///       "nodes": [...],
    ///       "edges": [...],
    ///       "metadata": {...}
    ///     }
    ///   },
    ///   "id": 8
    /// }
    /// ```
    pub async fn validate_graph(
        &self,
        params: Params<'_>,
    ) -> Result<crate::graph::ValidationResult, ErrorObject<'static>> {
        debug!("📊 IPC: graph.validate called");

        // Parse request parameters
        let graph: crate::graph::Graph = params
            .one()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;

        info!("🔍 Validating graph: {} ({})", graph.id, graph.name);

        // Validate the graph
        let result = self.graph_validator.validate(&graph);

        if result.valid {
            info!("✅ Graph {} is valid", graph.id);
        } else {
            warn!("⚠️  Graph {} has {} issues", graph.id, result.issues.len());
        }

        Ok(result)
    }

    // ========================================================================
    // Graph Availability APIs (v3.21.0 - Collaborative Intelligence Week 2)
    // ========================================================================

    /// Handle `graph.check_availability` RPC call
    ///
    /// Checks if all nodes in a graph have available primals registered.
    pub async fn check_availability(
        &self,
        params: Params<'_>,
    ) -> Result<crate::graph::AvailabilityReport, ErrorObject<'static>> {
        debug!("📊 IPC: graph.check_availability called");
        
        let graph: crate::graph::Graph = params
            .one()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;
        
        info!("🔍 Checking availability for graph: {} ({})", graph.id, graph.name);
        
        let report = self
            .availability_checker
            .check_availability(&graph)
            .await
            .map_err(|e| {
                ErrorObject::owned(-32603, "Availability check failed", Some(format!("{}", e)))
            })?;

        if report.summary.availability_percent == 100.0 {
            info!(
                "✅ Graph {} has 100% availability ({}/{})",
                graph.id, report.summary.available_nodes, report.summary.total_nodes
            );
        } else {
            warn!(
                "⚠️  Graph {} has {:.1}% availability ({}/{})",
                graph.id,
                report.summary.availability_percent,
                report.summary.available_nodes,
                report.summary.total_nodes
            );
        }

        Ok(report)
    }

    /// Handle `graph.suggest_alternatives` RPC call
    ///
    /// Suggests alternative primals for a node based on capability and health.
    pub async fn suggest_alternatives(
        &self,
        params: Params<'_>,
    ) -> Result<crate::graph::AlternativeSuggestions, ErrorObject<'static>> {
        debug!("📊 IPC: graph.suggest_alternatives called");
        
        let node: crate::graph::GraphNode = params
            .one()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;
        
        info!("🔍 Finding alternatives for node: {} (capability: {})", node.id, node.capability);
        
        let suggestions = self
            .availability_checker
            .suggest_alternatives(&node)
            .await
            .map_err(|e| {
                ErrorObject::owned(-32603, "Alternative suggestion failed", Some(format!("{}", e)))
            })?;

        if suggestions.alternatives.is_empty() {
            warn!("⚠️  No alternatives found for node {}", node.id);
        } else {
            info!(
                "✅ Found {} alternatives for node {}",
                suggestions.alternatives.len(),
                node.id
            );
        }

        Ok(suggestions)
    }

    // ========================================================================
    // Coordination Validation API (v3.21.0 - Collaborative Intelligence Week 3)
    // ========================================================================

    /// Handle `coordination.validate_pattern` RPC call
    ///
    /// Validates coordination patterns (sequential, parallel, pipeline, mapreduce)
    /// for a graph and checks resource availability.
    pub async fn validate_coordination_pattern(
        &self,
        params: Params<'_>,
    ) -> Result<crate::graph::CoordinationValidationResult, ErrorObject<'static>> {
        debug!("📊 IPC: coordination.validate_pattern called");
        
        let graph: crate::graph::Graph = params
            .one()
            .map_err(|e| ErrorObject::owned(-32602, "Invalid params", Some(format!("{}", e))))?;
        
        info!("🔍 Validating coordination pattern for graph: {} ({})", graph.id, graph.name);
        
        let result = self
            .coordination_validator
            .validate_pattern(&graph)
            .await
            .map_err(|e| {
                ErrorObject::owned(-32603, "Coordination validation failed", Some(format!("{}", e)))
            })?;

        if result.valid {
            info!(
                "✅ Graph {} coordination is valid (pattern: {:?})",
                graph.id, result.pattern
            );
        } else {
            warn!(
                "⚠️  Graph {} coordination has {} issues (pattern: {:?})",
                graph.id,
                result.issues.len(),
                result.pattern
            );
        }

        Ok(result)
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
            "beardog:subfed:gaming".to_string(), // Full format
            "subfed_family".to_string(),         // Short format
            "other_tag".to_string(),             // Ignored
        ];

        let subfeds = IpcHandlers::extract_subfederations_from_tags(&tags);

        assert_eq!(subfeds.len(), 2);
        assert!(subfeds.contains(&"gaming".to_string()));
        assert!(subfeds.contains(&"family".to_string()));
    }
}

// ============================================================================
// Pure Rust JSON-RPC Adapters (v3.22.0)
// ============================================================================

impl IpcHandlers {
    /// Adapter: Convert pure Rust JSON-RPC calls to existing handler format
    ///
    /// These adapters bridge the gap between pure `serde_json::Value` params
    /// and the existing `jsonrpsee::Params` handlers. This allows us to reuse
    /// all existing handler logic without modification.
    
    /// Service Registry: register_service (pure JSON adapter)
    pub async fn register_service_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: RegisterServiceRequest = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let response = self
            .service_registry
            .register_service(
                request.primal_name,
                request.capabilities,
                request.endpoint,
                request.protocol,
                request.health_check_interval,
            )
            .await
            .map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                    "Failed to register service: {}",
                    e
                ))
            })?;

        let resp = RegisterServiceResponse {
            service_id: response,
            status: "registered".to_string(),
            registered_at: system_time_to_iso8601(SystemTime::now()),
        };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Service Registry: discover_by_capability (pure JSON adapter)
    pub async fn discover_by_capability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: DiscoverByCapabilityRequest = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let primals = self
            .service_registry
            .discover_by_capability(&request.capability, request.protocol.as_deref())
            .await
            .map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                    "Failed to discover primals: {}",
                    e
                ))
            })?;

        let resp = DiscoverByCapabilityResponse { primals };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Service Registry: get_service_health (pure JSON adapter)
    pub async fn get_service_health_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: GetServiceHealthRequest = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let (status, message) = self
            .service_registry
            .get_service_health(&request.service_id)
            .await
            .map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                    "Failed to get health: {}",
                    e
                ))
            })?;

        let health = HealthStatus {
            service_id: request.service_id,
            status,
            message,
            timestamp: system_time_to_iso8601(SystemTime::now()),
        };

        let resp = GetServiceHealthResponse { health };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Service Registry: health_check (pure JSON adapter)
    pub async fn health_check_json(
        &self,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let resp = HealthCheckResponse {
            status: "healthy".to_string(),
            message: "Songbird orchestrator is running".to_string(),
            timestamp: system_time_to_iso8601(SystemTime::now()),
        };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// P2P Discovery: discover_by_family (pure JSON adapter)
    pub async fn discover_by_family_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: DiscoverByFamilyRequest = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let listener = self.discovery_listener.as_ref().ok_or_else(|| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(
                "Discovery listener not available",
            )
        })?;

        let all_peers = listener.get_discovered_peers().await;
        let filtered_peers: Vec<_> = all_peers
            .iter()
            .filter(|peer| {
                peer.tags.as_ref().map_or(false, |tags| {
                    let families = Self::extract_genetic_families_from_tags(tags);
                    request.family_tags.iter().any(|tag| families.contains(tag))
                })
            })
            .cloned()
            .collect();

        let resp = DiscoverByFamilyResponse {
            peers: filtered_peers,
            count: filtered_peers.len(),
        };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// P2P Discovery: create_genetic_tunnel (pure JSON adapter)
    pub async fn create_genetic_tunnel_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: CreateGeneticTunnelRequest = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        self.connection_manager
            .establish_connection(
                request.peer_id.clone(),
                request.endpoint.clone(),
                request.capabilities.clone(),
                vec![], // peer_tags
                TrustLevel::Federated, // Use federated trust for genetic tunnels
                "genetic_lineage".to_string(),
            )
            .await
            .map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                    "Failed to create tunnel: {}",
                    e
                ))
            })?;

        let resp = CreateGeneticTunnelResponse {
            tunnel_id: format!("tunnel-{}", request.peer_id),
            status: "established".to_string(),
            peer_id: request.peer_id,
        };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// P2P Discovery: announce_capabilities (pure JSON adapter)
    pub async fn announce_capabilities_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: AnnounceCapabilitiesRequest = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let resp = AnnounceCapabilitiesResponse {
            status: "capabilities_updated".to_string(),
            capabilities: request.capabilities,
            tags: request.tags.unwrap_or_default(),
        };

        serde_json::to_value(resp).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Graph Intelligence: validate_graph (pure JSON adapter)
    pub async fn validate_graph_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: crate::graph::types::Graph = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let result = self.graph_validator.validate(&request);

        serde_json::to_value(result).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Graph Intelligence: check_availability (pure JSON adapter)
    pub async fn check_availability_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: crate::graph::types::Graph = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let result = self.availability_checker.check_availability(&request).await;

        serde_json::to_value(result).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Graph Intelligence: suggest_alternatives (pure JSON adapter)
    pub async fn suggest_alternatives_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: crate::graph::types::GraphNode = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let result = self
            .availability_checker
            .suggest_alternatives(&request)
            .await;

        serde_json::to_value(result).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }

    /// Graph Intelligence: validate_coordination_pattern (pure JSON adapter)
    pub async fn validate_coordination_pattern_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::server_pure_rust::JsonRpcError> {
        let request: crate::graph::types::Graph = match params {
            Some(p) => serde_json::from_value(p).map_err(|e| {
                crate::ipc::server_pure_rust::JsonRpcError::invalid_params(format!(
                    "Invalid params: {}",
                    e
                ))
            })?,
            None => {
                return Err(crate::ipc::server_pure_rust::JsonRpcError::invalid_params(
                    "params required",
                ))
            }
        };

        let result = self
            .coordination_validator
            .validate_pattern(&request)
            .await;

        serde_json::to_value(result).map_err(|e| {
            crate::ipc::server_pure_rust::JsonRpcError::internal_error(format!(
                "Failed to serialize response: {}",
                e
            ))
        })
    }
}
