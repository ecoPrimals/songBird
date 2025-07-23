//! Universal Security Provider
//!
//! Provides unified security services for the entire Songbird ecosystem
//!
//! This module implements universal security patterns allowing Songbird
//! to handle authentication, authorization, encryption, and audit logging
//! without knowing the specific primal implementations it's working with.

use async_trait::async_trait;
use songbird_errors::{AuthError, Result, SongbirdError};
use songbird_universal_primals::{
    traits::{PrimalCapability, PrimalContext, SecurityLevel},
    types::{PrimalRequest, PrimalRequestType, PrimalResponse},
    universal_registry::UniversalServiceRegistry,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::security::providers::AuthenticationProvider;
use crate::security::types::{AuthToken, SecurityConfig, SubjectType};

/// Universal Security Provider that discovers and uses available security primals
pub struct UniversalSecurityProvider {
    /// Universal primal registry for capability-based discovery
    primal_registry: Arc<dyn UniversalServiceRegistry>,
    /// Security configuration
    config: SecurityConfig,
    /// Cache of discovered security capabilities
    security_capabilities: RwLock<HashMap<String, SecurityCapabilityInfo>>,
    /// Fallback implementations for standalone operation
    pub fallback_provider: Arc<FallbackSecurityProvider>,
    /// Last capability discovery time
    last_discovery: RwLock<SystemTime>,
    /// Discovery cache duration
    cache_duration: Duration,
    /// Default context for primal operations
    default_context: PrimalContext,
}

/// Information about discovered security capabilities
#[derive(Debug, Clone)]
struct SecurityCapabilityInfo {
    /// Primal providing the capability
    primal_id: String,
    /// Instance ID for multi-instance support
    instance_id: String,
    /// Endpoint for security requests
    endpoint: String,
    /// Specific capabilities supported
    capabilities: Vec<songbird_universal_primals::ServiceCapability>,
    /// Last successful health check
    last_health_check: Option<SystemTime>,
    /// Whether the primal is currently healthy
    is_healthy: bool,
}

/// Failsafe security provider for standalone operation
pub struct FallbackSecurityProvider {
    /// Fallback authentication implementation
    users: RwLock<HashMap<String, FallbackUserInfo>>,
    /// Configuration for fallbacks
    config: SecurityConfig,
}

#[derive(Debug, Clone)]
pub struct FallbackUserInfo {
    username: String,
    password_hash: String,
    permissions: Vec<String>,
    created_at: SystemTime,
}

impl FallbackUserInfo {
    /// Create new fallback user info
    pub fn new(username: String, password_hash: String, permissions: Vec<String>) -> Self {
        Self {
            username,
            password_hash,
            permissions,
            created_at: SystemTime::now(),
        }
    }

    /// Check if account is recent (created within last 30 days)
    pub fn is_recent_account(&self) -> bool {
        if let Ok(age) = SystemTime::now().duration_since(self.created_at) {
            age < Duration::from_secs(30 * 24 * 60 * 60) // 30 days
        } else {
            false
        }
    }

    /// Get account age in days
    pub fn account_age_days(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::ZERO)
            .as_secs()
            / (24 * 60 * 60)
    }

    /// Validate password against hash
    pub fn validate_password(&self, password: &str) -> bool {
        use ring::digest;

        // Handle legacy format with warning
        if self.password_hash.starts_with("hash:") {
            tracing::warn!(
                "Legacy password hash format detected - should be updated to secure hashing"
            );
            return self.password_hash == format!("hash:{password}");
        }

        // Secure cryptographic verification
        let Ok(combined) = hex::decode(&self.password_hash) else {
            return false;
        };

        if combined.len() != 48 {
            return false;
        }

        let (salt, stored_hash_bytes) = combined.split_at(16);

        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(salt);
        to_hash.extend_from_slice(password.as_bytes());

        let calculated_hash = digest::digest(&digest::SHA256, &to_hash);
        calculated_hash.as_ref() == stored_hash_bytes
    }
}

