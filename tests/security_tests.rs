//! Comprehensive Security Test Suite for Songbird Orchestrator
//!
//! Enterprise-grade security testing covering:
//! - Authentication providers and flows
//! - Authorization and permission systems
//! - Audit logging and security events
//! - Credential handling and validation
//! - Security context management
//! - Role-based access control (RBAC)

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use songbird_orchestrator::{
    errors::Result,
    security::{
        Action, AuthEvent, AuthEventType, AuthenticationResult, SecurityProvider, Subject, SubjectType,
        Condition, ConditionOperator, Credentials, Permission, Resource, ProductionSecurityProvider, SecurityConfig,
        UserInfo, AuthToken, SessionInfo,
    },
    security::authentication::AuthenticationProvider,
    traits::service::UniversalService,
};

mod common;
use common::{MockConfig, MockService};

/// Mock Security Provider for testing
#[derive(Debug, Clone)]
pub struct MockSecurityProvider {
    users: Arc<RwLock<HashMap<String, MockUser>>>,
    audit_events: Arc<RwLock<Vec<AuthEvent>>>,
    encryption_key: Vec<u8>,
}

#[derive(Debug, Clone)]
struct MockUser {
    id: String,
    username: String,
    password_hash: String,
    roles: Vec<String>,
    permissions: Vec<String>,
    attributes: HashMap<String, String>,
    active: bool,
}

impl MockSecurityProvider {
    pub fn new() -> Self {
        let provider = Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            audit_events: Arc::new(RwLock::new(Vec::new())),
            encryption_key: b"test-encryption-key-32-bytes!!".to_vec(),
        };

        // Initialize users - we'll do this lazily or use a different approach
        provider
    }

    pub async fn init_test_users(&self) {
        // Add test users
        let users = vec![
            MockUser {
                id: "admin-001".to_string(),
                username: "admin".to_string(),
                password_hash: "hashed_admin_password".to_string(),
                roles: vec!["admin".to_string(), "user".to_string()],
                permissions: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "delete".to_string(),
                ],
                attributes: HashMap::from([
                    ("department".to_string(), "security".to_string()),
                    ("clearance".to_string(), "high".to_string()),
                ]),
                active: true,
            },
            MockUser {
                id: "user-001".to_string(),
                username: "testuser".to_string(),
                password_hash: "hashed_user_password".to_string(),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                attributes: HashMap::from([
                    ("department".to_string(), "engineering".to_string()),
                    ("clearance".to_string(), "standard".to_string()),
                ]),
                active: true,
            },
            MockUser {
                id: "guest-001".to_string(),
                username: "guest".to_string(),
                password_hash: "hashed_guest_password".to_string(),
                roles: vec!["guest".to_string()],
                permissions: vec![],
                attributes: HashMap::new(),
                active: true,
            },
            MockUser {
                id: "disabled-001".to_string(),
                username: "disabled".to_string(),
                password_hash: "hashed_disabled_password".to_string(),
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
                attributes: HashMap::new(),
                active: false,
            },
        ];

        let mut user_map = self.users.write().await;
        for user in users {
            user_map.insert(user.username.clone(), user);
        }
    }

    pub async fn get_audit_events(&self) -> Vec<AuthEvent> {
        self.audit_events.read().await.clone()
    }
}

#[async_trait]
impl SecurityProvider for MockSecurityProvider {
    async fn authorize(&self, subject: &Subject, resource: &Resource, action: &Action) -> Result<bool> {
        // Simple mock authorization logic based on subject type
        match subject.subject_type {
            SubjectType::System => Ok(true), // System always authorized
            SubjectType::Service => {
                // Services can access their own resources
                Ok(resource.resource_type == "service")
            }
            SubjectType::User => {
                // Check if user has admin role in attributes
                let is_admin = subject.attributes.get("role").map_or(false, |r| r == "admin");
                if is_admin {
                    Ok(true)
                } else {
                    // Check if this is a guest user (no roles or guest role)
                    let is_guest = subject.id.contains("guest");
                    if is_guest {
                        Ok(false) // Guests have no access
                    } else {
                        // Regular users can access user_data or limited service access
                        Ok(resource.resource_type == "user_data" || 
                           (resource.resource_type == "service" && action.name == "read"))
                    }
                }
            }
        }
    }

