// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Authentication and token verification operations via the security provider.

use crate::error::Result;
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
}
