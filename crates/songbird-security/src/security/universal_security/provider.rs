//! Universal Security Provider - Main Coordinator Coordinator
//!
//! This module provides the main UniversalSecurityProvider that coordinates
//! all security operations across the modular security system.

use crate::security::universal_security::{
    capabilities::SecurityCapabilityDiscovery,
    types::{SecurityContext, SecurityLevel, SecurityResult, SubjectType},
};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal_primals::{
    traits::PrimalContext,
    universal_registry::UniversalServiceRegistry,
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Universal Security Provider that discovers and uses available security primals
pub struct UniversalSecurityProvider {
    /// Security capability discovery and management
    capability_discovery: SecurityCapabilityDiscovery,
    /// Default context for primal operations
    default_context: PrimalContext,
    /// Fallback security provider for standalone operation
    fallback_provider: Arc<FallbackSecurityProvider>,
}

impl UniversalSecurityProvider {
    /// Create a new universal security provider
    #[must_use]
    pub fn new(
        primal_registry: Arc<dyn UniversalServiceRegistry>,
        default_context: PrimalContext,
    ) -> Self {
        Self {
            capability_discovery: SecurityCapabilityDiscovery::new(primal_registry),
            default_context,
            fallback_provider: Arc::new(FallbackSecurityProvider::new()),
        }
    }

    /// Authenticate a user or service
    pub async fn authenticate(&self, credentials: &str, subject_type: SubjectType) -> SecurityResult<AuthenticationToken> {
        let context = SecurityContext {
            request_id: uuid::Uuid::new_v4(),
            subject: "system".to_string(),
            subject_type: SubjectType::System,
            operation: "authenticate".to_string(),
            resource: None,
            metadata: std::collections::HashMap::new(),
            required_level: SecurityLevel::Standard,
        };

        info!("Authenticating with universal security provider for subject type: {}",
              match context.subject_type {
                  SubjectType::User => "user",
                  SubjectType::Service => "service", 
                  SubjectType::System => "system",
                  SubjectType::Client => "client",
              });

        // Try to find a suitable security capability
        match self.capability_discovery
            .get_best_capability("authentication", context.required_level.clone())
            .await
        {
            Ok(Some(capability)) => {
                debug!("Using security capability from primal: {}", capability.primal_name);
                self.authenticate_with_primal(&capability, &context).await
            }
            Ok(None) => {
                debug!("No suitable security capability found, using fallback");
                self.fallback_provider.authenticate(&context).await
            }
            Err(e) => {
                debug!("Failed to discover security capabilities: {}, using fallback", e);
                self.fallback_provider.authenticate(&context).await
            }
        }
    }

    /// Authorize an operation
    pub async fn authorize(&self, token: &AuthenticationToken, operation: &str, resource: &str) -> SecurityResult<bool> {
        let context = SecurityContext {
            request_id: uuid::Uuid::new_v4(),
            subject: token.subject.clone(),
            subject_type: token.subject_type.clone(),
            operation: operation.to_string(),
            resource: Some(resource.to_string()),
            metadata: std::collections::HashMap::new(),
            required_level: SecurityLevel::Standard,
        };

        info!("Authorizing {} operation on {} for {}", operation, resource, token.subject);

        // Try to find a suitable security capability
        match self.capability_discovery
            .get_best_capability("authorization", context.required_level.clone())
            .await
        {
            Ok(Some(capability)) => {
                debug!("Using authorization capability from primal: {}", capability.primal_id);
                self.authorize_with_primal(&capability, token, resource, operation, &context).await
            }
            Ok(None) => {
                debug!("No suitable authorization capability found, using fallback");
                self.fallback_provider.authorize(token, resource, operation).await
            }
            Err(e) => {
                debug!("Failed to discover authorization capabilities: {}, using fallback", e);
                self.fallback_provider.authorize(token, resource, operation).await
            }
        }
    }

    /// Refresh security capabilities cache
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub async fn refresh_capabilities() -> Result<(), SongbirdError> {
        self.capability_discovery.refresh_capabilities().await;
    }

    /// Get security status and health information
    pub async fn get_security_status() -> SecurityResult<SecurityStatus> {
        let capabilities = match self.capability_discovery.discover_capabilities().await {
            Ok(caps) => caps,
            Err(e) => { return SecurityResult::failure(format!("Failed to discover capabilities: {}", e)); }
        };
        let status = SecurityStatus {
            available_capabilities: capabilities.len(),
            active_primals: capabilities.iter().map(|c| c.primal_id.clone()).collect(),
            fallback_active: true,
            last_capability_refresh: std::time::SystemTime::now()
        };
        SecurityResult::success(status)
    }

    /// Authenticate using a specific primal capability
    async fn authenticate_with_primal(&self,
        _capability: &crate::security::universal_security::types::SecurityCapabilityInfo,
        _context: &SecurityContext) -> SecurityResult<AuthenticationToken> {
        // This would implement actual primal-based authentication
        // For now, return a placeholder implementation
        SecurityResult::success(AuthenticationToken {
            token: "primal-auth-token".to_string(),
            subject: "authenticated-user".to_string(),
            subject_type: SubjectType::User,
            expires_at: std::time::SystemTime::now() + Duration::from_secs(3600)
        })
    }

    /// Authorize using a specific primal capability
    async fn authorize_with_primal(&self,
        _capability: &crate::security::universal_security::types::SecurityCapabilityInfo,
        _token: &AuthenticationToken,
        _resource: &str,
        _operation: &str,
        _context: &SecurityContext) -> SecurityResult<bool> { // This would implement actual primal-based authorization
        // For now, return a placeholder implementation
        SecurityResult::success(true)
    }
}

/// Authentication token returned by successful authentication
#[derive(Debug, Clone)]
#[must_use = "Guards and handles must be kept alive for their effect"]
pub struct AuthenticationToken {
    /// The authentication token string
    /// Token field
    pub token: String,
    /// Subject (user/service) that was authenticated
    pub subject: String,
    /// Type of subject
    pub subject_type: SubjectType,
    /// Token expiration time
    pub expires_at: std::time::SystemTime,
}

/// Security status information
#[derive(Debug, Clone)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct SecurityStatus {
    /// Number of available security capabilities
    pub available_capabilities: usize,
    /// List of active primal IDs providing security
    pub active_primals: Vec<String>,
    /// Whether fallback security is active
    pub last_capability_refresh: std::time::SystemTime,
}

