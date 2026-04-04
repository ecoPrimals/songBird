// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! L2CAP (Logical Link Control and Adaptation Protocol) layer
//!
//! Provides L2CAP channel management for ATT protocol communication.
//! Implements minimal L2CAP required for GATT operations over channel 0x0004.

use crate::error::{BluetoothError, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, trace, warn};

/// L2CAP Channel ID for ATT protocol
pub const ATT_CHANNEL_ID: u16 = 0x0004;

/// L2CAP Channel ID for signaling
pub const SIGNALING_CHANNEL_ID: u16 = 0x0001;

/// L2CAP Channel ID for LE signaling
pub const LE_SIGNALING_CHANNEL_ID: u16 = 0x0005;

/// Maximum L2CAP PDU size for LE
pub const LE_DEFAULT_MTU: u16 = 23;

/// L2CAP header size (length + channel ID)
const L2CAP_HEADER_SIZE: usize = 4;

/// ACL header size (handle/flags + data length)
const ACL_HEADER_SIZE: usize = 4;

/// L2CAP Channel
///
/// Represents an L2CAP channel for communication.
/// For ATT, this is always channel 0x0004.
#[derive(Debug, Clone)]
pub struct L2capChannel {
    /// Channel ID (0x0004 for ATT)
    pub channel_id: u16,
    /// Connection handle
    pub connection_handle: u16,
    /// Maximum Transmission Unit
    pub mtu: u16,
}

impl L2capChannel {
    /// Create new L2CAP channel for ATT
    #[must_use]
    pub const fn new_att(connection_handle: u16) -> Self {
        Self {
            channel_id: ATT_CHANNEL_ID,
            connection_handle,
            mtu: LE_DEFAULT_MTU,
        }
    }

    /// Create L2CAP channel with custom MTU
    #[must_use]
    pub const fn with_mtu(mut self, mtu: u16) -> Self {
        self.mtu = mtu;
        self
    }

    /// Build ACL data packet with L2CAP header
    ///
    /// Format:
    /// ```text
    /// ACL Header (4 bytes):
    ///   Handle + Flags (2 bytes, little-endian)
    ///   Data Length (2 bytes, little-endian)
    /// L2CAP Header (4 bytes):
    ///   PDU Length (2 bytes, little-endian)
    ///   Channel ID (2 bytes, little-endian)
    /// Payload:
    ///   Data (variable length)
    /// ```
    pub fn build_acl_packet(&self, payload: &[u8]) -> Vec<u8> {
        let l2cap_length = payload.len();
        let acl_data_length = L2CAP_HEADER_SIZE + l2cap_length;

        let mut packet = Vec::with_capacity(ACL_HEADER_SIZE + acl_data_length);

        // ACL Header
        // Handle and flags (bits 0-11: handle, 12-13: packet boundary, 14-15: broadcast)
        // For LE, we use packet boundary = 0b00 (first packet, non-automatically flushable)
        let handle_and_flags = self.connection_handle & 0x0FFF;
        packet.extend_from_slice(&handle_and_flags.to_le_bytes());

        // ACL data length
        packet.extend_from_slice(&u16::try_from(acl_data_length).unwrap_or(u16::MAX).to_le_bytes());

        // L2CAP Header
        // PDU length (payload only, not including L2CAP header)
        packet.extend_from_slice(&u16::try_from(l2cap_length).unwrap_or(u16::MAX).to_le_bytes());

        // Channel ID
        packet.extend_from_slice(&self.channel_id.to_le_bytes());

        // Payload
        packet.extend_from_slice(payload);

        trace!(
            "Built ACL packet: handle=0x{:04X}, channel=0x{:04X}, payload_len={}",
            self.connection_handle, self.channel_id, l2cap_length
        );

        packet
    }

