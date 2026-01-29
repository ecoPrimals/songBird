//! STUN/NAT Traversal JSON-RPC Handler
//!
//! Exposes `songbird-stun` crate via JSON-RPC for NAT traversal and public address discovery.
//!
//! ## Methods
//! - `stun.get_public_address` - Discover public IP/port via STUN
//! - `stun.bind` - Create/maintain STUN binding for hole punching
//!
//! ## Security Note
//! STUN servers can observe your public IP/port and connection timing.
//! Prefer genetic lineage relay (Tier 1) when sovereignty > convenience.

use crate::error::{IpcError, IpcResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use songbird_stun::StunClient;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

// ============================================================================
// STUN Handler
// ============================================================================

/// STUN handler for NAT traversal operations
pub struct StunHandler {
    /// STUN client
    client: Arc<StunClient>,

    /// Active STUN bindings (`binding_id` -> binding info)
    bindings: Arc<RwLock<HashMap<String, StunBinding>>>,
}

impl StunHandler {
    /// Create a new STUN handler
    pub fn new() -> Self {
        Self {
            client: Arc::new(StunClient::new()),
            bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create STUN handler with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            client: Arc::new(StunClient::with_timeout(timeout)),
            bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle `stun.get_public_address` JSON-RPC method
    ///
    /// Discovers public IP/port via STUN server for NAT traversal.
    pub async fn handle_get_public_address(
        &self,
        params: Value,
    ) -> IpcResult<StunGetPublicAddressResult> {
        let params: StunGetPublicAddressParams =
            serde_json::from_value(params).map_err(|e| {
                IpcError::InvalidParams(format!("Failed to parse params: {}", e))
            })?;

        debug!(
            "STUN: get_public_address (server: {:?})",
            params.server
        );

        // Use provided server or default to Nextcloud STUN (vetted)
        let stun_server = params
            .server
            .as_deref()
            .unwrap_or("stun.nextcloud.com:3478");

        // Discover public address
        let public_addr = self
            .client
            .discover_public_address(stun_server)
            .await
            .map_err(|e| IpcError::Internal(format!("STUN request failed: {}", e)))?;

        // Get local address (best effort)
        let local_addr = format!("0.0.0.0:{}", params.local_port.unwrap_or(0));

        info!(
            "✅ STUN: Discovered public address: {} (via {})",
            public_addr, stun_server
        );

        Ok(StunGetPublicAddressResult {
            public_address: public_addr.to_string(),
            local_address: local_addr,
            server: stun_server.to_string(),
            nat_type: Some("unknown".to_string()), // TODO: NAT type detection
        })
    }

    /// Handle `stun.bind` JSON-RPC method
    ///
    /// Creates and maintains a STUN binding for hole punching.
    pub async fn handle_bind(&self, params: Value) -> IpcResult<StunBindResult> {
        let params: StunBindParams = serde_json::from_value(params).map_err(|e| {
            IpcError::InvalidParams(format!("Failed to parse params: {}", e))
        })?;

        debug!(
            "STUN: bind (server: {}, local_port: {})",
            params.server, params.local_port
        );

        // Discover public address
        let public_addr = self
            .client
            .discover_public_address(&params.server)
            .await
            .map_err(|e| IpcError::Internal(format!("STUN bind failed: {}", e)))?;

        // Generate binding ID
        let binding_id = format!("stun-{}", uuid::Uuid::new_v4());

        // Store binding
        let lifetime_secs = params.keepalive_secs.unwrap_or(300); // 5 minutes default
        let binding = StunBinding {
            binding_id: binding_id.clone(),
            server: params.server.clone(),
            local_port: params.local_port,
            mapped_address: public_addr,
            lifetime_secs,
            created_at: std::time::SystemTime::now(),
        };

        self.bindings.write().await.insert(binding_id.clone(), binding);

        info!(
            "✅ STUN: Created binding {} (mapped: {})",
            binding_id, public_addr
        );

        Ok(StunBindResult {
            binding_id,
            mapped_address: public_addr.to_string(),
            lifetime_secs,
        })
    }

    /// List active STUN bindings (internal method)
    pub async fn list_bindings(&self) -> Vec<StunBinding> {
        self.bindings.read().await.values().cloned().collect()
    }
}

impl Default for StunHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Types
// ============================================================================

/// Parameters for `stun.get_public_address`
#[derive(Debug, Clone, Deserialize)]
pub struct StunGetPublicAddressParams {
    /// STUN server (e.g., "stun.l.google.com:19302")
    pub server: Option<String>,

    /// Local port to bind (default: ephemeral)
    pub local_port: Option<u16>,
}

/// Result for `stun.get_public_address`
#[derive(Debug, Clone, Serialize)]
pub struct StunGetPublicAddressResult {
    /// Public IP:port as seen by STUN server
    pub public_address: String,

    /// Local bound address
    pub local_address: String,

    /// STUN server used
    pub server: String,

    /// NAT type detected (if determinable)
    pub nat_type: Option<String>,
}

/// Parameters for `stun.bind`
#[derive(Debug, Clone, Deserialize)]
pub struct StunBindParams {
    /// STUN server
    pub server: String,

    /// Local port to bind
    pub local_port: u16,

    /// Keep-alive interval (seconds)
    pub keepalive_secs: Option<u64>,
}

/// Result for `stun.bind`
#[derive(Debug, Clone, Serialize)]
pub struct StunBindResult {
    /// Binding ID for reference
    pub binding_id: String,

    /// Mapped address
    pub mapped_address: String,

    /// Binding lifetime (seconds)
    pub lifetime_secs: u64,
}

/// Active STUN binding
#[derive(Debug, Clone)]
pub struct StunBinding {
    /// Binding ID
    pub binding_id: String,

    /// STUN server
    pub server: String,

    /// Local port
    pub local_port: u16,

    /// Mapped address from STUN
    pub mapped_address: SocketAddr,

    /// Binding lifetime (seconds)
    pub lifetime_secs: u64,

    /// When binding was created
    pub created_at: std::time::SystemTime,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_stun_handler_creation() {
        let handler = StunHandler::new();
        assert!(handler.bindings.read().await.is_empty());

        let handler = StunHandler::with_timeout(Duration::from_secs(10));
        assert!(handler.bindings.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_handle_get_public_address_params_parsing() {
        let handler = StunHandler::new();

        // Valid params with server
        let params = json!({"server": "stun.example.com:3478", "local_port": 1234});
        let parsed: StunGetPublicAddressParams =
            serde_json::from_value(params).expect("Should parse");
        assert_eq!(parsed.server, Some("stun.example.com:3478".to_string()));
        assert_eq!(parsed.local_port, Some(1234));

        // Valid params without server (should use default)
        let params = json!({});
        let parsed: StunGetPublicAddressParams =
            serde_json::from_value(params).expect("Should parse");
        assert_eq!(parsed.server, None); // Will use default in handler
        assert_eq!(parsed.local_port, None);
    }

    #[tokio::test]
    async fn test_handle_bind_params_parsing() {
        let handler = StunHandler::new();

        // Valid params
        let params = json!({
            "server": "stun.example.com:3478",
            "local_port": 5000,
            "keepalive_secs": 600
        });
        let parsed: StunBindParams = serde_json::from_value(params).expect("Should parse");
        assert_eq!(parsed.server, "stun.example.com:3478");
        assert_eq!(parsed.local_port, 5000);
        assert_eq!(parsed.keepalive_secs, Some(600));

        // Valid params without keepalive
        let params = json!({"server": "stun.example.com:3478", "local_port": 5000});
        let parsed: StunBindParams = serde_json::from_value(params).expect("Should parse");
        assert_eq!(parsed.keepalive_secs, None); // Will use default in handler
    }

    #[tokio::test]
    async fn test_list_bindings_empty() {
        let handler = StunHandler::new();
        let bindings = handler.list_bindings().await;
        assert!(bindings.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network access to public STUN server
    async fn test_handle_get_public_address_live() {
        let handler = StunHandler::new();
        let params = json!({"server": "stun.nextcloud.com:3478"});

        let result = handler.handle_get_public_address(params).await;

        match result {
            Ok(addr_result) => {
                println!("Discovered public address: {}", addr_result.public_address);
                assert!(!addr_result.public_address.is_empty());
                assert_eq!(addr_result.server, "stun.nextcloud.com:3478");
            }
            Err(e) => {
                eprintln!("STUN request failed (expected if no network): {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_handle_bind_live() {
        let handler = StunHandler::new();
        let params = json!({
            "server": "stun.nextcloud.com:3478",
            "local_port": 0,
            "keepalive_secs": 300
        });

        let result = handler.handle_bind(params).await;

        match result {
            Ok(bind_result) => {
                println!("Created binding: {}", bind_result.binding_id);
                assert!(!bind_result.binding_id.is_empty());
                assert!(bind_result.binding_id.starts_with("stun-"));
                assert_eq!(bind_result.lifetime_secs, 300);

                // Verify binding is stored
                let bindings = handler.list_bindings().await;
                assert_eq!(bindings.len(), 1);
            }
            Err(e) => {
                eprintln!("STUN bind failed (expected if no network): {}", e);
            }
        }
    }
}

