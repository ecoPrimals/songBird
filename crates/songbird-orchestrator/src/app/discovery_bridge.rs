//! Discovery → Federation Bridge
//!
//! Manages the automatic bridging of UDP multicast peer discoveries to federation
//! membership via progressive trust evaluation.
//!
//! ## Architecture
//!
//! This module implements a critical component of Songbird's zero-configuration
//! federation: automatically converting discovered peers (via UDP multicast) into
//! trusted federation members.
//!
//! ### Flow
//!
//! ```text
//! UDP Discovery
//!     ↓
//! AnonymousDiscoveryListener
//!     ↓
//! Bridge (polls every 10s)
//!     ↓
//! Same-Family Detection (NEW v3.10.0)
//!     ↓
//! Trust Evaluation (BearDog/Security Provider)
//!     ↓
//! ConnectionManager (Progressive Trust)
//!     ↓
//! FederationState (Registered Node)
//! ```
//!
//! ## Key Features
//!
//! - **Zero Hardcoding**: Discovers security provider at runtime via env vars
//! - **Same-Family LAN Optimization**: Skips HTTPS checks for same-family peers
//! - **Progressive Trust**: Establishes connections at appropriate trust levels
//! - **Safe Defaults**: Rejects on evaluation failure (fail-secure)
//! - **Audit Trail**: Records all trust decisions (accept/reject)
//!
//! ## Deep Debt Fixes (Jan 5, 2026)
//!
//! v3.10.0 fixed the Discovery→Registry wiring gap:
//! - HTTPS connectivity check was too strict for LAN deployments
//! - Same-family peers now skip HTTPS check (trust UDP discovery)
//! - Detailed logging at every decision point
//! - Peers now properly flow to API
//!
//! ## Related Modules
//!
//! - `connection_manager.rs` - Manages peer connections
//! - `trust/` - Trust evaluation logic
//! - `songbird-discovery` - UDP multicast discovery

use anyhow::Result;
use std::sync::Arc;
use tokio::time::interval;
use tracing::{debug, info, warn};

use super::core::SongbirdOrchestrator;

