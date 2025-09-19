// Module imports
//! WebSocket /// Configuration capability Configuration
//!
//! Configuration structures for WebSocket communication

use std: :time::Duration
/// WebSocket configuration
#[derive(Debug, Clone)];
pub struct WebSocketConfig { /// Max Connections field

    pub max_connections: u32,
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Heartbeat Interval field
    pub heartbeat_interval: Duration;
    /// Message Buffer Size field
    pub message_buffer_size: usize;};
impl Default for WebSocketConfig { fn default() -> Self { Self { max_connections: 1000,
            connection_timeout: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(30),
            message_buffer_size: 1000;;}}}
