//! Full Trust Connection (Trust Level 3)
//!
//! For peers with human entropy added (SoloKey, Phone HSM).
//!
//! ## Philosophy
//!
//! Human entropy enables all operations including sensitive ones.
//!
//! ## Allowed Operations
//!
//! - `*` - Everything (no restrictions)
//!
//! ## Denied Operations
//!
//! - None

use super::PeerConnection;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use songbird_types::TrustLevel;
use std::time::Duration;
use tracing::debug;

/// Full trust connection for peers with human entropy (Level 3)
///
/// Allows all operations with no restrictions.
pub struct FullTrustConnection {
    peer_id: String,
    endpoint: String,
    allowed_capabilities: Vec<String>,
    http_client: Client,
}

impl FullTrustConnection {
    /// Create a new full trust connection
    pub fn new(peer_id: String, endpoint: String) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            peer_id,
            endpoint,
            allowed_capabilities: vec!["*".to_string()],
            http_client,
        })
    }
}

#[async_trait]
impl PeerConnection for FullTrustConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Highest
    }
    
    fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }
    
    fn denied_capabilities(&self) -> &[String] {
        &[] // Nothing denied at highest trust
    }
    
    fn is_operation_allowed(&self, _operation: &str) -> bool {
        true // Everything allowed
    }
    
    async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        debug!(
            "🔓 Calling full-trust operation '{}' on peer '{}' (Level 3)",
            operation, self.peer_id
        );
        
        // Make HTTP call (no capability restrictions)
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
        
        debug!("✅ Full-trust operation '{}' succeeded on peer '{}'", operation, self.peer_id);
        Ok(result)
    }
    
    fn peer_id(&self) -> &str {
        &self.peer_id
    }
    
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
    
    async fn close(&self) -> Result<()> {
        debug!("Closing full-trust connection to peer '{}'", self.peer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_trust_allows_everything() {
        let conn = FullTrustConnection::new(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        ).unwrap();
        
        // Everything allowed
        assert!(conn.is_operation_allowed("data/read"));
        assert!(conn.is_operation_allowed("data/write"));
        assert!(conn.is_operation_allowed("commands/sensitive"));
        assert!(conn.is_operation_allowed("keys/access"));
        assert!(conn.is_operation_allowed("anything/at/all"));
    }

    #[test]
    fn test_trust_level() {
        let conn = FullTrustConnection::new(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        ).unwrap();
        
        assert_eq!(conn.trust_level(), TrustLevel::Highest);
    }
}

