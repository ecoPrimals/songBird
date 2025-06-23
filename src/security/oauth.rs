//! OAuth2 Integration Module
//!
//! Support for OAuth2/OIDC authentication providers

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub userinfo_endpoint: Option<String>,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes to request
    pub scopes: Vec<String>,
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            client_id: "songbird-orchestrator".to_string(),
            client_secret: "change-in-production".to_string(),
            auth_endpoint: "https://oauth.example.com/auth".to_string(),
            token_endpoint: "https://oauth.example.com/token".to_string(),
            userinfo_endpoint: Some("https://oauth.example.com/userinfo".to_string()),
            redirect_uri: "http://localhost:8080/auth/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        }
    }
}

/// OAuth2 provider trait
#[async_trait]
pub trait OAuth2Provider: Send + Sync {
    /// Get authorization URL
    fn get_auth_url(&self, state: &str) -> String;
    
    /// Exchange authorization code for access token
    async fn exchange_code(&self, code: &str, state: &str) -> Result<TokenResponse, Box<dyn std::error::Error>>;
    
    /// Get user info using access token
    async fn get_user_info(&self, access_token: &str) -> Result<super::UserInfo, Box<dyn std::error::Error>>;
    
    /// Refresh access token
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, Box<dyn std::error::Error>>;
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
    client: reqwest::Client,
}

impl GenericOAuth2Provider {
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
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
    
    async fn exchange_code(&self, code: &str, _state: &str) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];
        
        let response = self.client
            .post(&self.config.token_endpoint)
            .form(&params)
            .send()
            .await?;
            
        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;
            Ok(token_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Token exchange failed: {}", error_text).into())
        }
    }
    
    async fn get_user_info(&self, access_token: &str) -> Result<super::UserInfo, Box<dyn std::error::Error>> {
        if let Some(userinfo_endpoint) = &self.config.userinfo_endpoint {
            let response = self.client
                .get(userinfo_endpoint)
                .bearer_auth(access_token)
                .send()
                .await?;
                
            if response.status().is_success() {
                let user_data: serde_json::Value = response.json().await?;
                
                // Extract user info from standard OIDC claims
                let user = super::UserInfo {
                    id: user_data.get("sub")
                        .or_else(|| user_data.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    username: user_data.get("preferred_username")
                        .or_else(|| user_data.get("username"))
                        .or_else(|| user_data.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    email: user_data.get("email")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    roles: user_data.get("roles")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect())
                        .unwrap_or_else(|| vec!["user".to_string()]),
                    metadata: user_data.as_object()
                        .map(|obj| obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect())
                        .unwrap_or_default(),
                };
                
                Ok(user)
            } else {
                let error_text = response.text().await?;
                Err(format!("User info request failed: {}", error_text).into())
            }
        } else {
            // If no userinfo endpoint, create minimal user from token
            Ok(super::UserInfo {
                id: "oauth_user".to_string(),
                username: "oauth_user".to_string(),
                email: None,
                roles: vec!["user".to_string()],
                metadata: HashMap::new(),
            })
        }
    }
    
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];
        
        let response = self.client
            .post(&self.config.token_endpoint)
            .form(&params)
            .send()
            .await?;
            
        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;
            Ok(token_response)
        } else {
            let error_text = response.text().await?;
            Err(format!("Token refresh failed: {}", error_text).into())
        }
    }
}

/// Create an OAuth2 provider based on configuration
pub fn create_oauth_provider(config: OAuth2Config) -> Result<Box<dyn OAuth2Provider>, Box<dyn std::error::Error>> {
    Ok(Box::new(GenericOAuth2Provider::new(config)))
} 