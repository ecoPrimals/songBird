// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;

use crate::server::events::EventBroadcaster;

/// WebSocket API state
#[derive(Clone)]
pub struct WebSocketApiState {
    /// Federation state for status queries
    pub federation_state: Arc<FederationState>,

    /// Service registry for discovery
    pub service_registry: Arc<FederatedServiceRegistry>,

    /// Event broadcaster for real-time events
    pub event_broadcaster: Arc<EventBroadcaster>,

    /// Orchestrator for task events (optional - MVP Week 4)
    pub orchestrator: Option<Arc<crate::orchestrator::SongbirdOrchestrator>>,
}

impl WebSocketApiState {
    /// Create new WebSocket API state
    #[must_use]
    pub const fn new(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        event_broadcaster: Arc<EventBroadcaster>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            event_broadcaster,
            orchestrator: None,
        }
    }

    /// Create with orchestrator support (for MVP Week 4 task events)
    #[must_use]
    pub const fn with_orchestrator(
        federation_state: Arc<FederationState>,
        service_registry: Arc<FederatedServiceRegistry>,
        event_broadcaster: Arc<EventBroadcaster>,
        orchestrator: Arc<crate::orchestrator::SongbirdOrchestrator>,
    ) -> Self {
        Self {
            federation_state,
            service_registry,
            event_broadcaster,
            orchestrator: Some(orchestrator),
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

    /// Task event stream ready (MVP Week 4)
    TaskEventReady {
        /// Ready message
        message: String,
    },

    /// Task event (MVP Week 4: Observability)
    TaskEvent {
        /// Task ID
        task_id: String,

        /// User/owner ID
        user_id: String,

        /// Event type (Started, Completed, Failed, etc)
        event_type: String,

        /// Event timestamp (RFC3339)
        timestamp: String,
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
        message: Some(String::from("Connected to Songbird WebSocket API")),
    };

    if let Ok(json) = serde_json::to_string(&welcome)
        && let Err(e) = sender.send(Message::Text(json.into())).await
    {
        error!("Failed to send welcome message: {}", e);
        return;
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse and handle JSON message
                match serde_json::from_str::<WsMessage>(text.as_str()) {
                    Ok(ws_msg) => {
                        if let Some(response) = handle_ws_message(ws_msg, &state).await
                            && let Ok(json) = serde_json::to_string(&response)
                            && let Err(e) = sender.send(Message::Text(json.into())).await
                        {
                            error!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                        let error_msg = WsMessage::Error {
                            message: format!("Invalid message format: {e}"),
                            code: Some(String::from("INVALID_JSON")),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = sender.send(Message::Text(json.into())).await;
                        }
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                warn!("Binary messages not supported");
                let error_msg = WsMessage::Error {
                    message: String::from("Binary messages not supported"),
                    code: Some(String::from("UNSUPPORTED_FORMAT")),
                };
                if let Ok(json) = serde_json::to_string(&error_msg) {
                    let _ = sender.send(Message::Text(json.into())).await;
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
async fn handle_ws_message(msg: WsMessage, state: &WebSocketApiState) -> Option<WsMessage> {
    match msg {
        WsMessage::Subscribe {
            events,
        } => {
            info!("Client subscribed to events: {:?}", events);
            Some(WsMessage::Ack {
                message: Some(format!("Subscribed to {} event(s)", events.len())),
            })
        }

        WsMessage::Unsubscribe {
            events,
        } => {
            info!("Client unsubscribed from events: {:?}", events);
            Some(WsMessage::Ack {
                message: Some(format!("Unsubscribed from {} event(s)", events.len())),
            })
        }

        WsMessage::Ping {
            data,
        } => Some(WsMessage::Pong {
            data,
        }),

        WsMessage::QueryStatus => {
            // Get current federation status with real metrics
            let federation_stats = state.federation_state.get_stats().await;

            // Get total services from registry
            let registry_stats = state.service_registry.get_stats().await;
            let total_services = registry_stats.total_services;

            let uptime_seconds = federation_stats.uptime_seconds.unwrap_or(0);

            Some(WsMessage::FederationStatus {
                total_services,
                total_peers: federation_stats.active_nodes,
                uptime_seconds,
            })
        }

        WsMessage::QueryServices {
            capabilities,
        } => {
            info!("Querying services with capabilities: {capabilities:?}");
            let all_services = state.service_registry.get_all_services().await;
            let filtered: Vec<ServiceSummary> = all_services
                .into_iter()
                .filter(|svc| {
                    capabilities.is_empty()
                        || capabilities
                            .iter()
                            .any(|cap| svc.capabilities.iter().any(|c| c.contains(cap.as_str())))
                })
                .map(|svc| {
                    let (addr, port) = super::tarpc_server::parse_endpoint(&svc.endpoint);
                    ServiceSummary {
                        name: svc.service_name.clone(),
                        address: addr.to_string(),
                        port,
                        capabilities: svc.capabilities.clone(),
                    }
                })
                .collect();
            Some(WsMessage::ServiceList {
                services: filtered,
            })
        }

        // Server-to-client messages shouldn't come from client
        WsMessage::ServiceUpdate {
            ..
        }
        | WsMessage::HealthUpdate {
            ..
        }
        | WsMessage::FederationStatus {
            ..
        }
        | WsMessage::ServiceList {
            ..
        } => Some(WsMessage::Error {
            message: String::from("This message type can only be sent by server"),
            code: Some(String::from("INVALID_DIRECTION")),
        }),

        // These don't need responses
        WsMessage::Pong {
            ..
        }
        | WsMessage::Error {
            ..
        }
        | WsMessage::Ack {
            ..
        } => None,

        // Task event messages (handled by dedicated handler, not here)
        WsMessage::TaskEventReady {
            ..
        }
        | WsMessage::TaskEvent {
            ..
        } => Some(WsMessage::Error {
            message: String::from("Task events have a dedicated endpoint at /ws/tasks"),
            code: Some(String::from("WRONG_ENDPOINT")),
        }),
    }
}

/// Create WebSocket routes
pub fn websocket_routes() -> Router<WebSocketApiState> {
    Router::new().route("/ws", get(websocket_handler)).route("/ws/tasks", get(task_events_handler))
}

/// WebSocket handler for task events (MVP Week 4: Observability)
///
/// Provides real-time task lifecycle event streaming to clients.
/// Clients can subscribe to task events and receive live updates.
async fn task_events_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketApiState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_task_events(socket, state))
}

/// Handle task events WebSocket connection
async fn handle_task_events(socket: WebSocket, state: WebSocketApiState) {
    info!("🎯 Task events WebSocket connection established");

    let (mut sender, mut receiver) = socket.split();

    // Get orchestrator event stream
    let event_stream = if let Some(orch) = &state.orchestrator {
        if let Some(stream) = orch.get_event_stream() {
            stream
        } else {
            error!("Orchestrator event stream not available");
            let error_msg = WsMessage::Error {
                message: String::from("Event stream not available"),
                code: Some(String::from("NO_EVENT_STREAM")),
            };
            if let Ok(json) = serde_json::to_string(&error_msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
            return;
        }
    } else {
        error!("Orchestrator not available");
        let error_msg = WsMessage::Error {
            message: String::from("Task events not available (orchestrator not configured)"),
            code: Some(String::from("NO_ORCHESTRATOR")),
        };
        if let Ok(json) = serde_json::to_string(&error_msg) {
            let _ = sender.send(Message::Text(json.into())).await;
        }
        return;
    };

    // Subscribe to events
    use crate::observability::EventFilter;
    let mut event_rx = event_stream.subscribe_filtered(EventFilter::default());

    // Send welcome message
    let welcome = WsMessage::TaskEventReady {
        message: String::from("Connected to task event stream"),
    };
    if let Ok(json) = serde_json::to_string(&welcome)
        && let Err(e) = sender.send(Message::Text(json.into())).await
    {
        error!("Failed to send welcome message: {}", e);
        return;
    }

    // Handle concurrent event streaming and client messages
    loop {
        tokio::select! {
            // Forward events from orchestrator to client
            event_result = event_rx.recv() => {
                match event_result {
                    Ok(event) => {
                        // Convert to WebSocket message
                        let ws_msg = WsMessage::TaskEvent {
                            task_id: event.task_id.to_string(),
                            user_id: event.user_id.to_string(),
                            event_type: format!("{:?}", event.event_type),
                            timestamp: event.timestamp.to_rfc3339(),
                        };

                        if let Ok(json) = serde_json::to_string(&ws_msg)
                            && let Err(e) = sender.send(Message::Text(json.into())).await {
                                error!("Failed to send task event: {}", e);
                                break;
                            }
                    }
                    Err(e) => {
                        warn!("Event stream error: {}", e);
                        break;
                    }
                }
            }

            // Handle client messages (ping, close, etc)
            msg_result = receiver.next() => {
                match msg_result {
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = sender.send(Message::Pong(data)).await {
                            error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Task events WebSocket closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("Task events WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        info!("Task events WebSocket connection closed");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("🔌 Task events WebSocket connection closed");
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    fn assert_ws_roundtrip(msg: &WsMessage) {
        let json = serde_json::to_string(msg).unwrap();
        let back: WsMessage = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn ws_message_subscribe_roundtrip() {
        assert_ws_roundtrip(&WsMessage::Subscribe {
            events: vec![String::from("a"), String::from("b")],
        });
    }

    #[test]
    fn ws_message_unsubscribe_roundtrip() {
        assert_ws_roundtrip(&WsMessage::Unsubscribe {
            events: vec![String::from("x")],
        });
    }

    #[test]
    fn ws_message_ping_pong_roundtrip() {
        assert_ws_roundtrip(&WsMessage::Ping {
            data: Some(String::from("d")),
        });
        assert_ws_roundtrip(&WsMessage::Pong {
            data: None,
        });
    }

    #[test]
    fn ws_message_query_status_roundtrip() {
        assert_ws_roundtrip(&WsMessage::QueryStatus);
    }

    #[test]
    fn ws_message_query_services_roundtrip() {
        assert_ws_roundtrip(&WsMessage::QueryServices {
            capabilities: vec![String::from("compute")],
        });
    }

    #[test]
    fn ws_message_service_update_roundtrip() {
        assert_ws_roundtrip(&WsMessage::ServiceUpdate {
            service_name: String::from("n"),
            status: String::from("up"),
            address: String::from("127.0.0.1"),
        });
    }

    #[test]
    fn ws_message_health_update_roundtrip() {
        assert_ws_roundtrip(&WsMessage::HealthUpdate {
            service_name: String::from("n"),
            healthy: true,
            message: Some(String::from("ok")),
        });
    }

    #[test]
    fn ws_message_federation_status_roundtrip() {
        assert_ws_roundtrip(&WsMessage::FederationStatus {
            total_services: 3,
            total_peers: 2,
            uptime_seconds: 99,
        });
    }

    #[test]
    fn ws_message_service_list_roundtrip() {
        assert_ws_roundtrip(&WsMessage::ServiceList {
            services: vec![ServiceSummary {
                name: String::from("svc"),
                address: String::from("127.0.0.1"),
                port: 8080,
                capabilities: vec![String::from("c")],
            }],
        });
    }

    #[test]
    fn ws_message_error_ack_task_events_roundtrip() {
        assert_ws_roundtrip(&WsMessage::Error {
            message: String::from("e"),
            code: Some(String::from("E")),
        });
        assert_ws_roundtrip(&WsMessage::Ack {
            message: Some(String::from("m")),
        });
        assert_ws_roundtrip(&WsMessage::TaskEventReady {
            message: String::from("ready"),
        });
        assert_ws_roundtrip(&WsMessage::TaskEvent {
            task_id: String::from("t"),
            user_id: String::from("u"),
            event_type: String::from("Started"),
            timestamp: String::from("2025-01-01T00:00:00Z"),
        });
    }

    #[test]
    fn ws_message_unknown_type_fails() {
        let err = serde_json::from_str::<WsMessage>(r#"{"type":"not_a_real_variant"}"#);
        assert!(err.is_err());
    }

    #[test]
    fn ws_message_tag_is_snake_case_in_json() {
        let s = serde_json::to_string(&WsMessage::QueryStatus).unwrap();
        assert!(s.contains("query_status"));
    }
}
