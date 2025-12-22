//! WebSocket handler for real-time coordination

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::registry::SessionRegistry;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(_registry): State<Arc<SessionRegistry>>,
) -> Response {
    info!("🔌 WebSocket connection request: {}", &session_id[..8]);

    ws.on_upgrade(move |socket| handle_socket(socket, session_id))
}

async fn handle_socket(socket: WebSocket, session_id: String) {
    info!("✅ WebSocket connected: {}", &session_id[..8]);

    let (mut sender, mut receiver) = socket.split();

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "session_id": session_id,
        "message": "Connected to Songbird Rendezvous"
    });

    if sender.send(Message::Text(serde_json::to_string(&welcome).unwrap())).await.is_err() {
        warn!("⚠️  Failed to send welcome message");
        return;
    }

    // Message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("📨 Received: {} from {}", text, &session_id[..8]);

                // TODO: Handle connection coordination messages
                // TODO: Forward messages between peers

                // Echo for now
                if sender.send(Message::Text(format!("Echo: {}", text))).await.is_err() {
                    break;
                }
            }
            Ok(Message::Binary(_)) => {
                debug!("📦 Binary message from {}", &session_id[..8]);
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Pong(_)) => {}
            Ok(Message::Close(_)) => {
                info!("🔌 WebSocket closed: {}", &session_id[..8]);
                break;
            }
            Err(e) => {
                warn!("⚠️  WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("👋 WebSocket disconnected: {}", &session_id[..8]);
}
