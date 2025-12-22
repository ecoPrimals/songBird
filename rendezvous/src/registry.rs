//! Session Registry
//!
//! Manages ephemeral session registrations without storing IP addresses

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::messages::{
    NetworkContext as MessageNetworkContext, NodeIdentity as MessageNodeIdentity,
};

/// Node identity (re-export for internal use)
pub type NodeIdentity = MessageNodeIdentity;

/// Network context (re-export for internal use)
pub type NetworkContext = MessageNetworkContext;

/// Session information (NO IP ADDRESS!)
#[derive(Debug, Clone)]
pub struct Session {
    /// Ephemeral session ID
    pub session_id: String,

    /// Node identity
    pub identity: NodeIdentity,

    /// Network context (but no IP!)
    pub network_context: NetworkContext,

    /// When session was created (used for session management and expiry calculation)
    #[allow(dead_code)] // Used internally for session validation
    pub created_at: DateTime<Utc>,

    /// When session expires
    pub expires_at: DateTime<Utc>,

    /// Last heartbeat
    pub last_heartbeat: DateTime<Utc>,

    /// Public key for verification
    pub public_key_fingerprint: String,
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session needs heartbeat
    ///
    /// Used internally for connection health monitoring
    #[allow(dead_code)] // Used in connection health monitoring (Phase 4-5)
    pub fn needs_heartbeat(&self) -> bool {
        Utc::now() - self.last_heartbeat > Duration::seconds(30)
    }

    /// Update heartbeat
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        // Extend expiration by 60 seconds
        self.expires_at = Utc::now() + Duration::seconds(60);
    }
}

/// Session registry
pub struct SessionRegistry {
    /// Sessions by ephemeral session ID
    sessions: Arc<RwLock<HashMap<String, Session>>>,

    /// Index: node_id -> session_ids (for multi-interface nodes)
    node_index: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Index: capability -> session_ids
    capability_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SessionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        info!("📋 Initializing session registry");
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            node_index: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new session
    pub async fn register_session(
        &self,
        session_id: String,
        identity: NodeIdentity,
        network_context: NetworkContext,
    ) -> Session {
        let session = Session {
            session_id: session_id.clone(),
            identity: identity.clone(),
            network_context,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(60),
            last_heartbeat: Utc::now(),
            public_key_fingerprint: identity.public_key_fingerprint.clone(),
        };

        // Store session
        self.sessions.write().await.insert(session_id.clone(), session.clone());

        // Update node index
        self.node_index
            .write()
            .await
            .entry(identity.node_id.clone())
            .or_insert_with(Vec::new)
            .push(session_id.clone());

        // Update capability index
        let mut cap_index = self.capability_index.write().await;
        for capability in &identity.capabilities {
            cap_index.entry(capability.clone()).or_insert_with(Vec::new).push(session_id.clone());
        }

        info!(
            "✅ Registered session: {} (node: {}, caps: {:?})",
            &session_id[..8],
            &identity.node_id[..8],
            identity.capabilities
        );

        session
    }

    /// Update heartbeat for session
    pub async fn heartbeat(&self, session_id: &str) -> Option<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.update_heartbeat();
            debug!("💓 Heartbeat: {}", &session_id[..8]);
            Some(())
        } else {
            warn!("⚠️  Heartbeat for unknown session: {}", &session_id[..8]);
            None
        }
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Query peers by capability
    pub async fn query_peers(
        &self,
        required_capabilities: &[String],
        exclude_node_ids: &[String],
        max_results: usize,
    ) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        let cap_index = self.capability_index.read().await;

        // Find sessions with all required capabilities
        let mut matching_sessions: Vec<Session> = Vec::new();

        // Get all sessions that have at least one required capability
        let mut candidate_session_ids: Vec<String> = Vec::new();
        for capability in required_capabilities {
            if let Some(session_ids) = cap_index.get(capability) {
                candidate_session_ids.extend(session_ids.clone());
            }
        }

        // Deduplicate
        candidate_session_ids.sort();
        candidate_session_ids.dedup();

        // Filter candidates
        for session_id in candidate_session_ids {
            if let Some(session) = sessions.get(&session_id) {
                // Skip expired
                if session.is_expired() {
                    continue;
                }

                // Skip excluded nodes
                if exclude_node_ids.contains(&session.identity.node_id) {
                    continue;
                }

                // Check all required capabilities present
                let has_all_caps = required_capabilities
                    .iter()
                    .all(|req_cap| session.identity.capabilities.contains(req_cap));

                if has_all_caps {
                    matching_sessions.push(session.clone());

                    if matching_sessions.len() >= max_results {
                        break;
                    }
                }
            }
        }

        debug!("🔍 Query found {} peers", matching_sessions.len());
        matching_sessions
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let mut node_index = self.node_index.write().await;
        let mut cap_index = self.capability_index.write().await;

        let mut expired_count = 0;

        // Find expired sessions
        let expired_ids: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        // Remove expired sessions
        for session_id in expired_ids {
            if let Some(session) = sessions.remove(&session_id) {
                expired_count += 1;

                // Remove from node index
                if let Some(session_ids) = node_index.get_mut(&session.identity.node_id) {
                    session_ids.retain(|id| id != &session_id);
                    if session_ids.is_empty() {
                        node_index.remove(&session.identity.node_id);
                    }
                }

                // Remove from capability index
                for capability in &session.identity.capabilities {
                    if let Some(session_ids) = cap_index.get_mut(capability) {
                        session_ids.retain(|id| id != &session_id);
                        if session_ids.is_empty() {
                            cap_index.remove(capability);
                        }
                    }
                }

                debug!("🧹 Cleaned up expired session: {}", &session_id[..8]);
            }
        }

        if expired_count > 0 {
            info!("🧹 Cleaned up {} expired sessions", expired_count);
        }

        expired_count
    }

    /// Get statistics
    pub async fn stats(&self) -> RegistryStats {
        let sessions = self.sessions.read().await;
        let nodes = self.node_index.read().await;

        RegistryStats {
            total_sessions: sessions.len(),
            unique_nodes: nodes.len(),
            active_sessions: sessions.values().filter(|s| !s.is_expired()).count(),
        }
    }

    /// Start cleanup loop
    pub async fn start_cleanup_loop(self: Arc<Self>) {
        info!("🧹 Starting session cleanup loop (every 30s)");

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            let expired_count = self.cleanup_expired().await;

            if expired_count > 0 {
                let stats = self.stats().await;
                info!(
                    "📊 Registry stats: {} sessions, {} nodes, {} active",
                    stats.total_sessions, stats.unique_nodes, stats.active_sessions
                );
            }
        }
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_sessions: usize,
    pub unique_nodes: usize,
    pub active_sessions: usize,
}
