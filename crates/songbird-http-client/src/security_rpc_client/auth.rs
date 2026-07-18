// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Authentication, token verification, and enrollment proof operations via the security provider.

use crate::error::{Error, Result};
use serde_json::Value;

use super::core::SecurityRpcClient;

impl SecurityRpcClient {
    /// Verify an ionic token via `auth.verify_ionic` on the security provider.
    ///
    /// Returns the verification result containing subject, scopes, and expiration.
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the token is invalid.
    pub async fn verify_ionic(&self, token: &str) -> Result<Value> {
        let args = serde_json::json!({ "token": token });
        self.call("auth.verify_ionic", args).await
    }

    /// Verify an enrollment proof via `enrollment.verify` on the security provider.
    ///
    /// The enrolling node proves family membership by computing:
    /// `proof = HMAC-SHA256(family_seed, node_id || "|" || public_key || "|" || timestamp)`
    ///
    /// bearDog verifies using the same family seed. On success, the node is
    /// authorized to join the mesh.
    ///
    /// # Errors
    ///
    /// Returns an error if bearDog is unreachable or the proof is invalid.
    pub async fn verify_enrollment_proof(
        &self,
        node_id: &str,
        public_key: &str,
        timestamp: u64,
        proof: &str,
    ) -> Result<EnrollmentVerification> {
        let result = self
            .call(
                "enrollment.verify",
                serde_json::json!({
                    "node_id": node_id,
                    "public_key": public_key,
                    "timestamp": timestamp,
                    "proof": proof,
                }),
            )
            .await?;

        let verified = result["verified"].as_bool().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from(
                "Missing 'verified' in enrollment.verify response",
            ))
        })?;

        let reason = result["reason"].as_str().map(ToString::to_string);

        Ok(EnrollmentVerification {
            verified,
            reason,
        })
    }
}

/// Result of enrollment proof verification from bearDog.
#[derive(Debug, Clone)]
pub struct EnrollmentVerification {
    /// Whether the proof was valid (node has the family seed).
    pub verified: bool,
    /// Reason for rejection (if `verified` is false).
    pub reason: Option<String>,
}
