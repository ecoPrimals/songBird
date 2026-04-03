// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2025 ecoPrimals
//! Standard Methods (biomeOS-compliant)
//!
//! Handlers implementing biomeOS-standard methods required for all primals.
//! These methods provide primal identification and capability discovery.

use serde_json::Value;
use tracing::{info, warn};

use crate::ipc::jsonrpc::JsonRpcError;

/// Handle identity - Return primal identification (biomeOS standard)
///
/// NEW (Feb 4, 2026): Standard method required by biomeOS for all primals.
/// Provides primal name, version, family ID, and capability list.
///
/// **Response Format**:
/// ```json
/// {
///   "primal": "songbird",
///   "version": "0.1.0",
///   "family_id": "default",
///   "capabilities": [
///     "network.broadcast",
///     "network.listen",
///     "network.beacon_exchange",
///     "encrypt_discovery",
///     "decrypt_discovery"
///   ]
/// }
/// ```
pub async fn handle_identity() -> Result<Value, JsonRpcError> {
    info!("🔍 Identity request received");
    
    // Get family ID from canonical env_config (proper env chain, default: "default")
    let family_id = crate::env_config::family_id();
    
    // Songbird's capabilities for biomeOS integration
    let capabilities = vec![
        "network.broadcast",
        "network.listen",
        "network.beacon_exchange",
        "encrypt_discovery",
        "decrypt_discovery",
        "http.post",
        "http.get",
        "http.request",
        "discovery.announce",
        "discovery.query",
        "security.verify",
    ];
    
    Ok(serde_json::json!({
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": family_id,
        "capabilities": capabilities
    }))
}

/// Handle rpc.discover - Return all available RPC methods (biomeOS standard)
///
/// NEW (Feb 4, 2026): Standard method required by biomeOS for capability discovery.
/// Provides introspection of all available JSON-RPC methods with parameters.
///
/// **Response Format**:
/// ```json
/// {
///   "methods": [
///     {
///       "name": "health",
///       "params": [],
///       "description": "Health check"
///     },
///     {
///       "name": "network.beacon_exchange",
///       "params": ["endpoint", "beacon_id", "beacon_seed_encrypted"],
///       "description": "Exchange beacon seeds with peer"
///     }
///   ]
/// }
/// ```
pub async fn handle_rpc_discover() -> Result<Value, JsonRpcError> {
    info!("🔍 RPC method discovery request received");
    
    // Build method list with introspection
    let methods = vec![
        serde_json::json!({
            "name": "health",
            "params": [],
            "description": "Health check with uptime and connectivity"
        }),
        serde_json::json!({
            "name": "identity",
            "params": [],
            "description": "Primal identification and capabilities"
        }),
        serde_json::json!({
            "name": "rpc.discover",
            "params": [],
            "description": "Discover all available RPC methods"
        }),
        serde_json::json!({
            "name": "network.beacon_exchange",
            "params": ["endpoint", "beacon_id", "beacon_seed_encrypted"],
            "description": "Exchange beacon seeds with peer"
        }),
        serde_json::json!({
            "name": "network.broadcast",
            "params": ["payload_encrypted", "ttl", "channel"],
            "description": "Broadcast encrypted message to network"
        }),
        serde_json::json!({
            "name": "network.listen",
            "params": ["channel", "timeout_seconds"],
            "description": "Listen for network broadcasts"
        }),
        serde_json::json!({
            "name": "encrypt_discovery",
            "params": ["payload", "use_beacon_seed"],
            "description": "Encrypt payload for discovery broadcast"
        }),
        serde_json::json!({
            "name": "decrypt_discovery",
            "params": ["encrypted_b64", "known_beacon_seeds"],
            "description": "Decrypt discovery broadcast"
        }),
        serde_json::json!({
            "name": "http.request",
            "params": ["method", "url", "headers", "body"],
            "description": "Delegate HTTP request to external service"
        }),
        serde_json::json!({
            "name": "primal.register",
            "params": ["primal_id", "capabilities", "endpoint", "metadata"],
            "description": "Register primal with capabilities"
        }),
        serde_json::json!({
            "name": "primal.health",
            "params": [],
            "description": "Legacy health check (use 'health' instead)"
        }),
        serde_json::json!({
            "name": "discovery.list_peers",
            "params": [],
            "description": "List all discovered peers"
        }),
        serde_json::json!({
            "name": "discover_capabilities",
            "params": [],
            "description": "Legacy capability discovery (use 'identity' instead)"
        }),
    ];
    
    Ok(serde_json::json!({
        "methods": methods
    }))
}

/// Handle discover_capabilities - Return Songbird's capabilities
///
/// LEGACY (Jan 20, 2026): Upstream integration from biomeOS.
/// DEPRECATED: Use `identity` or `rpc.discover` instead.
/// Kept for backward compatibility with coordination clients that still call this method.
pub async fn handle_discover_capabilities() -> Result<Value, JsonRpcError> {
    warn!("⚠️  discover_capabilities is deprecated, use 'identity' or 'rpc.discover' instead");
    handle_identity().await
}
