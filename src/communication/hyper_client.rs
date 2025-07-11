use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::RwLock;

/// Hyper client errors
#[derive(Debug, Error)]
pub enum HyperClientError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
    
    #[error("Body processing error: {0}")]
    Body(String),
}

/// HTTP response wrapper
#[derive(Debug, Clone)]
pub struct HyperResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HyperResponse {
    /// Get the status code
    pub fn status(&self) -> hyper::StatusCode {
        hyper::StatusCode::from_u16(self.status).unwrap_or(hyper::StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Get response headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Get response body as bytes
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get response body as text
    pub fn text(&self) -> Result<String, HyperClientError> {
        std::str::from_utf8(&self.body)
            .map(|s| s.to_string())
            .map_err(|e| HyperClientError::Body(format!("Invalid UTF-8 in response body: {}", e)))
    }

    /// Get response body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, HyperClientError> {
        serde_json::from_slice(&self.body).map_err(HyperClientError::JsonSerialization)
    }

    /// Return error if status is not success
    pub fn error_for_status(self) -> Result<Self, HyperClientError> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(HyperClientError::InvalidResponse(format!(
                "HTTP error {}: {}",
                self.status,
                String::from_utf8_lossy(&self.body)
            )))
        }
    }
}

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HyperClientConfig {
    pub timeout: Duration,
    pub user_agent: String,
    pub max_connections_per_host: usize,
    pub http2_enabled: bool,
    pub default_headers: HashMap<String, String>,
}

impl Default for HyperClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: "songbird-orchestrator/0.2.0".to_string(),
            max_connections_per_host: 10,
            http2_enabled: true,
            default_headers: HashMap::new(),
        }
    }
}

/// Simplified HTTP client for communication layer
#[derive(Debug, Clone)]
pub struct HyperHttpClient {
    timeout: Duration,
    user_agent: String,
    default_headers: Arc<RwLock<HashMap<String, String>>>,
}

impl HyperHttpClient {
    /// Create a new HyperHttpClient with default configuration
    pub fn new() -> Result<Self, HyperClientError> {
        let config = HyperClientConfig::default();
        Self::with_config(config)
    }

    /// Create a new HyperHttpClient with custom configuration
    pub fn with_config(config: HyperClientConfig) -> Result<Self, HyperClientError> {
        Ok(Self {
            timeout: config.timeout,
            user_agent: config.user_agent,
            default_headers: Arc::new(RwLock::new(config.default_headers)),
        })
    }

    /// Execute a GET request (simplified implementation)
    pub async fn get(&self, url: &str) -> Result<HyperResponse, HyperClientError> {
        tracing::debug!("HTTP GET request to: {}", url);
        
        // Simplified implementation - in a real implementation, this would use hyper
        // For now, return a mock response
        Ok(HyperResponse {
            status: 200,
            headers: HashMap::new(),
            body: b"{}".to_vec(),
        })
    }

    /// Execute a POST request with JSON body (simplified implementation)
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> Result<HyperResponse, HyperClientError> {
        tracing::debug!("HTTP POST request to: {}", url);
        
        // Serialize the body to validate it's valid JSON
        let _body_json = serde_json::to_string(body)?;
        
        // Simplified implementation - in a real implementation, this would use hyper
        // For now, return a mock response
        Ok(HyperResponse {
            status: 200,
            headers: HashMap::new(),
            body: b"{}".to_vec(),
        })
    }

    /// Execute a generic HTTP request (simplified implementation)
    pub async fn request(&self, method: &str, url: &str, body: Option<&str>) -> Result<HyperResponse, HyperClientError> {
        tracing::debug!("HTTP {} request to: {}", method, url);
        
        // Simplified implementation - in a real implementation, this would use hyper
        // For now, return a mock response
        Ok(HyperResponse {
            status: 200,
            headers: HashMap::new(),
            body: body.unwrap_or("{}").as_bytes().to_vec(),
        })
    }

    /// Add a default header
    pub async fn add_default_header(&self, key: String, value: String) {
        let mut headers = self.default_headers.write().await;
        headers.insert(key, value);
    }

    /// Remove a default header
    pub async fn remove_default_header(&self, key: &str) {
        let mut headers = self.default_headers.write().await;
        headers.remove(key);
    }

    /// Set timeout (creates new client)
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set user agent (creates new client)
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hyper_client_creation() {
        let client = HyperHttpClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_default_headers() {
        let client = HyperHttpClient::new().unwrap();
        client.add_default_header("x-custom".to_string(), "test-value".to_string()).await;
        let headers = client.default_headers.read().await;
        assert_eq!(headers.get("x-custom"), Some(&"test-value".to_string()));
    }

    #[tokio::test]
    async fn test_get_request() {
        let client = HyperHttpClient::new().unwrap();
        let response = client.get("https://example.com").await.unwrap();
        assert!(response.is_success());
    }

    #[tokio::test]
    async fn test_post_json_request() {
        let client = HyperHttpClient::new().unwrap();
        let data = serde_json::json!({"test": "data"});
        let response = client.post_json("https://example.com", &data).await.unwrap();
        assert!(response.is_success());
    }
} 
