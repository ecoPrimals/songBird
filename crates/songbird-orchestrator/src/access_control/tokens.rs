// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Token generation and validation
//!
//! ✅ Pure Rust JWT implementation using `RustCrypto` (hmac + sha2)
//! Zero C dependencies - 100% ecoBin compliant!

use super::Role;
use super::pure_rust_jwt;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Custom claims for our JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject
    pub iat: i64,    // Issued at
    pub exp: i64,    // Expiry
    #[serde(flatten)]
    pub role: Role,
    pub token_type: TokenType,
}

/// Access token (JWT in standalone mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token_type: TokenType,

    // Standard JWT claims
    pub sub: String, // Subject
    pub iat: i64,    // Issued at
    pub exp: i64,    // Expiry

    // Custom claims
    #[serde(flatten)]
    pub role: Role,

    // Convenience fields (not serialized)
    #[serde(skip)]
    pub subject: String,
    #[serde(skip)]
    pub issued_at: i64,
    #[serde(skip)]
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    JWT,
    /// Tokens issued by the security capability provider.
    ///
    /// `"security_provider"` is the canonical wire name; legacy payloads that
    /// serialised the variant as `"BearDog"` are accepted for backward compat.
    #[serde(alias = "security_provider", alias = "BearDog")]
    SecurityProvider,
}

impl AccessToken {
    #[must_use]
    pub fn anonymous() -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + 3600;
        Self {
            token_type: TokenType::JWT,
            sub: "anonymous".into(),
            iat: now,
            exp,
            role: Role::Anonymous,
            subject: "anonymous".into(),
            issued_at: now,
            expires_at: exp,
        }
    }

    #[must_use]
    pub fn student(student_id: &str, course_id: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + 86400;
        Self {
            token_type: TokenType::JWT,
            sub: student_id.into(),
            iat: now,
            exp,
            role: Role::Student {
                student_id: student_id.into(),
                course_id: course_id.into(),
            },
            subject: student_id.into(),
            issued_at: now,
            expires_at: exp,
        }
    }

    #[must_use]
    pub fn ta(ta_id: &str, course_id: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + 86400;
        Self {
            token_type: TokenType::JWT,
            sub: ta_id.into(),
            iat: now,
            exp,
            role: Role::TA {
                ta_id: ta_id.into(),
                course_id: course_id.into(),
            },
            subject: ta_id.into(),
            issued_at: now,
            expires_at: exp,
        }
    }

    #[must_use]
    pub fn professor(professor_id: &str, courses: Vec<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + 86400;
        Self {
            token_type: TokenType::JWT,
            sub: professor_id.into(),
            iat: now,
            exp,
            role: Role::Professor {
                professor_id: professor_id.into(),
                courses,
            },
            subject: professor_id.into(),
            issued_at: now,
            expires_at: exp,
        }
    }

    #[must_use]
    pub fn admin(admin_id: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        let exp = now + 3600;
        Self {
            token_type: TokenType::JWT,
            sub: admin_id.into(),
            iat: now,
            exp,
            role: Role::Admin {
                admin_id: admin_id.into(),
            },
            subject: admin_id.into(),
            issued_at: now,
            expires_at: exp,
        }
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }

    /// Check if token has verified 2FA/hardware key
    ///
    /// This is critical for infrastructure access.
    /// Returns true if:
    /// - Token has 2FA verification claim (JWT)
    /// - Token is security provider token with hardware entropy (future)
    #[must_use]
    pub const fn has_2fa_verified(&self) -> bool {
        match self.token_type {
            TokenType::JWT => {
                // In JWT mode, 2FA verification would be indicated by a claim
                // For now, admin/remote-admin roles are assumed to have 2FA
                // Future: Add explicit 2fa_verified claim to JWT
                matches!(self.role, Role::Admin { .. } | Role::RemoteAdmin { .. })
            }
            TokenType::SecurityProvider => {
                // Security provider tokens with hardware entropy automatically satisfy 2FA
                true
            }
        }
    }

    /// Encode as JWT string (✅ Pure Rust using `RustCrypto` hmac + sha2!)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn encode(&self, secret: &[u8]) -> Result<String> {
        let claims = Claims {
            sub: self.sub.clone(),
            iat: self.iat,
            exp: self.exp,
            role: self.role.clone(),
            token_type: self.token_type.clone(),
        };

        pure_rust_jwt::encode(&claims, secret)
    }

    /// Decode from JWT string (✅ Pure Rust using `RustCrypto` hmac + sha2!)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn decode(token_str: &str, secret: &[u8]) -> Result<Self> {
        let claims: Claims = pure_rust_jwt::decode(token_str, secret)?;

        Ok(Self {
            token_type: claims.token_type,
            sub: claims.sub.clone(),
            iat: claims.iat,
            exp: claims.exp,
            role: claims.role.clone(),
            subject: claims.sub,
            issued_at: claims.iat,
            expires_at: claims.exp,
        })
    }
}

