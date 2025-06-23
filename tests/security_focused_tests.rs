//! Focused Security Module Tests - Phase 3
//! 
//! Target: High-impact security testing with working implementations
//! Focus: Authentication, JWT, Audit, Basic Security Operations
//! Expected: 30+ focused tests that actually compile and run

use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use songbird_orchestrator::{
    security::{
        AuditConfig, AuditLogger, AuthEvent, AuthEventType,
        AuthenticationProvider, Credentials, JwtAuthProvider, OAuth2Config,
        ProductionSecurityProvider, SecurityConfig, SecurityProvider,
        UserInfo,
    },
};

/// Security test utilities
mod security_test_utils {
    use super::*;

    /// Standard JWT secret for all tests (matches production default for compatibility)
    pub const TEST_JWT_SECRET: &str = "super-secret-jwt-key-change-in-production";

    /// Create test security config with known values
    pub fn create_test_security_config() -> SecurityConfig {
        SecurityConfig {
            jwt_secret: TEST_JWT_SECRET.to_string(),
            jwt_expiration: Duration::from_secs(3600),
            encryption_key: [1u8; 32], // Fixed key for testing
            enable_oauth: true,
            oauth_config: Some(OAuth2Config {
                client_id: "test-client-id".to_string(),
                client_secret: "test-client-secret".to_string(),
                auth_endpoint: "https://test-oauth.example.com/auth".to_string(),
                token_endpoint: "https://test-oauth.example.com/token".to_string(),
                userinfo_endpoint: Some("https://test-oauth.example.com/userinfo".to_string()),
                redirect_uri: "http://localhost:8080/auth/callback".to_string(),
                scopes: vec!["openid".to_string(), "profile".to_string()],
            }),
            enable_audit: true,
            audit_config: AuditConfig::default(),
        }
    }

    /// Create test user with various roles
    pub fn create_test_user(id: &str, roles: Vec<&str>) -> UserInfo {
        UserInfo {
            id: id.to_string(),
            username: format!("user_{}", id),
            email: Some(format!("{}@test.example.com", id)),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            metadata: HashMap::from([
                ("department".to_string(), json!("engineering")),
                ("clearance_level".to_string(), json!(3)),
                ("created_at".to_string(), json!(Utc::now())),
            ]),
        }
    }
}

use security_test_utils::*;

// ============================================================================
// AUTHENTICATION & JWT TESTING
// ============================================================================

