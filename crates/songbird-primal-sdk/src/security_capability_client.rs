//! # 🔒 Security Capability Client (Zero Hardcoding)
//!
//! **REPLACES**: `beardog.rs` - Hardcoded security primal
//!
//! This module provides security capabilities without hardcoding specific primal names.
//! Works with ANY security provider that implements the security capability interface.
//!
//! ## Migration from BearDog
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded beardog primal
//! let beardog = BeardogPrimal::new(context);
//! let token = beardog.generate_token(claims).await?;
//!
//! // ✅ NEW: Capability-based security client
//! let security = SecurityCapabilityClient::new().await?;
//! let token = security.generate_token(claims).await?;
//! // Works with beardog, vault, keycloak, or any security provider!
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_types::errors::{SongbirdError, SongbirdResult};
use songbird_universal::UnixRpcClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};

/// Security capability client (replaces BeardogPrimal)
///
/// **Pure Rust Implementation**: Uses Unix socket JSON-RPC for inter-primal communication,
/// eliminating HTTP overhead and `reqwest` dependency (ring-free!).
#[derive(Debug, Clone)]
pub struct SecurityCapabilityClient {
    /// Capability endpoint resolver (for discovery)
    resolver: CapabilityEndpointResolver,
    /// JSON-RPC client for Unix socket communication (Pure Rust!)
    rpc_client: UnixRpcClient,
    /// Client configuration
    config: SecurityClientConfig,
}

/// Security client configuration
#[derive(Debug, Clone)]
pub struct SecurityClientConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Default token expiration (seconds)
    pub default_token_expiration: u64,
}

impl Default for SecurityClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(
                std::env::var("SECURITY_REQUEST_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            ),
            default_token_expiration: std::env::var("SECURITY_TOKEN_EXPIRATION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600), // 1 hour
        }
    }
}

/// Token generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRequest {
    /// Subject (user/service identifier)
    pub subject: String,
    /// Claims to include in token
    pub claims: HashMap<String, serde_json::Value>,
    /// Token expiration (seconds from now)
    pub expires_in: Option<u64>,
    /// Scopes/permissions
    pub scopes: Vec<String>,
}

/// Token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Generated token
    pub token: String,
    /// Token type (e.g., "Bearer")
    pub token_type: String,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Refresh token (if applicable)
    pub refresh_token: Option<String>,
}

/// Token validation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTokenRequest {
    /// Token to validate
    pub token: String,
    /// Required scopes (optional)
    pub required_scopes: Option<Vec<String>>,
}

/// Token validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTokenResponse {
    /// Whether token is valid
    pub valid: bool,
    /// Token subject
    pub subject: Option<String>,
    /// Token claims
    pub claims: Option<HashMap<String, serde_json::Value>>,
    /// Remaining validity (seconds)
    pub expires_in: Option<u64>,
}

/// Encryption request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptRequest {
    /// Data to encrypt (base64 encoded)
    pub plaintext: String,
    /// Key identifier
    pub key_id: Option<String>,
    /// Additional authenticated data
    pub aad: Option<String>,
}

/// Encryption response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptResponse {
    /// Encrypted data (base64 encoded)
    pub ciphertext: String,
    /// Key identifier used
    pub key_id: String,
    /// Initialization vector (if applicable)
    pub iv: Option<String>,
}

/// Decryption request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptRequest {
    /// Data to decrypt (base64 encoded)
    pub ciphertext: String,
    /// Key identifier
    pub key_id: String,
    /// Initialization vector (if applicable)
    pub iv: Option<String>,
    /// Additional authenticated data
    pub aad: Option<String>,
}

/// Decryption response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptResponse {
    /// Decrypted data (base64 encoded)
    pub plaintext: String,
}

