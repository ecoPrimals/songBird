//! Limited Connection (Trust Level 1)
//!
//! For peers with same genetic family but no human approval.
//!
//! ## Philosophy
//!
//! "Same family = can hear the song, NOT enter the nest"
//!
//! ## Allowed Operations
//!
//! - `discovery` - Capability discovery
//! - `coordination/*` - BirdSong coordination
//! - `birdsong/*` - BirdSong protocol
//! - `health` - Health checks
//! - `capabilities` - Capability queries
//!
//! ## Denied Operations
//!
//! - `data/*` - No data access
//! - `commands/*` - No command execution
//! - `federation/*` - No full federation
//! - `keys/*` - No key access

use super::{check_operation_allowed, PeerConnection};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use songbird_types::TrustLevel;
use std::time::Duration;
use tracing::{debug, warn};

/// Limited connection for same-family peers (Level 1)
///
/// Allows BirdSong coordination only, no data access or full federation.
pub struct LimitedConnection {
    peer_id: String,
    endpoint: String,
    allowed_capabilities: Vec<String>,
    denied_capabilities: Vec<String>,
    http_client: Client,
}

impl LimitedConnection {
    /// Create a new limited connection
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Peer node ID
    /// * `endpoint` - Peer endpoint URL
    /// * `allowed_capabilities` - Capabilities allowed at this level
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::connections::LimitedConnection;
    /// # async fn example() -> anyhow::Result<()> {
    /// let conn = LimitedConnection::new(
    ///     "tower2".to_string(),
    ///     "https://192.168.1.100:8080".to_string(),
    ///     vec!["birdsong/*".to_string(), "health".to_string()],
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        peer_id: String,
        endpoint: String,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            peer_id,
            endpoint,
            allowed_capabilities,
            denied_capabilities: TrustLevel::Limited.default_denied_capabilities(),
            http_client,
        })
    }

    /// Create with default Level 1 capabilities
    pub fn with_defaults(peer_id: String, endpoint: String) -> Result<Self> {
        Self::new(peer_id, endpoint, TrustLevel::Limited.default_allowed_capabilities())
    }
}

#[async_trait]
impl PeerConnection for LimitedConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Limited
    }

    fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }

    fn denied_capabilities(&self) -> &[String] {
        &self.denied_capabilities
    }

    fn is_operation_allowed(&self, operation: &str) -> bool {
        check_operation_allowed(operation, &self.allowed_capabilities, &self.denied_capabilities)
    }

    async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        // Enforce capability restrictions
        if !self.is_operation_allowed(operation) {
            warn!(
                "🔒 Operation '{}' denied for peer '{}' at trust level 1 (Limited)",
                operation, self.peer_id
            );
            return Err(anyhow!(
                "Operation '{}' not allowed at trust level 1 (Limited). \
                 Allowed: {:?}. \
                 To enable this operation, elevate trust to level 2 (Elevated) via user approval.",
                operation,
                self.allowed_capabilities
            ));
        }

        debug!(
            "🎵 Calling limited operation '{}' on peer '{}' (BirdSong)",
            operation, self.peer_id
        );

        // Make HTTP call
        let url = format!("{}/api/v1/{}", self.endpoint, operation);

        let response = self.http_client.post(&url).json(&request).send().await.context(format!(
            "Failed to call operation '{}' on peer '{}'",
            operation, self.peer_id
        ))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Peer '{}' returned error {}: {}",
                self.peer_id,
                status,
                error_body
            ));
        }

        let result =
            response.json::<Value>().await.context("Failed to parse response from peer")?;

        debug!("✅ Limited operation '{}' succeeded on peer '{}'", operation, self.peer_id);
        Ok(result)
    }

    fn peer_id(&self) -> &str {
        &self.peer_id
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn close(&self) -> Result<()> {
        debug!("Closing limited connection to peer '{}'", self.peer_id);
        // HTTP client cleanup happens automatically
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limited_allows_birdsong() {
        let conn = LimitedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert!(conn.is_operation_allowed("birdsong/sync"));
        assert!(conn.is_operation_allowed("coordination/state"));
        assert!(conn.is_operation_allowed("health"));
        assert!(conn.is_operation_allowed("capabilities"));
    }

    #[test]
    fn test_limited_denies_data() {
        let conn = LimitedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert!(!conn.is_operation_allowed("data/read"));
        assert!(!conn.is_operation_allowed("data/write"));
        assert!(!conn.is_operation_allowed("commands/exec"));
        assert!(!conn.is_operation_allowed("federation/join"));
        assert!(!conn.is_operation_allowed("keys/access"));
    }

    #[test]
    fn test_trust_level() {
        let conn = LimitedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert_eq!(conn.trust_level(), TrustLevel::Limited);
    }
}
