// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Token verification infrastructure for the MethodGate (JH-11).
//!
//! Defines the [`TokenVerifier`] trait and concrete implementations:
//! - [`NoopVerifier`]: returns `NotConfigured` — for tests and when no security
//!   provider is available.
//! - [`SecurityProviderVerifier`]: calls `auth.verify_ionic` via IPC to verify ionic tokens.

use std::sync::Arc;

/// Claims extracted from a verified ionic token.
#[derive(Debug, Clone)]
pub struct TokenClaims {
    /// Subject identifier (the entity the token was issued to).
    pub subject: String,
    /// Scope patterns: `"*"`, `"domain.*"`, or exact method names.
    pub scopes: Vec<String>,
    /// Unix timestamp when the token expires (if bounded).
    pub expires_at: Option<u64>,
}

/// Errors from token verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenVerifyError {
    /// No verifier is configured (development/permissive mode).
    NotConfigured,
    /// The token's signature or structure is invalid.
    Invalid(String),
    /// The token has expired.
    Expired,
    /// The upstream verification endpoint is unreachable.
    Unavailable(String),
}

/// Abstraction over ionic token verification.
///
/// Production deployments wire [`SecurityProviderVerifier`] which calls
/// `auth.verify_ionic` on the security provider. Tests use [`NoopVerifier`].
pub trait TokenVerifier: Send + Sync {
    /// Verify an ionic token and extract its claims.
    fn verify(
        &self,
        token: &str,
    ) -> impl std::future::Future<Output = Result<TokenClaims, TokenVerifyError>> + Send;
}

/// Verifier that always returns `NotConfigured` — test-only.
///
/// Gated behind `#[cfg(test)]` to prevent accidental use in production
/// binaries. Production code should always wire [`BearDogVerifier`].
#[cfg(test)]
#[derive(Debug, Clone, Copy)]
pub struct NoopVerifier;

#[cfg(test)]
impl TokenVerifier for NoopVerifier {
    async fn verify(&self, _token: &str) -> Result<TokenClaims, TokenVerifyError> {
        Err(TokenVerifyError::NotConfigured)
    }
}

/// Verifier that calls the security provider's `auth.verify_ionic` via IPC.
///
/// Sends the token to whichever primal provides the `security` capability
/// for cryptographic verification, extracting subject, scopes, and expiration.
///
/// Expected response shape:
/// ```json
/// { "subject": "primal-name", "scopes": ["domain.*"], "expires_at": 1717000000 }
/// ```
#[derive(Debug, Clone)]
pub struct SecurityProviderVerifier {
    security_client: Arc<songbird_http_client::SecurityRpcClient>,
}

/// Legacy alias retained for downstream compatibility.
pub type BearDogVerifier = SecurityProviderVerifier;

impl SecurityProviderVerifier {
    /// Create a verifier backed by the given security provider client.
    #[must_use]
    pub fn new(client: Arc<songbird_http_client::SecurityRpcClient>) -> Self {
        Self {
            security_client: client,
        }
    }
}

impl TokenVerifier for SecurityProviderVerifier {
    async fn verify(&self, token: &str) -> Result<TokenClaims, TokenVerifyError> {
        let result = self
            .security_client
            .verify_ionic(token)
            .await
            .map_err(|e| TokenVerifyError::Unavailable(e.to_string()))?;

        let subject = result
            .get("subject")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();

        let scopes = result
            .get("scopes")
            .and_then(serde_json::Value::as_array)
            .map(|arr| arr.iter().filter_map(serde_json::Value::as_str).map(String::from).collect())
            .unwrap_or_default();

        let expires_at = result.get("expires_at").and_then(serde_json::Value::as_u64);

        if let Some(exp) = expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if exp <= now {
                return Err(TokenVerifyError::Expired);
            }
        }

        Ok(TokenClaims {
            subject,
            scopes,
            expires_at,
        })
    }
}
