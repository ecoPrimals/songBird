//! # Universal Security Adapter - Canonical Implementation
//!
//! This adapter provides security capabilities through universal routing to any
//! security provider (BearDog, custom implementations, etc.) based on capabilities
//! rather than hardcoded provider names.

use serde::{Deserialize, Serialize};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Universal security capability types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCapability {
    Encryption,
    Decryption, 
    KeyManagement,
    Authentication,
    Authorization,
    AuditLogging,
    ThreatDetection,
}

/// Security provider registration
#[derive(Debug, Clone)]
pub struct SecurityProvider {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<SecurityCapability>,
    pub endpoint: String,
    pub priority: u8,
    pub health_status: ProviderHealth,
}

#[derive(Debug, Clone)]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Universal security adapter - routes to any security provider
pub struct UniversalSecurityAdapter {
    providers: Arc<RwLock<HashMap<String, SecurityProvider>>>,
    client: reqwest::Client,
}

impl UniversalSecurityAdapter {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    /// Register a security provider (BearDog, custom, etc.)
    pub async fn register_provider(&self, provider: SecurityProvider) -> SongbirdResult<()> {
        let mut providers = self.providers.write().await;
        info!("Registering security provider: {} ({})", provider.name, provider.id);
        providers.insert(provider.id.clone(), provider);
        Ok(())
    }

    /// Find providers with specific capability
    pub async fn find_providers_with_capability(&self, capability: SecurityCapability) -> Vec<SecurityProvider> {
        let providers = self.providers.read().await;
        providers
            .values()
            .filter(|p| p.capabilities.contains(&capability) && matches!(p.health_status, ProviderHealth::Healthy))
            .cloned()
            .collect()
    }

