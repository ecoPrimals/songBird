//! Relay Protocol - Wire format for lineage-based packet relay
//!
//! **Pure Rust | Zero Unsafe | Modern Async**
//!
//! Evolution of TURN (RFC 5766) with genetic lineage authorization.
//!
//! ## Protocol Design
//!
//! Simple binary protocol optimized for low latency:
//! - 1-byte message type
//! - Minimal overhead
//! - Session ID for routing
//! - JSON payload for control messages
//!
//! ## Message Types
//!
//! | Type | Value | Purpose |
//! |------|-------|---------|
//! | ALLOCATE_REQUEST | 0x01 | Request relay session |
//! | ALLOCATE_RESPONSE | 0x02 | Allocation result |
//! | DATA_PACKET | 0x10 | Forward data through relay |
//! | REFRESH | 0x20 | Extend session TTL |
//! | DEALLOCATE | 0x30 | Close session |

use crate::error::{LineageRelayError, Result};
use crate::types::NodeId;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

/// Relay protocol message types
#[derive(Debug, Clone)]
pub enum RelayProtocol {
    /// Request relay allocation
    AllocateRequest(AllocationRequest),
    
    /// Allocation response
    AllocateResponse(AllocationResponse),
    
    /// Data packet to forward
    DataPacket {
        session_id: Uuid,
        data: Vec<u8>,
    },
    
    /// Refresh session (extend TTL)
    Refresh { session_id: Uuid },
    
    /// Deallocate (close session)
    Deallocate { session_id: Uuid },
}

impl RelayProtocol {
    /// Parse relay protocol message from bytes
    ///
    /// # Wire Format
    ///
    /// All messages start with 1-byte type:
    /// - 0x01: ALLOCATE_REQUEST [type(1)][json_payload]
    /// - 0x02: ALLOCATE_RESPONSE [type(1)][json_payload]
    /// - 0x10: DATA_PACKET [type(1)][session_id(16)][data]
    /// - 0x20: REFRESH [type(1)][session_id(16)]
    /// - 0x30: DEALLOCATE [type(1)][session_id(16)]
    ///
    /// # Errors
    ///
    /// Returns error if message is malformed or unknown type.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(LineageRelayError::InvalidProtocol(
                "Empty message".to_string()
            ));
        }
        
        let msg_type = bytes[0];
        let payload = &bytes[1..];
        
        match msg_type {
            0x01 => {
                // ALLOCATE_REQUEST - JSON payload
                let req: AllocationRequest = serde_json::from_slice(payload)
                    .map_err(|e| LineageRelayError::InvalidProtocol(format!(
                        "Failed to parse AllocationRequest: {}", e
                    )))?;
                Ok(Self::AllocateRequest(req))
            }
            0x02 => {
                // ALLOCATE_RESPONSE - JSON payload
                let resp: AllocationResponse = serde_json::from_slice(payload)
                    .map_err(|e| LineageRelayError::InvalidProtocol(format!(
                        "Failed to parse AllocationResponse: {}", e
                    )))?;
                Ok(Self::AllocateResponse(resp))
            }
            0x10 => {
                // DATA_PACKET - [session_id(16 bytes)][data]
                if payload.len() < 16 {
                    return Err(LineageRelayError::InvalidProtocol(
                        "DATA_PACKET too short (need 16 bytes for session_id)".to_string()
                    ));
                }
                
                let session_id = Uuid::from_slice(&payload[0..16])
                    .map_err(|e| LineageRelayError::InvalidProtocol(format!(
                        "Invalid session ID: {}", e
                    )))?;
                
                let data = payload[16..].to_vec();
                
                Ok(Self::DataPacket { session_id, data })
            }
            0x20 => {
                // REFRESH - [session_id(16 bytes)]
                if payload.len() != 16 {
                    return Err(LineageRelayError::InvalidProtocol(
                        "REFRESH must have exactly 16 bytes for session_id".to_string()
                    ));
                }
                
                let session_id = Uuid::from_slice(payload)
                    .map_err(|e| LineageRelayError::InvalidProtocol(format!(
                        "Invalid session ID: {}", e
                    )))?;
                
                Ok(Self::Refresh { session_id })
            }
            0x30 => {
                // DEALLOCATE - [session_id(16 bytes)]
                if payload.len() != 16 {
                    return Err(LineageRelayError::InvalidProtocol(
                        "DEALLOCATE must have exactly 16 bytes for session_id".to_string()
                    ));
                }
                
                let session_id = Uuid::from_slice(payload)
                    .map_err(|e| LineageRelayError::InvalidProtocol(format!(
                        "Invalid session ID: {}", e
                    )))?;
                
                Ok(Self::Deallocate { session_id })
            }
            _ => Err(LineageRelayError::InvalidProtocol(format!(
                "Unknown message type: 0x{:02x}",
                msg_type
            ))),
        }
    }
    
    /// Encode relay protocol message to bytes
    ///
    /// Returns wire format ready to send over UDP.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        match self {
            Self::AllocateRequest(req) => {
                buf.push(0x01);
                if let Ok(json) = serde_json::to_vec(req) {
                    buf.extend_from_slice(&json);
                }
            }
            Self::AllocateResponse(resp) => {
                buf.push(0x02);
                if let Ok(json) = serde_json::to_vec(resp) {
                    buf.extend_from_slice(&json);
                }
            }
            Self::DataPacket { session_id, data } => {
                buf.push(0x10);
                buf.extend_from_slice(session_id.as_bytes());
                buf.extend_from_slice(data);
            }
            Self::Refresh { session_id } => {
                buf.push(0x20);
                buf.extend_from_slice(session_id.as_bytes());
            }
            Self::Deallocate { session_id } => {
                buf.push(0x30);
                buf.extend_from_slice(session_id.as_bytes());
            }
        }
        
        buf
    }
}

