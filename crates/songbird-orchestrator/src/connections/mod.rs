//! Connection Management with Progressive Trust
//!
//! This module implements connection types for different trust levels:
//! - **Limited**: BirdSong coordination only (Level 1)
//! - **Federated**: Full federation (Level 2)
//! - **FullTrust**: All operations (Level 3)
//!
//! Each connection type enforces capability restrictions appropriate to its trust level.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use songbird_types::TrustLevel;

pub mod federated;
pub mod full_trust;
pub mod limited;

// v3.18.0: BTSP connection types (port-free, encrypted P2P)
pub mod federated_btsp;
pub mod full_trust_btsp;
pub mod limited_btsp;

pub use federated::FederatedConnection;
pub use full_trust::FullTrustConnection;
pub use limited::LimitedConnection;

// v3.18.0: Export BTSP connections
pub use federated_btsp::FederatedBtspConnection;
pub use full_trust_btsp::FullTrustBtspConnection;
pub use limited_btsp::LimitedBtspConnection;

/// Trait for all peer connections with trust-based capability enforcement
#[async_trait]
pub trait PeerConnection: Send + Sync {
    /// Get the trust level of this connection
    fn trust_level(&self) -> TrustLevel;

    /// Get allowed capabilities for this connection
    fn allowed_capabilities(&self) -> &[String];

    /// Get denied capabilities for this connection
    fn denied_capabilities(&self) -> &[String];

    /// Check if an operation is allowed at this trust level
    fn is_operation_allowed(&self, operation: &str) -> bool;

    /// Call a peer operation (with capability enforcement)
    async fn call(&self, operation: &str, request: Value) -> Result<Value>;

    /// Get peer ID
    fn peer_id(&self) -> &str;

    /// Get endpoint
    fn endpoint(&self) -> &str;

    /// Close the connection
    async fn close(&self) -> Result<()>;
}

/// Enum wrapping all connection types
///
/// v3.18.0: Added BTSP variants for port-free, encrypted P2P communication
pub enum Connection {
    // HTTPS connections (v3.0+)
    Limited(LimitedConnection),
    Federated(FederatedConnection),
    FullTrust(FullTrustConnection),

    // BTSP connections (v3.18.0+) - Port-free, NAT traversal built-in
    LimitedBtsp(LimitedBtspConnection),
    FederatedBtsp(FederatedBtspConnection),
    FullTrustBtsp(FullTrustBtspConnection),
}

impl Connection {
    /// Get the underlying peer connection trait object
    pub fn as_peer_connection(&self) -> &dyn PeerConnection {
        match self {
            // HTTPS connections
            Connection::Limited(conn) => conn,
            Connection::Federated(conn) => conn,
            Connection::FullTrust(conn) => conn,

            // BTSP connections (v3.18.0)
            Connection::LimitedBtsp(conn) => conn,
            Connection::FederatedBtsp(conn) => conn,
            Connection::FullTrustBtsp(conn) => conn,
        }
    }

    /// Get trust level
    pub fn trust_level(&self) -> TrustLevel {
        self.as_peer_connection().trust_level()
    }

    /// Check if operation is allowed
    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        self.as_peer_connection().is_operation_allowed(operation)
    }

    /// Call peer operation
    pub async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        self.as_peer_connection().call(operation, request).await
    }
}

/// Helper function to check if operation matches capability pattern
pub(crate) fn matches_capability_pattern(operation: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true; // Wildcard matches everything
    }

    if let Some(prefix) = pattern.strip_suffix("/*") {
        operation.starts_with(prefix)
    } else {
        operation == pattern
    }
}

/// Helper function to check if operation is allowed given allow/deny lists
pub(crate) fn check_operation_allowed(
    operation: &str,
    allowed: &[String],
    denied: &[String],
) -> bool {
    // Check denied first (explicit deny overrides allow)
    if denied.iter().any(|pattern| matches_capability_pattern(operation, pattern)) {
        return false;
    }

    // Check allowed
    allowed.iter().any(|pattern| matches_capability_pattern(operation, pattern))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_matching() {
        assert!(matches_capability_pattern("anything", "*"));
        assert!(matches_capability_pattern("data/read", "*"));
    }

    #[test]
    fn test_prefix_matching() {
        assert!(matches_capability_pattern("data/read", "data/*"));
        assert!(matches_capability_pattern("data/write", "data/*"));
        assert!(!matches_capability_pattern("commands/exec", "data/*"));
    }

    #[test]
    fn test_exact_matching() {
        assert!(matches_capability_pattern("health", "health"));
        assert!(!matches_capability_pattern("health", "capabilities"));
        assert!(!matches_capability_pattern("health/status", "health"));
    }

    #[test]
    fn test_deny_overrides_allow() {
        let allowed = vec!["data/*".to_string()];
        let denied = vec!["data/sensitive".to_string()];

        assert!(check_operation_allowed("data/read", &allowed, &denied));
        assert!(check_operation_allowed("data/write", &allowed, &denied));
        assert!(!check_operation_allowed("data/sensitive", &allowed, &denied));
    }

    #[test]
    fn test_no_match_denied() {
        let allowed = vec!["birdsong/*".to_string()];
        let denied = vec!["data/*".to_string()];

        assert!(!check_operation_allowed("commands/exec", &allowed, &denied));
        assert!(check_operation_allowed("birdsong/sync", &allowed, &denied));
    }
}