#[cfg(test)]
mod authentication_tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_token_generation_basic() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("test_user", vec!["user"]);

        let token = provider.generate_jwt(&user).unwrap();

        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.expires_in, 3600);
        assert!(!token.token.is_empty());
        assert!(token.token.len() > 50); // JWT tokens are typically long
    }

    #[tokio::test]
    async fn test_jwt_token_generation_with_complex_metadata() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        
        let mut user = create_test_user("complex_user", vec!["admin", "developer", "analyst"]);
        user.metadata.insert("permissions".to_string(), json!(["read", "write", "admin"]));
        user.metadata.insert("api_quota".to_string(), json!(1000));
        user.metadata.insert("features".to_string(), json!({"beta": true, "premium": false}));

        let token = provider.generate_jwt(&user).unwrap();
        let claims = provider.validate_jwt(&token.token).unwrap();

        assert_eq!(claims.sub, "complex_user");
        assert_eq!(claims.roles, vec!["admin", "developer", "analyst"]);
        assert!(claims.custom.contains_key("permissions"));
        assert!(claims.custom.contains_key("api_quota"));
        assert!(claims.custom.contains_key("features"));
    }

    #[tokio::test]
    async fn test_jwt_token_validation_success() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("valid_user", vec!["user", "tester"]);

        let token = provider.generate_jwt(&user).unwrap();
        let claims = provider.validate_jwt(&token.token).unwrap();

        assert_eq!(claims.sub, "valid_user");
        assert_eq!(claims.iss, "songbird-orchestrator");
        assert_eq!(claims.aud, "songbird-services");
        assert_eq!(claims.roles, vec!["user", "tester"]);
        assert!(claims.exp > claims.iat);
    }

    #[tokio::test]
    async fn test_jwt_token_validation_invalid_token() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let invalid_tokens = vec![
            "invalid.token.here",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
            "",
            "not-a-jwt-at-all",
        ];

        for invalid_token in invalid_tokens {
            let result = provider.validate_jwt(invalid_token);
            assert!(result.is_err(), "Token '{}' should be invalid", invalid_token);
        }
    }

    #[tokio::test]
    async fn test_jwt_token_validation_wrong_secret() {
        let config1 = create_test_security_config();
        let provider1 = ProductionSecurityProvider::new(config1).unwrap();
        
        let mut config2 = create_test_security_config();
        config2.jwt_secret = "different-secret-key-for-testing-32".to_string();
        let provider2 = ProductionSecurityProvider::new(config2).unwrap();
        
        let user = create_test_user("test_user", vec!["user"]);
        let token = provider1.generate_jwt(&user).unwrap();

        // Token generated with provider1 should not validate with provider2
        let result = provider2.validate_jwt(&token.token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwt_auth_provider_basic_authentication() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let credentials = Credentials::Basic {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = auth_provider.authenticate(&credentials).await.unwrap();
        assert!(result.success);
        assert!(result.user.is_some());
        assert!(result.token.is_some());
        assert_eq!(result.user.unwrap().username, "admin");
    }

    #[tokio::test]
    async fn test_jwt_auth_provider_basic_authentication_invalid_credentials() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let invalid_credentials = vec![
            Credentials::Basic { username: "admin".to_string(), password: "wrong".to_string() },
            Credentials::Basic { username: "wrong".to_string(), password: "admin123".to_string() },
            Credentials::Basic { username: "".to_string(), password: "".to_string() },
        ];

        for credentials in invalid_credentials {
            let result = auth_provider.authenticate(&credentials).await.unwrap();
            assert!(!result.success);
            assert!(result.user.is_none());
            assert!(result.token.is_none());
            assert!(result.error.is_some());
        }
    }

    #[tokio::test]
    async fn test_jwt_auth_provider_bearer_authentication() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("bearer_user", vec!["user"]);
        let token = provider.generate_jwt(&user).unwrap();

        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let credentials = Credentials::Bearer { token: token.token };
        let result = auth_provider.authenticate(&credentials).await.unwrap();

        assert!(result.success);
        assert!(result.user.is_some());
        assert_eq!(result.user.unwrap().id, "bearer_user");
    }

    #[tokio::test]
    async fn test_session_management_creation() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let credentials = Credentials::Basic {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = auth_provider.authenticate(&credentials).await.unwrap();
        let session = result.session.unwrap();

        assert!(!session.session_id.is_empty());
        assert_eq!(session.user_id, "admin");
        assert!(session.expires_at > session.created_at);
        assert!(!session.roles.is_empty());
    }

    #[tokio::test]
    async fn test_session_validation_token_lifecycle() {
        // Test that valid tokens work correctly
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("lifecycle_user", vec!["user"]);
        let token = provider.generate_jwt(&user).unwrap();

        // Valid token should work
        let result = provider.validate_token(&token.token).await;
        assert!(result.is_ok(), "Valid token should pass validation");
        
        let session = result.unwrap();
        assert_eq!(session.user_id, "lifecycle_user");
        assert!(session.roles.contains(&"user".to_string()));
        
        // Test that malformed tokens fail
        let invalid_token = "invalid.jwt.token";
        let result = provider.validate_token(invalid_token).await;
        assert!(result.is_err(), "Invalid token should fail validation");
        
        // Test that tokens with wrong signature fail
        let wrong_secret_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0IiwiaWF0IjoxNjAwMDAwMDAwLCJleHAiOjk5OTk5OTk5OTl9.invalid_signature";
        let result = provider.validate_token(wrong_secret_token).await;
        assert!(result.is_err(), "Token with wrong signature should fail validation");
    }

    #[tokio::test]
    async fn test_token_refresh_not_implemented() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let result = auth_provider.refresh_token("some-refresh-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        // Token revocation should succeed (even if it's a no-op currently)
        let result = auth_provider.revoke_token("some-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unsupported_credential_types() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let unsupported_credentials = vec![
            Credentials::ApiKey { key: "api-key".to_string(), secret: None },
            Credentials::Certificate { cert: vec![1, 2, 3], private_key: None },
        ];

        for credentials in unsupported_credentials {
            let result = auth_provider.authenticate(&credentials).await.unwrap();
            assert!(!result.success);
            assert!(result.error.is_some());
            assert!(result.error.unwrap().contains("Unsupported"));
        }
    }

    #[tokio::test]
    async fn test_oauth2_credential_structure() {
        let oauth2_creds = Credentials::OAuth2 {
            code: "auth-code-123".to_string(),
            state: Some("state-456".to_string()),
            redirect_uri: "http://localhost:8080/callback".to_string(),
        };

        match oauth2_creds {
            Credentials::OAuth2 { code, state, redirect_uri } => {
                assert_eq!(code, "auth-code-123");
                assert_eq!(state, Some("state-456".to_string()));
                assert_eq!(redirect_uri, "http://localhost:8080/callback");
            }
            _ => panic!("OAuth2 credentials not properly structured"),
        }
    }

    #[tokio::test]
    async fn test_authentication_result_comprehensive() {
        let auth_provider = JwtAuthProvider::new(
            TEST_JWT_SECRET.to_string(),
            Duration::from_secs(3600),
            "songbird-orchestrator".to_string(),
            "songbird-services".to_string(),
        );

        let credentials = Credentials::Basic {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = auth_provider.authenticate(&credentials).await.unwrap();

        // Verify all fields of AuthenticationResult
        assert!(result.success);
        assert!(result.user.is_some());
        assert!(result.token.is_some());
        assert!(result.session.is_some());
        assert!(result.error.is_none());
        assert!(!result.mfa_required);
        assert!(result.mfa_methods.is_empty());

        // Verify user details
        let user = result.user.unwrap();
        assert_eq!(user.username, "admin");
        assert!(user.email.is_some());
        assert!(!user.roles.is_empty());

        // Verify token details
        let token = result.token.unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert!(token.expires_in > 0);
        assert!(!token.token.is_empty());

        // Verify session details
        let session = result.session.unwrap();
        assert!(!session.session_id.is_empty());
        assert_eq!(session.user_id, "admin");
        assert!(session.expires_at > session.created_at);
    }

    #[tokio::test]
    async fn test_claims_structure_validation() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("claims_user", vec!["admin", "user"]);
        let token = provider.generate_jwt(&user).unwrap();
        let claims = provider.validate_jwt(&token.token).unwrap();

        // Verify all required JWT claims
        assert_eq!(claims.sub, "claims_user");
        assert_eq!(claims.iss, "songbird-orchestrator");
        assert_eq!(claims.aud, "songbird-services");
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.roles, vec!["admin", "user"]);
        assert!(!claims.custom.is_empty());

        // Verify custom claims contain user metadata
        assert!(claims.custom.contains_key("department"));
        assert!(claims.custom.contains_key("clearance_level"));
        assert!(claims.custom.contains_key("created_at"));
    }

    #[tokio::test]
    async fn test_production_security_provider_authentication_integration() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        // Test basic authentication
        let basic_creds = Credentials::Basic {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        };

        let result = provider.authenticate(&basic_creds).await.unwrap();
        assert!(result.success);
        assert!(result.user.is_some());
        assert!(result.token.is_some());

        // Test bearer authentication with generated token
        let token = result.token.unwrap();
        let bearer_creds = Credentials::Bearer { token: token.token };
        let bearer_result = provider.authenticate(&bearer_creds).await.unwrap();
        assert!(bearer_result.success);
        assert!(bearer_result.user.is_some());
    }

    #[tokio::test]
    async fn test_token_validation_integration() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("integration_user", vec!["user", "tester"]);
        let token = provider.generate_jwt(&user).unwrap();

        let session = provider.validate_token(&token.token).await.unwrap();

        assert_eq!(session.user_id, "integration_user");
        assert!(session.roles.contains(&"user".to_string()));
        assert!(session.roles.contains(&"tester".to_string()));
        assert!(session.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_refresh_token_placeholder() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        // Test the placeholder implementation
        let result = provider.refresh_token("dummy-refresh-token").await.unwrap();
        assert_eq!(result.token, "refreshed_token");
        assert_eq!(result.token_type, "Bearer");
        assert_eq!(result.expires_in, 3600);
        assert!(result.refresh_token.is_some());
    }

    #[tokio::test]
    async fn test_revoke_token_placeholder() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        // Test the placeholder implementation
        let result = provider.revoke_token("dummy-token").await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// ENCRYPTION & SECURITY OPERATIONS TESTING