/// Allocation request (client → server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRequest {
    /// Relay node offering service
    pub relay_node: NodeId,
    
    /// Node requesting relay
    pub requester: NodeId,
    
    /// Target peer address to relay to
    pub target_addr: SocketAddr,
    
    /// Lineage proof (BearDog signature)
    pub lineage_proof: Vec<u8>,
    
    /// Requested TTL (seconds)
    pub ttl_seconds: u32,
}

impl AllocationRequest {
    /// Create new allocation request
    pub fn new(
        relay_node: NodeId,
        requester: NodeId,
        target_addr: SocketAddr,
        lineage_proof: Vec<u8>,
        ttl_seconds: u32,
    ) -> Self {
        Self {
            relay_node,
            requester,
            target_addr,
            lineage_proof,
            ttl_seconds,
        }
    }
}

/// Allocation response (server → client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResponse {
    /// Success flag
    pub success: bool,
    
    /// Session ID (if successful)
    pub session_id: Option<Uuid>,
    
    /// Relay address to send data to (if successful)
    pub relay_addr: Option<SocketAddr>,
    
    /// Granted TTL (seconds)
    pub ttl_seconds: u32,
    
    /// Error message (if failed)
    pub error: Option<String>,
}

impl AllocationResponse {
    /// Create success response
    pub fn success(session_id: Uuid, relay_addr: SocketAddr, ttl_seconds: u32) -> Self {
        Self {
            success: true,
            session_id: Some(session_id),
            relay_addr: Some(relay_addr),
            ttl_seconds,
            error: None,
        }
    }
    
