use std::collections::HashMap;
// Module imports
use std::sync::Arc;
use std::time::Duration;
use hyper::{Method, StatusCode, Request, Response, Uri, HeaderMap};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use hyper_util::rt::TokioExecutor;
use http_body_util::{Full, BodyExt};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use bytes::Bytes;

/// Hyper-based HTTP client with connection pooling and timeout support
#[derive(Debug, Clone)]
pub struct HyperHttpClient {
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    timeout: Duration,
    user_agent: String,
    default_headers: Arc<RwLock<HashMap<String, String>>>,
}
/// HTTP client configuration
pub struct HyperClientConfig {
    pub timeout: Duration,
    pub user_agent: String,
    pub max_connections_per_host: usize,
    pub http2_enabled: bool,
    pub default_headers: HashMap<String, String>,
impl Default for HyperClientConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        Self {
            timeout: env_config.request_timeout(),
            user_agent: "songbird-orchestrator/0.2.0".to_string(),
            max_connections_per_host: std::env::var("SONGBIRD_MAX_CONNECTIONS_PER_HOST")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            http2_enabled: std::env::var("SONGBIRD_ENABLE_HTTP2")
                .map(|v| v.parse().unwrap_or(false))  // Security: Default TLS verification to secure
                .unwrap_or(true),
            default_headers: HashMap::new(),
        }
    }