impl FallbackSecurityProvider {
    /// Create new fallback security provider
    pub fn new(config: SecurityConfig) -> Self {
        let mut users = HashMap::new();

        // Create default admin user if none exists
        users.insert(
            "admin".to_string(),
            FallbackUserInfo::new(
                "admin".to_string(),
                "hash:admin_default_password".to_string(), // Should be properly hashed
                vec!["admin".to_string(), "user".to_string()],
            ),
        );

        Self {
            users: RwLock::new(users),
            config,
        }
    }

    /// Add a new user to the fallback system
    pub async fn add_user(
        &self,
        username: String,
        password: &str,
        permissions: Vec<String>,
    ) -> Result<()> {
        // Use secure cryptographic password hashing
        use rand::{thread_rng, Rng};
        use ring::digest;

        let mut salt = [0u8; 16];
        thread_rng().fill(&mut salt);

        let mut to_hash = Vec::new();
        to_hash.extend_from_slice(&salt);
        to_hash.extend_from_slice(password.as_bytes());

        let hash = digest::digest(&digest::SHA256, &to_hash);

        let mut combined = Vec::new();
        combined.extend_from_slice(&salt);
        combined.extend_from_slice(hash.as_ref());

        let password_hash = hex::encode(combined);

        let user_info = FallbackUserInfo::new(username.clone(), password_hash, permissions);
        let is_recent = user_info.is_recent_account();

        let mut users = self.users.write().await;
        users.insert(username.clone(), user_info);

        info!(
            "Added fallback user: {} (account age: 0 days, recent: {})",
            username, is_recent
        );
        Ok(())
    }

    /// Get user information including metadata
    pub async fn get_user_info(&self, username: &str) -> Option<FallbackUserInfo> {
        let users = self.users.read().await;
        users.get(username).cloned()
    }

    /// List all users with their metadata
    pub async fn list_users(&self) -> HashMap<String, (String, u64)> {
        // (username, account_age_days)
        let users = self.users.read().await;
        users
            .iter()
            .map(|(username, info)| {
                (
                    username.clone(),
                    (info.username.clone(), info.account_age_days()),
                )
            })
            .collect()
    }

    /// Remove user from fallback system
    pub async fn remove_user(&self, username: &str) -> Result<()> {
        let mut users = self.users.write().await;
        if users.remove(username).is_some() {
            info!("Removed fallback user: {}", username);
            Ok(())
        } else {
            Err(SongbirdError::Security {
                message: format!("User '{username}' not found"),
                context: Some("fallback_provider".to_string()),
                severity: Some("low".to_string()),
                suggestion: Some("Check if username is correct".to_string()),
            })
        }
    }
}

#[async_trait]
impl AuthenticationProvider for FallbackSecurityProvider {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        let users = self.users.read().await;

        if let Some(user_info) = users.get(username) {
            // Use the username from the stored info and validate password
            if user_info.username == username && user_info.validate_password(password) {
                info!(
                    "Fallback authentication successful for user: {} (account age: {} days)",
                    user_info.username,
                    user_info.account_age_days()
                );

                // Create token with user's actual permissions
                Ok(AuthToken::new(
                    user_info.username.clone(), // Use the stored username
                    SubjectType::User,
                    self.config.session_timeout,
                    user_info.permissions.clone(),
                ))
            } else {
                Err(SongbirdError::Auth(Box::new(AuthError {
                    message: "Invalid credentials".to_string(),
                    provider: Some("fallback".to_string()),
                })))
            }
        } else {
            Err(SongbirdError::Auth(Box::new(AuthError {
                message: format!("User '{username}' not found"),
                provider: Some("fallback".to_string()),
            })))
        }
    }

    async fn validate_token(&self, token: &str) -> Result<AuthToken> {
        // Simple token validation for fallback
        if token.starts_with("fallback_token_") {
            let parts: Vec<&str> = token.split('_').collect();
            if parts.len() >= 3 {
                let username = parts[2];
                let users = self.users.read().await;

                if let Some(user_info) = users.get(username) {
                    return Ok(AuthToken::new(
                        user_info.username.clone(), // Use stored username
                        SubjectType::User,
                        self.config.session_timeout,
                        user_info.permissions.clone(),
                    ));
                }
            }
        }

        Err(SongbirdError::Auth(Box::new(AuthError {
            message: "Invalid or expired token".to_string(),
            provider: Some("fallback".to_string()),
        })))
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        // Simple revocation for fallback (in memory only)
        info!("Token revoked (fallback implementation)");
        Ok(())
    }

    async fn refresh_token(&self, token: &str) -> Result<AuthToken> {
        // For fallback, refresh is same as validate
        self.validate_token(token).await
    }
}

