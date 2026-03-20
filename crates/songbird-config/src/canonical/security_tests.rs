// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for canonical security configuration types
//!
//! **PHASE 3 COVERAGE EXPANSION**: Targeting 0% → 80%+ coverage

use super::*;
use std::str::FromStr;

// =============================================================================
// SECURITY LEVEL TESTS
// =============================================================================

#[test]
fn test_security_level_default() {
    let level = SecurityLevel::default();
    assert_eq!(level, SecurityLevel::Public);
}

#[test]
fn test_security_level_display_all_variants() {
    assert_eq!(SecurityLevel::None.to_string(), "none");
    assert_eq!(SecurityLevel::Minimal.to_string(), "minimal");
    assert_eq!(SecurityLevel::Basic.to_string(), "basic");
    assert_eq!(SecurityLevel::Low.to_string(), "low");
    assert_eq!(SecurityLevel::Medium.to_string(), "medium");
    assert_eq!(SecurityLevel::Standard.to_string(), "standard");
    assert_eq!(SecurityLevel::Public.to_string(), "public");
    assert_eq!(SecurityLevel::High.to_string(), "high");
    assert_eq!(SecurityLevel::Private.to_string(), "private");
    assert_eq!(SecurityLevel::Critical.to_string(), "critical");
    assert_eq!(SecurityLevel::Confidential.to_string(), "confidential");
    assert_eq!(SecurityLevel::Enhanced.to_string(), "enhanced");
    assert_eq!(SecurityLevel::Maximum.to_string(), "maximum");
    assert_eq!(SecurityLevel::Classified.to_string(), "classified");
}

#[test]
fn test_security_level_from_str_all_variants() {
    assert_eq!(SecurityLevel::from_str("none").unwrap(), SecurityLevel::None);
    assert_eq!(SecurityLevel::from_str("minimal").unwrap(), SecurityLevel::Minimal);
    assert_eq!(SecurityLevel::from_str("basic").unwrap(), SecurityLevel::Basic);
    assert_eq!(SecurityLevel::from_str("low").unwrap(), SecurityLevel::Low);
    assert_eq!(SecurityLevel::from_str("medium").unwrap(), SecurityLevel::Medium);
    assert_eq!(SecurityLevel::from_str("standard").unwrap(), SecurityLevel::Standard);
    assert_eq!(SecurityLevel::from_str("public").unwrap(), SecurityLevel::Public);
    assert_eq!(SecurityLevel::from_str("high").unwrap(), SecurityLevel::High);
    assert_eq!(SecurityLevel::from_str("private").unwrap(), SecurityLevel::Private);
    assert_eq!(SecurityLevel::from_str("critical").unwrap(), SecurityLevel::Critical);
    assert_eq!(SecurityLevel::from_str("confidential").unwrap(), SecurityLevel::Confidential);
    assert_eq!(SecurityLevel::from_str("enhanced").unwrap(), SecurityLevel::Enhanced);
    assert_eq!(SecurityLevel::from_str("maximum").unwrap(), SecurityLevel::Maximum);
    assert_eq!(SecurityLevel::from_str("classified").unwrap(), SecurityLevel::Classified);
}

#[test]
fn test_security_level_from_str_case_insensitive() {
    assert_eq!(SecurityLevel::from_str("CRITICAL").unwrap(), SecurityLevel::Critical);
    assert_eq!(SecurityLevel::from_str("CrItIcAl").unwrap(), SecurityLevel::Critical);
    assert_eq!(SecurityLevel::from_str("HIGH").unwrap(), SecurityLevel::High);
}

#[test]
fn test_security_level_from_str_invalid() {
    assert!(SecurityLevel::from_str("invalid").is_err());
    assert!(SecurityLevel::from_str("").is_err());
    assert!(SecurityLevel::from_str("ultra").is_err());
}