// ============================================================================

#[cfg(test)]
mod encryption_tests {
    use super::*;

    #[tokio::test]
    async fn test_aes_encryption_basic() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = provider.encrypt(plaintext).unwrap();

        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() > plaintext.len()); // Should be longer due to nonce + tag
    }

    #[tokio::test]
    async fn test_aes_decryption_basic() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = provider.encrypt(plaintext).unwrap();
        let decrypted = provider.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn test_aes_encryption_decryption_large_data() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        // Test with larger data
        let large_data = vec![42u8; 10000];
        let encrypted = provider.encrypt(&large_data).unwrap();
        let decrypted = provider.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, large_data);
        assert!(encrypted.len() > large_data.len());
    }

    #[tokio::test]
    async fn test_aes_encryption_empty_data() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let empty_data = b"";
        let encrypted = provider.encrypt(empty_data).unwrap();
        let decrypted = provider.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, empty_data);
        assert!(encrypted.len() > 0); // Should contain nonce + tag even for empty data
    }

    #[tokio::test]
    async fn test_aes_encryption_different_plaintexts_different_ciphertexts() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let plaintext1 = b"Message 1";
        let plaintext2 = b"Message 2";

        let encrypted1 = provider.encrypt(plaintext1).unwrap();
        let encrypted2 = provider.encrypt(plaintext2).unwrap();

        // Different plaintexts should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But should decrypt correctly
        let decrypted1 = provider.decrypt(&encrypted1).unwrap();
        let decrypted2 = provider.decrypt(&encrypted2).unwrap();

        assert_eq!(decrypted1, plaintext1);
        assert_eq!(decrypted2, plaintext2);
    }

    #[tokio::test]
    async fn test_aes_encryption_same_plaintext_different_ciphertexts() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let plaintext = b"Same message";

        let encrypted1 = provider.encrypt(plaintext).unwrap();
        let encrypted2 = provider.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertexts due to random nonce
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        let decrypted1 = provider.decrypt(&encrypted1).unwrap();
        let decrypted2 = provider.decrypt(&encrypted2).unwrap();

        assert_eq!(decrypted1, plaintext);
        assert_eq!(decrypted2, plaintext);
    }

    #[tokio::test]
    async fn test_aes_decryption_invalid_ciphertext() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let invalid_ciphertexts = vec![
            vec![], // Empty
            vec![1, 2, 3], // Too short
            vec![0u8; 10], // Invalid but correct length
        ];

        for invalid_ciphertext in invalid_ciphertexts {
            let result = provider.decrypt(&invalid_ciphertext);
            assert!(result.is_err(), "Invalid ciphertext should fail decryption");
        }
    }

    #[tokio::test]
    async fn test_aes_decryption_tampered_ciphertext() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();

        let plaintext = b"Important message";
        let mut encrypted = provider.encrypt(plaintext).unwrap();

        // Tamper with the ciphertext
        if encrypted.len() > 20 {
            encrypted[20] ^= 1; // Flip a bit
        }

        let result = provider.decrypt(&encrypted);
        assert!(result.is_err(), "Tampered ciphertext should fail decryption");
    }

    #[tokio::test]
    async fn test_aes_encryption_with_different_keys() {
        let config1 = create_test_security_config();
        let mut config2 = create_test_security_config();
        config2.encryption_key = [2u8; 32]; // Different key

        let provider1 = ProductionSecurityProvider::new(config1).unwrap();
        let provider2 = ProductionSecurityProvider::new(config2).unwrap();

        let plaintext = b"Secret message";
        let encrypted = provider1.encrypt(plaintext).unwrap();

        // Decryption with different key should fail
        let result = provider2.decrypt(&encrypted);
        assert!(result.is_err(), "Different key should not decrypt successfully");
    }

    #[tokio::test]
    async fn test_security_config_defaults() {
        let config = SecurityConfig::default();

        assert!(!config.jwt_secret.is_empty());
        assert_eq!(config.jwt_expiration, Duration::from_secs(24 * 60 * 60));
        assert!(!config.enable_oauth);
        assert!(config.oauth_config.is_none());
        assert!(config.enable_audit);
    }

    #[tokio::test]
    async fn test_oauth2_config_structure() {
        let oauth_config = OAuth2Config::default();

        assert_eq!(oauth_config.client_id, "songbird-orchestrator");
        assert!(!oauth_config.client_secret.is_empty());
        assert!(oauth_config.auth_endpoint.starts_with("https://"));
        assert!(oauth_config.token_endpoint.starts_with("https://"));
        assert!(oauth_config.userinfo_endpoint.is_some());
        assert!(oauth_config.redirect_uri.contains("localhost"));
        assert!(!oauth_config.scopes.is_empty());
        assert!(oauth_config.scopes.contains(&"openid".to_string()));
    }
}