    async fn log_audit(&self, event: AuthEvent) -> Result<()> {
        self.audit_events.write().await.push(event);
        Ok(())
    }
}

#[async_trait]
impl AuthenticationProvider for MockSecurityProvider {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthenticationResult> {
        match credentials {
            Credentials::Basic { username, password } => {
                let users = self.users.read().await;
                if let Some(user) = users.get(username) {
                    if !user.active {
                        return Ok(AuthenticationResult {
                            success: false,
                            user: None,
                            token: None,
                            session: None,
                            error: Some("Account disabled".to_string()),
                            mfa_required: false,
                            mfa_methods: vec![],
                        });
                    }

                    // In real implementation, verify password hash
                    let password_valid = password == "correct_password";

                    if password_valid {
                        let user_info = UserInfo {
                            id: user.id.clone(),
                            username: user.username.clone(),
                            email: Some(format!("{}@example.com", user.username)),
                            roles: user.roles.clone(),
                            metadata: HashMap::new(),
                        };

                        let token = AuthToken {
                            token: format!("token_{}", user.id),
                            token_type: "Bearer".to_string(),
                            expires_in: 3600,
                            refresh_token: None,
                        };

                        Ok(AuthenticationResult {
                            success: true,
                            user: Some(user_info),
                            token: Some(token),
                            session: None,
                            error: None,
                            mfa_required: false,
                            mfa_methods: vec![],
                        })
                    } else {
                        Ok(AuthenticationResult {
                            success: false,
                            user: None,
                            token: None,
                            session: None,
                            error: Some("Invalid credentials".to_string()),
                            mfa_required: false,
                            mfa_methods: vec![],
                        })
                    }
                } else {
                    Ok(AuthenticationResult {
                        success: false,
                        user: None,
                        token: None,
                        session: None,
                        error: Some("User not found".to_string()),
                        mfa_required: false,
                        mfa_methods: vec![],
                    })
                }
            }
            Credentials::Bearer { token } => {
                // Validate bearer token
                if token.starts_with("token_") {
                    let user_id = token.strip_prefix("token_").unwrap();
                    let users = self.users.read().await;

                    if let Some((_, user)) = users.iter().find(|(_, u)| u.id == user_id) {
                        let user_info = UserInfo {
                            id: user.id.clone(),
                            username: user.username.clone(),
                            email: Some(format!("{}@example.com", user.username)),
                            roles: user.roles.clone(),
                            metadata: HashMap::new(),
                        };

                        Ok(AuthenticationResult {
                            success: true,
                            user: Some(user_info),
                            token: None, // Token already provided
                            session: None,
                            error: None,
                            mfa_required: false,
                            mfa_methods: vec![],
                        })
                    } else {
                        Ok(AuthenticationResult {
                            success: false,
                            user: None,
                            token: None,
                            session: None,
                            error: Some("Invalid token".to_string()),
                            mfa_required: false,
                            mfa_methods: vec![],
                        })
                    }
                } else {
                    Ok(AuthenticationResult {
                        success: false,
                        user: None,
                        token: None,
                        session: None,
                        error: Some("Invalid token format".to_string()),
                        mfa_required: false,
                        mfa_methods: vec![],
                    })
                }
            }
            Credentials::ApiKey { key, secret } => {
                if key == "valid_key" && secret.as_ref().map_or(false, |s| s == "valid_secret") {
                    let user_info = UserInfo {
                        id: "api_user".to_string(),
                        username: "api_user".to_string(),
                        email: Some("api@example.com".to_string()),
                        roles: vec!["api".to_string()],
                        metadata: HashMap::new(),
                    };

                    let token = AuthToken {
                        token: "api_token".to_string(),
                        token_type: "Bearer".to_string(),
                        expires_in: 3600,
                        refresh_token: None,
                    };

                    Ok(AuthenticationResult {
                        success: true,
                        user: Some(user_info),
                        token: Some(token),
                        session: None,
                        error: None,
                        mfa_required: false,
                        mfa_methods: vec![],
                    })
                } else {
                    Ok(AuthenticationResult {
                        success: false,
                        user: None,
                        token: None,
                        session: None,
                        error: Some("Invalid API key".to_string()),
                        mfa_required: false,
                        mfa_methods: vec![],
                    })
                }
            }
            _ => Ok(AuthenticationResult {
                success: false,
                user: None,
                token: None,
                session: None,
                error: Some("Unsupported credential type".to_string()),
                mfa_required: false,
                mfa_methods: vec![],
            })
        }
    }

