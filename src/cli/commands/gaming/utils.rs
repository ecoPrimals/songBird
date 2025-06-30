//! Gaming utilities and helper functions

use crate::network::gaming::GameProtocolClass;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub session_code: String,
    pub host_address: SocketAddr,
    pub game_name: String,
    pub protocol_class: GameProtocolClass,
    pub max_players: u8,
    pub current_players: u8,
}

/// Generate a random session code
pub fn generate_session_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    
    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Pad string to specified length
pub fn pad_string(s: &str, length: usize) -> String {
    if s.len() >= length {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(length - s.len()))
    }
}

/// Get local IP address
pub async fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}