#[test]
fn test_security_level_as_u8_all_variants() {
    assert_eq!(SecurityLevel::None.as_u8(), 0);
    assert_eq!(SecurityLevel::Minimal.as_u8(), 1);
    assert_eq!(SecurityLevel::Basic.as_u8(), 2);
    assert_eq!(SecurityLevel::Low.as_u8(), 3);
    assert_eq!(SecurityLevel::Medium.as_u8(), 4);
    assert_eq!(SecurityLevel::Standard.as_u8(), 5);
    assert_eq!(SecurityLevel::Public.as_u8(), 6);
    assert_eq!(SecurityLevel::High.as_u8(), 7);
    assert_eq!(SecurityLevel::Private.as_u8(), 8);
    assert_eq!(SecurityLevel::Critical.as_u8(), 9);
    assert_eq!(SecurityLevel::Confidential.as_u8(), 10);
    assert_eq!(SecurityLevel::Enhanced.as_u8(), 11);
    assert_eq!(SecurityLevel::Maximum.as_u8(), 12);
    assert_eq!(SecurityLevel::Classified.as_u8(), 13);
}

#[test]
fn test_security_level_from_u8_all_valid_values() {
    assert_eq!(SecurityLevel::from_u8(0), Some(SecurityLevel::None));
    assert_eq!(SecurityLevel::from_u8(1), Some(SecurityLevel::Minimal));
    assert_eq!(SecurityLevel::from_u8(2), Some(SecurityLevel::Basic));
    assert_eq!(SecurityLevel::from_u8(3), Some(SecurityLevel::Low));
    assert_eq!(SecurityLevel::from_u8(4), Some(SecurityLevel::Medium));
    assert_eq!(SecurityLevel::from_u8(5), Some(SecurityLevel::Standard));
    assert_eq!(SecurityLevel::from_u8(6), Some(SecurityLevel::Public));
    assert_eq!(SecurityLevel::from_u8(7), Some(SecurityLevel::High));
    assert_eq!(SecurityLevel::from_u8(8), Some(SecurityLevel::Private));
    assert_eq!(SecurityLevel::from_u8(9), Some(SecurityLevel::Critical));
    assert_eq!(SecurityLevel::from_u8(10), Some(SecurityLevel::Confidential));
    assert_eq!(SecurityLevel::from_u8(11), Some(SecurityLevel::Enhanced));
    assert_eq!(SecurityLevel::from_u8(12), Some(SecurityLevel::Maximum));
    assert_eq!(SecurityLevel::from_u8(13), Some(SecurityLevel::Classified));
}

#[test]
fn test_security_level_from_u8_invalid_values() {
    assert_eq!(SecurityLevel::from_u8(14), None);
    assert_eq!(SecurityLevel::from_u8(255), None);
    assert_eq!(SecurityLevel::from_u8(100), None);
}

#[test]
fn test_security_level_requires_authentication() {
    // Levels that don't require authentication
    assert!(!SecurityLevel::None.requires_authentication());
    assert!(!SecurityLevel::Public.requires_authentication());

    // Levels that require authentication
    assert!(SecurityLevel::Minimal.requires_authentication());
    assert!(SecurityLevel::Basic.requires_authentication());
    assert!(SecurityLevel::Low.requires_authentication());
    assert!(SecurityLevel::Medium.requires_authentication());
    assert!(SecurityLevel::Standard.requires_authentication());
    assert!(SecurityLevel::High.requires_authentication());
    assert!(SecurityLevel::Private.requires_authentication());
    assert!(SecurityLevel::Critical.requires_authentication());
    assert!(SecurityLevel::Confidential.requires_authentication());
    assert!(SecurityLevel::Enhanced.requires_authentication());
    assert!(SecurityLevel::Maximum.requires_authentication());
    assert!(SecurityLevel::Classified.requires_authentication());
}

#[test]
fn test_security_level_round_trip_u8() {
    for i in 0..=13 {
        let level = SecurityLevel::from_u8(i).unwrap();
        assert_eq!(level.as_u8(), i);
    }
}

#[test]
fn test_security_level_round_trip_string() {
    let levels = [
        SecurityLevel::None,
        SecurityLevel::Minimal,
        SecurityLevel::Basic,
        SecurityLevel::Low,
        SecurityLevel::Medium,
        SecurityLevel::Standard,
        SecurityLevel::Public,
        SecurityLevel::High,
        SecurityLevel::Private,
        SecurityLevel::Critical,
        SecurityLevel::Confidential,
        SecurityLevel::Enhanced,
        SecurityLevel::Maximum,
        SecurityLevel::Classified,
    ];

    for level in &levels {
        let string = level.to_string();
        let parsed = SecurityLevel::from_str(&string).unwrap();
        assert_eq!(*level, parsed);
    }
}