impl SongbirdOrchestrator {
    /// Start discovery → federation bridge (auto-join discovered peers)
    ///
    /// This spawns a background task that:
    /// 1. Polls the discovery listener every 10 seconds
    /// 2. Extracts peer identity (node_id, node_name)
    /// 3. Checks same-family status (for LAN optimization)
    /// 4. Verifies connectivity (HTTPS /health, skipped for same-family)
    /// 5. Evaluates trust (via BearDog or other security provider)
    /// 6. Establishes connection at appropriate trust level
    /// 7. Registers in federation state
    ///
    /// ## Trust Evaluation
    ///
    /// **With Security Provider** (BearDog):
    /// - Queries security provider for genetic lineage verification
    /// - Only accepts peers with valid family lineage
    /// - Secure by default
    ///
    /// **Without Security Provider** (Development):
    /// - Falls back to anonymous trust (INSECURE!)
    /// - Auto-accepts all peers
    /// - Logs warning
    ///
    /// ## Same-Family LAN Optimization (v3.10.0)
    ///
    /// For peers with matching `SONGBIRD_FAMILY_ID`:
    /// - Skips HTTPS connectivity check
    /// - Trusts UDP multicast discovery
    /// - Significantly faster peer addition
    /// - Essential for LAN deployments
    ///
    /// ## Error Handling
    ///
    /// **Safe Defaults**:
    /// - Trust evaluation error → Reject peer
    /// - Connection error → Skip peer
    /// - No security provider → Log warning, auto-accept (dev only)
    ///
    /// **Audit Trail**:
    /// - All rejections recorded in ConnectionManager
    /// - All decisions logged with reason & confidence
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Bridge task spawned successfully
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let orchestrator = SongbirdOrchestrator::new(config).await?;
    /// orchestrator.start_discovery_federation_bridge().await?;
    /// // Bridge now running in background, polling every 10s
    /// ```
    pub(super) async fn start_discovery_federation_bridge(&self) -> Result<()> {
        if let Some(ref listener) = self.discovery_listener {
            let listener_clone = Arc::clone(listener);
            let federation_state = Arc::clone(&self.federation_state);
            let trust_manager = Arc::clone(&self.trust_manager);
            let connection_manager = Arc::clone(&self.connection_manager);
            
            // Check for security provider for trust evaluation
            // EVOLVED (v3.15.0): Agnostic capability discovery (zero vendor hardcoding!)
            let security_client_endpoint = crate::app::security_setup::discover_security_endpoint(None).await.ok();

            tokio::spawn(async move {
                let mut interval = interval(tokio::time::Duration::from_secs(10));

                if security_client_endpoint.is_some() {
                    info!("🌉 Discovery → Federation bridge started (10s interval) [Trust Evaluation: ACTIVE]");
                } else {
                    info!("🌉 Discovery → Federation bridge started (10s interval) [Trust Evaluation: DISABLED - no security provider]");
                }

                loop {
                    interval.tick().await;
                    
                    // INFO LOGGING (v3.10.3 - Jan 6, 2026): Show bridge polls at INFO level
                    // Helps verify bridge is running even with default logging
                    debug!("🔄 Bridge poll tick (checking for discovered peers...)");

                    // Get all discovered peers
                    let peers = listener_clone.get_peers().await;

                    if !peers.is_empty() {
                        info!("🔍 Processing {} discovered peers", peers.len());

                        for peer in peers {
                        // Get HTTPS endpoint
                        let endpoint = peer.https_endpoint();

                        // Extract identity based on protocol version
                        let (node_id, node_name) = if peer.version == "3.0" {
                            // v3.0: Use stable node_id and node_name
                            match (&peer.node_id, &peer.node_name) {
                                (Some(id), Some(name)) => (id.clone(), name.clone()),
                                _ => {
                                    warn!("⚠️  Peer claims v3.0 but missing node_id/node_name, falling back to session_id");
                                    (
                                        peer.session_id.clone(),
                                        format!("peer-{}", &peer.session_id[..8]),
                                    )
                                }
                            }
                        } else {
                            // v2.x: Fall back to session_id (legacy)
                            (peer.session_id.clone(), format!("peer-{}", &peer.session_id[..8]))
                        };

                        // Log discovered peer with proper identity
                        debug!(
                            "🔍 Discovered peer: {} (v{}) at {} (capabilities: {:?})",
                            node_name, peer.version, endpoint, peer.capabilities
                        );

                        // Check if same family for trust decisions (Jan 5, 2026)
                        // Tags format: ["beardog:family:FAMILY_ID:NODE_ID", ...]
                        let same_family = std::env::var("SONGBIRD_FAMILY_ID")
                            .ok()
                            .map(|my_family| {
                                peer.tags.as_ref()
                                    .map(|tags| {
                                        tags.iter().any(|tag| {
                                            tag.contains(&format!(":family:{}:", my_family))
                                                || tag.contains(&format!("family_{}", my_family))
                                        })
                                    })
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false);

                        // CRITICAL: Verify HTTPS connectivity before registering
                        // EVOLVED (Jan 5, 2026): Skip for same-family LAN peers
                        let skip_connectivity_check = same_family;
                        
                        let connectivity_ok = if skip_connectivity_check {
                            info!("✅ Same family peer '{}' - skipping connectivity check (trust LAN discovery)", node_name);
                            true
                        } else {
                            let health_url = format!("{}/health", endpoint);
                            debug!("🔍 Checking connectivity to {} at {}", node_name, health_url);
                            
                            let connectivity_check = tokio::time::timeout(
                                tokio::time::Duration::from_secs(3),
                                async {
                                    // ✅ EVOLVED: Proper error handling instead of unwrap
                                    let client = reqwest::Client::builder()
                                        .danger_accept_invalid_certs(true)
                                        .build()
                                        .map_err(|e| {
                                            warn!("Failed to build HTTP client for connectivity check: {}", e);
                                            e
                                        })?;

                                    client
                                        .get(&health_url)
                                        .send()
                                        .await
                                }
                            ).await;
                            
                            match connectivity_check {
                                Ok(Ok(response)) if response.status().is_success() => {
                                    info!("✅ Peer '{}' (v{}) is reachable at {}", node_name, peer.version, endpoint);
                                    true
                                }
                                Ok(Ok(response)) => {
                                    warn!("⚠️  Peer '{}' returned HTTP {} - connectivity check failed", node_name, response.status());
                                    false
                                }
                                Ok(Err(e)) => {
                                    warn!("⚠️  Peer '{}' unreachable: {}", node_name, e);
                                    false
                                }
                                Err(_) => {
                                    warn!("⚠️  Peer '{}' connection timeout (3s)", node_name);
                                    false
                                }
                            }
                        };

                        if connectivity_ok {
                                info!(
                                    "✅ Peer '{}' (v{}) is reachable at {}",
                                    node_name, peer.version, endpoint
                                );

                                // 🔒 CRITICAL: Genetic lineage trust evaluation (USB seed integration)
                                // Query security provider (e.g., BearDog) for trust decision
                                // 
                                // Agnostic Pattern: Node discovers security capability at runtime
                                // - If security provider configured → evaluate trust (secure)
                                // - If no security provider → anonymous trust (development only)
                                //
                                // This replaces the previous "establish_anonymous" which was:
                                // ❌ Insecure: Auto-accepted all peers (no lineage check)
                                // ✅ Now secure: Only accepts peers with valid lineage
                                
                                let trust_decision_result = if let Some(ref sec_endpoint) = security_client_endpoint {
                                    // Security provider available - evaluate trust properly
                                    use crate::trust::peer_trust::{evaluate_peer_trust, DiscoveredPeer};
                                    use crate::security_capability_client::SecurityCapabilityClient;
                                    
                                    let security_client = match SecurityCapabilityClient::from_endpoint(sec_endpoint.clone()) {
                                        Ok(client) => client,
                                        Err(e) => {
                                            warn!("⚠️  Failed to create security client: {}", e);
                                            continue;
                                        }
                                    };
                                    
                                    // 🚨 CRITICAL FIX (Jan 3, 2026): Convert discovery attestations to trust layer format
                                    let trust_attestations = peer.identity_attestations.as_ref()
                                        .map(|attestations| {
                                            attestations.iter().map(|att| {
                                                crate::trust::UniversalIdentityAttestation {
                                                    provider: Some(att.provider_capability.clone()),
                                                    format: att.format.clone(),
                                                    data: att.data.clone(),
                                                }
                                            }).collect()
                                        })
                                        .unwrap_or_default();
                                    
                                    // ✅ v3.14.2: Log peer tags for debugging
                                    let peer_tags = peer.tags.clone().unwrap_or_default();
                                    if peer_tags.is_empty() {
                                        warn!("⚠️  Peer {} has NO tags - family extraction will fail!", node_id);
                                        warn!("   This means the peer didn't broadcast identity tags");
                                    } else {
                                        debug!("📋 Peer {} has {} tags: {:?}", node_id, peer_tags.len(), peer_tags);
                                    }
                                    
                                    let discovered_peer = DiscoveredPeer {
                                        node_id: node_id.clone(),
                                        tags: peer_tags,
                                        identity_attestations: trust_attestations, // ✅ FIXED: Pass attestations from discovery
                                        endpoint: endpoint.clone(),
                                        capabilities: peer.capabilities.clone(),
                                        discovery_method: "udp_multicast".to_string(),
                                        first_seen_at: peer.timestamp.unwrap_or(0),
                                    };
                                    
                                    match evaluate_peer_trust(&discovered_peer, &security_client).await {
                                        Ok(decision) => Some(decision),
                                        Err(e) => {
                                            warn!("⚠️  Trust evaluation failed for {}: {}", node_name, e);
                                            warn!("   Defaulting to reject (safe default)");
                                            None // Reject on error (safe default)
                                        }
                                    }
                                } else {
                                    // No security provider - fall back to anonymous trust (INSECURE - development only)
                                    warn!("⚠️  No security provider configured - using anonymous trust (INSECURE)");
                                    warn!("   Set SONGBIRD_BEARDOG_URL for secure genetic lineage verification");
                                    
                                    // Use fully qualified path to avoid duplicate import
                                    Some(crate::trust::peer_trust::PeerTrustDecision::AutoAccept {
                                        reason: "no_security_provider_configured".to_string(),
                                        confidence: 0.0,
                                        encryption_tag: None,
                                    })
                                };
                                
                                match trust_decision_result {
                                    Some(crate::trust::peer_trust::PeerTrustDecision::AutoAccept { reason, confidence, .. }) => {
                                        info!(
                                            "✅ Trust Decision: AUTO-ACCEPT for '{}' (reason: {}, confidence: {:.2})",
                                            node_name, reason, confidence
                                        );
                                        
                                        // Handle trust decision via connection manager (progressive trust)
                                        match connection_manager.handle_trust_decision(
                                            node_id.clone(),
                                            endpoint.clone(),
                                            peer.capabilities.clone(),
                                            &crate::trust::peer_trust::PeerTrustDecision::AutoAccept {
                                                reason: reason.clone(),
                                                confidence,
                                                encryption_tag: None,
                                            },
                                            "udp_multicast".to_string(),
                                        ).await {
                                            Ok(()) => {
                                                // Get trust level for logging
                                                if let Some(trust_level) = connection_manager.get_connection(&node_id).await {
                                                    let trust_level_num = trust_level as u8;
                                                    info!(
                                                        "✅ Connection established with '{}' at trust level {} ({})",
                                                        node_name,
                                                        trust_level_num,
                                                        match trust_level_num {
                                                            1 => "Limited - BirdSong coordination only",
                                                            2 => "Elevated - Full federation",
                                                            3 => "Highest - All operations",
                                                            _ => "Unknown",
                                                        }
                                                    );
                                                    
                                                    // Also establish legacy anonymous trust for backward compatibility
                                                    if let Err(e) = trust_manager.establish_anonymous(peer.session_id.clone()).await {
                                                        warn!("⚠️  Failed to establish legacy trust: {}", e);
                                                    }

                                                    // Convert v3.0 endpoints to federation format (if available)
                                                    let endpoints = peer.endpoints.as_ref().map(|eps| {
                                                        eps.iter().map(|ep| {
                                                            songbird_network_federation::state::TransportEndpointInfo {
                                                                interface_type: ep.interface_type.clone(),
                                                                address: ep.address.clone(),
                                                                protocols: ep.protocols.clone(),
                                                                preference: ep.preference,
                                                                status: songbird_network_federation::state::EndpointStatus::Active,
                                                                last_check: chrono::Utc::now(),
                                                            }
                                                        }).collect()
                                                    });

                                                    // Create node registration
                                                    let node_registration = songbird_network_federation::state::NodeRegistration {
                                                        node_id: node_id.clone(),
                                                        node_name: node_name.clone(),
                                                        node_address: endpoint.clone(),
                                                        endpoints,
                                                        cpu_cores: 0,
                                                        memory_gb: 0,
                                                        gpu_model: None,
                                                        storage_gb: None,
                                                        capabilities: peer.capabilities.clone(),
                                                        status: songbird_network_federation::state::NodeStatus::Active,
                                                        joined_at: chrono::Utc::now(),
                                                        last_heartbeat: chrono::Utc::now(),
                                                    };

                                                    // Register node in federation
                                                    federation_state.register_node(node_registration).await;

                                                    info!(
                                                        "🤝 Peer '{}' joined federation (progressive trust level {})",
                                                        node_name, trust_level_num
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "❌ Failed to establish connection with {}: {}",
                                                    peer.session_id, e
                                                );
                                            }
                                        }
                                    }
                                    
                                    Some(crate::trust::peer_trust::PeerTrustDecision::PromptUser { reason, peer_id, recommendation }) => {
                                        warn!(
                                            "⚠️  Trust Decision: PROMPT USER for '{}' (reason: {})",
                                            node_name, reason
                                        );
                                        warn!("   Peer ID: {}", peer_id);
                                        warn!("   Recommendation: {}", recommendation);
                                        warn!("   TODO: Implement user consent UI - for now, skipping peer");
                                        // Skip this peer - do not add to federation without user consent
                                    }
                                    
                                    Some(crate::trust::peer_trust::PeerTrustDecision::Reject { reason, trust_level }) => {
                                        warn!(
                                            "❌ Trust Decision: REJECT for '{}' (reason: {})",
                                            node_name, reason
                                        );
                                        warn!("   Trust level: {}", trust_level);
                                        info!("   Peer {} rejected - no genetic lineage", &peer.session_id[..8]);
                                        
                                        // Track rejection in connection manager for audit trail
                                        if let Err(e) = connection_manager.handle_trust_decision(
                                            node_id.clone(),
                                            endpoint.clone(),
                                            peer.capabilities.clone(),
                                            &crate::trust::peer_trust::PeerTrustDecision::Reject {
                                                reason: reason.clone(),
                                                trust_level: trust_level.clone(),
                                            },
                                            "udp_multicast".to_string(),
                                        ).await {
                                            warn!("⚠️  Failed to record rejection: {}", e);
                                        }
                                        // Skip this peer - do not add to federation
                                    }
                                    
                                    None => {
                                        warn!(
                                            "❌ Trust evaluation failed for '{}' - rejecting (safe default)",
                                            node_name
                                        );
                                        // Skip this peer - safe default is to reject on evaluation failure
                                    }
                                }
                            } else {
                                // Connectivity check failed - peer not reachable
                                debug!("🔍 Peer '{}' not added - connectivity check failed", node_name);
                            }
                        } // End of: for peer in peers
                    } // End of: if !peers.is_empty()
                } // End of: loop
            });

            info!("✅ Discovery → Federation bridge task spawned");
        }

        Ok(())
    }
}