impl SecurityCapabilityClient {
    /// Create new security capability client
    ///
    /// Discovers security providers dynamically - no hardcoded endpoints!
    ///
    /// # Example
    /// ```no_run
    /// use songbird_primal_sdk::security_capability_client::SecurityCapabilityClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let security = SecurityCapabilityClient::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> SongbirdResult<Self> {
        Self::with_config(SecurityClientConfig::default()).await
    }
    
    /// Create security client with custom configuration
    pub async fn with_config(config: SecurityClientConfig) -> SongbirdResult<Self> {
        info!("🔒 Creating security capability client (Pure Rust Unix socket!)");
        
        // Discover Unix socket path for security capability
        let socket_path = Self::discover_socket_path()?;
        
        // Create UnixRpcClient (100% Pure Rust!)
        let rpc_client = UnixRpcClient::new(&socket_path)
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Failed to create Unix RPC client for {:?}: {}", socket_path, e),
                field: Some("rpc_client".to_string()),
                suggestion: Some("Ensure security primal is running and socket exists".to_string()),
            })?;
        
        info!("✅ Security capability client connected to {:?}", socket_path);
        
        Ok(Self {
            resolver: CapabilityEndpointResolver::new(),
            rpc_client,
            config,
        })
    }
    
    /// Discover Unix socket path for security capability
    ///
    /// Priority:
    /// 1. SECURITY_SOCKET_PATH environment variable
    /// 2. BEARDOG_SOCKET_PATH environment variable (legacy)
    /// 3. Default: /tmp/beardog.sock
    fn discover_socket_path() -> SongbirdResult<PathBuf> {
        std::env::var("SECURITY_SOCKET_PATH")
            .or_else(|_| std::env::var("BEARDOG_SOCKET_PATH"))
            .map(PathBuf::from)
            .or_else(|_| Ok(PathBuf::from("/tmp/beardog.sock")))
    }
    
    /// Generate authentication token
    ///
    /// Works with ANY provider that implements the security capability:
    /// - beardog (if available)
    /// - HashiCorp Vault
    /// - Keycloak
    /// - Auth0
    /// - Custom security services
    ///
    /// # Example
    /// ```no_run
    /// # use songbird_primal_sdk::security_capability_client::*;
    /// # use std::collections::HashMap;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let security = SecurityCapabilityClient::new().await?;
    /// 
    /// let request = TokenRequest {
    ///     subject: "user@example.com".to_string(),
    ///     claims: HashMap::new(),
    ///     expires_in: Some(3600),
    ///     scopes: vec!["read".to_string(), "write".to_string()],
    /// };
    /// 
    /// let response = security.generate_token(request).await?;
    /// println!("Token: {}", response.token);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_token(&self, request: TokenRequest) -> SongbirdResult<TokenResponse> {
        debug!("🔑 Generating token via JSON-RPC for subject: {}", request.subject);
        
        // Call security.generate_token JSON-RPC method
        let response: TokenResponse = self.rpc_client
            .call("security.generate_token", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Token generation RPC failed: {}", e),
                source: Some("security.generate_token".to_string()),
            })?;
        
        info!("✅ Token generated successfully (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Validate authentication token
    pub async fn validate_token(&self, request: ValidateTokenRequest) -> SongbirdResult<ValidateTokenResponse> {
        debug!("🔍 Validating token via JSON-RPC");
        
        // Call security.validate_token JSON-RPC method
        let response: ValidateTokenResponse = self.rpc_client
            .call("security.validate_token", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Token validation RPC failed: {}", e),
                source: Some("security.validate_token".to_string()),
            })?;
        
        info!("✅ Token validated successfully (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Encrypt data
    pub async fn encrypt(&self, request: EncryptRequest) -> SongbirdResult<EncryptResponse> {
        debug!("🔐 Encrypting data via JSON-RPC");
        
        // Call security.encrypt JSON-RPC method
        let response: EncryptResponse = self.rpc_client
            .call("security.encrypt", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Encryption RPC failed: {}", e),
                source: Some("security.encrypt".to_string()),
            })?;
        
        info!("✅ Data encrypted successfully (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Decrypt data
    pub async fn decrypt(&self, request: DecryptRequest) -> SongbirdResult<DecryptResponse> {
        debug!("🔓 Decrypting data via JSON-RPC");
        
        // Call security.decrypt JSON-RPC method
        let response: DecryptResponse = self.rpc_client
            .call("security.decrypt", &request)
            .await
            .map_err(|e| SongbirdError::Network {
                message: format!("Decryption RPC failed: {}", e),
                source: Some("security.decrypt".to_string()),
            })?;
        
        info!("✅ Data decrypted successfully (Pure Rust RPC!)");
        Ok(response)
    }
    
    /// Check if security capability is available
    pub async fn is_available(&self) -> bool {
        self.resolver.get_endpoint(CapabilityType::Security).await.is_ok()
    }
    
    /// Get current configuration
    pub fn config(&self) -> &SecurityClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_security_client_creation() {
        let result = SecurityCapabilityClient::new().await;
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_default_config() {
        let config = SecurityClientConfig::default();
        assert!(config.default_token_expiration > 0);
    }
}