#[test]
fn test_security_level_serialization() {
    let level = SecurityLevel::Critical;
    let json = serde_json::to_string(&level).unwrap();
    let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(level, deserialized);
}

#[test]
fn test_security_level_clone_copy() {
    let level = SecurityLevel::High;
    let cloned = level;
    let copied = level;
    assert_eq!(level, cloned);
    assert_eq!(level, copied);
}

#[test]
fn test_security_level_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(SecurityLevel::Critical, "critical_data");
    map.insert(SecurityLevel::Public, "public_data");

    assert_eq!(map.get(&SecurityLevel::Critical), Some(&"critical_data"));
    assert_eq!(map.get(&SecurityLevel::Public), Some(&"public_data"));
}

// =============================================================================
// UNIVERSAL SECURITY CONFIG TESTS
// =============================================================================

#[test]
fn test_universal_security_config_default() {
    let config = UniversalSecurityConfig::default();

    assert!(!config.capability_requirements.encryption_capabilities.is_empty());
    assert!(!config.capability_requirements.authentication_capabilities.is_empty());
    assert!(config.authentication.enabled);
    assert!(config.encryption.enabled);
    assert!(config.access_control.enabled);
    assert!(config.provider_discovery.auto_discovery);
}

#[test]
fn test_universal_security_config_serialization() {
    let config = UniversalSecurityConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: UniversalSecurityConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(
        config.capability_requirements.minimum_security_level,
        deserialized.capability_requirements.minimum_security_level
    );
}

#[test]
fn test_universal_security_config_clone() {
    let config = UniversalSecurityConfig::default();
    let cloned = config.clone();

    assert_eq!(
        config.capability_requirements.encryption_capabilities.len(),
        cloned.capability_requirements.encryption_capabilities.len()
    );
}

// =============================================================================
// SECURITY CAPABILITY REQUIREMENTS TESTS
// =============================================================================

#[test]
fn test_security_capability_requirements_default() {
    let requirements = SecurityCapabilityRequirements::default();

    assert_eq!(requirements.encryption_capabilities.len(), 3);
    assert!(requirements.encryption_capabilities.contains(&"aes_256".to_string()));
    assert!(requirements.encryption_capabilities.contains(&"rsa_2048".to_string()));
    assert!(requirements.encryption_capabilities.contains(&"tls_1_3".to_string()));

    assert_eq!(requirements.authentication_capabilities.len(), 3);
    assert!(requirements.authentication_capabilities.contains(&"multi_factor".to_string()));

    assert_eq!(requirements.access_control_capabilities.len(), 2);
    assert!(requirements.access_control_capabilities.contains(&"role_based".to_string()));

    assert_eq!(requirements.minimum_security_level, "enterprise");
    assert_eq!(requirements.preferred_security_level, Some("quantum_resistant".to_string()));
}

#[test]
fn test_security_capability_requirements_custom() {
    let requirements = SecurityCapabilityRequirements {
        encryption_capabilities: vec!["custom_encryption".to_string()],
        authentication_capabilities: vec!["custom_auth".to_string()],
        access_control_capabilities: vec!["custom_access".to_string()],
        minimum_security_level: "high".to_string(),
        preferred_security_level: None,
    };

    assert_eq!(requirements.encryption_capabilities.len(), 1);
    assert!(requirements.preferred_security_level.is_none());
}

// =============================================================================
// AUTHENTICATION CONFIG TESTS
// =============================================================================

#[test]
fn test_authentication_config_default() {
    let config = AuthenticationConfig::default();

    assert!(config.enabled);
    assert_eq!(config.preferred_methods.len(), 3);
    assert_eq!(config.token_config.expiration_secs, 3600);
    assert_eq!(config.session_config.timeout_secs, 28800);
}

#[test]
fn test_authentication_config_serialization() {
    let config = AuthenticationConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AuthenticationConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.token_config.expiration_secs, deserialized.token_config.expiration_secs);
}

// =============================================================================
// TOKEN CONFIG TESTS
// =============================================================================

#[test]
fn test_token_config_default() {
    let config = TokenConfig::default();

    assert_eq!(config.expiration_secs, 3600);
    assert_eq!(config.refresh_threshold, 0.8);
    assert!(config.enable_rotation);
}