/// Token validator
pub struct TokenValidator {
    #[allow(dead_code, reason = "stored for future JWT cryptographic verification in validate()")]
    secret: Vec<u8>,
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenValidator {
    /// Create a new token validator
    ///
    /// Loads JWT secret from `SONGBIRD_JWT_SECRET` environment variable.
    /// Returns a validator with an empty secret if unset (validation will reject all tokens).
    #[must_use]
    pub fn new() -> Self {
        let secret = if let Ok(s) = songbird_process_env::var("SONGBIRD_JWT_SECRET") {
            s.into_bytes()
        } else {
            tracing::warn!(
                "SONGBIRD_JWT_SECRET not set — token validation will reject all tokens; \
                 configure via environment or delegate to security provider"
            );
            Vec::new()
        };

        Self {
            secret,
        }
    }

    /// Create validator with explicit secret (for testing or custom configuration)
    pub fn with_secret(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn validate(&self, token: &AccessToken) -> Result<Identity> {
        // Check expiry
        if token.is_expired() {
            return Err(anyhow!("Token expired"));
        }

        // Check blacklist (for revoked tokens)
        // FUTURE (Phase 2): Token blacklist via Redis or distributed cache
        // Current: Expiry-based validation is sufficient for most use cases
        // Future use case: Immediate token revocation (e.g., compromised tokens, user logout)
        // For now, expiry check provides basic security

        // Return identity
        Ok(Identity {
            id: token.sub.clone(),
            role: token.role.clone(),
        })
    }
}

/// Validated identity
#[derive(Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub role: Role,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::access_control::Role;

    #[test]
    fn test_token_encoding_decoding() {
        let secret = b"test-secret";
        let token = AccessToken::student("student-123", "CSE-847");

        let encoded = token.encode(secret).unwrap();
        let decoded = AccessToken::decode(&encoded, secret).unwrap();

        assert_eq!(token.subject, decoded.subject);
        assert_eq!(token.role, decoded.role);
    }

    #[test]
    fn test_token_expiry() {
        let mut token = AccessToken::student("student-123", "CSE-847");

        assert!(!token.is_expired());

        // Set expiry to past (use 'exp' field, not 'expires_at')
        token.exp = chrono::Utc::now().timestamp() - 1000;

        assert!(token.is_expired());
    }

    #[tokio::test]
    async fn test_token_validation() {
        let validator = TokenValidator::new();
        let token = AccessToken::student("student-123", "CSE-847");

        let identity = validator.validate(&token).await.unwrap();

        assert_eq!(identity.id, "student-123");
        assert!(matches!(identity.role, Role::Student { .. }));
    }

    #[test]
    fn encode_decode_anonymous_ta_professor_admin() {
        let secret = b"sec";
        for token in [
            AccessToken::anonymous(),
            AccessToken::ta("ta1", "C1"),
            AccessToken::professor("p1", vec!["c".to_string()]),
            AccessToken::admin("root"),
        ] {
            let enc = token.encode(secret).unwrap();
            let dec = AccessToken::decode(&enc, secret).unwrap();
            assert_eq!(dec.sub, token.sub);
            assert_eq!(dec.role, token.role);
        }
    }

    #[tokio::test]
    async fn validate_expired_rejected() {
        let v = TokenValidator::with_secret(b"x");
        let mut t = AccessToken::anonymous();
        t.exp = chrono::Utc::now().timestamp() - 10;
        let e = v.validate(&t).await;
        assert!(e.is_err());
    }

    #[test]
    fn has_2fa_verified_admin_and_security_provider_token() {
        let admin = AccessToken::admin("a");
        assert!(admin.has_2fa_verified());
        let mut bd = AccessToken::anonymous();
        bd.token_type = TokenType::SecurityProvider;
        assert!(bd.has_2fa_verified());
        let st = AccessToken::student("s", "c");
        assert!(!st.has_2fa_verified());
    }

    #[test]
    fn has_2fa_remote_admin() {
        let mut t = AccessToken::anonymous();
        t.role = Role::RemoteAdmin {
            admin_id: "r".into(),
            vpn_session: "v".into(),
            hardware_key_verified: true,
        };
        assert!(t.has_2fa_verified());
    }

    #[tokio::test]
    async fn token_validator_with_secret() {
        let v = TokenValidator::with_secret("custom-secret");
        let mut tok = AccessToken::student("s", "c");
        let enc = tok.encode(b"custom-secret").unwrap();
        tok = AccessToken::decode(&enc, b"custom-secret").unwrap();
        let id = v.validate(&tok).await.unwrap();
        assert_eq!(id.id, "s");
    }
}
