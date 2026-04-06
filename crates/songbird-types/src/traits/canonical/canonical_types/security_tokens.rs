// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Credentials, tokens, and validation results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Hold secrets or API keys for authentication flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Credential kind (password, `api_key`, `oauth_refresh`, etc.).
    pub credential_type: String,
    /// Opaque credential fields; never log verbatim.
    pub data: HashMap<String, serde_json::Value>,
}

/// Represent a bearer or session token returned by an auth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Serialized token string.
    pub token: String,
    /// Token profile (Bearer, MAC, etc.).
    pub token_type: String,
    /// Absolute expiry when known.
    pub expires_at: Option<SystemTime>,
    /// Issuer metadata and scopes.
    pub metadata: HashMap<String, String>,
}

/// Standard JWT-style claims used by internal auth adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject identifier (user or service id).
    pub subject: String,
    /// Intended recipients or audiences.
    pub audience: Vec<String>,
    /// Absolute expiry when present.
    pub expires_at: Option<SystemTime>,
    /// Extension claims as JSON.
    pub custom_claims: HashMap<String, serde_json::Value>,
}

/// Report whether a token parsed and validated successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    /// True when signature and time bounds are valid.
    pub valid: bool,
    /// Parsed claims when validation succeeded.
    pub claims: Option<TokenClaims>,
    /// Failure reason when validation failed.
    pub error: Option<String>,
}
