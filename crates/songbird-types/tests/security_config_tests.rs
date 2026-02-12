//! Tests for security configuration types

use songbird_types::config::security::*;
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_canonical_security_config_default() {
    let config = CanonicalSecurityConfig::default();
    // Default should have security enabled
    assert!(config.enabled);
    // Nested configs should have defaults
    assert!(config.authentication.enabled);
    assert!(config.authorization.enabled);
    assert!(config.encryption.enabled);
}

#[test]
fn test_authentication_config_default() {
    let config = AuthenticationConfig::default();
    assert!(config.enabled);
    assert_eq!(config.session_timeout, Duration::from_secs(3600));
}

#[test]
fn test_authentication_config_clone() {
    let config = AuthenticationConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.session_timeout, cloned.session_timeout);
}

#[test]
fn test_authentication_config_serialization() {
    let config = AuthenticationConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("enabled"));
    assert!(json.contains("session_timeout"));
}

#[test]
fn test_authentication_method_default() {
    let method = AuthenticationMethod::default();
    assert!(matches!(method, AuthenticationMethod::Jwt));
}

#[test]
fn test_authentication_method_variants() {
    let _none = AuthenticationMethod::None;
    let _basic = AuthenticationMethod::Basic;
    let _oauth2 = AuthenticationMethod::OAuth2;
    let _jwt = AuthenticationMethod::Jwt;
    let _mfa = AuthenticationMethod::Mfa;
}

#[test]
fn test_authentication_method_serialization() {
    let method = AuthenticationMethod::OAuth2;
    let json = serde_json::to_string(&method).expect("Serialization should succeed");
    assert!(json.contains("OAuth2"));
}

#[test]
fn test_authorization_config_default() {
    let config = AuthorizationConfig::default();
    assert!(config.enabled);
    assert!(config.rbac_enabled);
    assert_eq!(config.default_role, "user");
}

#[test]
fn test_authorization_config_clone() {
    let config = AuthorizationConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.rbac_enabled, cloned.rbac_enabled);
    assert_eq!(config.default_role, cloned.default_role);
}

#[test]
fn test_authorization_config_serialization() {
    let config = AuthorizationConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("rbac_enabled"));
    assert!(json.contains("default_role"));
}

#[test]
fn test_encryption_config_default() {
    let config = EncryptionConfig::default();
    assert!(config.enabled);
    assert_eq!(config.algorithm, "AES-256-GCM");
    assert_eq!(config.key_size, 256);
}

#[test]
fn test_encryption_config_clone() {
    let config = EncryptionConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.algorithm, cloned.algorithm);
    assert_eq!(config.key_size, cloned.key_size);
}

#[test]
fn test_encryption_config_serialization() {
    let config = EncryptionConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("AES-256-GCM"));
    assert!(json.contains("256"));
}

#[test]
fn test_security_provider_integration_config_default() {
    let config = SecurityProviderIntegrationConfig::default();
    assert!(!config.enabled);
    assert!(config.providers.is_empty());
}

#[test]
fn test_security_provider_config_default() {
    let config = SecurityProviderConfig::default();
    assert_eq!(config.name, "default");
    assert!(config.endpoint.contains("localhost"));
    assert!(config.credentials.is_empty());
}

#[test]
fn test_security_provider_config_clone() {
    let config = SecurityProviderConfig::default();
    let cloned = config.clone();
    assert_eq!(config.name, cloned.name);
    assert_eq!(config.endpoint, cloned.endpoint);
}

#[test]
fn test_security_provider_config_serialization() {
    let config = SecurityProviderConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("name"));
    assert!(json.contains("endpoint"));
}

#[test]
fn test_mfa_method_default() {
    let method = MfaMethod::default();
    assert!(matches!(method, MfaMethod::Totp));
}

#[test]
fn test_mfa_method_variants() {
    let _totp = MfaMethod::Totp;
    let _sms = MfaMethod::Sms;
    let _email = MfaMethod::Email;
    let _hardware = MfaMethod::Hardware;
}

#[test]
fn test_mfa_method_serialization() {
    let method = MfaMethod::Hardware;
    let json = serde_json::to_string(&method).expect("Serialization should succeed");
    assert!(json.contains("Hardware"));
}

#[test]
fn test_mfa_settings_default() {
    let settings = MfaSettings::default();
    assert!(!settings.enabled);
    assert!(settings.required_for_admin);
    assert_eq!(settings.methods.len(), 1);
    assert!(matches!(settings.methods[0], MfaMethod::Totp));
}

#[test]
fn test_mfa_settings_clone() {
    let settings = MfaSettings::default();
    let cloned = settings.clone();
    assert_eq!(settings.enabled, cloned.enabled);
    assert_eq!(settings.required_for_admin, cloned.required_for_admin);
    assert_eq!(settings.methods.len(), cloned.methods.len());
}

#[test]
fn test_mfa_settings_serialization() {
    let settings = MfaSettings::default();
    let json = serde_json::to_string(&settings).expect("Serialization should succeed");
    assert!(json.contains("required_for_admin"));
    assert!(json.contains("methods"));
}

#[test]
fn test_security_config_with_custom_values() {
    let config = CanonicalSecurityConfig {
        enabled: false,
        authentication: AuthenticationConfig {
            enabled: false,
            method: AuthenticationMethod::Basic,
            session_timeout: Duration::from_secs(7200),
        },
        authorization: AuthorizationConfig {
            enabled: false,
            rbac_enabled: false,
            default_role: "admin".to_string(),
        },
        encryption: EncryptionConfig {
            enabled: false,
            algorithm: "ChaCha20-Poly1305".to_string(),
            key_size: 256,
        },
        security_provider_integration: SecurityProviderIntegrationConfig {
            enabled: true,
            providers: {
                let mut providers = HashMap::new();
                providers.insert(
                    "beardog".to_string(),
                    SecurityProviderConfig {
                        name: "beardog".to_string(),
                        endpoint: "https://security.example.com:443".to_string(),
                        credentials: HashMap::new(),
                    },
                );
                providers
            },
        },
    };

    assert!(!config.enabled);
    assert!(!config.authentication.enabled);
    assert_eq!(config.authorization.default_role, "admin");
    assert_eq!(config.encryption.algorithm, "ChaCha20-Poly1305");
    assert!(config.security_provider_integration.enabled);
    assert_eq!(config.security_provider_integration.providers.len(), 1);
}