    /// Parse ACL data packet to extract L2CAP payload
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Packet is too short
    /// - Channel ID doesn't match
    /// - Length field is invalid
    pub fn parse_acl_packet(&self, packet: &[u8]) -> Result<Vec<u8>> {
        if packet.len() < ACL_HEADER_SIZE + L2CAP_HEADER_SIZE {
            return Err(BluetoothError::InvalidData {
                context: format!(
                    "ACL packet too short: {} bytes (need at least {})",
                    packet.len(),
                    ACL_HEADER_SIZE + L2CAP_HEADER_SIZE
                ),
            });
        }

        // Parse ACL header
        let handle_and_flags = u16::from_le_bytes([packet[0], packet[1]]);
        let received_handle = handle_and_flags & 0x0FFF;
        let acl_data_length = u16::from_le_bytes([packet[2], packet[3]]) as usize;

        if received_handle != self.connection_handle {
            warn!(
                "ACL packet handle mismatch: expected 0x{:04X}, got 0x{:04X}",
                self.connection_handle, received_handle
            );
        }

        // Verify packet size
        if packet.len() < ACL_HEADER_SIZE + acl_data_length {
            return Err(BluetoothError::InvalidData {
                context: format!(
                    "ACL packet size mismatch: packet has {} bytes, header says {}",
                    packet.len() - ACL_HEADER_SIZE,
                    acl_data_length
                ),
            });
        }

        // Parse L2CAP header
        let l2cap_start = ACL_HEADER_SIZE;
        let pdu_length =
            u16::from_le_bytes([packet[l2cap_start], packet[l2cap_start + 1]]) as usize;
        let channel_id = u16::from_le_bytes([packet[l2cap_start + 2], packet[l2cap_start + 3]]);

        if channel_id != self.channel_id {
            return Err(BluetoothError::InvalidData {
                context: format!(
                    "L2CAP channel mismatch: expected 0x{:04X}, got 0x{:04X}",
                    self.channel_id, channel_id
                ),
            });
        }

        // Extract payload
        let payload_start = l2cap_start + L2CAP_HEADER_SIZE;
        let payload_end = payload_start + pdu_length;

        if payload_end > packet.len() {
            return Err(BluetoothError::InvalidData {
                context: format!(
                    "L2CAP payload exceeds packet: need {} bytes, have {}",
                    payload_end - l2cap_start,
                    packet.len() - l2cap_start
                ),
            });
        }

        let payload = packet[payload_start..payload_end].to_vec();

        trace!(
            "Parsed ACL packet: handle=0x{:04X}, channel=0x{:04X}, payload_len={}",
            received_handle,
            channel_id,
            payload.len()
        );

        Ok(payload)
    }
}

/// L2CAP Manager
///
/// Manages L2CAP channels for connections.
pub struct L2capManager {
    channels: Arc<Mutex<Vec<L2capChannel>>>,
}

impl L2capManager {
    /// Create new L2CAP manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create ATT channel for a connection
    ///
    /// # Errors
    ///
    /// Returns error if channel already exists
    pub async fn create_att_channel(&self, connection_handle: u16) -> Result<L2capChannel> {
        let channel = {
            let mut channels = self.channels.lock().await;

            // Check if channel already exists
            if channels.iter().any(|c| c.connection_handle == connection_handle) {
                return Err(BluetoothError::InvalidOperation(format!(
                    "ATT channel already exists for handle 0x{connection_handle:04X}"
                )));
            }

            let channel = L2capChannel::new_att(connection_handle);
            channels.push(channel.clone());
            channel
        }; // Lock dropped here

        debug!("Created ATT channel for handle 0x{:04X}", connection_handle);

        Ok(channel)
    }

    /// Get ATT channel for a connection
    ///
    /// # Errors
    ///
    /// Returns error if channel doesn't exist
    pub async fn get_att_channel(&self, connection_handle: u16) -> Result<L2capChannel> {
        let channels = self.channels.lock().await;

        channels.iter().find(|c| c.connection_handle == connection_handle).cloned().ok_or_else(
            || {
                BluetoothError::InvalidOperation(format!(
                    "No ATT channel for handle 0x{connection_handle:04X}"
                ))
            },
        )
    }

    /// Remove channel for a connection
    pub async fn remove_channel(&self, connection_handle: u16) {
        self.channels.lock().await.retain(|c| c.connection_handle != connection_handle);
        debug!("Removed channel for handle 0x{:04X}", connection_handle);
    }

    /// Get all channels
    pub async fn channels(&self) -> Vec<L2capChannel> {
        self.channels.lock().await.clone()
    }
}

impl Default for L2capManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_build_acl_packet() {
        let channel = L2capChannel::new_att(0x0040);
        let payload = vec![0x0A, 0x01, 0x00]; // ATT Read Request, handle 0x0001

        let packet = channel.build_acl_packet(&payload);

        // ACL Header (4 bytes)
        assert_eq!(packet[0], 0x40); // Handle low byte
        assert_eq!(packet[1], 0x00); // Handle high byte
        assert_eq!(packet[2], 0x07); // ACL data length low (4 + 3 = 7)
        assert_eq!(packet[3], 0x00); // ACL data length high

        // L2CAP Header (4 bytes)
        assert_eq!(packet[4], 0x03); // PDU length low (3 bytes payload)
        assert_eq!(packet[5], 0x00); // PDU length high
        assert_eq!(packet[6], 0x04); // Channel ID low (0x0004)
        assert_eq!(packet[7], 0x00); // Channel ID high

        // Payload (3 bytes)
        assert_eq!(packet[8], 0x0A); // ATT Read Request opcode
        assert_eq!(packet[9], 0x01); // Handle low
        assert_eq!(packet[10], 0x00); // Handle high

