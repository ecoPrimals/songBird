//! # Protocol Detector Module
//!
//! **PEDANTIC COMPLETION** ✅
//!
//! This module was missing and causing import errors. Created to resolve
//! unresolved import issues in the gaming network system.

use crate::network::gaming::types::*;
use songbird_errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Protocol detector for gaming network protocols
pub struct ProtocolDetector {
    detected_protocols: Arc<RwLock<HashMap<String, GameProtocolType>>>,
}

impl ProtocolDetector {
    /// Create a new protocol detector
    pub fn new() -> Self {
        Self {
            detected_protocols: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Detect protocol from packet data
    pub async fn detect_protocol(&self, packet: &[u8]) -> SongbirdResult<GameProtocolType> {
        // Simple protocol detection based on packet patterns
        if packet.len() < 4 {
            return Ok(GameProtocolType::Unknown);
        }

        match &packet[0..4] {
            [0x00, 0x02, _, _] => Ok(GameProtocolType::IPX),
            [0x45, 0x00, _, _] => Ok(GameProtocolType::UDP),
            b"DPLP" => Ok(GameProtocolType::DirectPlay),
            _ => Ok(GameProtocolType::Unknown),
        }
    }

    /// Cache detected protocol
    pub async fn cache_protocol(&self, key: String, protocol: GameProtocolType) -> SongbirdResult<()> {
        let mut protocols = self.detected_protocols.write().await;
        protocols.insert(key, protocol);
        Ok(())
    }
}

impl Default for ProtocolDetector {
    fn default() -> Self {
        Self::new()
    }
} 