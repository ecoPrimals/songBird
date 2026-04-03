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
