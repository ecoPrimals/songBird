//! OAuth2 Integration Module
//!
//! Support for OAuth2/OIDC authentication providers

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use thiserror::Error;
/// OAuth2 errors
#[derive(Debug, Error)]
pub enum OAuth2Error {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}
/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Authorization endpoint
    pub auth_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// User info endpoint (for OIDC)
    pub _userinfo_endpoint: Option<String>,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes to request
    pub scopes: Vec<String>,
}
impl Default for OAuth2Config {
    fn default() -> Self {
        // Get configuration from environment variables with secure defaults
        let host = std::env::var("SONGBIRD_OAUTH_BIND_ADDRESS")
            .unwrap_or_else(|_| {
                // Default to secure localhost binding instead of 0.0.0.0
                std::env::var("SONGBIRD_BIND_ADDRESS")
                    .unwrap_or_else(|_| crate::config::environment::EnvironmentConfig::default().bind_address.to_string())
            });
        let env_config = crate::config::environment::EnvironmentConfig::default();
        let port = std::env::var("SONGBIRD_PORT").unwrap_or_else(|_| env_config.bind_port.to_string());

        Self {
            client_id: std::env::var("SONGBIRD_OAUTH_CLIENT_ID")
                .unwrap_or_else(|_| "songbird-orchestrator".to_string()),
            client_secret: std::env::var("SONGBIRD_OAUTH_CLIENT_SECRET")
                .unwrap_or_else(|_| {
                    tracing::warn!("No SONGBIRD_OAUTH_CLIENT_SECRET provided - using default (not secure for production)");
                    "change-in-production".to_string()
                }),
            auth_endpoint: std::env::var("SONGBIRD_OAUTH_AUTH_ENDPOINT")
                .unwrap_or_else(|_| "https://oauth.example.com/auth".to_string()),
            token_endpoint: std::env::var("SONGBIRD_OAUTH_TOKEN_ENDPOINT")
                .unwrap_or_else(|_| "https://oauth.example.com/token".to_string()),
            _userinfo_endpoint: std::env::var("SONGBIRD_OAUTH_USERINFO_ENDPOINT").ok()
                .or_else(|| Some("https://oauth.example.com/userinfo".to_string())),
            redirect_uri: std::env::var("SONGBIRD_OAUTH_REDIRECT_URI")
                .unwrap_or_else(|_| format!("http://{}:{}/auth/callback", host, port)),
            scopes: std::env::var("SONGBIRD_OAUTH_SCOPES")
                .map(|s| s.split(',').map(|scope| scope.trim().to_string()).collect())
                .unwrap_or_else(|_| vec!["openid".to_string(), "profile".to_string(), "email".to_string()]),
        }
    }
}
/// OAuth2 provider trait
#[async_trait]
pub trait OAuth2Provider: Send + Sync {
    /// Get authorization URL
    fn get_auth_url(&self, state: &str) -> String;
    /// Exchange authorization code for access token
    async fn exchange_code(
        &self,
        code: &str,
        state: &str,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>>;
    /// Get user info using access token
    async fn get_user_info(
        &self,
        _access_token: &str,
    ) -> Result<super::UserInfo, Box<dyn std::error::Error>>;
    /// Refresh access token
    async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>>;
}
/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}
/// Generic OAuth2 provider implementation
pub struct GenericOAuth2Provider {
    config: OAuth2Config,
    client: crate::communication::HyperHttpClient,
}
impl GenericOAuth2Provider {
    pub fn new(config: OAuth2Config) -> Result<Self, OAuth2Error> {
        Ok(Self {
            config,
            client: crate::communication::HyperHttpClient::new().map_err(|e| {
                OAuth2Error::Network(format!("Failed to create HTTP client: {}", e))
            })?,
        })
    }
}
#[async_trait]
impl OAuth2Provider for GenericOAuth2Provider {
    fn get_auth_url(&self, state: &str) -> String {
        let scopes = self.config.scopes.join(" ");
        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            self.config.auth_endpoint,
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state)
        )
    }
    async fn exchange_code(
        &self,
        code: &str,
        _state: &str,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let params = serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": self.config.redirect_uri,
            "client_id": self.config.client_id,
            "client_secret": self.config.client_secret
        });
        let response = self
            .client
            .request(
                "POST",
                &self.config.token_endpoint,
                Some(&serde_json::to_string(&params)?),
            )
            .await?;

        if response.is_success() {
            let token_response: TokenResponse = response.json()?;
            Ok(token_response)
        } else {
            let error_text = response.text()?;
            Err(format!("Token exchange failed: {}", error_text).into())
        }
    }
    async fn get_user_info(
        &self,
        _access_token: &str,
    ) -> Result<super::UserInfo, Box<dyn std::error::Error>> {
        if let Some(_userinfo_endpoint) = &self.config._userinfo_endpoint {
            // Create a temporary client with the auth header
            // Simplified approach - just return basic user info for now
            return Ok(super::UserInfo {
                user_id: "oauth_user".to_string(),
                username: "oauth_user".to_string(),
                password_hash: "oauth_no_password".to_string(),
                permissions: vec!["user".to_string()],
                enabled: true,
                mfa_enabled: false,
                mfa_secret: None,
            });
        } else {
            // If no userinfo endpoint, create minimal user from token
            Ok(super::UserInfo {
                user_id: "oauth_user".to_string(),
                username: "oauth_user".to_string(),
                password_hash: "oauth_no_password".to_string(),
                permissions: vec!["user".to_string()],
                enabled: true,
                mfa_enabled: false,
                mfa_secret: None,
            })
        }
    }
    async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let params = serde_json::json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": self.config.client_id,
            "client_secret": self.config.client_secret
        });

        let response = self
            .client
            .request(
                "POST",
                &self.config.token_endpoint,
                Some(&serde_json::to_string(&params)?),
            )
            .await?;

        if response.is_success() {
            let token_response: TokenResponse = response.json()?;
            Ok(token_response)
        } else {
            let error_text = response.text()?;
            Err(format!("Token refresh failed: {}", error_text).into())
        }
    }
}
/// Create an OAuth2 provider based on configuration
pub fn create_oauth_provider(
    config: OAuth2Config,
) -> Result<Box<dyn OAuth2Provider>, Box<dyn std::error::Error>> {
    Ok(Box::new(GenericOAuth2Provider::new(config)?))
}
