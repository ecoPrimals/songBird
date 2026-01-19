//! Trust evaluation and connection establishment

use super::btsp::BtspConnectionFactory;
use super::peer::PeerRegistry;
use anyhow::{anyhow, Result};
use crate::connections::*;
use crate::trust::peer_trust::PeerTrustDecision;
use songbird_types::TrustLevel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Trust Evaluator - Evaluates trust and establishes connections
///
/// **Responsibilities**:
/// - Interpret trust decisions
/// - Map decisions to trust levels
/// - Coordinate connection establishment
/// - Choose connection protocol (BTSP vs HTTP)
pub struct TrustEvaluator;

impl TrustEvaluator {
    /// Create new trust evaluator
    pub fn new() -> Self {
        Self
    }

    /// Handle trust decision from peer discovery
    ///
    /// **v3.21.0**: Matches original API for compatibility
    /// **Modern pattern**: Interprets PeerTrustDecision and delegates
    #[allow(clippy::too_many_arguments)]
    pub async fn handle_decision(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        trust_decision: &PeerTrustDecision,
        discovery_method: String,
        connections: &Arc<RwLock<HashMap<String, Connection>>>,
        peer_registry: &PeerRegistry,
        btsp_factory: &BtspConnectionFactory,
    ) -> Result<()> {
        match trust_decision {
            PeerTrustDecision::AutoAccept {
                reason,
                confidence,
                ..
            } => {
                info!("✅ Auto-accept: {} (confidence: {:.2})", peer_id, confidence);

                // Determine trust level from decision
                let trust_level = if reason.contains("same_genetic_family")
                    || reason.contains("same_family")
                {
                    TrustLevel::Limited
                } else if confidence >= &0.9 {
                    TrustLevel::Elevated
                } else {
                    TrustLevel::Limited
                };

                self.establish_connection(
                    peer_id,
                    endpoint,
                    trust_level,
                    discovery_method,
                    capabilities,
                    peer_tags,
                    connections,
                    peer_registry,
                    btsp_factory,
                )
                .await
            }

            PeerTrustDecision::PromptUser {
                reason,
                ..
            } => {
                warn!("⏳ User prompt needed for peer '{}': {}", peer_id, reason);

                // TODO: Implement user prompt in Phase 6
                // For now, establish limited trust connection
                self.establish_connection(
                    peer_id,
                    endpoint,
                    TrustLevel::Limited,
                    discovery_method,
                    capabilities,
                    peer_tags,
                    connections,
                    peer_registry,
                    btsp_factory,
                )
                .await
            }

            PeerTrustDecision::Reject { reason, .. } => {
                info!("❌ Rejecting peer '{}': {}", peer_id, reason);
                peer_registry.reject(peer_id, reason.clone()).await;
                Ok(())
            }
        }
    }

    /// Establish connection at specified trust level
    ///
    /// **Modern architecture**:
    /// - BTSP-first strategy (encrypted P2P)
    /// - Graceful fallback to HTTP
    /// - Capability-based protocol selection
    #[allow(clippy::too_many_arguments)]
    pub async fn establish_connection(
        &self,
        peer_id: String,
        endpoint: String,
        trust_level: TrustLevel,
        discovery_method: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        connections: &Arc<RwLock<HashMap<String, Connection>>>,
        peer_registry: &PeerRegistry,
        btsp_factory: &BtspConnectionFactory,
    ) -> Result<()> {
        info!(
            "🔗 Establishing connection to '{}' at trust level {} ({})",
            peer_id,
            trust_level.as_u8(),
            trust_level.name()
        );

        // Determine connection protocol (BTSP vs HTTP)
        let connection = if btsp_factory.should_use_btsp(&peer_tags) {
            info!("🔐 Peer '{}' supports BTSP - attempting encrypted tunnel", peer_id);

            // Try BTSP connection
            match btsp_factory
                .create_connection(peer_id.clone(), peer_tags.clone(), trust_level)
                .await
            {
                Ok(conn) => {
                    info!("✅ BTSP connection established for '{}'", peer_id);
                    conn
                }
                Err(e) => {
                    warn!("⚠️  BTSP connection failed: {} - falling back to HTTPS", e);
                    self.create_http_connection(&peer_id, &endpoint, trust_level)?
                }
            }
        } else {
            // Use HTTP fallback
            info!("🌐 Peer '{}' does not support BTSP - using HTTPS", peer_id);
            self.create_http_connection(&peer_id, &endpoint, trust_level)?
        };

        // Register peer metadata
        peer_registry
            .register(
                peer_id.clone(),
                endpoint,
                trust_level,
                discovery_method,
                capabilities,
            )
            .await;

        // Store connection
        connections.write().await.insert(peer_id.clone(), connection);

        info!("✅ Connection established with '{}'", peer_id);
        Ok(())
    }

    /// Create HTTP-based connection (fallback when BTSP unavailable)
    ///
    /// **Modern pattern**: Type-safe connection creation
    fn create_http_connection(
        &self,
        peer_id: &str,
        endpoint: &str,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        match trust_level {
            TrustLevel::None => Err(anyhow!("Cannot create connection at trust level None")),

            TrustLevel::Limited => {
                debug!("🎵 Creating Limited HTTPS connection (BirdSong only)");
                let conn =
                    LimitedConnection::with_defaults(peer_id.to_string(), endpoint.to_string())?;
                Ok(Connection::Limited(conn))
            }

            TrustLevel::Elevated => {
                debug!("✅ Creating Federated HTTPS connection (full federation)");
                let conn =
                    FederatedConnection::with_defaults(peer_id.to_string(), endpoint.to_string())?;
                Ok(Connection::Federated(conn))
            }

            TrustLevel::Highest => {
                debug!("🔓 Creating Full Trust HTTPS connection (all operations)");
                let conn = FullTrustConnection::new(peer_id.to_string(), endpoint.to_string())?;
                Ok(Connection::FullTrust(conn))
            }
        }
    }
}

impl Default for TrustEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

