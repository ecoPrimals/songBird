// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN binding request/response transaction helpers (RFC 5389).

use crate::error::{StunError, StunResult};
use crate::message::{StunAttribute, StunMessage};
use bytes::Bytes;
use std::net::SocketAddr;

/// A single STUN binding exchange: one request and validated response handling.
#[derive(Debug)]
pub struct BindingTransaction {
    request: StunMessage,
    key: Option<Vec<u8>>,
}

impl BindingTransaction {
    /// Create an unauthenticated binding transaction.
    #[must_use]
    pub fn new() -> Self {
        Self {
            request: StunMessage::new_binding_request(),
            key: None,
        }
    }

    /// Create an authenticated binding transaction with beacon-tier credentials.
    ///
    /// Adds USERNAME to the request. When a key is present, `encode_request`
    /// produces a message with MESSAGE-INTEGRITY and FINGERPRINT per RFC 5389.
    /// Per `DARK_FOREST_BEACON_GENETICS_STANDARD.md`, these MUST be beacon-tier
    /// credentials — never nuclear/lineage material.
    #[must_use]
    pub fn with_credentials(credentials: &crate::types::StunCredentials) -> Self {
        let mut request = StunMessage::new_binding_request();
        request.attributes.push(StunAttribute::Username(credentials.username.clone()));
        Self {
            request,
            key: if credentials.key.is_empty() {
                None
            } else {
                Some(credentials.key.clone())
            },
        }
    }

    /// Encode the binding request. When credentials with a key are attached,
    /// the message includes MESSAGE-INTEGRITY (HMAC-SHA1) and FINGERPRINT (CRC32).
    #[must_use]
    pub fn encode_request(&self) -> Bytes {
        self.key
            .as_ref()
            .map_or_else(|| self.request.encode(), |k| self.request.encode_authenticated(k))
    }

    /// Decode a binding response, verify the transaction ID, and return the mapped address.
    pub fn parse_response(&self, response_bytes: &[u8]) -> StunResult<SocketAddr> {
        let response = StunMessage::decode(response_bytes)?;
        if response.transaction_id != self.request.transaction_id {
            return Err(StunError::InvalidResponse(String::from("Transaction ID mismatch")));
        }
        response.get_any_mapped_address().ok_or_else(|| {
            StunError::InvalidResponse(String::from("No mapped address in response"))
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::message::{MessageType, StunAttribute, StunMessage};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn parse_response_rejects_transaction_id_mismatch() {
        let txn = BindingTransaction::new();
        let mut response = StunMessage::new_binding_request();
        response.message_type = MessageType::BindingResponse;
        response.transaction_id = [0xEE; 12];
        response.attributes.push(StunAttribute::XorMappedAddress(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
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
    #[allow(clippy::unwrap_used, reason = "test assertion")]
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
    #[allow(clippy::unwrap_used, reason = "test assertion")]
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
    #[allow(clippy::unwrap_used, reason = "test assertion")]
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

    #[test]
    fn with_credentials_produces_mi_and_fp() {
        let creds = crate::types::StunCredentials {
            username: String::from("beacon-user"),
            key: b"beacon-stun-key".to_vec(),
        };
        let txn = BindingTransaction::with_credentials(&creds);
        let wire = txn.encode_request();

        let decoded = StunMessage::decode(&wire).expect("decode authenticated request");
        assert_eq!(decoded.message_type, MessageType::BindingRequest);

        let has_username = decoded
            .attributes
            .iter()
            .any(|a| matches!(a, StunAttribute::Username(u) if u == "beacon-user"));
        let has_mi =
            decoded.attributes.iter().any(|a| matches!(a, StunAttribute::MessageIntegrity(_)));
        let has_fp = decoded.attributes.iter().any(|a| matches!(a, StunAttribute::Fingerprint(_)));

        assert!(has_username, "USERNAME attribute required");
        assert!(has_mi, "MESSAGE-INTEGRITY required for authenticated request");
        assert!(has_fp, "FINGERPRINT required for authenticated request");
    }

    #[test]
    fn with_credentials_empty_key_omits_integrity() {
        let creds = crate::types::StunCredentials {
            username: String::from("user-no-key"),
            key: vec![],
        };
        let txn = BindingTransaction::with_credentials(&creds);
        let wire = txn.encode_request();

        let decoded = StunMessage::decode(&wire).expect("decode request");
        let has_mi =
            decoded.attributes.iter().any(|a| matches!(a, StunAttribute::MessageIntegrity(_)));
        let has_fp = decoded.attributes.iter().any(|a| matches!(a, StunAttribute::Fingerprint(_)));

        assert!(!has_mi, "empty key must not add MESSAGE-INTEGRITY");
        assert!(!has_fp, "empty key must not add FINGERPRINT");
    }

    #[test]
    fn with_credentials_username_in_request() {
        let creds = crate::types::StunCredentials {
            username: String::from("txn-user"),
            key: b"secret-key".to_vec(),
        };
        let txn = BindingTransaction::with_credentials(&creds);
        let decoded = StunMessage::decode(&txn.encode_request()).expect("decode");
        let username = decoded.attributes.iter().find_map(|a| {
            if let StunAttribute::Username(u) = a {
                Some(u.clone())
            } else {
                None
            }
        });
        assert_eq!(username.as_deref(), Some("txn-user"));
    }

    #[test]
    fn encode_request_unauthenticated_has_no_mi_or_fp() {
        let txn = BindingTransaction::new();
        let decoded = StunMessage::decode(&txn.encode_request()).expect("decode");
        let has_mi =
            decoded.attributes.iter().any(|a| matches!(a, StunAttribute::MessageIntegrity(_)));
        let has_fp = decoded.attributes.iter().any(|a| matches!(a, StunAttribute::Fingerprint(_)));
        assert!(!has_mi);
        assert!(!has_fp);
    }
}
