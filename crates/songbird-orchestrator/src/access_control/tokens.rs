//! Token generation and validation

use super::Role;
use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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
    BearDog, // For future integration
}

impl AccessToken {
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

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }

    /// Check if token has verified 2FA/hardware key
    ///
    /// This is critical for infrastructure access.
    /// Returns true if:
    /// - Token has 2FA verification claim (JWT)
    /// - Token is security provider token with hardware entropy (future)
    pub fn has_2fa_verified(&self) -> bool {
        match self.token_type {
            TokenType::JWT => {
                // In JWT mode, 2FA verification would be indicated by a claim
                // For now, admin/remote-admin roles are assumed to have 2FA
                // Future: Add explicit 2fa_verified claim to JWT
                matches!(self.role, Role::Admin { .. } | Role::RemoteAdmin { .. })
            }
            TokenType::BearDog => {
                // security provider tokens with hardware entropy automatically satisfy 2FA
                // This will be properly implemented with security provider integration (Q1 2025)
                true
            }
        }
    }

    /// Encode as JWT string
    pub fn encode(&self, secret: &[u8]) -> Result<String> {
        let header = Header::new(Algorithm::HS256);
        let key = EncodingKey::from_secret(secret);

        encode(&header, self, &key).map_err(|e| anyhow!("Failed to encode token: {}", e))
    }

    /// Decode from JWT string
    pub fn decode(token_str: &str, secret: &[u8]) -> Result<Self> {
        let key = DecodingKey::from_secret(secret);
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<AccessToken>(token_str, &key, &validation)
            .map_err(|e| anyhow!("Failed to decode token: {}", e))?;

        Ok(token_data.claims)
    }
}

/// Token validator
pub struct TokenValidator {
    secret: Vec<u8>,
}

impl TokenValidator {
    /// Create a new token validator
    ///
    /// Loads JWT secret from (in priority order):
    /// 1. `SONGBIRD_JWT_SECRET` environment variable
    /// 2. Default development secret (INSECURE - development only)
    ///
    /// **SECURITY**: In production, ALWAYS set `SONGBIRD_JWT_SECRET` to a strong secret
    pub fn new() -> Self {
        let secret = std::env::var("SONGBIRD_JWT_SECRET").unwrap_or_else(|_| {
            tracing::warn!(
                "SONGBIRD_JWT_SECRET not set. Using development secret. \
                     DO NOT USE IN PRODUCTION. Set SONGBIRD_JWT_SECRET environment variable."
            );
            "songbird-dev-secret-change-in-production".to_string()
        });

        Self {
            secret: secret.into_bytes(),
        }
    }

    /// Create validator with explicit secret (for testing or custom configuration)
    pub fn with_secret(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    pub async fn validate(&self, token: &AccessToken) -> Result<Identity> {
        // Check expiry
        if token.is_expired() {
            return Err(anyhow!("Token expired"));
        }

        // Check blacklist (for revoked tokens)
        // Future enhancement: Implement token blacklist via Redis or database
        // For now, expiry check provides basic security
        // Blacklist implementation tracking: https://github.com/ecoPrimals/songbird/issues/XXX

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
    use super::*;

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

        // Set expiry to past
        token.expires_at = chrono::Utc::now().timestamp() - 1000;

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
}
