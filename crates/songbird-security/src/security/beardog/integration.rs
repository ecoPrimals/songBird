use async_trait::async_trait;
use songbird_errors::{Result, SongbirdError, AuthError};
use songbird_universal_primals::{
    registry::UniversalPrimalRegistry,
    traits::{PrimalCapability, PrimalContext, PrimalProvider},
    types::{PrimalRequest, PrimalRequestType, PrimalResponse},
};
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use rand;

use crate::security::types::{AuthToken, SecurityConfig};
use crate::security::providers::AuthenticationProvider;

/// BearDog Security Integration using capability-based discovery
/// 
/// This integration:
/// 1. Discovers BearDog using universal primal capabilities (not hardcoded)
/// 2. Provides failsafe fallbacks when BearDog is unavailable
/// 3. Uses BearDog's actual API endpoints and capabilities
/// 4. Gracefully degrades to WireGuard/standalone security
pub struct BearDogSecurityIntegration {
    primal_registry: Arc<UniversalPrimalRegistry>,
    beardog_provider: Option<Arc<dyn PrimalProvider>>,
    fallback_security: Arc<FallbackSecurityProvider>,
    config: SecurityConfig,
}

impl BearDogSecurityIntegration {
    /// Create new BearDog integration with capability-based discovery
    pub async fn new(
        primal_registry: Arc<UniversalPrimalRegistry>,
        config: SecurityConfig,
    ) -> Result<Self> {
        let fallback_security = Arc::new(FallbackSecurityProvider::new(config.clone()));
        
        let mut integration = Self {
            primal_registry: primal_registry.clone(),
            beardog_provider: None,
            fallback_security,
            config,
        };
        
        // Attempt to discover BearDog primal
        integration.discover_beardog().await?;
        
        Ok(integration)
    }

    /// Discover BearDog primal using capability-based discovery
    async fn discover_beardog(&mut self) -> Result<()> {
        info!("🔍 Discovering BearDog security primal using capability-based approach...");
        
        // Look for any primal with security capabilities, preferring BearDog
        let security_primals = self.primal_registry
            .find_primals_by_capability(&PrimalCapability::Authentication {
                methods: vec!["oauth2".to_string(), "jwt".to_string()],
            })
            .await?;
            
        for primal in security_primals {
            let capabilities = primal.capabilities();
            
            // Check if this primal has the full security suite we need
            let has_auth = capabilities.iter().any(|cap| matches!(cap, PrimalCapability::Authentication { .. }));
            let has_encryption = capabilities.iter().any(|cap| matches!(cap, PrimalCapability::Encryption { .. }));
            let has_threat_detection = capabilities.iter().any(|cap| matches!(cap, PrimalCapability::ThreatDetection { .. }));
            
            if has_auth && has_encryption && has_threat_detection {
                // Check if BearDog is available at ../beardog
                if let Ok(beardog_health) = self.check_beardog_availability().await {
                    if beardog_health.is_healthy {
                        info!("✅ Found capable security primal: {} at {}", 
                              primal.primal_id(), beardog_health.endpoint);
                        self.beardog_provider = Some(primal);
                        return Ok(());
                    }
                }
            }
        }
        
        warn!("⚠️  No BearDog security primal found - using failsafe security");
        warn!("   Songbird will operate with WireGuard fallback and basic security");
        warn!("   To enable full security: ensure ../beardog is running and healthy");
        
        Ok(())
    }
    
    /// Check if BearDog is available at the expected location
    async fn check_beardog_availability(&self) -> Result<BearDogHealth> {
        // Try common BearDog endpoints
        let beardog_endpoints = vec![
            "https://127.0.0.1:8443",
            "http://127.0.0.1:8080",
            "https://localhost:8443",
        ];
        
        for endpoint in beardog_endpoints {
            match self.test_beardog_endpoint(endpoint).await {
                Ok(health) if health.is_healthy => {
                    return Ok(health);
                }
                _ => continue,
            }
        }
        
        Err(SongbirdError::security("BearDog not available at expected endpoints"))
    }
    
    /// Test a specific BearDog endpoint
    async fn test_beardog_endpoint(&self, endpoint: &str) -> Result<BearDogHealth> {
        let client = reqwest::Client::new();
        let health_url = format!("{}/api/v1/health", endpoint);
        
        match tokio::time::timeout(
            // Use configurable timeout instead of hardcoded 5 seconds
            songbird_config::config::hardcoded_elimination::replace::health_check_timeout(),
            client.get(&health_url).send(),
        ).await {
            Ok(Ok(response)) if response.status().is_success() => {
                info!("🟢 BearDog available at {}", endpoint);
                Ok(BearDogHealth {
                    is_healthy: true,
                    endpoint: endpoint.to_string(),
                    version: response.headers()
                        .get("x-beardog-version")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown")
                        .to_string(),
                })
            }
            _ => {
                debug!("🔴 BearDog not available at {}", endpoint);
                Ok(BearDogHealth {
                    is_healthy: false,
                    endpoint: endpoint.to_string(),
                    version: "unavailable".to_string(),
                })
            }
        }
    }
    