#[test]
fn test_token_config_custom() {
    let config = TokenConfig {
        expiration_secs: 7200,
        refresh_threshold: 0.5,
        enable_rotation: false,
    };

    assert_eq!(config.expiration_secs, 7200);
    assert_eq!(config.refresh_threshold, 0.5);
    assert!(!config.enable_rotation);
}

#[test]
fn test_token_config_serialization() {
    let config = TokenConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TokenConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.expiration_secs, deserialized.expiration_secs);
    assert_eq!(config.refresh_threshold, deserialized.refresh_threshold);
}

// =============================================================================
// SESSION CONFIG TESTS
// =============================================================================

#[test]
fn test_session_config_default() {
    let config = SessionConfig::default();

    assert_eq!(config.timeout_secs, 28800);
    assert!(!config.persistent);
    assert_eq!(config.max_concurrent_sessions, Some(5));
}

#[test]
fn test_session_config_custom() {
    let config = SessionConfig {
        timeout_secs: 3600,
        persistent: true,
        max_concurrent_sessions: None,
    };

    assert_eq!(config.timeout_secs, 3600);
    assert!(config.persistent);
    assert!(config.max_concurrent_sessions.is_none());
}

// =============================================================================
// ENCRYPTION CONFIG TESTS
// =============================================================================

#[test]
fn test_encryption_config_default() {
    let config = EncryptionConfig::default();

    assert!(config.enabled);
    assert_eq!(config.preferred_algorithms.len(), 3);
    assert_eq!(config.key_management.rotation_interval_secs, 86400 * 30);
    assert!(config.transport.require_tls);
}

#[test]
fn test_encryption_config_serialization() {
    let config = EncryptionConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: EncryptionConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
}

// =============================================================================
// KEY MANAGEMENT CONFIG TESTS
// =============================================================================

#[test]
fn test_key_management_config_default() {
    let config = KeyManagementConfig::default();

    assert_eq!(config.rotation_interval_secs, 86400 * 30); // 30 days
    assert!(config.auto_rotation);
}

#[test]
fn test_key_management_config_custom() {
    let config = KeyManagementConfig {
        rotation_interval_secs: 86400 * 7, // 7 days
        auto_rotation: false,
        key_derivation: KeyDerivationFunction::Argon2,
        storage_backend: KeyStorageBackend::Hsm,
    };

    assert_eq!(config.rotation_interval_secs, 86400 * 7);
    assert!(!config.auto_rotation);
}

// =============================================================================
// TRANSPORT ENCRYPTION CONFIG TESTS
// =============================================================================

#[test]
fn test_transport_encryption_config_default() {
    let config = TransportEncryptionConfig::default();

    assert!(config.require_tls);
    assert_eq!(config.preferred_cipher_suites.len(), 3);
    assert!(!config.certificate_pinning);
}

#[test]
fn test_transport_encryption_config_custom() {
    let config = TransportEncryptionConfig {
        require_tls: true,
        min_tls_version: TlsVersion::Tls12,
        preferred_cipher_suites: vec!["CUSTOM_CIPHER".to_string()],
        certificate_pinning: true,
    };

    assert!(config.require_tls);
    assert_eq!(config.preferred_cipher_suites.len(), 1);
    assert!(config.certificate_pinning);
}

// =============================================================================
// ACCESS CONTROL CONFIG TESTS
// =============================================================================

#[test]
fn test_access_control_config_default() {
    let config = AccessControlConfig::default();

    assert!(config.enabled);
    assert!(config.rbac.enabled);
    assert!(!config.abac.enabled); // ABAC disabled by default
}

#[test]
fn test_access_control_config_serialization() {
    let config = AccessControlConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AccessControlConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
}

// =============================================================================
// RBAC CONFIG TESTS
// =============================================================================

#[test]
fn test_rbac_config_default() {
    let config = RbacConfig::default();

    assert!(config.enabled);
    assert_eq!(config.default_roles.len(), 3);
    assert!(config.default_roles.contains(&"user".to_string()));
    assert!(config.default_roles.contains(&"admin".to_string()));
    assert!(config.default_roles.contains(&"system".to_string()));

    assert_eq!(config.role_hierarchy.len(), 2);
    assert!(config.role_hierarchy.contains_key("admin"));
    assert!(config.role_hierarchy.contains_key("system"));
}