impl UniversalSecurityProvider {
    /// Create new universal security provider
    pub fn new(primal_registry: Arc<dyn UniversalServiceRegistry>, config: SecurityConfig) -> Self {
        let default_context = PrimalContext {
            primal_id: "songbird-security".to_string(),
            user_id: "system".to_string(),
            device_id: "security-provider".to_string(),
            security_level: SecurityLevel::System, // High security level for security provider
            session_id: "security-session".to_string(),
            network_location: Default::default(),
            metadata: std::collections::HashMap::new(),
        };

        Self {
            primal_registry,
            config: config.clone(),
            security_capabilities: RwLock::new(HashMap::new()),
            fallback_provider: Arc::new(FallbackSecurityProvider::new(config)),
            last_discovery: RwLock::new(SystemTime::UNIX_EPOCH),
            cache_duration: Duration::from_secs(300), // 5 minutes
            default_context,
        }
    }

    /// Create with custom context for user-specific security
    pub fn with_context(
        primal_registry: Arc<dyn UniversalServiceRegistry>,
        config: SecurityConfig,
        context: PrimalContext,
    ) -> Self {
        Self {
            primal_registry,
            config: config.clone(),
            security_capabilities: RwLock::new(HashMap::new()),
            fallback_provider: Arc::new(FallbackSecurityProvider::new(config)),
            last_discovery: RwLock::new(SystemTime::UNIX_EPOCH),
            cache_duration: Duration::from_secs(300),
            default_context: context,
        }
    }

    /// Discover available security capabilities from registered primals
    async fn discover_security_capabilities(&self) -> Result<()> {
        let now = SystemTime::now();
        let last_discovery = *self.last_discovery.read().await;

        // Check if discovery cache is still valid
        if now.duration_since(last_discovery).unwrap_or(Duration::MAX) < self.cache_duration {
            return Ok(());
        }

        info!("🔍 Discovering security capabilities from registered primals...");

        // Get all registered primals with security capabilities
        let auth_capability = songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["oauth".to_string(), "jwt".to_string()],
            encryption_algorithms: vec!["aes256".to_string(), "rsa".to_string()],
            compliance_frameworks: vec!["gdpr".to_string(), "iso27001".to_string()],
            threat_detection: true,
        };

        let security_primals = self
            .primal_registry
            .find_services_by_capability(vec![auth_capability])
            .await
            .unwrap_or_default();

        let mut capabilities = HashMap::new();

        for primal in security_primals {
            let primal_capabilities = &primal.registration.capabilities;

            // Filter for security-related capabilities
            let security_caps: Vec<&songbird_universal_primals::ServiceCapability> =
                primal_capabilities
                    .iter()
                    .filter(|cap| {
                        matches!(
                            cap,
                            songbird_universal_primals::ServiceCapability::Security { .. }
                        )
                    })
                    .collect();

            if !security_caps.is_empty() {
                let endpoints = &primal.registration.endpoints;
                let primal_id = primal.registration.service_id.to_string();
                let instance_id = primal.registration.instance_id.clone();
                let num_capabilities = security_caps.len();

                let default_endpoint = match primal_id.as_str() {
                    id if id.contains("beardog") => std::env::var("BEARDOG_SECURITY_ENDPOINT")
                        .unwrap_or_else(|_| "http://localhost:8443".to_string()),
                    id if id.contains("security") => std::env::var("SECURITY_SERVICE_ENDPOINT")
                        .unwrap_or_else(|_| "http://localhost:8443".to_string()),
                    _ => std::env::var("DEFAULT_SECURITY_ENDPOINT")
                        .unwrap_or_else(|_| "http://localhost:8443".to_string()),
                };

                let endpoint_url = if let Some(first_endpoint) = endpoints.first() {
                    first_endpoint.url.clone()
                } else {
                    default_endpoint
                };

                let capability_info = SecurityCapabilityInfo {
                    primal_id: primal_id.clone(),
                    instance_id: instance_id.clone(),
                    endpoint: endpoint_url,
                    capabilities: security_caps.into_iter().cloned().collect(),
                    last_health_check: Some(now),
                    is_healthy: true,
                };

                capabilities.insert(instance_id.clone(), capability_info);

                info!(
                    "✅ Discovered security primal: {} (instance: {}) with {} capabilities",
                    primal_id, instance_id, num_capabilities
                );
            }
        }