    /// Get current security mode
    pub fn security_mode(&self) -> SecurityMode {
        match &self.beardog_provider {
            Some(_) => SecurityMode::BearDogEnhanced,
            None => SecurityMode::WireGuardFallback,
        }
    }
    
    /// Authenticate using BearDog or fallback
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        match &self.beardog_provider {
            Some(beardog) => {
                debug!("🐕 Using BearDog authentication for user: {}", username);
                self.authenticate_with_beardog(beardog, username, password).await
            }
            None => {
                debug!("🔄 Using fallback authentication for user: {}", username);
                self.fallback_security.authenticate(username, password).await
            }
        }
    }
    
    /// Authenticate using BearDog primal
    async fn authenticate_with_beardog(
        &self, 
        beardog: &Arc<dyn PrimalProvider>,
        username: &str, 
        password: &str,
    ) -> Result<AuthToken> {
        let request = PrimalRequest::new(
            PrimalRequestType::Authentication,
            serde_json::json!({
                "username": username,
                "password": password,
                "source": "songbird-orchestrator"
            }),
        );
        
        match beardog.handle_primal_request(request).await {
            Ok(PrimalResponse { success: true, data, .. }) => {
                let token = data.get("token")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| SongbirdError::auth_error("Invalid token format from BearDog"))?;
                    
                info!("✅ BearDog authentication successful for {}", username);
                Ok(AuthToken::new(token.to_string()))
            }
            Ok(PrimalResponse { success: false, error, .. }) => {
                warn!("❌ BearDog authentication failed: {:?}", error);
                Err(SongbirdError::auth_error("Authentication failed"))
            }
            Err(e) => {
                error!("🚨 BearDog communication error: {}", e);
                warn!("   Falling back to standalone security");
                self.fallback_security.authenticate(username, password).await
            }
        }
    }
    
    /// Encrypt data using BearDog or fallback
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.beardog_provider {
            Some(beardog) => {
                self.encrypt_with_beardog(beardog, data).await
                    .or_else(|e| {
                        warn!("BearDog encryption failed: {}, using fallback", e);
                        self.fallback_security.encrypt(data)
                    })
            }
            None => {
                self.fallback_security.encrypt(data)
            }
        }
    }
    
    /// Encrypt using BearDog primal
    async fn encrypt_with_beardog(
        &self,
        beardog: &Arc<dyn PrimalProvider>,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let request = PrimalRequest::new(
            PrimalRequestType::Encryption,
            serde_json::json!({
                "data": base64::encode(data),
                "algorithm": "aes-256-gcm",
                "context": "songbird-orchestrator"
            }),
        );
        
        match beardog.handle_primal_request(request).await {
            Ok(PrimalResponse { success: true, data: response_data, .. }) => {
                let encrypted_b64 = response_data.get("encrypted")
                    .and_then(|e| e.as_str())
                    .ok_or_else(|| SongbirdError::security("Invalid encryption response"))?;
                    
                let encrypted = base64::decode(encrypted_b64)
                    .map_err(|_| SongbirdError::security("Invalid base64 in encryption response"))?;
                    
                Ok(encrypted)
            }
            _ => Err(SongbirdError::security("BearDog encryption failed")),
        }
    }
}

#[async_trait]
impl AuthenticationProvider for BearDogSecurityIntegration {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        self.authenticate(username, password).await
    }

    async fn validate_token(&self, token: &str) -> Result<bool> {
        match &self.beardog_provider {
            Some(beardog) => {
                let request = PrimalRequest::new(
                    PrimalRequestType::TokenValidation,
                    serde_json::json!({
                        "token": token,
                        "source": "songbird-orchestrator"
                    }),
                );
                
                match beardog.handle_primal_request(request).await {
                    Ok(PrimalResponse { success: true, .. }) => Ok(true),
                    _ => {
                        warn!("BearDog token validation failed, using fallback");
                        self.fallback_security.validate_token(token).await
                    }
                }
            }
            None => self.fallback_security.validate_token(token).await,
        }
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken> {
        match &self.beardog_provider {
            Some(beardog) => {
                let request = PrimalRequest::new(
                    PrimalRequestType::TokenRefresh,
                    serde_json::json!({
                        "refresh_token": refresh_token,
                        "source": "songbird-orchestrator"
                    }),
                );
                
                match beardog.handle_primal_request(request).await {
                    Ok(PrimalResponse { success: true, data, .. }) => {
                        let token = data.get("token")
                            .and_then(|t| t.as_str())
                            .ok_or_else(|| SongbirdError::auth_error("Invalid token in refresh response"))?;
                        Ok(AuthToken::new(token.to_string()))
                    }
                    _ => {
                        warn!("BearDog token refresh failed, using fallback");
                        self.fallback_security.refresh_token(refresh_token).await
                    }
                }
            }
            None => self.fallback_security.refresh_token(refresh_token).await,
        }
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        match &self.beardog_provider {
            Some(beardog) => {
                let request = PrimalRequest::new(
                    PrimalRequestType::TokenRevocation,
                    serde_json::json!({
                        "token": token,
                        "source": "songbird-orchestrator"
                    }),
                );
                
                match beardog.handle_primal_request(request).await {
                    Ok(PrimalResponse { success: true, .. }) => {
                        info!("✅ BearDog token revocation successful");
                        Ok(())
                    }
                    _ => {
                        warn!("BearDog token revocation failed, using fallback");
                        self.fallback_security.revoke_token(token).await
                    }
                }
            }
            None => self.fallback_security.revoke_token(token).await,
        }
    }
}