/// Fallback security provider for standalone operation
pub struct FallbackSecurityProvider {
    // Minimal implementation for when no primals are available
}

impl FallbackSecurityProvider { /// Create a new fallback security provider
    #[must_use]
    pub fn new() -> Self { Self {} }

    /// Fallback authentication implementation
    pub async fn authenticate(&self, _context: &SecurityContext) -> SecurityResult<AuthenticationToken> {
        // Simple fallback authentication - in production this would be more sophisticated
        SecurityResult::success(AuthenticationToken {
            token: "fallback-auth-token".to_string(),
            subject: "fallback-user".to_string(),
            subject_type: SubjectType::User,
            expires_at: std::time::SystemTime::now() + Duration::from_secs(3600)
        })
    }

    /// Fallback authorization implementation
    pub async fn authorize(&self,
        _token: &AuthenticationToken,
        _resource: &str,
        _operation: &str) -> SecurityResult<bool> { // Simple fallback authorization - allows all operations in fallback mode
        SecurityResult::success(true)
    }
}

impl Default for FallbackSecurityProvider { 
    fn default() -> Self { 
        Self::new() 
    } 
}
#[cfg(test)]
mod tests {
    use super::*;

    // Mock registry for testing;
    struct MockRegistry;

    #[async_trait::async_trait]
    impl UniversalServiceRegistry for MockRegistry {
        async fn register_service(&self, _service: songbird_universal_primals::types::ServiceInfo) -> SongbirdResult<()> { Ok(()) }
        async fn discover_services(&self, _capability: songbird_universal_primals::traits::PrimalCapability) -> SongbirdResult<Vec<songbird_universal_primals::types::ServiceInfo>> { Ok(vec![]) }
        async fn get_service_health(&self, _service_id: &str) -> SongbirdResult<songbird_universal_primals::types::ServiceHealth> { Ok(songbird_universal_primals::types::ServiceHealth::Healthy) }
    }

    #[tokio::test]
    async fn test_universal_security_provider_creation() {
        let registry = Arc::new(MockRegistry);
        let context = PrimalContext {
            user_id: "test-user".to_string(),
            device_id: "test-device".to_string(),
            session_id: "test-session".to_string(),
        };
        let provider = UniversalSecurityProvider::new(registry, context);
        
        // Test that we can create the provider
        let status = provider.get_security_status().await;
        assert!(status.success);
    }

    #[tokio::test]
    async fn test_fallback_authentication() {
        let registry = Arc::new(MockRegistry);
        let context = PrimalContext { user_id: "test-user".to_string(),
            device_id: "test-device".to_string(),
            session_id: "test-session".to_string() };
    let provider = UniversalSecurityProvider::new(registry, context);
        
        // Test fallback authentication
        let result = provider.authenticate("test-credentials", SubjectType::User).await;
        assert!(result.success);
        
        if let Some(token) = result.data { assert_eq!(token.subject_type, SubjectType::User);
            assert!(!token.token.is_empty()); }
    }
}
