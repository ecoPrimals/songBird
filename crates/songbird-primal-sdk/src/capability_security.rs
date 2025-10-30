//! Security Capability Provider (Primal-Agnostic)
//!
//! Provides security, authentication, and authorization capabilities through
//! pure capability-based discovery. No hardcoded primal names.
//!
//! # Philosophy
//!
//! This module requests "security" capability without knowing or caring which
//! primal provides it. Could be beardog, could be something else, could be
//! multiple providers. We only care about the CAPABILITY, not the PROVIDER.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use songbird_types::{SongbirdError, SongbirdResult};

/// Security capability configuration (vendor/primal agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCapabilityConfig {
    /// Required security capabilities
    pub required_capabilities: Vec<String>,
    /// Optional API authentication method
    pub auth_method: Option<AuthMethod>,
    /// Whether to verify TLS certificates
    pub verify_tls: bool,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Discovery hints (environment variables to check)
    pub discovery_hints: Vec<String>,
}

/// Authentication method (vendor-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// Bearer token
    BearerToken { token_source: TokenSource },
    /// API key
    ApiKey { key_source: TokenSource },
    /// Mutual TLS
    MutualTls { cert_path: String, key_path: String },
    /// OAuth 2.0
    OAuth2 { client_id: String, client_secret_source: TokenSource },
}

/// Token source (environment variable or file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenSource {
    /// Environment variable
    Environment { var_name: String },
    /// File path
    File { path: String },
    /// Inline (not recommended for production)
    Inline { value: String },
}