// ============================================================================
// AUDIT & SECURITY EVENTS TESTING
// ============================================================================

#[cfg(test)]
mod audit_tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_config_default() {
        let audit_config = AuditConfig::default();
        
        // Verify default audit configuration
        assert!(audit_config.enabled);
        assert!(matches!(audit_config.format, songbird_orchestrator::security::AuditFormat::Json));
        assert!(!audit_config.include_sensitive);
    }

    #[tokio::test]
    async fn test_auth_event_creation() {
        let auth_event = AuthEvent {
            event_type: AuthEventType::TokenGenerated,
            user_id: "test-user-123".to_string(),
            timestamp: Utc::now(),
            details: HashMap::from([
                ("token_type".to_string(), json!("Bearer")),
                ("expires_in".to_string(), json!(3600)),
                ("scopes".to_string(), json!(["read", "write"])),
            ]),
            success: true,
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("TestAgent/1.0".to_string()),
        };

        assert_eq!(auth_event.user_id, "test-user-123");
        assert!(matches!(auth_event.event_type, AuthEventType::TokenGenerated));
        assert!(auth_event.success);
        assert!(auth_event.ip_address.is_some());
        assert!(auth_event.user_agent.is_some());
        assert_eq!(auth_event.details.len(), 3);
    }

    #[tokio::test]
    async fn test_auth_event_types_comprehensive() {
        let event_types = vec![
            AuthEventType::Login,
            AuthEventType::LoginAttempt,
            AuthEventType::LoginFailed,
            AuthEventType::Logout,
            AuthEventType::TokenGenerated,
            AuthEventType::TokenValidated,
            AuthEventType::TokenRefreshed,
            AuthEventType::TokenRevoked,
            AuthEventType::MfaRequired,
            AuthEventType::MfaSuccess,
            AuthEventType::MfaFailed,
            AuthEventType::PasswordChanged,
            AuthEventType::AccountLocked,
            AuthEventType::AccountUnlocked,
            AuthEventType::AccessGranted,
            AuthEventType::AccessDenied,
        ];

        for event_type in event_types {
            let auth_event = AuthEvent {
                event_type: event_type.clone(),
                user_id: "test-user".to_string(),
                timestamp: Utc::now(),
                details: HashMap::new(),
                success: true,
                ip_address: None,
                user_agent: None,
            };

            // Verify event can be created with each type (using pattern matching instead of equality)
            match (&auth_event.event_type, &event_type) {
                (AuthEventType::Login, AuthEventType::Login) => {},
                (AuthEventType::LoginAttempt, AuthEventType::LoginAttempt) => {},
                (AuthEventType::LoginFailed, AuthEventType::LoginFailed) => {},
                (AuthEventType::Logout, AuthEventType::Logout) => {},
                (AuthEventType::TokenGenerated, AuthEventType::TokenGenerated) => {},
                (AuthEventType::TokenValidated, AuthEventType::TokenValidated) => {},
                (AuthEventType::TokenRefreshed, AuthEventType::TokenRefreshed) => {},
                (AuthEventType::TokenRevoked, AuthEventType::TokenRevoked) => {},
                (AuthEventType::MfaRequired, AuthEventType::MfaRequired) => {},
                (AuthEventType::MfaSuccess, AuthEventType::MfaSuccess) => {},
                (AuthEventType::MfaFailed, AuthEventType::MfaFailed) => {},
                (AuthEventType::PasswordChanged, AuthEventType::PasswordChanged) => {},
                (AuthEventType::AccountLocked, AuthEventType::AccountLocked) => {},
                (AuthEventType::AccountUnlocked, AuthEventType::AccountUnlocked) => {},
                (AuthEventType::AccessGranted, AuthEventType::AccessGranted) => {},
                (AuthEventType::AccessDenied, AuthEventType::AccessDenied) => {},
                _ => panic!("Event type mismatch"),
            }
        }
    }

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let audit_config = AuditConfig::default();
        let audit_logger = AuditLogger::new(audit_config);
        
        assert!(audit_logger.is_ok(), "Audit logger should be created successfully");
    }

    #[tokio::test]
    async fn test_audit_event_logging() {
        let audit_config = AuditConfig::default();
        let audit_logger = AuditLogger::new(audit_config).unwrap();

        let auth_event = AuthEvent {
            event_type: AuthEventType::Login,
            user_id: "audit-test-user".to_string(),
            timestamp: Utc::now(),
            details: HashMap::from([
                ("login_method".to_string(), json!("password")),
                ("session_duration".to_string(), json!(3600)),
            ]),
            success: true,
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        // Audit logging should not fail
        audit_logger.log_auth_event(auth_event);
    }

    #[tokio::test]
    async fn test_security_provider_audit_integration() {
        let config = create_test_security_config();
        let provider = ProductionSecurityProvider::new(config).unwrap();
        let user = create_test_user("audit_integration_user", vec!["user"]);

        // Generate token (should trigger audit event)
        let token = provider.generate_jwt(&user).unwrap();
        assert!(!token.token.is_empty());

        // Validate token (should trigger audit event)
        let claims = provider.validate_jwt(&token.token).unwrap();
        assert_eq!(claims.sub, "audit_integration_user");

        // Audit events should be logged internally (we can't easily verify without file access)
        // But we can verify the operations completed successfully
    }

    #[tokio::test]
    async fn test_audit_event_serialization() {
        let auth_event = AuthEvent {
            event_type: AuthEventType::AccessDenied,
            user_id: "serialization-test-user".to_string(),
            timestamp: Utc::now(),
            details: HashMap::from([
                ("resource".to_string(), json!("sensitive-document")),
                ("action".to_string(), json!("delete")),
                ("reason".to_string(), json!("insufficient_permissions")),
            ]),
            success: false,
            ip_address: Some("172.16.0.50".to_string()),
            user_agent: Some("CurlBot/1.0".to_string()),
        };

        // Test that auth events can be serialized to JSON
        let serialized = serde_json::to_string(&auth_event);
        assert!(serialized.is_ok(), "Auth event should be serializable");

        let json_str = serialized.unwrap();
        assert!(json_str.contains("AccessDenied"));
        assert!(json_str.contains("serialization-test-user"));
        assert!(json_str.contains("sensitive-document"));
        assert!(json_str.contains("172.16.0.50"));
    }

    #[tokio::test]
    async fn test_audit_event_deserialization() {
        let json_str = r#"{
            "event_type": "LoginFailed",
            "user_id": "deserialization-test-user",
            "timestamp": "2024-01-01T12:00:00Z",
            "details": {
                "failure_reason": "invalid_password",
                "attempt_count": 3
            },
            "success": false,
            "ip_address": "203.0.113.42",
            "user_agent": "TestClient/2.0"
        }"#;

        let deserialized: std::result::Result<AuthEvent, _> = serde_json::from_str(json_str);
        assert!(deserialized.is_ok(), "Auth event should be deserializable");

        let auth_event = deserialized.unwrap();
        assert_eq!(auth_event.user_id, "deserialization-test-user");
        assert!(matches!(auth_event.event_type, AuthEventType::LoginFailed));
        assert!(!auth_event.success);
        assert_eq!(auth_event.ip_address, Some("203.0.113.42".to_string()));
        assert!(auth_event.details.contains_key("failure_reason"));
    }
} 