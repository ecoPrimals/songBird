// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! STUN binding request/response transaction helpers (RFC 5389).

use crate::error::{StunError, StunResult};
use crate::message::StunMessage;
use bytes::Bytes;
use std::net::SocketAddr;

/// A single STUN binding exchange: one request and validated response handling.
#[derive(Debug)]
pub struct BindingTransaction {
    request: StunMessage,
}

impl BindingTransaction {
    #[must_use]
    pub fn new() -> Self {
        Self {
            request: StunMessage::new_binding_request(),
        }
    }

    #[must_use]
    pub fn encode_request(&self) -> Bytes {
        self.request.encode()
    }

    /// Decode a binding response, verify the transaction ID, and return the mapped address.
    pub fn parse_response(&self, response_bytes: &[u8]) -> StunResult<SocketAddr> {
        let response = StunMessage::decode(response_bytes)?;
        if response.transaction_id != self.request.transaction_id {
            return Err(StunError::InvalidResponse("Transaction ID mismatch".to_string()));
        }
        response
            .get_any_mapped_address()
            .ok_or_else(|| StunError::InvalidResponse("No mapped address in response".to_string()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::message::{MessageType, StunAttribute, StunMessage};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn parse_response_rejects_transaction_id_mismatch() {
        let txn = BindingTransaction::new();
        let mut response = StunMessage::new_binding_request();
        response.message_type = MessageType::BindingResponse;
        response.transaction_id = [0xEE; 12];
        response.attributes.push(StunAttribute::XorMappedAddress(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            9,
        )));
        let wire = response.encode();
        let err = txn.parse_response(&wire).expect_err("txn id must match");
        assert!(
            err.to_string().contains("Transaction") || err.to_string().contains("mismatch"),
            "unexpected: {err}"
        );
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn parse_response_requires_mapped_address() {
        let txn = BindingTransaction::new();
        let mut response = StunMessage::new_binding_request();
        response.message_type = MessageType::BindingResponse;
        response.transaction_id = txn.encode_request().as_ref()[8..20].try_into().unwrap();
        assert!(response.attributes.is_empty());
        let wire = response.encode();
        let err = txn.parse_response(&wire).expect_err("no XOR/MAPPED address");
        assert!(
            err.to_string().contains("mapped") || err.to_string().contains("address"),
            "unexpected: {err}"
        );
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn parse_response_accepts_xor_mapped_on_success() {
        let txn = BindingTransaction::new();
        let tid: [u8; 12] = txn.encode_request().as_ref()[8..20].try_into().unwrap();
        let mut response = StunMessage::new_binding_request();
        response.message_type = MessageType::BindingResponse;
        response.transaction_id = tid;
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 9)), 44_000);
        response.attributes.push(StunAttribute::XorMappedAddress(addr));
        let wire = response.encode();
        let got = txn.parse_response(&wire).expect("valid binding response");
        assert_eq!(got, addr);
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn parse_response_rejects_binding_error_without_address() {
        let txn = BindingTransaction::new();
        let tid: [u8; 12] = txn.encode_request().as_ref()[8..20].try_into().unwrap();
        let mut response = StunMessage::new_binding_request();
        response.message_type = MessageType::BindingError;
        response.transaction_id = tid;
        let wire = response.encode();
        let err = txn.parse_response(&wire).expect_err("error response has no mapped addr");
        assert!(
            err.to_string().contains("mapped") || err.to_string().contains("address"),
            "unexpected: {err}"
        );
    }
}
