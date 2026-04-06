// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN client protocol helpers: server resolution, port-pattern inference.

use crate::error::{StunError, StunResult};
use crate::types::PortPattern;
use std::net::SocketAddr;

/// Classify observed external ports from repeated STUN probes (same logic as [`crate::StunClient::probe_port_pattern`]).
pub fn infer_port_pattern_from_mapped_ports(ports: &[u16]) -> PortPattern {
    if ports.len() < 2 {
        return PortPattern::Unknown;
    }

    let deltas: Vec<i32> = ports.windows(2).map(|w| i32::from(w[1]) - i32::from(w[0])).collect();

    if deltas.is_empty() {
        return PortPattern::Unknown;
    }

    let first_delta = deltas[0];
    let consistent_count = deltas.iter().filter(|d| **d == first_delta).count();
    let consistency = f64::from(u32::try_from(consistent_count).unwrap_or(0))
        / f64::from(u32::try_from(deltas.len()).unwrap_or(1));

    if consistency >= 0.7 && first_delta.unsigned_abs() <= 100 {
        let Some(&last_port) = ports.last() else {
            return PortPattern::Unknown;
        };
        let predicted = i32::from(last_port) + first_delta;
        let predicted_next = u16::try_from(predicted.clamp(1, 65535)).unwrap_or(last_port);

        let confidence = consistency
            * if first_delta.unsigned_abs() <= 10 {
                0.95
            } else {
                0.75
            };

        PortPattern::Sequential {
            step: first_delta,
            last_port,
            predicted_next,
            confidence,
        }
    } else {
        PortPattern::Random {
            observed: ports.to_vec(),
        }
    }
}

/// Resolve a STUN server hostname and pick a preferred address (IPv4 when available).
pub async fn resolve_stun_server(stun_server: &str) -> StunResult<SocketAddr> {
    let all_addrs: Vec<SocketAddr> = tokio::net::lookup_host(stun_server)
        .await
        .map_err(|e| StunError::Network(format!("Failed to resolve STUN server: {e}")))?
        .collect();

    if all_addrs.is_empty() {
        return Err(StunError::Network(format!("No addresses found for: {stun_server}")));
    }

    all_addrs
        .iter()
        .find(|a| a.is_ipv4())
        .or_else(|| all_addrs.first())
        .copied()
        .ok_or_else(|| StunError::Network(format!("No usable addresses found for: {stun_server}")))
}

/// UDP bind address matching the server address family.
#[must_use]
pub fn local_bind_addr_for_peer(server_addr: SocketAddr) -> &'static str {
    if server_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::message::StunMessage;
    use crate::message::{MAGIC_COOKIE, MessageType, StunAttribute};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn binding_request_encode_decode_roundtrip() {
        let msg = StunMessage {
            message_type: MessageType::BindingRequest,
            transaction_id: [7u8; 12],
            attributes: Vec::new(),
        };
        let wire = msg.encode();
        assert_eq!(wire.len(), 20);
        assert_eq!(u16::from_be_bytes([wire[0], wire[1]]), MessageType::BindingRequest.to_u16());
        assert_eq!(u32::from_be_bytes([wire[4], wire[5], wire[6], wire[7]]), MAGIC_COOKIE);

        let decoded = StunMessage::decode(&wire).expect("decode");
        assert_eq!(decoded.message_type, MessageType::BindingRequest);
        assert_eq!(decoded.transaction_id, [7u8; 12]);
    }

    #[test]
    fn binding_response_parses_xor_mapped_address() {
        let addr = std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 10)), 49_152);
        let msg = StunMessage {
            message_type: MessageType::BindingResponse,
            transaction_id: [9u8; 12],
            attributes: vec![StunAttribute::XorMappedAddress(addr)],
        };
        let decoded = StunMessage::decode(&msg.encode()).expect("decode response");
        assert_eq!(decoded.get_any_mapped_address(), Some(addr));
    }

    #[test]
    fn decode_shows_transaction_id_mismatch_against_request() {
        let req = StunMessage {
            message_type: MessageType::BindingRequest,
            transaction_id: [1u8; 12],
            attributes: Vec::new(),
        };
        let resp = StunMessage {
            message_type: MessageType::BindingResponse,
            transaction_id: [2u8; 12],
            attributes: vec![StunAttribute::XorMappedAddress(std::net::SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2)),
                40_000,
            ))],
        };
        let parsed = StunMessage::decode(&resp.encode()).expect("decode");
        assert_ne!(parsed.transaction_id, req.transaction_id);
    }

    #[test]
    fn infer_port_pattern_sequential_small_step() {
        let p = infer_port_pattern_from_mapped_ports(&[10_000, 10_001, 10_002, 10_003]);
        match p {
            PortPattern::Sequential {
                step,
                predicted_next,
                ..
            } => {
                assert_eq!(step, 1);
                assert_eq!(predicted_next, 10_004);
            }
            _ => panic!("expected sequential pattern, got {p:?}"),
        }
    }

    #[test]
    fn infer_port_pattern_random_high_jitter() {
        let p = infer_port_pattern_from_mapped_ports(&[1000, 5000, 1200, 8000]);
        assert!(matches!(p, PortPattern::Random { .. }));
    }

    #[test]
    fn infer_port_pattern_insufficient_samples() {
        assert!(matches!(infer_port_pattern_from_mapped_ports(&[42]), PortPattern::Unknown));
    }

    #[test]
    fn infer_port_pattern_large_step_treated_as_random() {
        let p = infer_port_pattern_from_mapped_ports(&[1000, 1101, 1202, 1303]);
        assert!(
            matches!(p, PortPattern::Random { .. }),
            "expected Random for |step| > 100, got {p:?}"
        );
    }

    #[test]
    fn infer_port_pattern_inconsistent_deltas_yield_random() {
        let p = infer_port_pattern_from_mapped_ports(&[10_000, 10_001, 10_010, 10_011]);
        assert!(
            matches!(p, PortPattern::Random { .. }),
            "expected Random when deltas disagree, got {p:?}"
        );
    }
}
