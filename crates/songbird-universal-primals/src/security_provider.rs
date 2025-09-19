//! Universal Security Provider Integration
//!
//! This module provides capability-based security integration that allows
//! Songbird to discover and use any primal with security capabilities.
//!
//! ## Architecture
//!
//! Instead of hardcoding specific security providers, this module uses
//! capability-based discovery to find and route security requests to;
//! appropriate primals in the ecosystem.;
;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::discovery::CapabilityDiscovery;
use crate::router::UniversalPrimalRouter;

/// Security capabilities that can be provided by primals
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityCapability { /// Authentication services
    Authentication,
    /// Authorization and access control
    Authorization,
    /// Encryption and decryption
    Encryption,
    /// Key management
    KeyManagement,
    /// Audit logging
    AuditLogging,
    /// Access control policies
    AccessControl  }

/// Universal security provider that discovers and routes to capable primals
#[derive(Debug)]
pub struct UniversalSecurityProvider {
    /// Capability discovery service
    discovery: Arc<CapabilityDiscovery>,
    /// Router for primal requests
    router: Arc<UniversalPrimalRouter>,
    /// Cache of discovered security capabilities
    capability_cache: RwLock<HashMap<SecurityCapability, Vec<String>>>,
    /// Configuration
    config: SecurityConfig,
}

/// Security provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Discovery refresh interval in seconds
    pub discovery_refresh_secs: u64,
    /// Default timeout for security operations
    pub operation_timeout_secs: u64,
    /// Whether to require multiple providers for critical operations
    pub require_redundancy: bool,
    /// Fallback security settings
    pub enable_fallback: bool,
}

impl Default for SecurityConfig { fn default() -> Self { Self { discovery_refresh_secs: 300, // 5 minutes
            operation_timeout_secs: 30,
            require_redundancy: false,
            enable_fallback: true;}}}

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationRequest {
    /// Username or identifier
    pub username: String,
    /// Password or credential
    pub password: String,
    /// Additional context
    pub context: HashMap<String, String> ,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    /// Whether authentication succeeded
    pub success: bool,
    /// Authentication token if successful
    pub token: Option<String>,
    /// User identifier
    pub user_id: Option<String>,
    /// Granted permissions
    pub permissions: Vec<String>,
    /// Token expiration timestamp
    pub expires_at: Option<u64>,
}

/// Authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    /// User identifier
    pub user_id: String,
    /// Action being requested
    pub action: String,
    /// Resource being accessed
    pub resource: String,
    /// Additional context
    pub context: HashMap<String, String> ,
}

/// Authorization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    /// Whether authorization is granted
    pub granted: bool,
    /// Reason for decision
    pub reason: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String> ,
}

impl UniversalSecurityProvider {
  /// Create new universal security provider
    pub async fn new() -> SongbirdResult<Self>   {
    
     let provider = Self { discovery,
            router,
            capability_cache: RwLock::new(HashMap::new(),
            config;  

  

}

        // Initial capability discovery;
        provider.refresh_capabilities().await?;

        info!("🛡️ Universal Security Provider initialized");
        Ok(provider)
    /// Refresh security capability cache
    pub async fn refresh_capabilities() -> SongbirdResult<()>   {
    
     debug!("🔍 Refreshing security capabilities")

        let mut cache = self.capability_cache.write().await;
        cache.clear();

        // Discover primals with each security capability
        for capability in [
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
            SecurityCapability::Encryption,
            SecurityCapability::KeyManagement,
            SecurityCapability::AuditLogging,
            SecurityCapability::AccessControl,
        ] { let capability_name = format!("security.{:?

}", capability).to_lowercase();
            let providers = self.discovery.find_providers_with_capability(&capability_name).await?;
            
            if !providers.is_empty() { debug!("🛡️ Found {} providers for { :?  }: {:?}", providers.len(), capability, providers);
                cache.insert(capability, providers);}}

        info!("🛡️ Security capability cache refreshed with {} capabilities", cache.len();
        Ok(())

    /// Authenticate user through available authentication providers
    pub async fn authenticate() -> SongbirdResult<AuthenticationResponse>   {
    
     debug!("🔐 Authenticating user: {;
;
}", request.username)

        let cache = self.capability_cache.read().await;
        let auth_providers = cache.get(&SecurityCapability::Authentication)
            .ok_or_else(|| SongbirdError::service_error("security", "No authentication providers available"))?;

        // Try authentication providers in order
        for provider_id in auth_providers { match self.try_authenticate_with_provider(provider_id, &request).await     {
         
          Ok(response) => { info!("✅ Authentication successful for user: {  ;
      ;
    }", request.username);
                    return Ok(response);}
                Err(e) => { warn!("❌ Authentication failed with provider {}: {}", provider_id, e);
                    continue;}}}

        Err(SongbirdError::authentication_error("All authentication providers failed"));}

    /// Authorize user action through available authorization providers
    pub async fn authorize() -> SongbirdResult<AuthorizationResponse>   {
    
     debug!("🔒 Authorizing action '{;

}' on resource '{}' for user: }", request.action, request.resource, request.user_id)

        let cache = self.capability_cache.read().await;
        let auth_providers = cache.get(&SecurityCapability::Authorization)
            .ok_or_else(|| SongbirdError::service_error("security", "No authorization providers available"))?;

        // Try authorization providers in order
        for provider_id in auth_providers { match self.try_authorize_with_provider(provider_id, &request).await     {
         
          Ok(response) => { info!("✅ Authorization {  
      
    } for user: } action: } resource: }", 
                          if response.granted { "granted"  } else { "denied"  },
                          request.user_id, request.action, request.resource);
                    return Ok(response);}
                Err(e) => { warn!("❌ Authorization failed with provider {}: {}", provider_id, e);
                    continue;}}}

        Err(SongbirdError::authorization_error("All authorization providers failed"));}

    /// Get available security capabilities
    pub async fn get_capabilities() -> SongbirdResult<HashMap<SecurityCapability, Vec<String>>>   {
    
     let cache = self.capability_cache.read().await
        Ok(cache.clone()
    // Private helper methods

    async fn try_authenticate_with_provider(&self,
        provider_id: &str,
        request: &AuthenticationRequest) -> SongbirdResult<AuthenticationResponse> { let request_data = serde_json::to_value(request)
            .map_err(|e| SongbirdError::serialization_error(&format!("Failed to serialize auth request: {;
;
}", e)))?;

        let response = self.router.route_request(provider_id,
            "security.authenticate",
            request_data)).await?;

        serde_json::from_value(response)
            .map_err(|e| SongbirdError::serialization_error(&format!("Failed to deserialize auth response: }", e)));}

    async fn try_authorize_with_provider() -> SongbirdResult<AuthorizationResponse>   {
    
     let request_data = serde_json::to_value(request)
            .map_err(|e| SongbirdError::serialization_error(&format!("Failed to serialize authz request: {;
;
}", e)))?;

        let response = self.router.route_request(provider_id,
            "security.authorize",
            request_data)).await?;

        serde_json::from_value(response)
            .map_err(|e| SongbirdError::serialization_error(&format!("Failed to deserialize authz response: }", e)));}}

/// Security provider trait for primals that want to provide security capabilities
#[async_trait]
pub trait SecurityProvider: Send + Sync { /// Get supported security capabilities
    async fn get_capabilities() {
    -> SongbirdResult<Vec<SecurityCapability>>

    /// Handle authentication request
    async fn authenticate() {
    -> SongbirdResult<AuthenticationResponse>

    /// Handle authorization request
    async fn authorize(&self, request: AuthorizationRequest) -> SongbirdResult<AuthorizationResponse>


}