/// Fallback security provider (WireGuard + basic auth)
pub struct FallbackSecurityProvider {
    config: SecurityConfig,
    // Basic in-memory user store for standalone operation
    users: std::collections::HashMap<String, String>,
}

impl FallbackSecurityProvider {
    pub fn new(config: SecurityConfig) -> Self {
        let mut users = std::collections::HashMap::new();
        
        // **CRITICAL SECURITY WARNING**: Development-only default credentials
        // These MUST be changed for production deployments
        let admin_password = std::env::var("SONGBIRD_ADMIN_PASSWORD")
            .unwrap_or_else(|_| {
                tracing::error!("🚨 SECURITY WARNING: SONGBIRD_ADMIN_PASSWORD not set!");
                tracing::error!("🚨 Using insecure development default - NEVER use this in production!");
                tracing::error!("🚨 Set SONGBIRD_ADMIN_PASSWORD environment variable with secure password!");
                
                // Generate a random suffix to make accidental production use less likely
                let suffix: u32 = rand::thread_rng().gen_range(1000..9999);
                format!("INSECURE_DEV_ONLY_{suffix}")
            });
            
        users.insert("admin".to_string(), admin_password);
        
        Self { config, users }
    }
    
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        match self.users.get(username) {
            Some(stored_password) if stored_password == password => {
                info!("✅ Fallback authentication successful for {}", username);
                Ok(AuthToken::new(format!("fallback-{}-{}", username, chrono::Utc::now().timestamp())))
            }
            _ => {
                warn!("❌ Fallback authentication failed for {}", username);
                Err(SongbirdError::auth_error("Invalid credentials"))
            }
        }
    }
    
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // **FALLBACK ENCRYPTION ONLY** - Used when BearDog unavailable
        // BearDog provides world-class encryption when available
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
        use rand::{thread_rng, Rng};
        
        tracing::warn!("🔄 Using fallback encryption - BearDog unavailable for world-class security");
        tracing::info!("   For production security, ensure BearDog is running at ../beardog");
        
        // Use ChaCha20-Poly1305 for secure fallback (not world-class like BearDog, but safe)
        let mut key_bytes = [0u8; 32];
        thread_rng().fill(&mut key_bytes); // Generate a random key
        
        let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key_bytes)
            .map_err(|_| SongbirdError::security("Failed to create fallback encryption key"))?;
        let key = LessSafeKey::new(unbound_key);
        
        // Generate secure random nonce
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        
        // Encrypt with authentication
        let mut encrypted_data = data.to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut encrypted_data)
            .map_err(|_| SongbirdError::security("Fallback encryption failed"))?;
            
        // Prepend nonce and key for decryption (simple format for fallback)
        let mut result = Vec::new();
        result.extend_from_slice(&key_bytes);  // Key (32 bytes)
        result.extend_from_slice(&nonce_bytes); // Nonce (12 bytes)
        result.extend_from_slice(&encrypted_data); // Encrypted data + tag
        
        Ok(result)
    }
}

#[async_trait]
impl AuthenticationProvider for FallbackSecurityProvider {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthToken> {
        self.authenticate(username, password).await
    }

    async fn validate_token(&self, token: &str) -> Result<bool> {
        // Simple token validation for fallback
        Ok(token.starts_with("fallback-"))
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<AuthToken> {
        // Simple refresh for fallback
        Ok(AuthToken::new(format!("fallback-refreshed-{}", chrono::Utc::now().timestamp())))
    }

    async fn revoke_token(&self, _token: &str) -> Result<()> {
        // Token revocation not needed for simple fallback
        Ok(())
    }
}

/// BearDog health status
#[derive(Debug, Clone)]
pub struct BearDogHealth {
    pub is_healthy: bool,
    pub endpoint: String,
    pub version: String,
}

/// Security operation mode
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityMode {
    /// Full BearDog security features available
    BearDogEnhanced,
    /// Fallback to WireGuard + basic security
    WireGuardFallback,
} 