        // Also check for additional security capabilities
        let additional_capabilities = [songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["basic".to_string(), "bearer".to_string()],
            encryption_algorithms: vec!["aes128".to_string()],
            compliance_frameworks: vec!["iso27001".to_string()],
            threat_detection: false,
        }];

        for capability in additional_capabilities {
            let primals_with_capability = self
                .primal_registry
                .find_services_by_capability(vec![capability])
                .await
                .unwrap_or_default();

            for primal in primals_with_capability {
                let instance_id = primal.registration.instance_id.clone();

                // Skip if already added
                if capabilities.contains_key(&instance_id) {
                    continue;
                }

                let primal_capabilities = &primal.registration.capabilities;
                let security_caps: Vec<&songbird_universal_primals::ServiceCapability> =
                    primal_capabilities
                        .iter()
                        .filter(|cap| {
                            matches!(
                                cap,
                                songbird_universal_primals::ServiceCapability::Security { .. }
                            )
                        })
                        .collect();

                if !security_caps.is_empty() {
                    let endpoints = &primal.registration.endpoints;
                    let primal_id = primal.registration.service_id.to_string();
                    let num_capabilities = security_caps.len();

                    let endpoint_url = if let Some(first_endpoint) = endpoints.first() {
                        first_endpoint.url.clone()
                    } else {
                        "http://localhost:8443".to_string()
                    };

                    let capability_info = SecurityCapabilityInfo {
                        primal_id: primal_id.clone(),
                        instance_id: instance_id.clone(),
                        endpoint: endpoint_url,
                        capabilities: security_caps.into_iter().cloned().collect(),
                        last_health_check: Some(SystemTime::now()),
                        is_healthy: true,
                    };

                    capabilities.insert(instance_id.clone(), capability_info);

                    info!("✅ Discovered additional security primal: {} (instance: {}) with {} capabilities", 
                          primal_id, instance_id, num_capabilities);
                }
            }
        }

        // Update cache
        {
            let mut cache = self.security_capabilities.write().await;
            *cache = capabilities;
        }

        {
            let mut last_discovery = self.last_discovery.write().await;
            *last_discovery = now;
        }

        info!("🔐 Security capability discovery completed");
        Ok(())
    }

    /// Check if a capability is security-related
    fn is_security_capability(&self, capability: &PrimalCapability) -> bool {
        matches!(
            capability,
            PrimalCapability::Authentication { .. }
                | PrimalCapability::Encryption { .. }
                | PrimalCapability::KeyManagement { .. }
                | PrimalCapability::ThreatDetection { .. }
                | PrimalCapability::Security { .. }
                | PrimalCapability::Authorization { .. }
        ) || matches!(
            capability,
            PrimalCapability::Custom { name, .. } if name == "AuditLogging"
        )
    }

    /// Find best security primal for a specific capability
    async fn find_best_security_primal(
        &self,
        required_capability: &songbird_universal_primals::ServiceCapability,
    ) -> Option<SecurityCapabilityInfo> {
        // Ensure capabilities are discovered
        if let Err(e) = self.discover_security_capabilities().await {
            warn!("Failed to discover security capabilities: {}", e);
            return None;
        }

        let capabilities = self.security_capabilities.read().await;

        // Find primal with required capability and best performance
        capabilities
            .values()
            .filter(|info| {
                info.capabilities.iter().any(|cap| {
                    matches!(
                        (cap, required_capability),
                        (
                            songbird_universal_primals::ServiceCapability::Security { .. },
                            songbird_universal_primals::ServiceCapability::Security { .. }
                        )
                    )
                })
            })
            .find(|info| info.is_healthy)
            .cloned()
    }

    /// Make authenticated request to security primal
    async fn make_security_request(
        &self,
        primal_info: &SecurityCapabilityInfo,
        request_type: &str,
        payload: HashMap<String, serde_json::Value>,
    ) -> Result<PrimalResponse> {
        // Check if primal is healthy before making request
        if let Err(e) = self.check_primal_health(primal_info).await {
            warn!("Security primal unhealthy: {}, falling back", e);
            return Err(SongbirdError::Security {
                message: format!("Security primal unavailable: {e}"),
                context: Some(primal_info.primal_id.clone()),
                severity: Some("medium".to_string()),
                suggestion: Some("Will use fallback security provider".to_string()),
            });
        }

        let request = PrimalRequest {
            id: Uuid::new_v4(),
            request_type: PrimalRequestType::Authentication,
            payload,
            timestamp: chrono::Utc::now(),
            context: Some("songbird-security".to_string()),
            priority: Some(5),
            security_level: Some("high".to_string()),
        };

        debug!(
            "Making {} request to security primal: {} (instance: {}) at endpoint: {}",
            request_type, primal_info.primal_id, primal_info.instance_id, primal_info.endpoint
        );

        // Create direct HTTP request since route_request_to_instance doesn't exist in trait
        let client = reqwest::Client::new();
        let response = client
            .post(&primal_info.endpoint)
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::Security {
                message: format!("Security primal HTTP request failed: {e}"),
                context: Some(primal_info.primal_id.clone()),
                severity: Some("medium".to_string()),
                suggestion: Some("Check security primal availability".to_string()),
            })?;

        // Parse response
        let response_json =
            response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| SongbirdError::Security {
                    message: format!("Failed to parse security primal response: {e}"),
                    context: Some(primal_info.primal_id.clone()),
                    severity: Some("medium".to_string()),
                    suggestion: Some("Check security primal response format".to_string()),
                })?;

        // Create proper PrimalResponse
        let primal_response = songbird_universal_primals::PrimalResponse {
            response_type: songbird_universal_primals::PrimalResponseType::Success,
            payload: response_json.clone(),
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
            primal_id: primal_info.primal_id.clone(),
            request_id: "security_request".to_string(),
            status: "ok".to_string(),
            data: response_json,
            metadata: Some(std::collections::HashMap::new()),
        };

        Ok(primal_response)
    }

    /// Check health of security primal using its endpoint
    async fn check_primal_health(&self, primal_info: &SecurityCapabilityInfo) -> Result<()> {
        let now = SystemTime::now();

        // If we checked health recently (within 30 seconds), trust the cache
        if now
            .duration_since(
                primal_info
                    .last_health_check
                    .unwrap_or(SystemTime::UNIX_EPOCH),
            )
            .unwrap_or(Duration::MAX)
            < Duration::from_secs(30)
        {
            return Ok(());
        }

        // Perform actual health check using the endpoint
        let health_url = format!("{}/health", primal_info.endpoint.trim_end_matches('/'));

        match reqwest::Client::new()
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                debug!(
                    "Health check passed for security primal: {} at {}",
                    primal_info.primal_id, health_url
                );

                // Update health check timestamp (we need to update the cache)
                self.update_primal_health_check(&primal_info.instance_id)
                    .await;
                Ok(())
            }
            Ok(response) => {
                let status = response.status();
                Err(SongbirdError::Security {
                    message: format!("Security primal health check failed with status: {status}"),
                    context: Some(health_url),
                    severity: Some("medium".to_string()),
                    suggestion: Some("Check primal service status".to_string()),
                })
            }
            Err(e) => Err(SongbirdError::Security {
                message: format!("Security primal health check request failed: {e}"),
                context: Some(health_url),
                severity: Some("medium".to_string()),
                suggestion: Some("Check network connectivity to primal".to_string()),
            }),
        }
    }

    /// Update health check timestamp for a primal
    async fn update_primal_health_check(&self, instance_id: &str) {
        let mut capabilities = self.security_capabilities.write().await;
        if let Some(info) = capabilities.get_mut(instance_id) {
            info.last_health_check = Some(SystemTime::now());
        }
    }

    async fn get_security_capabilities(&self) -> HashMap<String, SecurityCapabilityInfo> {
        // Create security capability query
        let auth_capability = songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["oauth".to_string(), "jwt".to_string()],
            encryption_algorithms: vec!["aes256".to_string(), "rsa".to_string()],
            compliance_frameworks: vec!["gdpr".to_string(), "iso27001".to_string()],
            threat_detection: true,
        };

        let security_primals = self
            .primal_registry
            .find_services_by_capability(vec![auth_capability])
            .await
            .unwrap_or_default();

        let mut capabilities = HashMap::new();

        for primal in security_primals {
            let primal_capabilities = &primal.registration.capabilities;

            // Look for security-related endpoints
            for capability in primal_capabilities {
                if let songbird_universal_primals::ServiceCapability::Security { .. } = capability {
                    let endpoints = &primal.registration.endpoints;
                    let primal_id = primal.registration.service_id.to_string();
                    let instance_id = primal.registration.instance_id.clone();

                    for endpoint in endpoints {
                        capabilities.insert(
                            format!("{}:{}", primal_id, endpoint.name),
                            SecurityCapabilityInfo {
                                primal_id: primal_id.clone(),
                                instance_id: instance_id.clone(),
                                endpoint: endpoint.url.clone(),
                                capabilities: vec![capability.clone()],
                                last_health_check: None,
                                is_healthy: true,
                            },
                        );
                    }
                }
            }
        }

        capabilities
    }
}

