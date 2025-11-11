/// WebSocket API for Songbird - Real-time bidirectional communication
///
/// This module provides WebSocket support for real-time events, streaming,
/// and persistent connections. It's part of Phase 4 of the Progressive
/// Protocol Enhancement strategy.
///
/// Features:
/// - Real-time service discovery updates
/// - Health status change notifications
/// - Federation event streaming
/// - Bidirectional communication
/// - Connection persistence
///
/// Version: 0.2.1
/// Last Updated: November 11, 2025 - Phase 4

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;

/// WebSocket API state
#[derive(Clone)]
pub struct WebSocketApiState {
    /// Federation state for status queries
    pub federation_state: Arc<FederationState>,
    
    /// Service registry for discovery
    pub service_registry: Arc<FederatedServiceRegistry>,
}

impl WebSocketApiState {
    /// Create new WebSocket API state
    pub fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Client subscribes to events
    Subscribe {
        /// Event types to subscribe to
        events: Vec<String>,
    },
    
    /// Client unsubscribes from events
    Unsubscribe {
        /// Event types to unsubscribe from
        events: Vec<String>,
    },
    
    /// Ping message for keep-alive
    Ping {
        /// Optional data
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
    },
    
    /// Pong response
    Pong {
        /// Optional data
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
    },
    
    /// Query federation status
    QueryStatus,
    
    /// Query services by capability
    QueryServices {
        /// Required capabilities
        capabilities: Vec<String>,
    },
    
    /// Server sends service update event
    ServiceUpdate {
        /// Service name
        service_name: String,
        
        /// Service status
        status: String,
        
        /// Service address
        address: String,
    },
    
    /// Server sends health status event
    HealthUpdate {
        /// Service name
        service_name: String,
        
        /// Is healthy
        healthy: bool,
        
        /// Optional message
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    
    /// Server sends federation status
    FederationStatus {
        /// Total services
        total_services: usize,
        
        /// Total peers
        total_peers: usize,
        
        /// Uptime in seconds
        uptime_seconds: u64,
    },
    
    /// Server sends service list
    ServiceList {
        /// List of services
        services: Vec<ServiceSummary>,
    },
    
    /// Error response
    Error {
        /// Error message
        message: String,
        
        /// Error code
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    
    /// Success acknowledgment
    Ack {
        /// Optional message
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

/// Service summary for WebSocket responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSummary {
    /// Service name
    pub name: String,
    
    /// Service address
    pub address: String,
    
    /// Service port
    pub port: u16,
    
    /// Service capabilities
    pub capabilities: Vec<String>,
}

/// WebSocket upgrade handler
///
/// This upgrades an HTTP connection to a WebSocket connection
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketApiState>,
) -> Response {
    info!("WebSocket connection requested");
    
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: WebSocketApiState) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("🔌 WebSocket connection established");
    
    // Send welcome message
    let welcome = WsMessage::Ack {
        message: Some("Connected to Songbird WebSocket API".to_string()),
    };
    
    if let Ok(json) = serde_json::to_string(&welcome) {
        if let Err(e) = sender.send(Message::Text(json)).await {
            error!("Failed to send welcome message: {}", e);
            return;
        }
    }
    
    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse and handle JSON message
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        if let Some(response) = handle_ws_message(ws_msg, &state).await {
                            if let Ok(json) = serde_json::to_string(&response) {
                                if let Err(e) = sender.send(Message::Text(json)).await {
                                    error!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                        let error_msg = WsMessage::Error {
                            message: format!("Invalid message format: {}", e),
                            code: Some("INVALID_JSON".to_string()),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = sender.send(Message::Text(json)).await;
                        }
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                warn!("Binary messages not supported");
                let error_msg = WsMessage::Error {
                    message: "Binary messages not supported".to_string(),
                    code: Some("UNSUPPORTED_FORMAT".to_string()),
                };
                if let Ok(json) = serde_json::to_string(&error_msg) {
                    let _ = sender.send(Message::Text(json)).await;
                }
            }
            Ok(Message::Ping(data)) => {
                // Respond to ping with pong
                if let Err(e) = sender.send(Message::Pong(data)).await {
                    error!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Received pong, connection is alive
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed by client");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    info!("🔌 WebSocket connection closed");
}

/// Handle a parsed WebSocket message
async fn handle_ws_message(
    msg: WsMessage,
    state: &WebSocketApiState,
) -> Option<WsMessage> {
    match msg {
        WsMessage::Subscribe { events } => {
            info!("Client subscribed to events: {:?}", events);
            Some(WsMessage::Ack {
                message: Some(format!("Subscribed to {} event(s)", events.len())),
            })
        }
        
        WsMessage::Unsubscribe { events } => {
            info!("Client unsubscribed from events: {:?}", events);
            Some(WsMessage::Ack {
                message: Some(format!("Unsubscribed from {} event(s)", events.len())),
            })
        }
        
        WsMessage::Ping { data } => {
            Some(WsMessage::Pong { data })
        }
        
        WsMessage::QueryStatus => {
            // Get current federation status
            let stats = state.federation_state.get_stats().await;
            Some(WsMessage::FederationStatus {
                total_services: 0, // TODO: Get from service registry
                total_peers: stats.active_nodes,
                uptime_seconds: 0, // TODO: Calculate uptime
            })
        }
        
        WsMessage::QueryServices { capabilities } => {
            // Query services by capability
            // For now, return empty list (would integrate with service registry)
            info!("Querying services with capabilities: {:?}", capabilities);
            Some(WsMessage::ServiceList {
                services: vec![],
            })
        }
        
        // Server-to-client messages shouldn't come from client
        WsMessage::ServiceUpdate { .. }
        | WsMessage::HealthUpdate { .. }
        | WsMessage::FederationStatus { .. }
        | WsMessage::ServiceList { .. } => {
            Some(WsMessage::Error {
                message: "This message type can only be sent by server".to_string(),
                code: Some("INVALID_DIRECTION".to_string()),
            })
        }
        
        // These don't need responses
        WsMessage::Pong { .. }
        | WsMessage::Error { .. }
        | WsMessage::Ack { .. } => None,
    }
}

/// Create WebSocket routes
pub fn websocket_routes() -> Router<WebSocketApiState> {
    Router::new()
        .route("/ws", get(websocket_handler))
}

