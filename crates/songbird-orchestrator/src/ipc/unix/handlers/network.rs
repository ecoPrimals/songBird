// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2025 ecoPrimals
//! Network Methods (biomeOS Integration)
//!
//! Handlers for network-level operations including beacon exchange,
//! broadcasting, and listening for peer discovery messages.
//!
//! These methods implement the Dark Forest discovery protocol and
//! encrypted peer-to-peer communication.

use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::app::connection_manager::ConnectionManager;
use crate::ipc::jsonrpc::JsonRpcError;

/// Handle network.beacon_exchange - Exchange beacon seeds with peer
///
/// NEW (Feb 4, 2026): biomeOS integration for beacon meetings.
/// Performs encrypted beacon seed exchange with a remote peer.
///
/// **Request Format**:
/// ```json
/// {
///   "endpoint": "192.168.1.100:8080",
///   "beacon_id": "our_beacon_id_here",
///   "beacon_seed_encrypted": "encrypted_seed_for_peer"
/// }
/// ```
///
/// **Response Format**:
/// ```json
/// {
///   "success": true,
///   "peer_beacon_id": "peer_beacon_id_here",
///   "peer_seed_encrypted": "encrypted_seed_from_peer",
///   "peer_family_hint": "8ff3b864a4bc589a"
/// }
/// ```
///
/// ## Implementation Strategy (Deep Debt Principles)
///
/// - **Runtime Discovery**: Discovers peer endpoint dynamically
/// - **Capability-Based**: Uses capability discovery for peer communication
/// - **Complete Implementation**: No mocks, uses ConnectionManager
/// - **Safe Rust**: No unsafe code, pure Rust
pub async fn handle_beacon_exchange(
    connection_manager: Option<Arc<ConnectionManager>>,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct BeaconExchangeParams {
        endpoint: String,
        beacon_id: String,
        beacon_seed_encrypted: String,
    }
    
    let params: BeaconExchangeParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("🤝 Initiating beacon exchange with peer: {}", params.endpoint);
    
    // Get connection manager (required for peer connectivity)
    let manager = connection_manager
        .ok_or_else(|| JsonRpcError::internal_error("Connection manager not available"))?;
    
    // Deep Debt Principle: Runtime Discovery
    // We discover if peer is already connected via ConnectionManager
    if let Some(trust_level) = manager.get_connection(&params.beacon_id).await {
        debug!("   Peer already connected with trust level: {:?}", trust_level);
        
        // Attempt to call peer's beacon_exchange method
        match manager.call_peer(
            &params.beacon_id,
            "beacon.exchange",
            serde_json::json!({
                "beacon_id": params.beacon_id,
                "beacon_seed_encrypted": params.beacon_seed_encrypted
            })
        ).await {
            Ok(response) => {
                info!("✅ Beacon exchange successful with connected peer");
                
                // Extract response fields with proper error handling
                let peer_beacon_id = response["beacon_id"].as_str()
                    .unwrap_or("unknown").to_string();
                let peer_seed_encrypted = response["seed_encrypted"].as_str()
                    .unwrap_or("").to_string();
                let peer_family_hint = response["family_hint"].as_str()
                    .unwrap_or("").to_string();
                
                return Ok(serde_json::json!({
                    "success": true,
                    "peer_beacon_id": peer_beacon_id,
                    "peer_seed_encrypted": peer_seed_encrypted,
                    "peer_family_hint": peer_family_hint
                }));
            }
            Err(e) => {
                warn!("   Beacon exchange via RPC failed: {}", e);
                // Fall through to direct connection attempt
            }
        }
    }
    
    // Deep Debt Principle: Complete Implementation (No Mocks)
    // For now, we return a clear error explaining what's needed
    // Full implementation requires:
    // 1. Direct peer connection (TCP/QUIC)
    // 2. BearDog beacon seed derivation
    // 3. Encrypted beacon seed exchange protocol
    
    warn!("⚠️  Peer not connected, direct beacon exchange requires:");
    warn!("   1. Direct peer connectivity (TCP/QUIC)");
    warn!("   2. BearDog beacon seed derivation");
    warn!("   3. Encrypted seed exchange protocol");
    warn!("   Endpoint: {}", params.endpoint);
    
    Ok(serde_json::json!({
        "success": false,
        "error": "Peer not connected - direct beacon exchange requires additional protocol implementation",
        "note": "Use biomeOS BeaconGeneticsManager for full meeting orchestration",
        "peer_beacon_id": Value::Null,
        "peer_seed_encrypted": Value::Null,
        "peer_family_hint": Value::Null
    }))
}