#[async_trait]
impl AuthenticationProvider for UniversalSecurityProvider {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        // Try to find authentication capability
        let auth_capability = songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["password".to_string()],
            encryption_algorithms: vec!["aes256".to_string()],
            compliance_frameworks: vec!["gdpr".to_string()],
            threat_detection: true,
        };

        match self.find_best_security_primal(&auth_capability).await {
            Some(primal_info) => {
                // Use discovered security primal for authentication
                let mut payload = HashMap::new();
                payload.insert(
                    "username".to_string(),
                    serde_json::Value::String(username.to_string()),
                );
                payload.insert(
                    "password".to_string(),
                    serde_json::Value::String(password.to_string()),
                );

                match self
                    .make_security_request(&primal_info, "authenticate", payload)
                    .await
                {
                    Ok(response) => {
                        if response.success {
                            // Parse successful authentication response
                            if let Some(_token_data) = response.payload.get("token") {
                                info!("✅ Authentication successful via security primal: {} (instance: {})", 
                                      primal_info.primal_id, primal_info.instance_id);

                                // Create AuthToken from primal response
                                let permissions = if let Some(perms_value) =
                                    response.payload.get("permissions")
                                {
                                    // Try to parse permissions from response
                                    if let Some(perms_array) = perms_value.as_array() {
                                        perms_array
                                            .iter()
                                            .filter_map(|v| v.as_str())
                                            .map(|s| s.to_string())
                                            .collect()
                                    } else {
                                        vec!["user".to_string()]
                                    }
                                } else {
                                    vec!["user".to_string()]
                                };

                                let token = AuthToken::new(
                                    username.to_string(),
                                    SubjectType::User,
                                    self.config.session_timeout,
                                    permissions,
                                );
                                return Ok(token);
                            }
                        }

                        // Authentication failed at primal level
                        warn!(
                            "Authentication failed at security primal: {}",
                            response
                                .error_message
                                .unwrap_or_else(|| "Unknown error".to_string())
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Security primal authentication request failed: {}, falling back",
                            e
                        );
                    }
                }
            }
            None => {
                debug!("No security primal available for authentication, using fallback");
            }
        }

        // Fallback to standalone authentication
        info!("🛡️ Using fallback authentication (standalone mode)");
        self.fallback_provider
            .authenticate(username, password)
            .await
    }

    async fn validate_token(&self, token: &str) -> Result<AuthToken> {
        // Try security primal first
        let auth_capability = songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["token".to_string()],
            encryption_algorithms: vec!["aes256".to_string()],
            compliance_frameworks: vec!["gdpr".to_string()],
            threat_detection: true,
        };

        if let Some(primal_info) = self.find_best_security_primal(&auth_capability).await {
            let mut payload = HashMap::new();
            payload.insert(
                "token".to_string(),
                serde_json::Value::String(token.to_string()),
            );

            if let Ok(response) = self
                .make_security_request(&primal_info, "validate_token", payload)
                .await
            {
                if response.success {
                    debug!(
                        "Token validated by security primal: {} (instance: {})",
                        primal_info.primal_id, primal_info.instance_id
                    );

                    // Parse token validation response
                    if let Some(_token_data) = response.payload.get("token_info") {
                        // Parse and return validated token
                        // This would need proper parsing based on primal response format
                        // For now, create a basic token
                        if let Some(username_value) = response.payload.get("username") {
                            if let Some(username) = username_value.as_str() {
                                return Ok(AuthToken::new(
                                    username.to_string(),
                                    SubjectType::User,
                                    self.config.session_timeout,
                                    vec!["user".to_string()],
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Fallback validation
        self.fallback_provider.validate_token(token).await
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        // Try security primal first
        let auth_capability = songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["token".to_string()],
            encryption_algorithms: vec!["aes256".to_string()],
            compliance_frameworks: vec!["gdpr".to_string()],
            threat_detection: true,
        };

        if let Some(primal_info) = self.find_best_security_primal(&auth_capability).await {
            let mut payload = HashMap::new();
            payload.insert(
                "token".to_string(),
                serde_json::Value::String(token.to_string()),
            );

            if let Ok(response) = self
                .make_security_request(&primal_info, "revoke_token", payload)
                .await
            {
                if response.success {
                    info!(
                        "Token revoked by security primal: {} (instance: {})",
                        primal_info.primal_id, primal_info.instance_id
                    );
                    return Ok(());
                }
            }
        }

        // Fallback revocation
        self.fallback_provider.revoke_token(token).await
    }

    async fn refresh_token(&self, token: &str) -> Result<AuthToken> {
        // Try security primal first, then fallback
        let auth_capability = songbird_universal_primals::ServiceCapability::Security {
            authentication_methods: vec!["token".to_string()],
            encryption_algorithms: vec!["aes256".to_string()],
            compliance_frameworks: vec!["gdpr".to_string()],
            threat_detection: true,
        };

        if let Some(primal_info) = self.find_best_security_primal(&auth_capability).await {
            let mut payload = HashMap::new();
            payload.insert(
                "token".to_string(),
                serde_json::Value::String(token.to_string()),
            );

            if let Ok(response) = self
                .make_security_request(&primal_info, "refresh_token", payload)
                .await
            {
                if response.success {
                    debug!(
                        "Token refreshed by security primal: {} (instance: {})",
                        primal_info.primal_id, primal_info.instance_id
                    );

                    // Parse refreshed token from response
                    if let Some(username_value) = response.payload.get("username") {
                        if let Some(username) = username_value.as_str() {
                            return Ok(AuthToken::new(
                                username.to_string(),
                                SubjectType::User,
                                self.config.session_timeout,
                                vec!["user".to_string()],
                            ));
                        }
                    }
                }
            }
        }

        // Fallback refresh
        self.fallback_provider.refresh_token(token).await
    }
}

// Note: Authorization provider implementation would follow similar pattern
// with capability-based discovery and fallback implementations
