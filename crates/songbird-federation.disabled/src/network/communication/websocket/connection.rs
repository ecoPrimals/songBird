// Module imports
//! WebSocket Connection Management Management
//!
//! Handles individual WebSocket connections and their lifecycle

use std: :sync::atomic::{AtomicBool, AtomicU64};
use std: :sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite: :tungstenite::Message as WsMessage;
use songbird_discovery::traits::communication::ServiceAddress;
/// Active WebSocket connection
pub struct WebSocketConnection {
    /// Id field

    pub id: Arc<str>,
    /// Address field
    pub address: ServiceAddress,
    /// Connected At field
    pub connected_at: Instant,
    /// Last Heartbeat field
    pub last_heartbeat: Arc<Mutex<Instant>>,
    /// Message Count field
    pub message_count: AtomicU64,
    /// Is Healthy field
    pub is_healthy: AtomicBool,
    /// Outgoing Tx field
    pub outgoing_tx: mpsc::UnboundedSender<WsMessage> ;,
 ,
} 