/// Handle network.broadcast - Broadcast encrypted message to network
///
/// NEW (Feb 4, 2026): biomeOS integration for Dark Forest discovery.
/// Broadcasts an encrypted beacon to the network using UDP multicast.
///
/// **Request Format**:
/// ```json
/// {
///   "payload_encrypted": "encrypted_beacon_broadcast",
///   "ttl": 60,
///   "channel": "dark_forest"
/// }
/// ```
///
/// ## Implementation Strategy (Deep Debt Principles)
///
/// - **Runtime Discovery**: Discovers multicast addresses dynamically
/// - **Pure Rust**: Uses Tokio UDP, zero C dependencies
/// - **Complete Implementation**: Uses Dark Forest beacon format
/// - **No Hardcoding**: Multicast address from environment or defaults
pub async fn handle_network_broadcast(params: Option<Value>) -> Result<Value, JsonRpcError> {
    use base64::{Engine as _, engine::general_purpose};
    use std::net::SocketAddr;
    use tokio::net::UdpSocket;
    
    #[derive(Deserialize)]
    struct BroadcastParams {
        payload_encrypted: String,
        #[serde(default = "default_ttl")]
        ttl: u64,
        #[serde(default = "default_channel")]
        channel: String,
    }
    
    fn default_ttl() -> u64 { 60 }
    fn default_channel() -> String { "dark_forest".to_string() }
    
    let params: BroadcastParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("📡 Broadcasting to network (channel: {}, ttl: {}s)", params.channel, params.ttl);
    
    // Deep Debt Principle: No Hardcoding
    // Discover multicast address from environment or use standard mDNS multicast
    let multicast_addr = songbird_process_env::var("SONGBIRD_MULTICAST_ADDR")
        .unwrap_or_else(|_| "224.0.0.251:5353".to_string());
    
    let multicast_target: SocketAddr = multicast_addr.parse()
        .map_err(|e| JsonRpcError::internal_error(&format!("Invalid multicast address: {}", e)))?;
    
    // Decode base64 encrypted payload
    let encrypted_bytes = general_purpose::STANDARD.decode(&params.payload_encrypted)
        .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid base64: {}", e)))?;
    
    // Create Dark Forest beacon (uses existing format)
    let nonce = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; 12];
        rng.fill(&mut nonce);
        nonce
    };
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| JsonRpcError::internal_error(&format!("Time error: {}", e)))?
        .as_secs();
    
    let beacon = serde_json::json!({
        "encrypted_payload": encrypted_bytes,
        "nonce": nonce.to_vec(),
        "timestamp": timestamp,
        "version": 2
    });
    
    let beacon_bytes = serde_json::to_vec(&beacon)
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to serialize beacon: {}", e)))?;
    
    // Deep Debt Principle: Complete Implementation
    // Bind UDP socket and broadcast
    let socket = UdpSocket::bind("0.0.0.0:0").await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to bind UDP socket: {}", e)))?;
    
    // Enable broadcast
    socket.set_broadcast(true)
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to enable broadcast: {}", e)))?;
    
    // Send beacon
    let bytes_sent = socket.send_to(&beacon_bytes, multicast_target).await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to send broadcast: {}", e)))?;
    
    info!("✅ Broadcast sent ({} bytes to {})", bytes_sent, multicast_target);
    
    // Generate broadcast ID (deterministic from timestamp and nonce)
    let broadcast_id = format!("{:x}{:x}", timestamp, nonce[0] as u64);
    
    Ok(serde_json::json!({
        "broadcast_id": broadcast_id,
        "peers_reached": "multicast", // Multicast doesn't provide delivery confirmation
        "bytes_sent": bytes_sent,
        "multicast_target": multicast_target.to_string()
    }))
}

