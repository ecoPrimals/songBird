// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Connection Management with Progressive Trust
//!
//! This module implements connection types for different trust levels:
//! - **Limited**: `BirdSong` coordination only (Level 1)
//! - **Federated**: Full federation (Level 2)
//! - **`FullTrust`**: All operations (Level 3)
//!
//! Each connection type enforces capability restrictions appropriate to its trust level.

use anyhow::Result;
use serde_json::Value;
use songbird_types::TrustLevel;

pub mod federated;
pub mod full_trust;
pub mod limited;

pub mod federated_btsp;
pub mod full_trust_btsp;
pub mod limited_btsp;

pub use federated::FederatedConnection;
pub use full_trust::FullTrustConnection;
pub use limited::LimitedConnection;

pub use federated_btsp::FederatedBtspConnection;
pub use full_trust_btsp::FullTrustBtspConnection;
pub use limited_btsp::LimitedBtspConnection;

/// Contract for peer connections with trust-based capability enforcement.
///
/// Static dispatch only — no `dyn PeerConnection`. All dispatch goes through
/// the `Connection` enum which forwards to concrete types via match arms.
pub trait PeerConnection: Send + Sync {
    fn trust_level(&self) -> TrustLevel;
    fn allowed_capabilities(&self) -> &[String];
    fn denied_capabilities(&self) -> &[String];
    fn is_operation_allowed(&self, operation: &str) -> bool;
    fn call(
        &self,
        operation: &str,
        request: Value,
    ) -> impl std::future::Future<Output = Result<Value>> + Send;
    fn peer_id(&self) -> &str;
    fn endpoint(&self) -> &str;
    fn close(&self) -> impl std::future::Future<Output = Result<()>> + Send;
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

/// Dispatch macro — forwards a method call to the inner connection type.
macro_rules! dispatch {
    ($self:ident, $method:ident $(, $arg:expr)*) => {
        match $self {
            Self::Limited(c) => c.$method($($arg),*),
            Self::Federated(c) => c.$method($($arg),*),
            Self::FullTrust(c) => c.$method($($arg),*),
            Self::LimitedBtsp(c) => c.$method($($arg),*),
            Self::FederatedBtsp(c) => c.$method($($arg),*),
            Self::FullTrustBtsp(c) => c.$method($($arg),*),
        }
    };
}

impl Connection {
    #[must_use]
    pub fn trust_level(&self) -> TrustLevel {
        dispatch!(self, trust_level)
    }

    #[must_use]
    pub fn allowed_capabilities(&self) -> &[String] {
        dispatch!(self, allowed_capabilities)
    }

    #[must_use]
    pub fn denied_capabilities(&self) -> &[String] {
        dispatch!(self, denied_capabilities)
    }

    #[must_use]
    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        dispatch!(self, is_operation_allowed, operation)
    }

    #[must_use]
    pub fn peer_id(&self) -> &str {
        dispatch!(self, peer_id)
    }

    #[must_use]
    pub fn endpoint(&self) -> &str {
        dispatch!(self, endpoint)
    }

    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the operation is denied.
    pub async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        match self {
            Self::Limited(c) => c.call(operation, request).await,
            Self::Federated(c) => c.call(operation, request).await,
            Self::FullTrust(c) => c.call(operation, request).await,
            Self::LimitedBtsp(c) => c.call(operation, request).await,
            Self::FederatedBtsp(c) => c.call(operation, request).await,
            Self::FullTrustBtsp(c) => c.call(operation, request).await,
        }
    }

    /// # Errors
    ///
    /// Returns an error if the close operation fails.
    pub async fn close(&self) -> Result<()> {
        match self {
            Self::Limited(c) => c.close().await,
            Self::Federated(c) => c.close().await,
            Self::FullTrust(c) => c.close().await,
            Self::LimitedBtsp(c) => c.close().await,
            Self::FederatedBtsp(c) => c.close().await,
            Self::FullTrustBtsp(c) => c.close().await,
        }
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