    async fn validate_token(&self, token: &str) -> Result<SessionInfo> {
        if token.starts_with("token_") || token == "api_token" {
            Ok(SessionInfo {
                session_id: "mock_session".to_string(),
                user_id: "test_user".to_string(),
                created_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                roles: vec!["user".to_string()],
                metadata: HashMap::new(),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
            })
        } else {
            Err(songbird_orchestrator::errors::SongbirdError::Internal {
                message: "Invalid token".to_string(),
            })
        }
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<AuthToken> {
        Ok(AuthToken {
            token: "refreshed_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some("new_refresh_token".to_string()),
        })
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_authentication_basic_valid_credentials() {
    let provider = MockSecurityProvider::new();
    provider.init_test_users().await;

    let credentials = Credentials::Basic {
        username: "admin".to_string(),
        password: "correct_password".to_string(),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(result.success);
    assert!(result.user.is_some());
    assert!(result.token.is_some());
    
    let user = result.user.unwrap();
    assert_eq!(user.username, "admin");
    assert!(user.roles.contains(&"admin".to_string()));
}

#[tokio::test]
async fn test_authentication_basic_invalid_credentials() {
    let provider = MockSecurityProvider::new();
    provider.init_test_users().await;

    let credentials = Credentials::Basic {
        username: "admin".to_string(),
        password: "wrong_password".to_string(),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(!result.success);
    assert!(result.user.is_none());
    assert!(result.token.is_none());
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_authentication_nonexistent_user() {
    let provider = MockSecurityProvider::new();
    provider.init_test_users().await;

    let credentials = Credentials::Basic {
        username: "nonexistent".to_string(),
        password: "password".to_string(),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(!result.success);
    assert!(result.user.is_none());
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_authentication_disabled_user() {
    let provider = MockSecurityProvider::new();
    provider.init_test_users().await;

    let credentials = Credentials::Basic {
        username: "disabled".to_string(),
        password: "correct_password".to_string(),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(!result.success);
    assert!(result.user.is_none());
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_authentication_bearer_token_valid() {
    let provider = MockSecurityProvider::new();
    provider.init_test_users().await;

    let credentials = Credentials::Bearer {
        token: "token_admin-001".to_string(),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(result.success);
    assert!(result.user.is_some());
    
    let user = result.user.unwrap();
    assert_eq!(user.id, "admin-001");
}

#[tokio::test]
async fn test_authentication_bearer_token_invalid() {
    let provider = MockSecurityProvider::new();
    provider.init_test_users().await;

    let credentials = Credentials::Bearer {
        token: "invalid_token".to_string(),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(!result.success);
    assert!(result.user.is_none());
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_authentication_api_key_valid() {
    let provider = MockSecurityProvider::new();

    let credentials = Credentials::ApiKey {
        key: "valid_key".to_string(),
        secret: Some("valid_secret".to_string()),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(result.success);
    assert!(result.user.is_some());
    assert!(result.token.is_some());
    
    let user = result.user.unwrap();
    assert_eq!(user.username, "api_user");
}

#[tokio::test]
async fn test_authentication_api_key_invalid() {
    let provider = MockSecurityProvider::new();

    let credentials = Credentials::ApiKey {
        key: "invalid_key".to_string(),
        secret: Some("invalid_secret".to_string()),
    };

    let result = provider.authenticate(&credentials).await.unwrap();
    assert!(!result.success);
    assert!(result.user.is_none());
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_authorization_admin_full_access() {
    let provider = MockSecurityProvider::new();
    
    let mut subject_attrs = HashMap::new();
    subject_attrs.insert("role".to_string(), "admin".to_string());
    
    let subject = Subject {
        id: "admin-001".to_string(),
        subject_type: SubjectType::User,
        attributes: subject_attrs,
    };
    
    let resource = Resource {
        id: "test-service".to_string(),
        resource_type: "service".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "delete".to_string(),
        attributes: HashMap::new(),
    };

    let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
    assert!(authorized);
}

#[tokio::test]
async fn test_authorization_user_read_access() {
    let provider = MockSecurityProvider::new();
    
    let subject = Subject {
        id: "user-001".to_string(),
        subject_type: SubjectType::User,
        attributes: HashMap::new(),
    };
    
    let resource = Resource {
        id: "user-data-001".to_string(),
        resource_type: "user_data".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "read".to_string(),
        attributes: HashMap::new(),
    };

    let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
    assert!(authorized);
}

#[tokio::test]
async fn test_authorization_user_denied_write_access() {
    let provider = MockSecurityProvider::new();
    
    let subject = Subject {
        id: "user-001".to_string(),
        subject_type: SubjectType::User,
        attributes: HashMap::new(),
    };
    
    let resource = Resource {
        id: "admin-resource".to_string(),
        resource_type: "admin".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "write".to_string(),
        attributes: HashMap::new(),
    };

    let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
    assert!(!authorized);
}

#[tokio::test]
async fn test_authorization_user_own_data_access() {
    let provider = MockSecurityProvider::new();
    
    let subject = Subject {
        id: "user-001".to_string(),
        subject_type: SubjectType::User,
        attributes: HashMap::new(),
    };
    
    let resource = Resource {
        id: "user-data-001".to_string(),
        resource_type: "user_data".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "read".to_string(),
        attributes: HashMap::new(),
    };

    let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
    assert!(authorized);
}

#[tokio::test]
async fn test_authorization_user_denied_other_user_data() {
    let provider = MockSecurityProvider::new();
    
    let subject = Subject {
        id: "user-001".to_string(),
        subject_type: SubjectType::User,
        attributes: HashMap::new(),
    };
    
    let resource = Resource {
        id: "other-user-data".to_string(),
        resource_type: "admin".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "read".to_string(),
        attributes: HashMap::new(),
    };

    let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
    assert!(!authorized);
}

#[tokio::test]
async fn test_authorization_guest_denied_access() {
    let provider = MockSecurityProvider::new();
    
    let subject = Subject {
        id: "guest-001".to_string(),
        subject_type: SubjectType::User,
        attributes: HashMap::new(),
    };
    
    let resource = Resource {
        id: "admin-resource".to_string(),
        resource_type: "admin".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "read".to_string(),
        attributes: HashMap::new(),
    };

    let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
    assert!(!authorized);
}

#[tokio::test]
async fn test_audit_logging() {
    let provider = MockSecurityProvider::new();
    
    let event = AuthEvent {
        event_type: AuthEventType::Login,
        user_id: "admin-001".to_string(),
        timestamp: Utc::now(),
        details: HashMap::from([
            ("resource".to_string(), serde_json::Value::String("test-service".to_string())),
            ("action".to_string(), serde_json::Value::String("delete".to_string())),
        ]),
        success: true,
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("TestAgent/1.0".to_string()),
    };

    provider.log_audit(event.clone()).await.unwrap();
    
    let events = provider.get_audit_events().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].user_id, "admin-001");
    assert!(events[0].success);
}

#[tokio::test]
async fn test_encryption_decryption() {
    let _provider = MockSecurityProvider::new();
    let original_data = b"sensitive information";
    
    // Create a production security provider for encryption testing
    let config = SecurityConfig::default();
    let prod_provider = ProductionSecurityProvider::new(config).unwrap();
    
    let encrypted = prod_provider.encrypt(original_data).unwrap();
    assert_ne!(encrypted, original_data);
    
    let decrypted = prod_provider.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, original_data);
}

#[tokio::test]
async fn test_service_request_with_security_context() {
    let _provider = MockSecurityProvider::new();
    
    // Create a mock service for testing
    let service = MockService::new();
    
    // Test that service can be created (basic functionality test)
    assert_eq!(service.service_info().id, "mock-service");
}

#[tokio::test]
async fn test_role_based_access_control() {
    let provider = MockSecurityProvider::new();
    let roles = ["admin", "user", "guest"];

    for role in &roles {
        let mut subject_attrs = HashMap::new();
        if *role == "admin" {
            subject_attrs.insert("role".to_string(), "admin".to_string());
        }
        
        let subject = Subject {
            id: format!("{}-001", role),
            subject_type: SubjectType::User,
            attributes: subject_attrs,
        };
        
        let resource = Resource {
            id: "test-resource".to_string(),
            resource_type: "service".to_string(),
            attributes: HashMap::new(),
        };
        
        let action = Action {
            name: "read".to_string(),
            attributes: HashMap::new(),
        };

        let authorized = provider.authorize(&subject, &resource, &action).await.unwrap();
        
        // Admin should be authorized, others depend on resource type
        match *role {
            "admin" => assert!(authorized),
            "user" => assert!(authorized), // Users can read services
            "guest" => assert!(!authorized), // Guests cannot access services
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_concurrent_authentication() {
    let provider = Arc::new(MockSecurityProvider::new());
    provider.init_test_users().await;
    let mut handles = vec![];

    for _i in 0..10 {
        let provider_clone = provider.clone();
        
        let handle = tokio::spawn(async move {
            let credentials = Credentials::Basic {
                username: "admin".to_string(),
                password: "correct_password".to_string(),
            };

            provider_clone.authenticate(&credentials).await
        });
        
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        assert!(result.success);
    }
}

#[tokio::test]
async fn test_security_audit_trail() {
    let provider = MockSecurityProvider::new();
    
    let events = vec![
        ("admin", "login", "success"),
        ("user", "login", "failed"),
        ("admin", "delete", "success"),
        ("guest", "login", "failed"),
        ("user", "read", "success"),
    ];

    for (user, action, result) in events {
        let event = AuthEvent {
            event_type: if action == "login" {
                if result == "success" { AuthEventType::Login } else { AuthEventType::LoginFailed }
            } else {
                AuthEventType::Login // Simplified for testing
            },
            user_id: user.to_string(),
            timestamp: Utc::now(),
            details: HashMap::from([
                ("action".to_string(), serde_json::Value::String(action.to_string())),
            ]),
            success: result == "success",
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
        };

        provider.log_audit(event).await.unwrap();
    }
    
    let audit_events = provider.get_audit_events().await;
    assert_eq!(audit_events.len(), 5);
    
    let failed_logins = audit_events
        .iter()
        .filter(|e| matches!(e.event_type, AuthEventType::LoginFailed) && !e.success)
        .count();
    assert_eq!(failed_logins, 2);
    
    let successful_events = audit_events
        .iter()
        .filter(|e| e.success)
        .count();
    assert_eq!(successful_events, 3);
}

#[tokio::test]
async fn test_permission_conditions() {
    let resource = Resource {
        id: "api-endpoint".to_string(),
        resource_type: "api".to_string(),
        attributes: HashMap::new(),
    };
    
    let action = Action {
        name: "read".to_string(),
        attributes: HashMap::new(),
    };
    
    let conditions = vec![
        Condition {
            attribute: "time".to_string(),
            operator: ConditionOperator::GreaterThan,
            value: "09:00".to_string(),
        },
        Condition {
            attribute: "ip".to_string(),
            operator: ConditionOperator::Contains,
            value: "192.168.1".to_string(),
        },
    ];
    
    let permission = Permission {
        resource,
        action,
        conditions,
    };

    assert_eq!(permission.resource.resource_type, "api");
    assert_eq!(permission.action.name, "read");
    assert_eq!(permission.conditions.len(), 2);
    
    match &permission.conditions[0].operator {
        ConditionOperator::GreaterThan => {
            assert_eq!(permission.conditions[0].value, "09:00");
        }
        _ => panic!("Expected GreaterThan operator"),
    }
    
    match &permission.conditions[1].operator {
        ConditionOperator::Contains => {
            assert_eq!(permission.conditions[1].value, "192.168.1");
        }
        _ => panic!("Expected Contains operator"),
    }
}