#[test]
fn test_rbac_config_role_hierarchy() {
    let config = RbacConfig::default();

    let admin_inherits = config.role_hierarchy.get("admin").unwrap();
    assert_eq!(admin_inherits.len(), 1);
    assert!(admin_inherits.contains(&"user".to_string()));

    let system_inherits = config.role_hierarchy.get("system").unwrap();
    assert_eq!(system_inherits.len(), 2);
    assert!(system_inherits.contains(&"admin".to_string()));
    assert!(system_inherits.contains(&"user".to_string()));
}

// =============================================================================
// ABAC CONFIG TESTS
// =============================================================================

#[test]
fn test_abac_config_default() {
    let config = AbacConfig::default();

    assert!(!config.enabled); // Disabled by default
    assert_eq!(config.attribute_sources.len(), 3);
}

#[test]
fn test_abac_config_custom() {
    let config = AbacConfig {
        enabled: true,
        policy_engine: PolicyEngine::Opa,
        attribute_sources: vec![
            AttributeSource::User,
            AttributeSource::External("ldap".to_string()),
        ],
    };

    assert!(config.enabled);
    assert_eq!(config.attribute_sources.len(), 2);
}

// =============================================================================
// PROVIDER DISCOVERY CONFIG TESTS
// =============================================================================

#[test]
fn test_provider_discovery_config_default() {
    let config = ProviderDiscoveryConfig::default();

    assert!(config.auto_discovery);
    assert_eq!(config.discovery_interval_secs, 60);
    assert_eq!(config.health_check_interval_secs, 30);
    assert!(config.fallback.enable_builtin_fallback);
}

#[test]
fn test_provider_discovery_config_custom() {
    let config = ProviderDiscoveryConfig {
        auto_discovery: false,
        discovery_interval_secs: 120,
        health_check_interval_secs: 60,
        selection_strategy: ProviderSelectionStrategy::LoadBalance,
        fallback: FallbackConfig::default(),
    };

    assert!(!config.auto_discovery);
    assert_eq!(config.discovery_interval_secs, 120);
}

// =============================================================================
// FALLBACK CONFIG TESTS
// =============================================================================

#[test]
fn test_fallback_config_default() {
    let config = FallbackConfig::default();

    assert!(config.enable_builtin_fallback);
    assert_eq!(config.fallback_timeout_secs, 30);
    assert_eq!(config.max_fallback_attempts, 3);
}

#[test]
fn test_fallback_config_custom() {
    let config = FallbackConfig {
        enable_builtin_fallback: false,
        fallback_timeout_secs: 60,
        max_fallback_attempts: 5,
    };

    assert!(!config.enable_builtin_fallback);
    assert_eq!(config.fallback_timeout_secs, 60);
    assert_eq!(config.max_fallback_attempts, 5);
}

// =============================================================================
// ENUM SERIALIZATION TESTS
// =============================================================================