/// Security capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequest {
    /// Operation to perform
    pub operation: SecurityOperation,
    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Request ID for tracking
    pub request_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Security operations (capability-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityOperation {
    /// Authenticate user/service
    Authenticate { credentials: Credentials },
    /// Authorize action
    Authorize { principal: String, resource: String, action: String },
    /// Encrypt data
    Encrypt { data: Vec<u8>, algorithm: Option<String> },
    /// Decrypt data
    Decrypt { encrypted_data: Vec<u8> },
    /// Sign data
    Sign { data: Vec<u8> },
    /// Verify signature
    Verify { data: Vec<u8>, signature: Vec<u8> },
    /// Generate token
    GenerateToken { principal: String, scopes: Vec<String> },
    /// Validate token
    ValidateToken { token: String },
    /// Audit log
    AuditLog { event: AuditEvent },
}

/// Credentials (type-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    /// Username and password
    UsernamePassword { username: String, password: String },
    /// API key
    ApiKey { key: String },
    /// Token
    Token { token: String },
    /// Certificate
    Certificate { cert_pem: String },
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event type
    pub event_type: String,
    /// Principal (user/service)
    pub principal: String,
    /// Resource affected
    pub resource: String,
    /// Action performed
    pub action: String,
    /// Result (success/failure)
    pub result: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Security capability response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResponse {
    /// Request ID
    pub request_id: String,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Provider ID (learned through discovery)
    pub provider_id: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl Default for SecurityCapabilityConfig {
    fn default() -> Self {
        Self {
            required_capabilities: vec![
                "authentication".to_string(),
                "authorization".to_string(),
            ],
            auth_method: None,
            verify_tls: std::env::var("SECURITY_VERIFY_TLS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            timeout_secs: std::env::var("SECURITY_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            max_retries: std::env::var("SECURITY_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            discovery_hints: vec![
                "SONGBIRD_SECURITY_DISCOVERY".to_string(),
                "SECURITY_ENDPOINT".to_string(),
                "AUTH_SERVICE_URL".to_string(),
                // Legacy compatibility (for migration period only)
                "BEARDOG_ENDPOINT".to_string(),
            ],
        }
    }
}

/// Request security capability from discovered provider
///
/// This function uses the infant discovery engine to find a provider that
/// offers "security" capability. It doesn't know or care about primal names.
pub async fn request_security_capability(
    request: SecurityRequest,
) -> SongbirdResult<SecurityResponse> {
    // Import the infant discovery engine
    use songbird_universal::InfantDiscoveryEngine;

    // Get or create discovery engine
    let discovery = InfantDiscoveryEngine::new();

    // Request security capability (no primal name needed!)
    let response = discovery
        .request_capability(
            "security",
            &serde_json::to_string(&request.operation)
                .map_err(|e| SongbirdError::internal_error(&format!("Serialization failed: {}", e)))?,
            &serde_json::to_value(&request.parameters)
                .map_err(|e| SongbirdError::internal_error(&format!("Value conversion failed: {}", e)))?,
        )
        .await?;

    // Parse response
    let security_response: SecurityResponse = serde_json::from_value(response.response_data)
        .map_err(|e| SongbirdError::internal_error(&format!("Failed to parse security response: {}", e)))?;

    Ok(security_response)
}

/// Helper: Authenticate user
pub async fn authenticate(credentials: Credentials) -> SongbirdResult<SecurityResponse> {
    let request = SecurityRequest {
        operation: SecurityOperation::Authenticate { credentials },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    request_security_capability(request).await
}

/// Helper: Authorize action
pub async fn authorize(
    principal: String,
    resource: String,
    action: String,
) -> SongbirdResult<SecurityResponse> {
    let request = SecurityRequest {
        operation: SecurityOperation::Authorize {
            principal,
            resource,
            action,
        },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    request_security_capability(request).await
}

/// Helper: Encrypt data
pub async fn encrypt(data: Vec<u8>) -> SongbirdResult<Vec<u8>> {
    let request = SecurityRequest {
        operation: SecurityOperation::Encrypt {
            data,
            algorithm: None,
        },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_security_capability(request).await?;

    if response.success {
        // Extract encrypted data from response
        let encrypted = response.data
            .get("encrypted_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SongbirdError::internal_error("No encrypted data in response"))?;

        // Decode from base64
        base64::decode(encrypted)
            .map_err(|e| SongbirdError::internal_error(&format!("Failed to decode encrypted data: {}", e)))
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "Encryption failed".to_string()),
        ))
    }
}

/// Helper: Decrypt data
pub async fn decrypt(encrypted_data: Vec<u8>) -> SongbirdResult<Vec<u8>> {
    let request = SecurityRequest {
        operation: SecurityOperation::Decrypt { encrypted_data },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_security_capability(request).await?;

    if response.success {
        // Extract decrypted data from response
        let decrypted = response.data
            .get("decrypted_data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SongbirdError::internal_error("No decrypted data in response"))?;

        // Decode from base64
        base64::decode(decrypted)
            .map_err(|e| SongbirdError::internal_error(&format!("Failed to decode decrypted data: {}", e)))
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "Decryption failed".to_string()),
        ))
    }
}

/// Helper: Generate token
pub async fn generate_token(
    principal: String,
    scopes: Vec<String>,
) -> SongbirdResult<String> {
    let request = SecurityRequest {
        operation: SecurityOperation::GenerateToken { principal, scopes },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_security_capability(request).await?;

    if response.success {
        response.data
            .get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| SongbirdError::internal_error("No token in response"))
    } else {
        Err(SongbirdError::internal_error(
            &response.error.unwrap_or_else(|| "Token generation failed".to_string()),
        ))
    }
}

/// Helper: Validate token
pub async fn validate_token(token: String) -> SongbirdResult<bool> {
    let request = SecurityRequest {
        operation: SecurityOperation::ValidateToken { token },
        parameters: HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
    };

    let response = request_security_capability(request).await?;

    if response.success {
        response.data
            .get("valid")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| SongbirdError::internal_error("No validation result in response"))
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityCapabilityConfig::default();
        assert!(config.verify_tls);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.required_capabilities.is_empty());
    }

    #[test]
    fn test_security_request_creation() {
        let request = SecurityRequest {
            operation: SecurityOperation::Authenticate {
                credentials: Credentials::UsernamePassword {
                    username: "test".to_string(),
                    password: "pass".to_string(),
                },
            },
            parameters: HashMap::new(),
            request_id: "test-123".to_string(),
            timestamp: Utc::now(),
        };

        assert_eq!(request.request_id, "test-123");
    }
}