        assert_eq!(packet.len(), 11);
    }

    #[test]
    fn test_parse_acl_packet() {
        let channel = L2capChannel::new_att(0x0040);

        // Build a sample ACL packet
        let packet = vec![
            0x40, 0x00, // Handle 0x0040
            0x07, 0x00, // ACL data length 7
            0x03, 0x00, // L2CAP length 3
            0x04, 0x00, // Channel ID 0x0004
            0x0A, 0x01, 0x00, // ATT Read Request
        ];

        let payload = match channel.parse_acl_packet(&packet) {
            Ok(p) => p,
            Err(e) => panic!("parse_acl_packet: {e:?}"),
        };

        assert_eq!(payload.len(), 3);
        assert_eq!(payload[0], 0x0A); // ATT Read Request opcode
        assert_eq!(payload[1], 0x01); // Handle low
        assert_eq!(payload[2], 0x00); // Handle high
    }

    #[test]
    fn test_parse_short_packet() {
        let channel = L2capChannel::new_att(0x0040);
        let packet = vec![0x40, 0x00, 0x07]; // Too short

        let result = channel.parse_acl_packet(&packet);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wrong_channel() {
        let channel = L2capChannel::new_att(0x0040);

        let packet = vec![
            0x40, 0x00, // Handle 0x0040
            0x07, 0x00, // ACL data length 7
            0x03, 0x00, // L2CAP length 3
            0x05, 0x00, // Wrong channel ID 0x0005
            0x0A, 0x01, 0x00,
        ];

        let result = channel.parse_acl_packet(&packet);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_l2cap_manager() {
        let manager = L2capManager::new();

        // Create channel
        let channel = match manager.create_att_channel(0x0040).await {
            Ok(c) => c,
            Err(e) => panic!("create_att_channel: {e:?}"),
        };
        assert_eq!(channel.connection_handle, 0x0040);
        assert_eq!(channel.channel_id, ATT_CHANNEL_ID);

        // Get channel
        let retrieved = match manager.get_att_channel(0x0040).await {
            Ok(c) => c,
            Err(e) => panic!("get_att_channel: {e:?}"),
        };
        assert_eq!(retrieved.connection_handle, 0x0040);

        // Remove channel
        manager.remove_channel(0x0040).await;
        let result = manager.get_att_channel(0x0040).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_channel() {
        let manager = L2capManager::new();

        match manager.create_att_channel(0x0040).await {
            Ok(_channel) => {}
            Err(e) => panic!("create_att_channel: {e:?}"),
        }
        let result = manager.create_att_channel(0x0040).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_with_mtu() {
        let channel = L2capChannel::new_att(0x0040).with_mtu(512);
        assert_eq!(channel.mtu, 512);
    }

    #[test]
    fn l2cap_channel_constants_match_le_att_usage() {
        assert_eq!(ATT_CHANNEL_ID, 0x0004);
        assert_eq!(SIGNALING_CHANNEL_ID, 0x0001);
        assert_eq!(LE_SIGNALING_CHANNEL_ID, 0x0005);
        assert_eq!(LE_DEFAULT_MTU, 23);
    }

    #[test]
    fn parse_acl_rejects_packet_shorter_than_acl_header_claims() {
        let ch = L2capChannel::new_att(0x0040);
        let bad = vec![0x40u8, 0x00, 0x64, 0x00];
        assert!(ch.parse_acl_packet(&bad).is_err());
    }

    #[test]
    fn parse_acl_rejects_truncated_l2cap_payload() {
        let ch = L2capChannel::new_att(0x0040);
        let packet = vec![
            0x40, 0x00, 0x0B, 0x00, // ACL data length 11 bytes after header
            0x05, 0x00, // L2CAP PDU len 5
            0x04, 0x00, // CID ATT
            0x01, 0x02, // truncated payload (need 5 bytes)
        ];
        assert!(ch.parse_acl_packet(&packet).is_err());
    }

    #[test]
    fn build_and_parse_acl_round_trips_payload() {
        let ch = L2capChannel::new_att(0x00AA).with_mtu(247);
        let payload = vec![0x12u8, 0x34];
        let pkt = ch.build_acl_packet(&payload);
        let out = ch.parse_acl_packet(&pkt).expect("round-trip");
        assert_eq!(out, payload);
    }

    #[tokio::test]
    async fn l2cap_manager_channels_lists_all() {
        let m = L2capManager::new();
        m.create_att_channel(0x10).await.expect("c1");
        m.create_att_channel(0x20).await.expect("c2");
        let list = m.channels().await;
        assert_eq!(list.len(), 2);
    }
}
