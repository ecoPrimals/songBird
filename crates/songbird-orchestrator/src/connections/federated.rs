//! Federated Connection (Trust Level 2)
//!
//! For peers approved by a human for full federation.
//!
//! ## Philosophy
//!
//! Human approval grants full federation capabilities.
//!
//! ## Allowed Operations
//!
//! - All Level 1 operations (BirdSong, coordination, health)
//! - `federation/*` - Full federation
//! - `data/read` - Read-only data access
//!
//! ## Denied Operations
//!
//! - `data/write` - No data modification
//! - `commands/sensitive` - No sensitive commands
//! - `keys/*` - No key access

use super::{check_operation_allowed, PeerConnection};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use songbird_types::TrustLevel;
use std::time::Duration;
use tracing::{debug, warn};

/// Federated connection for human-approved peers (Level 2)
///
/// Allows full federation and read-only data access.
pub struct FederatedConnection {
    peer_id: String,
    endpoint: String,
    allowed_capabilities: Vec<String>,
    denied_capabilities: Vec<String>,
    http_client: Client,
}

impl FederatedConnection {
    /// Create a new federated connection
    pub fn new(
        peer_id: String,
        endpoint: String,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            peer_id,
            endpoint,
            allowed_capabilities,
            denied_capabilities: TrustLevel::Elevated.default_denied_capabilities(),
            http_client,
        })
    }
    
    /// Create with default Level 2 capabilities
    pub fn with_defaults(peer_id: String, endpoint: String) -> Result<Self> {
        Self::new(
            peer_id,
            endpoint,
            TrustLevel::Elevated.default_allowed_capabilities(),
        )
    }
}

#[async_trait]
impl PeerConnection for FederatedConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Elevated
    }
    
    fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }
    
    fn denied_capabilities(&self) -> &[String] {
        &self.denied_capabilities
    }
    
    fn is_operation_allowed(&self, operation: &str) -> bool {
        check_operation_allowed(
            operation,
            &self.allowed_capabilities,
            &self.denied_capabilities,
        )
    }
    
    async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        // Enforce capability restrictions
        if !self.is_operation_allowed(operation) {
            warn!(
                "🔒 Operation '{}' denied for peer '{}' at trust level 2 (Elevated)",
                operation, self.peer_id
            );
            return Err(anyhow!(
                "Operation '{}' not allowed at trust level 2 (Elevated). \
                 Allowed: {:?}. \
                 To enable sensitive operations, elevate trust to level 3 (Highest) via human entropy.",
                operation,
                self.allowed_capabilities
            ));
        }
        
        debug!(
            "✅ Calling federated operation '{}' on peer '{}'",
            operation, self.peer_id
        );
        
        // Make HTTP call
        let url = format!("{}/api/v1/{}", self.endpoint, operation);
        
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context(format!("Failed to call operation '{}' on peer '{}'", operation, self.peer_id))?;
        
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
        
        let result = response.json::<Value>().await
            .context("Failed to parse response from peer")?;
        
        debug!("✅ Federated operation '{}' succeeded on peer '{}'", operation, self.peer_id);
        Ok(result)
    }
    
    fn peer_id(&self) -> &str {
        &self.peer_id
    }
    
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
    
    async fn close(&self) -> Result<()> {
        debug!("Closing federated connection to peer '{}'", self.peer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_allows_level1_plus_federation() {
        let conn = FederatedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        ).unwrap();
        
        // Level 1 operations
        assert!(conn.is_operation_allowed("birdsong/sync"));
        assert!(conn.is_operation_allowed("health"));
        
        // Level 2 operations
        assert!(conn.is_operation_allowed("federation/join"));
        assert!(conn.is_operation_allowed("data/read"));
    }

    #[test]
    fn test_federated_denies_sensitive() {
        let conn = FederatedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        ).unwrap();
        
        assert!(!conn.is_operation_allowed("data/write"));
        assert!(!conn.is_operation_allowed("commands/sensitive"));
        assert!(!conn.is_operation_allowed("keys/access"));
    }

    #[test]
    fn test_trust_level() {
        let conn = FederatedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        ).unwrap();
        
        assert_eq!(conn.trust_level(), TrustLevel::Elevated);
    }
}