    /// Create error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            session_id: None,
            relay_addr: None,
            ttl_seconds: 0,
            error: Some(error),
        }
    }
    
    /// Create unauthorized response
    pub fn unauthorized(reason: &str) -> Self {
        Self::error(format!("Unauthorized: {}", reason))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_encode_decode_allocate_request() {
        let req = AllocationRequest::new(
            "tower".into(),
            "pixel".into(),
            "192.168.1.100:12345".parse().unwrap(),
            vec![1, 2, 3, 4],
            300,
        );
        
        let msg = RelayProtocol::AllocateRequest(req.clone());
        let encoded = msg.encode();
        
        // Should start with 0x01
        assert_eq!(encoded[0], 0x01);
        
        // Should decode back
        let decoded = RelayProtocol::parse(&encoded).unwrap();
        
        match decoded {
            RelayProtocol::AllocateRequest(decoded_req) => {
                assert_eq!(decoded_req.relay_node, req.relay_node);
                assert_eq!(decoded_req.requester, req.requester);
                assert_eq!(decoded_req.target_addr, req.target_addr);
                assert_eq!(decoded_req.ttl_seconds, req.ttl_seconds);
            }
            _ => panic!("Expected AllocateRequest"),
        }
    }
    
    #[test]
    fn test_protocol_encode_decode_allocate_response() {
        let session_id = Uuid::new_v4();
        let relay_addr: SocketAddr = "162.226.225.148:3479".parse().unwrap();
        
        let resp = AllocationResponse::success(session_id, relay_addr, 300);
        let msg = RelayProtocol::AllocateResponse(resp.clone());
        let encoded = msg.encode();
        
        // Should start with 0x02
        assert_eq!(encoded[0], 0x02);
        
        // Should decode back
        let decoded = RelayProtocol::parse(&encoded).unwrap();
        
        match decoded {
            RelayProtocol::AllocateResponse(decoded_resp) => {
                assert!(decoded_resp.success);
                assert_eq!(decoded_resp.session_id, Some(session_id));
                assert_eq!(decoded_resp.relay_addr, Some(relay_addr));
                assert_eq!(decoded_resp.ttl_seconds, 300);
            }
            _ => panic!("Expected AllocateResponse"),
        }
    }
    
    #[test]
    fn test_protocol_encode_decode_data_packet() {
        let session_id = Uuid::new_v4();
        let data = b"Hello, World!".to_vec();
        
        let msg = RelayProtocol::DataPacket {
            session_id,
            data: data.clone(),
        };
        let encoded = msg.encode();
        
        // Should start with 0x10
        assert_eq!(encoded[0], 0x10);
        
        // Should have session_id + data
        assert_eq!(encoded.len(), 1 + 16 + data.len());
        
        // Should decode back
        let decoded = RelayProtocol::parse(&encoded).unwrap();
        
        match decoded {
            RelayProtocol::DataPacket { session_id: decoded_id, data: decoded_data } => {
                assert_eq!(decoded_id, session_id);
                assert_eq!(decoded_data, data);
            }
            _ => panic!("Expected DataPacket"),
        }
    }
    
    #[test]
    fn test_protocol_encode_decode_refresh() {
        let session_id = Uuid::new_v4();
        
        let msg = RelayProtocol::Refresh { session_id };
        let encoded = msg.encode();
        
        // Should start with 0x20
        assert_eq!(encoded[0], 0x20);
        
        // Should have exactly 17 bytes (type + uuid)
        assert_eq!(encoded.len(), 17);
        
        // Should decode back
        let decoded = RelayProtocol::parse(&encoded).unwrap();
        
        match decoded {
            RelayProtocol::Refresh { session_id: decoded_id } => {
                assert_eq!(decoded_id, session_id);
            }
            _ => panic!("Expected Refresh"),
        }
    }
    
    #[test]
    fn test_protocol_encode_decode_deallocate() {
        let session_id = Uuid::new_v4();
        
        let msg = RelayProtocol::Deallocate { session_id };
        let encoded = msg.encode();
        
        // Should start with 0x30
        assert_eq!(encoded[0], 0x30);
        
        // Should decode back
        let decoded = RelayProtocol::parse(&encoded).unwrap();
        
        match decoded {
            RelayProtocol::Deallocate { session_id: decoded_id } => {
                assert_eq!(decoded_id, session_id);
            }
            _ => panic!("Expected Deallocate"),
        }
    }
    
    #[test]
    fn test_protocol_invalid_type() {
        let bytes = vec![0xFF, 1, 2, 3]; // Unknown type
        let result = RelayProtocol::parse(&bytes);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_protocol_empty_message() {
        let bytes = vec![];
        let result = RelayProtocol::parse(&bytes);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_allocation_response_success() {
        let session_id = Uuid::new_v4();
        let relay_addr: SocketAddr = "192.168.1.1:3479".parse().unwrap();
        
        let resp = AllocationResponse::success(session_id, relay_addr, 300);
        
        assert!(resp.success);
        assert_eq!(resp.session_id, Some(session_id));
        assert_eq!(resp.relay_addr, Some(relay_addr));
        assert_eq!(resp.ttl_seconds, 300);
        assert!(resp.error.is_none());
    }
    
    #[test]
    fn test_allocation_response_error() {
        let resp = AllocationResponse::error("Test error".to_string());
        
        assert!(!resp.success);
        assert_eq!(resp.session_id, None);
        assert_eq!(resp.relay_addr, None);
        assert_eq!(resp.error, Some("Test error".to_string()));
    }
    
    #[test]
    fn test_allocation_response_unauthorized() {
        let resp = AllocationResponse::unauthorized("Not family");
        
        assert!(!resp.success);
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().contains("Unauthorized"));
    }
    
    #[test]
    fn test_data_packet_large_payload() {
        let session_id = Uuid::new_v4();
        let data = vec![42u8; 65000]; // Large packet
        
        let msg = RelayProtocol::DataPacket {
            session_id,
            data: data.clone(),
        };
        let encoded = msg.encode();
        let decoded = RelayProtocol::parse(&encoded).unwrap();
        
        match decoded {
            RelayProtocol::DataPacket { data: decoded_data, .. } => {
                assert_eq!(decoded_data.len(), 65000);
                assert_eq!(decoded_data, data);
            }
            _ => panic!("Expected DataPacket"),
        }
    }
}
