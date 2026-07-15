// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Trust evaluation and connection establishment

use super::btsp::BtspConnectionFactory;
use super::peer::PeerRegistry;
use crate::connections::{
    Connection, FederatedConnection, FullTrustConnection, HttpRemoteConnection, LimitedConnection,
};
use crate::trust::peer_trust::PeerTrustDecision;
use anyhow::{Result, anyhow};
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
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Handle trust decision from peer discovery
    ///
    /// **v3.21.0**: Matches original API for compatibility
    /// **Modern pattern**: Interprets `PeerTrustDecision` and delegates
    #[expect(
        clippy::too_many_arguments,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    /// # Errors
    ///
    /// Returns an error if the operation fails.
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
                let trust_level =
                    if reason.contains("same_genetic_family") || reason.contains("same_family") {
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

                // FUTURE (Phase 6): Interactive user consent UI for manual peer approval
                // Current: Automatic trust decisions (safer, no user interaction needed)
                // Future use case: Manual override for sensitive peer connections
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

            PeerTrustDecision::Reject {
                reason,
                ..
            } => {
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
    #[expect(
        clippy::too_many_arguments,
        reason = "intentional pattern; clippy false positive for this API"
    )]
    /// # Errors
    ///
    /// Returns an error if the operation fails.
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
                    warn!(
                        "⚠️  BTSP connection failed for '{}': {} — falling back to plain HTTP",
                        peer_id, e
                    );
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
            .register(peer_id.clone(), endpoint, trust_level, discovery_method, capabilities)
            .await;

        // Store connection
        connections.write().await.insert(peer_id.clone(), connection);

        info!("✅ Connection established with '{}'", peer_id);
        Ok(())
    }

    /// Create connection to peer (fallback when BTSP unavailable)
    ///
    /// For remote network peers (http:// endpoints), uses HTTP JSON-RPC transport.
    /// For local peers (UDS-based), uses local socket connections.
    fn create_http_connection(
        &self,
        peer_id: &str,
        endpoint: &str,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        if trust_level == TrustLevel::None {
            return Err(anyhow!("Cannot create connection at trust level None"));
        }

        let is_remote = endpoint.starts_with("http://") || endpoint.starts_with("https://");

        if is_remote {
            info!(
                "🌐 Creating HTTP remote connection to '{}' at {} (trust: {})",
                peer_id,
                endpoint,
                trust_level.name()
            );
            let conn =
                HttpRemoteConnection::new(peer_id.to_string(), endpoint.to_string(), trust_level);
            return Ok(Connection::HttpRemote(conn));
        }

        match trust_level {
            TrustLevel::None => Err(anyhow!("Cannot create connection at trust level None")),

            TrustLevel::Limited => {
                debug!("🎵 Creating Limited local connection (BirdSong only)");
                let conn =
                    LimitedConnection::with_defaults(peer_id.to_string(), endpoint.to_string())?;
                Ok(Connection::Limited(conn))
            }

            TrustLevel::Elevated => {
                debug!("✅ Creating Federated local connection (full federation)");
                let conn =
                    FederatedConnection::with_defaults(peer_id.to_string(), endpoint.to_string())?;
                Ok(Connection::Federated(conn))
            }

            TrustLevel::Highest => {
                debug!("🔓 Creating Full Trust local connection (all operations)");
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