/// Hyper client errors
#[derive(Debug, Error)]
pub enum HyperClientError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] hyper_util::client::legacy::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
    #[error("Request timeout")]
    Timeout,
    #[error("HTTP error {status}: {message}")]
    HttpStatus { status: u16, message: String },
    #[error("Body processing error: {0}")]
    Body(String),
    #[error("URI parsing error: {0}")]
    UriError(#[from] hyper::http::uri::InvalidUri),
    #[error("Header error: {0}")]
    HeaderError(#[from] hyper::http::header::InvalidHeaderValue),
    #[error("HTTP building error: {0}")]
    HttpError(#[from] hyper::http::Error),
impl HyperHttpClient {
    /// Create a new HyperHttpClient with default configuration
    pub fn new() -> Result<Self, HyperClientError> {
        let config = HyperClientConfig::default();
        Self::with_config(config)
    /// Create a new HyperHttpClient with custom configuration
    pub fn with_config(config: HyperClientConfig) -> Result<Self, HyperClientError> {
        // Build HTTPS connector with rustls
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| HyperClientError::Body(format!("Failed to create HTTPS connector: {}", e)))?
            .https_or_http()
            .enable_http1()
            .build();
        // Build client with connection pooling
        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(config.max_connections_per_host)
            .build(https);
        Ok(Self {
            client,
            timeout: config.timeout,
            user_agent: config.user_agent,
            default_headers: Arc::new(RwLock::new(config.default_headers)),
        })
    /// Execute a GET request
    pub async fn get(&self, url: &str) -> Result<HyperResponse, HyperClientError> {
        let uri: Uri = url.parse()?;
        let mut request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("user-agent", &self.user_agent);
        // Add default headers
        let default_headers = self.default_headers.read().await;
        for (key, value) in default_headers.iter() {
            request = request.header(key, value);
        let request = request.body(Full::new(Bytes::new()))?;
        let response = tokio::time::timeout(self.timeout, self.client.request(request)).await
            .map_err(|_| HyperClientError::Timeout)?
            .map_err(HyperClientError::Request)?;
        self.convert_response(response).await
    /// Execute a POST request with JSON body
    pub async fn post_json<T: Serialize>(&self, url: &str, body: &T) -> Result<HyperResponse, HyperClientError> {
        let body_bytes = serde_json::to_vec(body)?;
        let body = Full::new(Bytes::from(body_bytes));
            .method(Method::POST)
            .header("content-type", "application/json")
        let request = request.body(body)?;
    /// Execute a PUT request with JSON body
    pub async fn put_json<T: Serialize>(&self, url: &str, body: &T) -> Result<HyperResponse, HyperClientError> {
            .method(Method::PUT)
    /// Execute a request with optional JSON body
    pub async fn request<T: Serialize>(
        &self,
        method: Method,
        url: &str,
        body: Option<&T>,
    ) -> Result<HyperResponse, HyperClientError> {
        match method {
            Method::GET => self.get(url).await,
            Method::POST => {
                if let Some(body) = body {
                    self.post_json(url, body).await
                } else {
                    self.post_json(url, &serde_json::json!({})).await
                }
            }
            Method::PUT => {
                    self.put_json(url, body).await
                    self.put_json(url, &serde_json::json!({})).await
            _ => Err(HyperClientError::Body("Unsupported HTTP method".to_string())),
    /// Add a default header
    pub async fn add_default_header(&self, key: String, value: String) {
        let mut headers = self.default_headers.write().await;
        headers.insert(key, value);
    /// Remove a default header
    pub async fn remove_default_header(&self, key: &str) {
        headers.remove(key);
    /// Set timeout (creates new client)
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    /// Set user agent (creates new client)
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
    /// Convert hyper response to our response type
    async fn convert_response(&self, response: Response<hyper::body::Incoming>) -> Result<HyperResponse, HyperClientError> {
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response.collect().await
            .map_err(|e| HyperClientError::Body(format!("Failed to read response body: {}", e)))?
            .to_bytes()
            .to_vec();
        Ok(HyperResponse {
            status,
            headers,
            body: body_bytes,
/// HTTP response wrapper
pub struct HyperResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
impl HyperResponse {
    /// Get the status code
    pub fn status(&self) -> StatusCode {
        self.status
    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    /// Get response headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    /// Get response body as bytes
    pub fn body(&self) -> &[u8] {
        &self.body
    /// Get response body as text
    pub fn text(&self) -> Result<String, HyperClientError> {
        std::str::from_utf8(&self.body)
            .map(|s| s.to_string())
            .map_err(|e| HyperClientError::Body(format!("Invalid UTF-8 in response body: {}", e)))
    /// Get response body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, HyperClientError> {
        serde_json::from_slice(&self.body).map_err(HyperClientError::JsonSerialization)
    /// Get content length
    pub fn content_length(&self) -> Option<u64> {
        self.headers
            .get("content-length")?
            .to_str()
            .ok()?
            .parse()
            .ok()
    /// Return error if status is not success
    pub fn error_for_status(self) -> Result<Self, HyperClientError> {
        if self.status.is_success() {
            Ok(self)
        } else {
            let message = self.text().unwrap_or_else(|_| "Unknown error".to_string());
            Err(HyperClientError::HttpStatus {
                status: self.status.as_u16(),
                message,
            })
/// Builder for configuring HyperHttpClient
#[derive(Debug)]
pub struct HyperClientBuilder {
    config: HyperClientConfig,
impl HyperClientBuilder {
    /// Create a new builder
    pub fn new() -> Self {
            config: HyperClientConfig::default(),
    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
    /// Set user agent
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.config.user_agent = user_agent.into();
    /// Set max connections per host
    pub fn max_connections_per_host(mut self, max: usize) -> Self {
        self.config.max_connections_per_host = max;
    /// Enable/disable HTTP/2
    pub fn http2_enabled(mut self, enabled: bool) -> Self {
        self.config.http2_enabled = enabled;
    /// Add default header
    pub fn default_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.config.default_headers.insert(key.into(), value.into());
    /// Build the client
    pub fn build(self) -> Result<HyperHttpClient, HyperClientError> {
        HyperHttpClient::with_config(self.config)
impl Default for HyperClientBuilder {
        Self::new()
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_hyper_client_creation() {
        let client = HyperHttpClient::new();
        assert!(client.is_ok());
    async fn test_hyper_client_builder() {
        let client = HyperClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .user_agent("test-client")
            .max_connections_per_host(5)
            .default_header("x-test", "value")
        
        async fn test_default_headers() {
        let client = HyperHttpClient::new()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))
            .map_err(|e| { tracing::error!("HTTP client setup failed: {}", e); e })?;
        client.add_default_header("x-custom".to_string(), "test-value".to_string()).await;
        let headers = client.default_headers.read().await;
        assert_eq!(headers.get("x-custom"), Some(&"test-value".to_string()));
    } 