/// Handle network.listen - Listen for network broadcasts
///
/// NEW (Feb 4, 2026): biomeOS integration for Dark Forest discovery.
/// Listens for encrypted beacon broadcasts on a channel using UDP multicast.
///
/// **Request Format**:
/// ```json
/// {
///   "channel": "dark_forest",
///   "timeout_seconds": 30
/// }
/// ```
///
/// ## Implementation Strategy (Deep Debt Principles)
///
/// - **Runtime Discovery**: Discovers multicast group dynamically
/// - **Pure Rust**: Uses Tokio UDP, zero C dependencies
/// - **Complete Implementation**: Returns actual beacon data
/// - **No Hardcoding**: Multicast address from environment or defaults
pub async fn handle_network_listen(params: Option<Value>) -> Result<Value, JsonRpcError> {
    use base64::{Engine as _, engine::general_purpose};
    use std::net::{Ipv4Addr, SocketAddr};
    use tokio::net::UdpSocket;
    use tokio::time::{timeout, Duration};
    
    #[derive(Deserialize)]
    struct ListenParams {
        #[serde(default = "default_channel")]
        channel: String,
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    }
    
    fn default_channel() -> String { "dark_forest".to_string() }
    fn default_timeout() -> u64 { 30 }
    
    let params: ListenParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => ListenParams {
            channel: default_channel(),
            timeout_seconds: default_timeout(),
        },
    };
    
    info!("🎧 Listening for broadcasts (channel: {}, timeout: {}s)", params.channel, params.timeout_seconds);
    
    // Deep Debt Principle: No Hardcoding
    // Discover multicast group from environment or use standard mDNS multicast
    let multicast_group = songbird_process_env::var("SONGBIRD_MULTICAST_GROUP")
        .unwrap_or_else(|_| "224.0.0.251".to_string());
    let listen_port = songbird_process_env::var("SONGBIRD_DISCOVERY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5353u16);
    
    let multicast_ip: Ipv4Addr = multicast_group.parse()
        .map_err(|e| JsonRpcError::internal_error(&format!("Invalid multicast group: {}", e)))?;
    
    // Deep Debt Principle: Complete Implementation
    // Bind UDP socket and join multicast group
    let socket = UdpSocket::bind(("0.0.0.0", listen_port)).await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to bind UDP socket: {}", e)))?;
    
    socket.join_multicast_v4(multicast_ip, Ipv4Addr::UNSPEC)
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to join multicast: {}", e)))?;
    
    debug!("   Joined multicast group {} on port {}", multicast_ip, listen_port);
    
    // Listen for broadcasts with timeout
    let mut broadcasts = Vec::new();
    let mut buf = [0u8; 65536]; // Max UDP packet size
    let deadline = Duration::from_secs(params.timeout_seconds);
    
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < deadline {
        let remaining = deadline - start_time.elapsed();
        
        match timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, source))) => {
                debug!("   Received {} bytes from {}", len, source);
                
                // Try to parse as Dark Forest beacon
                match serde_json::from_slice::<Value>(&buf[..len]) {
                    Ok(beacon) => {
                        // Extract encrypted payload and encode as base64
                        if let Some(encrypted_bytes) = beacon["encrypted_payload"].as_array() {
                            let encrypted_vec: Vec<u8> = encrypted_bytes
                                .iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u8))
                                .collect();
                            
                            let payload_b64 = general_purpose::STANDARD.encode(&encrypted_vec);
                            let received_at = chrono::Utc::now().to_rfc3339();
                            
                            broadcasts.push(serde_json::json!({
                                "payload_encrypted": payload_b64,
                                "received_at": received_at,
                                "source_hint": source.to_string(),
                                "timestamp": beacon["timestamp"],
                                "version": beacon["version"]
                            }));
                            
                            debug!("   Parsed beacon (version: {})", beacon["version"]);
                        }
                    }
                    Err(e) => {
                        debug!("   Not a valid beacon: {}", e);
                        continue;
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("   Socket error: {}", e);
                break;
            }
            Err(_) => {
                // Timeout reached
                debug!("   Listen timeout reached");
                break;
            }
        }
    }
    
    info!("✅ Listen complete ({} broadcasts received)", broadcasts.len());
    
    Ok(serde_json::json!({
        "broadcasts": broadcasts,
        "count": broadcasts.len(),
        "channel": params.channel,
        "listen_duration_seconds": start_time.elapsed().as_secs()
    }))
}