    /// Encrypt data using any available encryption provider
    pub async fn encrypt_data(&self, data: &[u8], context: EncryptionContext) -> SongbirdResult<EncryptedData> {
        debug!("🔒 Requesting encryption for context: {:?}", context);

        let providers = self.find_providers_with_capability(SecurityCapability::Encryption).await;
        if providers.is_empty() {
            return Err(SongbirdError::internal_error(service_error(
                "security", 
                "No encryption providers available"
            ));
        }

        // Use highest priority provider
        let provider = providers.into_iter()
            .max_by_key(|p| p.priority)
            .unwrap();

        self.encrypt_with_provider(&provider, data, context).await
    }

    /// Decrypt data using any available decryption provider
    pub async fn decrypt_data(&self, encrypted_data: &EncryptedData) -> SongbirdResult<Vec<u8>> {
        debug!("🔓 Requesting decryption for data from provider: {}", encrypted_data.provider_id);

        let providers = self.providers.read().await;
        let provider = providers.get(&encrypted_data.provider_id)
            .ok_or_else(|| SongbirdError::service_error(
                "security",
                format!("Provider {} not found for decryption", encrypted_data.provider_id)
            ))?;

        self.decrypt_with_provider(provider, encrypted_data).await
    }

    /// Authenticate using any available authentication provider
    pub async fn authenticate(&self, credentials: AuthCredentials) -> SongbirdResult<AuthToken> {
        debug!("🔐 Requesting authentication");

        let providers = self.find_providers_with_capability(SecurityCapability::Authentication).await;
        if providers.is_empty() {
            return Err(SongbirdError::internal_error(service_error(
                "security",
                "No authentication providers available"
            ));
        }

        // Try providers in priority order
        for provider in providers.iter().rev() {
            match self.authenticate_with_provider(provider, &credentials).await {
                Ok(token) => return Ok(token),
                Err(e) => {
                    warn!("Authentication failed with provider {}: {}", provider.name, e);
                    continue;
                }
            }
        }

        Err(SongbirdError::internal_error(authentication_error("All authentication providers failed"))
    }

    /// Check authorization using any available authorization provider  
    pub async fn authorize(&self, token: &AuthToken, resource: &str, action: &str) -> SongbirdResult<bool> {
        debug!("🔒 Checking authorization for resource: {}, action: {}", resource, action);

        let providers = self.find_providers_with_capability(SecurityCapability::Authorization).await;
        if providers.is_empty() {
            warn!("No authorization providers available, denying access");
            return Ok(false);
        }

        // Use highest priority provider for authorization
        let provider = providers.into_iter()
            .max_by_key(|p| p.priority)
            .unwrap();

        self.authorize_with_provider(&provider, token, resource, action).await
    }

    /// Health check all providers
    pub async fn health_check(&self) -> SongbirdResult<SecurityHealthReport> {
        let mut providers = self.providers.write().await;
        let mut report = SecurityHealthReport {
            total_providers: providers.len(),
            healthy_providers: 0,
            provider_status: HashMap::new(),
        };

        for (id, provider) in providers.iter_mut() {
            let health = self.check_provider_health(provider).await;
            provider.health_status = health.clone();
            
            if matches!(health, ProviderHealth::Healthy) {
                report.healthy_providers += 1;
            }
            
            report.provider_status.insert(id.clone(), health);
        }

        Ok(report)
    }

    // Private implementation methods
    async fn encrypt_with_provider(
        &self, 
        provider: &SecurityProvider, 
        data: &[u8], 
        context: EncryptionContext
    ) -> SongbirdResult<EncryptedData> {
        let request = EncryptionRequest {
            data: data.to_vec(),
            algorithm: context.algorithm,
            key_id: context.key_id,
        };

        let response = self.client
            .post(&format!("{}/encrypt", provider.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to call provider {}: {}", provider.name, e)))?;

        if !response.status().is_success() {
            return Err(SongbirdError::internal_error(service_error(
                &provider.name,
                format!("Encryption failed with status: {}", response.status())
            ));
        }

        let encrypted: EncryptedData = response.json().await
            .map_err(|e| SongbirdError::service_error(&provider.name, format!("Invalid response: {}", e)))?;

        Ok(encrypted)
    }

    async fn decrypt_with_provider(
        &self,
        provider: &SecurityProvider,
        encrypted_data: &EncryptedData
    ) -> SongbirdResult<Vec<u8>> {
        let response = self.client
            .post(&format!("{}/decrypt", provider.endpoint))
            .json(encrypted_data)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to call provider {}: {}", provider.name, e)))?;

        if !response.status().is_success() {
            return Err(SongbirdError::internal_error(service_error(
                &provider.name,
                format!("Decryption failed with status: {}", response.status())
            ));
        }

        let decrypted: DecryptionResponse = response.json().await
            .map_err(|e| SongbirdError::service_error(&provider.name, format!("Invalid response: {}", e)))?;

        Ok(decrypted.data)
    }

    async fn authenticate_with_provider(
        &self,
        provider: &SecurityProvider,
        credentials: &AuthCredentials
    ) -> SongbirdResult<AuthToken> {
        let response = self.client
            .post(&format!("{}/authenticate", provider.endpoint))
            .json(credentials)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to call provider {}: {}", provider.name, e)))?;

        if !response.status().is_success() {
            return Err(SongbirdError::internal_error(authentication_error(
                format!("Authentication failed with provider {}", provider.name)
            ));
        }

        let auth_response: AuthResponse = response.json().await
            .map_err(|e| SongbirdError::service_error(&provider.name, format!("Invalid response: {}", e)))?;

        Ok(auth_response.token)
    }

    async fn authorize_with_provider(
        &self,
        provider: &SecurityProvider,
        token: &AuthToken,
        resource: &str,
        action: &str
    ) -> SongbirdResult<bool> {
        let request = AuthorizationRequest {
            token: token.clone(),
            resource: resource.to_string(),
            action: action.to_string(),
        };

        let response = self.client
            .post(&format!("{}/authorize", provider.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to call provider {}: {}", provider.name, e)))?;

        if !response.status().is_success() {
            return Ok(false); // Deny on error
        }

        let auth_response: AuthorizationResponse = response.json().await
            .map_err(|_| Ok(false))?; // Deny on parse error

        Ok(auth_response.authorized)
    }

    async fn check_provider_health(&self, provider: &SecurityProvider) -> ProviderHealth {
        match self.client
            .get(&format!("{}/health", provider.endpoint))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => ProviderHealth::Healthy,
            Ok(_) => ProviderHealth::Degraded,
            Err(_) => ProviderHealth::Unhealthy,
        }
    }
}

impl Default for UniversalSecurityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// Supporting types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionContext {
    pub algorithm: String,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub algorithm: String,
    pub key_id: String,
    pub provider_id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequest {
    pub data: Vec<u8>,
    pub algorithm: String,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptionResponse {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: AuthToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequest {
    pub token: AuthToken,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResponse {
    pub authorized: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SecurityHealthReport {
    pub total_providers: usize,
    pub healthy_providers: usize,
    pub provider_status: HashMap<String, ProviderHealth>,
}

// Export the main adapter type
pub use UniversalSecurityAdapter as SecurityCapabilityAdapter;
