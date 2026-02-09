//! WebSocket handler for real-time coordination and beacon relay
//!
//! **EVOLVED**: Now supports beacon forwarding between peers for NAT traversal.
//!
//! ## How It Works
//!
//! 1. Peers connect via WebSocket with their session_id
//! 2. Peers send `forward` messages targeting other session_ids
//! 3. Server forwards encrypted beacons between connected peers
//! 4. End-to-end encryption (BirdSong) - server can't read content

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use crate::registry::SessionRegistry;

/// Connected peer with message channel
struct ConnectedPeer {
    session_id: String,
    tx: mpsc::Sender<String>,
}

/// Global peer connections for message forwarding
type PeerConnections = Arc<RwLock<HashMap<String, ConnectedPeer>>>;

lazy_static::lazy_static! {
    static ref CONNECTIONS: PeerConnections = Arc::new(RwLock::new(HashMap::new()));
}

/// Message types for beacon relay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelayMessage {
    /// Forward encrypted beacon to target peer
    #[serde(rename = "forward")]
    Forward {
        target_session_id: String,
        #[serde(default)]
        encrypted_beacon: String,
        /// Optional: base64 binary data for larger payloads
        #[serde(default)]
        payload: Option<String>,
    },
    /// Beacon received from another peer
    #[serde(rename = "beacon")]
    Beacon {
        from_session_id: String,
        encrypted_beacon: String,
        #[serde(default)]
        payload: Option<String>,
    },
    /// List online peers (family discovery)
    #[serde(rename = "list_peers")]
    ListPeers,
    /// Response with online peer list
    #[serde(rename = "peers")]
    Peers { session_ids: Vec<String> },
    /// Ping for keepalive
    #[serde(rename = "ping")]
    Ping,
    /// Pong response
    #[serde(rename = "pong")]
    Pong,
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
    /// Welcome message
    #[serde(rename = "welcome")]
    Welcome {
        session_id: String,
        message: String,
    },
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(_registry): State<Arc<SessionRegistry>>,
) -> Response {
    info!("🔌 WebSocket connection request: {}", &session_id[..8.min(session_id.len())]);

    ws.on_upgrade(move |socket| handle_socket(socket, session_id))
}

async fn handle_socket(socket: WebSocket, session_id: String) {
    let session_short = &session_id[..8.min(session_id.len())];
    info!("✅ WebSocket connected: {}", session_short);

    let (mut sender, mut receiver) = socket.split();

    // Create channel for receiving forwarded messages
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Register this connection
    {
        let mut connections = CONNECTIONS.write().await;
        connections.insert(
            session_id.clone(),
            ConnectedPeer {
                session_id: session_id.clone(),
                tx,
            },
        );
        info!("📊 Active connections: {}", connections.len());
    }

    // Send welcome message
    let welcome = RelayMessage::Welcome {
        session_id: session_id.clone(),
        message: "Connected to Songbird Beacon Relay".to_string(),
    };

    if sender
        .send(Message::Text(serde_json::to_string(&welcome).unwrap()))
        .await
        .is_err()
    {
        warn!("⚠️  Failed to send welcome message");
        cleanup_connection(&session_id).await;
        return;
    }

    // Spawn task to forward incoming messages from other peers
    let session_id_clone = session_id.clone();
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Message loop - receive from this peer
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("📨 Received from {}: {}", session_short, &text[..100.min(text.len())]);

                match serde_json::from_str::<RelayMessage>(&text) {
                    Ok(relay_msg) => {
                        handle_relay_message(&session_id, relay_msg).await;
                    }
                    Err(e) => {
                        warn!("⚠️  Invalid message from {}: {}", session_short, e);
                        // Try to send error back
                        let error = RelayMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        };
                        let _ = forward_to_peer(&session_id, &serde_json::to_string(&error).unwrap()).await;
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                debug!("📦 Binary message ({} bytes) from {}", data.len(), session_short);
                // Binary messages could be used for larger relay payloads
            }
            Ok(Message::Ping(data)) => {
                // Pong is handled automatically by axum-ws
                debug!("🏓 Ping from {}", session_short);
                let _ = data; // Suppress unused warning
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) => {
                info!("🔌 WebSocket closed: {}", session_short);
                break;
            }
            Err(e) => {
                warn!("⚠️  WebSocket error from {}: {}", session_short, e);
                break;
            }
        }
    }

    // Cleanup
    forward_task.abort();
    cleanup_connection(&session_id).await;
    info!("👋 WebSocket disconnected: {}", session_short);
}

async fn handle_relay_message(from_session_id: &str, msg: RelayMessage) {
    let from_short = &from_session_id[..8.min(from_session_id.len())];
    
    match msg {
        RelayMessage::Forward {
            target_session_id,
            encrypted_beacon,
            payload,
        } => {
            let target_short = &target_session_id[..8.min(target_session_id.len())];
            info!("📤 Forwarding beacon from {} to {}", from_short, target_short);

            // Create beacon message for target
            let beacon = RelayMessage::Beacon {
                from_session_id: from_session_id.to_string(),
                encrypted_beacon,
                payload,
            };

            let msg_str = serde_json::to_string(&beacon).unwrap();
            if forward_to_peer(&target_session_id, &msg_str).await {
                debug!("✅ Beacon forwarded to {}", target_short);
            } else {
                warn!("⚠️  Target {} not connected", target_short);
                // Notify sender that target is offline
                let error = RelayMessage::Error {
                    message: format!("Target {} not connected", target_short),
                };
                let _ = forward_to_peer(from_session_id, &serde_json::to_string(&error).unwrap()).await;
            }
        }
        RelayMessage::ListPeers => {
            let connections = CONNECTIONS.read().await;
            let session_ids: Vec<String> = connections
                .keys()
                .filter(|k| *k != from_session_id)
                .cloned()
                .collect();
            
            let response = RelayMessage::Peers { session_ids };
            let _ = forward_to_peer(from_session_id, &serde_json::to_string(&response).unwrap()).await;
        }
        RelayMessage::Ping => {
            let pong = RelayMessage::Pong;
            let _ = forward_to_peer(from_session_id, &serde_json::to_string(&pong).unwrap()).await;
        }
        _ => {
            // Other message types handled inline
        }
    }
}

async fn forward_to_peer(session_id: &str, message: &str) -> bool {
    let connections = CONNECTIONS.read().await;
    if let Some(peer) = connections.get(session_id) {
        peer.tx.send(message.to_string()).await.is_ok()
    } else {
        false
    }
}

async fn cleanup_connection(session_id: &str) {
    let mut connections = CONNECTIONS.write().await;
    connections.remove(session_id);
    info!("📊 Active connections after cleanup: {}", connections.len());
}
