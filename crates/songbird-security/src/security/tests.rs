//! Security Module Tests
//!
//! Comprehensive test suite for the security module components.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::security::beardog::*;
    use crate::security::types::*;
    use crate::security::providers::*;
    use crate::security::managers::*;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_auth_token_creation() {
        let token = AuthToken::new(

            SubjectType::User,
            Duration::from_secs(3600),
            vec!["read".to_string(), "write".to_string()],
        );

        assert_eq!(token.subject, "test_user");
        assert_eq!(token.subject_type, SubjectType::User);
        assert_eq!(token.permissions.len(), 2);
        assert!(token.has_permission("read"));
        assert!(token.has_permission("write"));
        assert!(!token.has_permission("admin"));
    }

    #[test]
    fn test_password_policy_validation() {
        let config = SecurityConfig {
            password_policy: PasswordPolicy {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special_chars: true,
                max_age_days: 90,
            },
            ..Default::default()
        };

        let mut provider = InMemoryAuthProvider::new(config);

        // Valid password
        assert!(provider.add_user("user1".to_string(), "Password123!".to_string(), vec![]).is_ok());

        // Invalid passwords
        assert!(provider.add_user("user2".to_string(), "pass".to_string(), vec![]).is_err()); // Too short
        assert!(provider.add_user("user3".to_string(), "password123!".to_string(), vec![]).is_err()); // No uppercase
        assert!(provider.add_user("user4".to_string(), "PASSWORD123!".to_string(), vec![]).is_err()); // No lowercase
        assert!(provider.add_user("user5".to_string(), "Password!".to_string(), vec![]).is_err()); // No numbers
        assert!(provider.add_user("user6".to_string(), "Password123".to_string(), vec![]).is_err()); // No special chars
    }

    #[tokio::test]
    async fn test_in_memory_auth_provider() {
        let config = SecurityConfig::default();
        let mut provider = InMemoryAuthProvider::new(config);

        // Add a user
        provider.add_user(


            vec!["read".to_string(), "write".to_string()],
        ).expect("Failed to add user");

        // Test authentication
        let token = provider.authenticate("testuser", "TestPass123!").await.expect("Authentication failed");
        assert_eq!(token.subject, "testuser");
        assert_eq!(token.permissions.len(), 2);

        // Test invalid password
        let result = provider.authenticate("testuser", "wrongpass").await;
        assert!(result.is_err());

        // Test non-existent user
        let result = provider.authenticate("nonexistent", "password").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_and_action_creation() {
        let resource = Resource {
            resource_type: "file".to_string(),
            resource_id: "document.txt".to_string(),
            attributes: HashMap::new(),
        };

        let action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        assert_eq!(resource.resource_type, "file");
        assert_eq!(resource.resource_id, "document.txt");
        assert_eq!(action.action_type, "read");
    }

    #[test]
    fn test_permission_creation() {
        let resource = Resource {
            resource_type: "file".to_string(),
            resource_id: "document.txt".to_string(),
            attributes: HashMap::new(),
        };

        let action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        let permission = Permission {
            id: "perm_1".to_string(),
            subject: "user1".to_string(),
            subject_type: SubjectType::User,
            resource,
            action,
            effect: PermissionEffect::Allow,
            conditions: vec![],
        };

        assert_eq!(permission.id, "perm_1");
        assert_eq!(permission.subject, "user1");
        assert_eq!(permission.effect, PermissionEffect::Allow);
    }

    #[tokio::test]
    async fn test_security_manager() {
        let config = SecurityConfig::default();
        let mut auth_provider = InMemoryAuthProvider::new(config.clone());
        let authz_provider = InMemoryAuthzProvider::new();

        auth_provider.add_user("testuser".to_string(), "TestPass123!".to_string(), vec!["read".to_string()]).expect("Failed to add user");

        let manager = SecurityManager::new(
            Box::new(auth_provider),
            Box::new(authz_provider),
            config,
        );

        let token = manager.authenticate("testuser", "TestPass123!").await.expect("Authentication failed");
        assert_eq!(token.subject, "testuser");
    }

    #[tokio::test]
    async fn test_authentication_flow_comprehensive() {
        let config = SecurityConfig {
            authentication_enabled: true,
            authorization_enabled: true,
            session_timeout: Duration::from_secs(3600),
            max_login_attempts: 3,
            password_policy: PasswordPolicy {
                min_length: 8,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special_chars: true,
                max_age_days: 90,
            },
            ..Default::default()
        };

        let mut auth_provider = InMemoryAuthProvider::new(config.clone());
        let authz_provider = InMemoryAuthzProvider::new();

        // Add test users
        auth_provider.add_user("alice".to_string(), "SecurePass123!".to_string(), vec!["read".to_string(), "write".to_string()]).expect("Failed to add alice");
        auth_provider.add_user("bob".to_string(), "AnotherPass456#".to_string(), vec!["read".to_string()]).expect("Failed to add bob");

        let security_manager = SecurityManager::new(
            Box::new(auth_provider),
            Box::new(authz_provider),
            config,
        );

        // Test successful authentication
        let alice_token = security_manager.authenticate("alice", "SecurePass123!").await.expect("Alice authentication failed");
        assert_eq!(alice_token.subject, "alice");
        assert_eq!(alice_token.permissions.len(), 2);
        assert!(alice_token.has_permission("read"));
        assert!(alice_token.has_permission("write"));

        // Test Bob's authentication
        let bob_token = security_manager.authenticate("bob", "AnotherPass456#").await.expect("Bob authentication failed");
        assert_eq!(bob_token.subject, "bob");
        assert_eq!(bob_token.permissions.len(), 1);
        assert!(bob_token.has_permission("read"));
        assert!(!bob_token.has_permission("write"));

        // Test invalid credentials
        let invalid_result = security_manager.authenticate("alice", "wrongpassword").await;
        assert!(invalid_result.is_err());

        // Test non-existent user
        let nonexistent_result = security_manager.authenticate("charlie", "password").await;
        assert!(nonexistent_result.is_err());
    }

    #[tokio::test]
    async fn test_token_validation_and_expiration() {
        let config = SecurityConfig {
            session_timeout: Duration::from_millis(100), // Very short timeout for testing
            ..Default::default()
        };

        let mut auth_provider = InMemoryAuthProvider::new(config.clone());
        auth_provider.add_user("testuser".to_string(), "TestPass123!".to_string(), vec!["read".to_string()]).expect("Failed to add user");

        // Create a token that will expire quickly
        let short_lived_token = AuthToken::new(

            SubjectType::User,
            Duration::from_millis(100),
            vec!["read".to_string()],
        );

        // Token should be valid initially
        assert!(!short_lived_token.is_expired());

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        // Token should now be expired
        assert!(short_lived_token.is_expired());

        // Test token refresh
        let refreshed_token = auth_provider.refresh_token(&short_lived_token.token).await;
        assert!(refreshed_token.is_err()); // Should fail because original token is expired
    }

    #[tokio::test]
    async fn test_authorization_provider() {
        let mut authz_provider = InMemoryAuthzProvider::new();

        let resource = Resource {
            resource_type: "file".to_string(),
            resource_id: "document.txt".to_string(),
            attributes: HashMap::new(),
        };

        let read_action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        let write_action = Action {
            action_type: "write".to_string(),
            attributes: HashMap::new(),
        };

        // Add a permission
        let permission = Permission {
            id: "perm_1".to_string(),
            subject: "alice".to_string(),
            subject_type: SubjectType::User,
            resource: resource.clone(),
            action: read_action.clone(),
            effect: PermissionEffect::Allow,
            conditions: vec![],
        };

        authz_provider.add_permission(permission).await.expect("Failed to add permission");

        // Test authorization
        let context = HashMap::new();
        
        // Should be authorized for read
        let read_authorized = authz_provider.authorize("alice", SubjectType::User, &read_action, &resource, &context).await.expect("Authorization check failed");
        assert!(read_authorized);

        // Should not be authorized for write (no permission)
        let write_authorized = authz_provider.authorize("alice", SubjectType::User, &write_action, &resource, &context).await.expect("Authorization check failed");
        assert!(!write_authorized);

        // Different user should not be authorized
        let other_user_authorized = authz_provider.authorize("bob", SubjectType::User, &read_action, &resource, &context).await.expect("Authorization check failed");
        assert!(!other_user_authorized);
    }

    #[tokio::test]
    async fn test_security_manager_authentication_disabled() {
        let config = SecurityConfig {
            authentication_enabled: false,
            ..Default::default()
        };

        let auth_provider = InMemoryAuthProvider::new(config.clone());
        let authz_provider = InMemoryAuthzProvider::new();

        let security_manager = SecurityManager::new(
            Box::new(auth_provider),
            Box::new(authz_provider),
            config,
        );

        let result = security_manager.authenticate("testuser", "password").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_security_manager_authorization_flow() {
        let config = SecurityConfig::default();
        let mut auth_provider = InMemoryAuthProvider::new(config.clone());
        let authz_provider = InMemoryAuthzProvider::new();

        auth_provider.add_user("testuser".to_string(), "TestPass123!".to_string(), vec!["read".to_string()]).expect("Failed to add user");

        let security_manager = SecurityManager::new(
            Box::new(auth_provider),
            Box::new(authz_provider),
            config,
        );

        // Authenticate to get a token
        let token = security_manager.authenticate("testuser", "TestPass123!").await.expect("Authentication failed");

        // Test authorization
        let resource = Resource {
            resource_type: "file".to_string(),
            resource_id: "test.txt".to_string(),
            attributes: HashMap::new(),
        };

        let action = Action {
            action_type: "read".to_string(),
            attributes: HashMap::new(),
        };

        let context = HashMap::new();

        let authorized = security_manager.authorize(&token.token, &action, &resource, &context).await.expect("Authorization check failed");
        // Should be false because no explicit permission is set
        assert!(!authorized);
    }

    #[test]
    fn test_beardog_config_creation() {
        let config = BearDogConfig {
            endpoint: "https://beardog.example.com".to_string(),
            api_key: "test_key".to_string(),
            security_level: BearDogSecurityLevel::High,
            audit_level: BearDogAuditLevel::Detailed,
            compliance_mode: BearDogComplianceMode::SOC2,
            metadata: HashMap::new(),
        };

        assert_eq!(config.endpoint, "https://beardog.example.com");
        assert_eq!(config.security_level, BearDogSecurityLevel::High);
        assert_eq!(config.audit_level, BearDogAuditLevel::Detailed);
        assert_eq!(config.compliance_mode, BearDogComplianceMode::SOC2);
    }

    #[test]
    fn test_beardog_security_context() {
        let context = BearDogSecurityContext {
            security_level: BearDogSecurityLevel::Confidential,
            use_bstp: true,
            metadata: HashMap::new(),
        };

        assert_eq!(context.security_level, BearDogSecurityLevel::Confidential);
        assert!(context.use_bstp);
    }

    #[test]
    fn test_beardog_key_spec() {
        let key_spec = BearDogKeySpec {
            algorithm: "AES-256".to_string(),
            key_size: 256,
            purpose: BearDogKeyPurpose::DataEncryption,
            rotation_policy: BearDogRotationPolicy {
                interval_days: 30,
                auto_rotate: true,
            },
        };

        assert_eq!(key_spec.algorithm, "AES-256");
        assert_eq!(key_spec.key_size, 256);
        assert_eq!(key_spec.purpose, BearDogKeyPurpose::DataEncryption);
        assert_eq!(key_spec.rotation_policy.interval_days, 30);
        assert!(key_spec.rotation_policy.auto_rotate);
    }

    #[test]
    fn test_beardog_principal_and_resource() {
        let principal = BearDogPrincipal {
            id: "user123".to_string(),
            principal_type: BearDogPrincipalType::User,
            attributes: HashMap::new(),
        };

        let resource = BearDogResource {
            id: "resource456".to_string(),
            resource_type: "file".to_string(),
            owner: "user123".to_string(),
            attributes: HashMap::new(),
        };

        assert_eq!(principal.id, "user123");
        assert_eq!(principal.principal_type, BearDogPrincipalType::User);
        assert_eq!(resource.id, "resource456");
        assert_eq!(resource.resource_type, "file");
        assert_eq!(resource.owner, "user123");
    }

    #[test]
    fn test_beardog_security_event() {
        use chrono::Utc;

        let event = BearDogSecurityEvent {
            event_id: "event123".to_string(),
            event_type: BearDogSecurityEventType::Authentication,
            principal: BearDogPrincipal {
                id: "user123".to_string(),
                principal_type: BearDogPrincipalType::User,
                attributes: HashMap::new(),
            },
            resource: None,
            action: None,
            timestamp: Utc::now(),
            outcome: BearDogSecurityOutcome::Success,
            details: HashMap::new(),
        };

        assert_eq!(event.event_id, "event123");
        assert_eq!(event.event_type, BearDogSecurityEventType::Authentication);
        assert_eq!(event.outcome, BearDogSecurityOutcome::Success);
    }

    #[test]
    fn test_beardog_encrypted_data() {
        let encrypted_data = BearDogEncryptedData {
            data: vec![1, 2, 3, 4, 5],
            algorithm: "AES-256-GCM".to_string(),
            key_id: "key123".to_string(),
        };

        assert_eq!(encrypted_data.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(encrypted_data.algorithm, "AES-256-GCM");
        assert_eq!(encrypted_data.key_id, "key123");
    }

    #[test]
    fn test_condition_operators() {
        let condition = Condition {
            attribute: "department".to_string(),
            operator: ConditionOperator::Equals,
            value: "engineering".to_string(),
        };

        assert_eq!(condition.attribute, "department");
        assert_eq!(condition.value, "engineering");
        matches!(condition.operator, ConditionOperator::Equals);
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();

        assert!(config.authentication_enabled);
        assert!(config.authorization_enabled);
        assert!(config.encryption_enabled);
        assert!(config.audit_logging);
        assert_eq!(config.session_timeout, Duration::from_secs(3600));
        assert_eq!(config.max_login_attempts, 3);
    }

    #[test]
    fn test_password_policy_defaults() {
        let policy = PasswordPolicy::default();

        assert_eq!(policy.min_length, 8);
        assert!(policy.require_uppercase);
        assert!(policy.require_lowercase);
        assert!(policy.require_numbers);
        assert!(policy.require_special_chars);
        assert_eq!(policy.max_age_days, 90);
    }

    #[test]
    fn test_subject_types() {
        let user = SubjectType::User;
        let service = SubjectType::Service;
        let role = SubjectType::Role;
        let group = SubjectType::Group;

        assert_eq!(user, SubjectType::User);
        assert_eq!(service, SubjectType::Service);
        assert_eq!(role, SubjectType::Role);
        assert_eq!(group, SubjectType::Group);
    }

    #[test]
    fn test_permission_effects() {
        let allow = PermissionEffect::Allow;
        let deny = PermissionEffect::Deny;

        assert_eq!(allow, PermissionEffect::Allow);
        assert_eq!(deny, PermissionEffect::Deny);
    }

    #[test]
    fn test_beardog_compliance_modes() {
        let standard = BearDogComplianceMode::Standard;
        let fips = BearDogComplianceMode::FIPS140;
        let soc2 = BearDogComplianceMode::SOC2;
        let gdpr = BearDogComplianceMode::GDPR;

        assert_eq!(standard, BearDogComplianceMode::Standard);
        assert_eq!(fips, BearDogComplianceMode::FIPS140);
        assert_eq!(soc2, BearDogComplianceMode::SOC2);
        assert_eq!(gdpr, BearDogComplianceMode::GDPR);
    }

    #[test]
    fn test_beardog_audit_levels() {
        let minimal = BearDogAuditLevel::Minimal;
        let standard = BearDogAuditLevel::Standard;
        let detailed = BearDogAuditLevel::Detailed;
        let comprehensive = BearDogAuditLevel::Comprehensive;
        let paranoid = BearDogAuditLevel::Paranoid;

        assert_eq!(minimal, BearDogAuditLevel::Minimal);
        assert_eq!(standard, BearDogAuditLevel::Standard);
        assert_eq!(detailed, BearDogAuditLevel::Detailed);
        assert_eq!(comprehensive, BearDogAuditLevel::Comprehensive);
        assert_eq!(paranoid, BearDogAuditLevel::Paranoid);
    }

    #[test]
    fn test_auth_token_expiration() {
        let token = AuthToken::new(

            SubjectType::User,
            Duration::from_secs(0), // Immediate expiration
            vec!["read".to_string()],
        );

        // Should be expired immediately
        assert!(token.is_expired());
    }
} 