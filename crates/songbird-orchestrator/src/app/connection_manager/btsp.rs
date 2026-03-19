//! BTSP connection factory and client management

use crate::btsp_client::BtspClient;
use crate::connections::{
    Connection, FederatedBtspConnection, FullTrustBtspConnection, LimitedBtspConnection,
};
use anyhow::Result;
use songbird_types::TrustLevel;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::{debug, info};

/// BTSP Connection Factory - Creates encrypted P2P connections
///
/// **Modern Pattern** (v3.21.0):
/// - Lazy initialization via `OnceCell`
/// - Thread-safe, async-aware
/// - Protocol capability detection
/// - Graceful fallback to HTTP
pub struct BtspConnectionFactory {
    /// Lazy-initialized BTSP client (thread-safe, async-aware)
    ///
    /// **Modern Rust**: `OnceCell` for lazy initialization
    /// - Initialize once on first use
    /// - Thread-safe without blocking
    /// - Async-friendly
    client: Arc<OnceCell<Arc<BtspClient>>>,
}

impl BtspConnectionFactory {
    /// Create new BTSP connection factory
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Arc::new(OnceCell::new()),
        }
    }

    /// Get or initialize BTSP client (lazy, thread-safe)
    ///
    /// **Modern Async Rust Pattern**:
    /// - `OnceCell::get_or_try_init` ensures single initialization
    /// - No blocking, fully async
    /// - Multiple concurrent calls are safe (first wins, others wait)
    /// - Initialization happens only when needed
    ///
    /// **v3.20.0**: Migrated to Unix socket (Jan 16, 2026)
    /// - Discovers `BearDog` via Unix socket at runtime
    /// - Gracefully handles absence of `BearDog`
    pub async fn get_or_init_client(&self) -> Result<Arc<BtspClient>> {
        self.client
            .get_or_try_init(|| async {
                info!("🔐 Initializing BTSP client (Unix socket)");

                // BtspClient::new() discovers socket path automatically
                let client = BtspClient::new();

                debug!("✅ BTSP client initialized");
                Ok(Arc::new(client))
            })
            .await
            .map(Arc::clone)
    }

    /// Get BTSP client if already initialized
    ///
    /// **Non-blocking**: Returns None if not yet initialized
    pub fn get_client(&self) -> Option<Arc<BtspClient>> {
        self.client.get().map(Arc::clone)
    }

    /// Check if peer supports BTSP protocol
    ///
    /// **Capability-based**: Runtime protocol detection via peer tags
    #[must_use]
    pub fn should_use_btsp(&self, peer_tags: &[String]) -> bool {
        peer_tags.iter().any(|tag| tag == "btsp_enabled" || tag == "btsp")
    }

    /// Create BTSP connection at specified trust level
    ///
    /// **Modern pattern**: Delegates to specific connection types
    /// **Note**: `peer_tags` passed as endpoint was a bug in original, using `peer_tags` now
    pub async fn create_connection(
        &self,
        peer_id: String,
        peer_tags: Vec<String>,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        let client = self.get_or_init_client().await?;

        match trust_level {
            TrustLevel::Highest => {
                let conn = FullTrustBtspConnection::new(peer_id, peer_tags, client).await?;
                Ok(Connection::FullTrustBtsp(conn))
            }
            TrustLevel::Limited => {
                let conn = LimitedBtspConnection::new(peer_id, peer_tags, client, vec![]).await?;
                Ok(Connection::LimitedBtsp(conn))
            }
            TrustLevel::Elevated => {
                let conn = FederatedBtspConnection::new(peer_id, peer_tags, client, vec![]).await?;
                Ok(Connection::FederatedBtsp(conn))
            }
            TrustLevel::None => {
                Err(anyhow::anyhow!("Cannot create BTSP connection at trust level None"))
            }
        }
    }
}

impl Default for BtspConnectionFactory {
    fn default() -> Self {
        Self::new()
    }
}
