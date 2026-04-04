// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability roles, token matching, and wire-id mapping for runtime primal discovery.

use anyhow::{Result, anyhow};

/// Capability types for primal discovery (functional roles, not primal names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Cryptographic operations (signing, encryption, hashing)
    Crypto,
    /// Security operations (JWT, auth, trust evaluation)
    Security,
    /// HTTP/HTTPS requests (external API delegation)
    Http,
    /// AI operations (LLM inference, routing)
    Ai,
    /// Storage operations (key-value, blob)
    Storage,
    /// Messaging operations (pub/sub, queues)
    Messaging,
}

impl Capability {
    /// Get environment variable name for this capability
    pub(crate) const fn env_var_name(&self) -> &'static str {
        match self {
            Self::Crypto => "CRYPTO_PROVIDER_SOCKET",
            Self::Security => "SECURITY_PROVIDER_SOCKET",
            Self::Http => "HTTP_PROVIDER_SOCKET",
            Self::Ai => "AI_PROVIDER_SOCKET",
            Self::Storage => "STORAGE_PROVIDER_SOCKET",
            Self::Messaging => "MESSAGING_PROVIDER_SOCKET",
        }
    }

    /// Alternative environment variable names (backward compatibility).
    pub(crate) fn alt_env_vars(&self) -> Vec<&'static str> {
        match self {
            Self::Crypto => {
                vec!["SECURITY_PROVIDER_SOCKET", "BEARDOG_CRYPTO_SOCKET", "BEARDOG_SOCKET"]
            }
            Self::Security => {
                vec!["SECURITY_PROVIDER_SOCKET", "SONGBIRD_SECURITY_PROVIDER", "BEARDOG_SOCKET"]
            }
            Self::Http => vec!["HTTP_CLIENT_SOCKET", "SONGBIRD_SOCKET"],
            Self::Ai => vec!["AI_PROVIDER_SOCKETS", "SQUIRREL_SOCKET"],
            Self::Storage => vec!["STORAGE_SOCKET", "NESTGATE_SOCKET"],
            Self::Messaging => vec!["MESSENGER_SOCKET", "PUBSUB_SOCKET"],
        }
    }

    /// Returns true if a flat `capabilities.list` response satisfies this role.
    #[must_use]
    pub fn matches_capability_tokens(&self, tokens: &[String]) -> bool {
        let lowered: Vec<String> = tokens.iter().map(|t| t.to_ascii_lowercase()).collect();
        match self {
            Self::Crypto => lowered.iter().any(|t| {
                t.contains("crypto.delegate")
                    || t.starts_with("crypto.")
                    || t == "crypto"
                    || t.contains("encryption")
            }),
            Self::Security => lowered.iter().any(|t| {
                t.starts_with("security.")
                    || t.contains("jwt")
                    || t.contains("btsp.")
                    || t == "security"
            }),
            Self::Http => lowered.iter().any(|t| {
                t == "http.request" || t == "http.get" || t == "http.post" || t.starts_with("http.")
            }),
            Self::Ai => lowered.iter().any(|t| {
                t.starts_with("ai.")
                    || t.contains("llm")
                    || t.contains("mcp")
                    || t.contains("inference")
                    || t.contains("model")
            }),
            Self::Storage => {
                lowered.iter().any(|t| t.starts_with("storage.") || t.contains("persist"))
            }
            Self::Messaging => lowered.iter().any(|t| {
                t.contains("messaging")
                    || t.contains("pubsub")
                    || t.contains("queue")
                    || t.starts_with("message.")
            }),
        }
    }
}

/// `sovereign-storage`: storage role plus an explicit sovereign / edge token.
#[must_use]
pub fn matches_sovereign_storage_tokens(tokens: &[String]) -> bool {
    Capability::Storage.matches_capability_tokens(tokens)
        && tokens.iter().any(|t| t.to_ascii_lowercase().contains("sovereign"))
}

/// Map doctor / CLI capability keys to [`Capability`] (excludes sovereign-storage).
pub fn capability_from_wire_id(id: &str) -> Result<Capability> {
    match id {
        "crypto" => Ok(Capability::Crypto),
        "ai" => Ok(Capability::Ai),
        "storage" => Ok(Capability::Storage),
        "messaging" => Ok(Capability::Messaging),
        "http" => Ok(Capability::Http),
        "security" => Ok(Capability::Security),
        other => Err(anyhow!("Unknown capability id for discovery: {other}")),
    }
}