#[test]
fn test_authentication_method_serialization() {
    let methods = vec![
        AuthenticationMethod::BearerToken,
        AuthenticationMethod::ApiKey,
        AuthenticationMethod::Certificate,
        AuthenticationMethod::OAuth2,
        AuthenticationMethod::Saml,
        AuthenticationMethod::Custom("custom".to_string()),
    ];

    for method in methods {
        let json = serde_json::to_string(&method).unwrap();
        let _deserialized: AuthenticationMethod = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_encryption_algorithm_serialization() {
    let algorithms = vec![
        EncryptionAlgorithm::Aes256Gcm,
        EncryptionAlgorithm::Aes256Cbc,
        EncryptionAlgorithm::ChaCha20Poly1305,
        EncryptionAlgorithm::Rsa2048,
        EncryptionAlgorithm::Rsa4096,
        EncryptionAlgorithm::EccP256,
        EncryptionAlgorithm::EccP384,
        EncryptionAlgorithm::Custom("quantum".to_string()),
    ];

    for algorithm in algorithms {
        let json = serde_json::to_string(&algorithm).unwrap();
        let _deserialized: EncryptionAlgorithm = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_key_derivation_function_serialization() {
    let kdfs = vec![
        KeyDerivationFunction::Pbkdf2,
        KeyDerivationFunction::Scrypt,
        KeyDerivationFunction::Argon2,
        KeyDerivationFunction::Custom("custom_kdf".to_string()),
    ];

    for kdf in kdfs {
        let json = serde_json::to_string(&kdf).unwrap();
        let _deserialized: KeyDerivationFunction = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_key_storage_backend_serialization() {
    let backends = vec![
        KeyStorageBackend::CapabilityBased,
        KeyStorageBackend::Hsm,
        KeyStorageBackend::SecureEnclave,
        KeyStorageBackend::Custom("vault".to_string()),
    ];

    for backend in backends {
        let json = serde_json::to_string(&backend).unwrap();
        let _deserialized: KeyStorageBackend = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_tls_version_serialization() {
    let versions = vec![TlsVersion::Tls12, TlsVersion::Tls13];

    for version in versions {
        let json = serde_json::to_string(&version).unwrap();
        let _deserialized: TlsVersion = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_access_policy_serialization() {
    let policies = vec![
        AccessPolicy::Allow,
        AccessPolicy::Deny,
        AccessPolicy::Conditional(HashMap::from([(
            "department".to_string(),
            "engineering".to_string(),
        )])),
    ];

    for policy in policies {
        let json = serde_json::to_string(&policy).unwrap();
        let _deserialized: AccessPolicy = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_policy_engine_serialization() {
    let engines = vec![
        PolicyEngine::Simple,
        PolicyEngine::Xacml,
        PolicyEngine::Opa,
        PolicyEngine::Custom("custom_engine".to_string()),
    ];

    for engine in engines {
        let json = serde_json::to_string(&engine).unwrap();
        let _deserialized: PolicyEngine = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_attribute_source_serialization() {
    let sources = vec![
        AttributeSource::User,
        AttributeSource::Resource,
        AttributeSource::Environment,
        AttributeSource::External("ldap".to_string()),
    ];

    for source in sources {
        let json = serde_json::to_string(&source).unwrap();
        let _deserialized: AttributeSource = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_provider_selection_strategy_serialization() {
    let strategies = vec![
        ProviderSelectionStrategy::BestCapability,
        ProviderSelectionStrategy::FastestResponse,
        ProviderSelectionStrategy::LoadBalance,
        ProviderSelectionStrategy::FirstAvailable,
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy).unwrap();
        let _deserialized: ProviderSelectionStrategy = serde_json::from_str(&json).unwrap();
    }
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_full_security_config_round_trip() {
    let config = UniversalSecurityConfig::default();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&config).unwrap();

    // Deserialize back
    let deserialized: UniversalSecurityConfig = serde_json::from_str(&json).unwrap();

    // Verify key fields
    assert_eq!(config.authentication.enabled, deserialized.authentication.enabled);
    assert_eq!(config.encryption.enabled, deserialized.encryption.enabled);
    assert_eq!(config.access_control.enabled, deserialized.access_control.enabled);
}

#[test]
fn test_security_config_with_disabled_features() {
    let mut config = UniversalSecurityConfig::default();
    config.authentication.enabled = false;
    config.encryption.enabled = false;
    config.access_control.enabled = false;

    assert!(!config.authentication.enabled);
    assert!(!config.encryption.enabled);
    assert!(!config.access_control.enabled);
}

#[test]
fn test_security_config_debug_output() {
    let config = UniversalSecurityConfig::default();
    let debug_output = format!("{config:?}");

    assert!(!debug_output.is_empty());
    assert!(debug_output.contains("UniversalSecurityConfig"));
}

#[test]
fn test_security_level_ordering_by_value() {
    // Verify numeric ordering matches expected security hierarchy
    assert!(SecurityLevel::None.as_u8() < SecurityLevel::Minimal.as_u8());
    assert!(SecurityLevel::Low.as_u8() < SecurityLevel::Medium.as_u8());
    assert!(SecurityLevel::Medium.as_u8() < SecurityLevel::High.as_u8());
    assert!(SecurityLevel::High.as_u8() < SecurityLevel::Critical.as_u8());
    assert!(SecurityLevel::Critical.as_u8() < SecurityLevel::Maximum.as_u8());
    assert!(SecurityLevel::Maximum.as_u8() < SecurityLevel::Classified.as_u8());
}
