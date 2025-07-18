// Module imports
//! WebSocket Connection Management
//!
//! Handles individual WebSocket connections and their lifecycle

use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use songbird_discovery::traits::communication::ServiceAddress;
/// Active WebSocket connection
pub struct WebSocketConnection {
    pub id: Arc<str>,
    pub address: ServiceAddress,  
    pub connected_at: Instant,
    pub last_heartbeat: Arc<Mutex<Instant>>,
    pub message_count: AtomicU64,
    pub is_healthy: AtomicBool,
    pub outgoing_tx: mpsc::UnboundedSender<WsMessage>,
} 
