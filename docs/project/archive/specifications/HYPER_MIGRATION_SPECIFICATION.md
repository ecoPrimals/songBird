# 🚀 **Hyper Client Migration Specification**

**Document Version**: 1.0  
**Target Release**: v0.2.0  
**Implementation Team**: Core HTTP Team  
**Estimated Effort**: 1 Day  
**Priority**: High  

## 📋 **Executive Summary**

This specification outlines the complete migration from `reqwest` to native `hyper` client for all HTTP communication in Songbird Orchestrator. The migration will eliminate external HTTP client dependencies while maintaining full compatibility with existing federation and institutional integration features.

## 🎯 **Migration Objectives**

### **Primary Goals**
- **100% Pure Rust**: Remove reqwest dependency, use hyper directly
- **Zero Breaking Changes**: Maintain all existing API contracts
- **Performance Improvement**: Reduce memory footprint by 15-20%
- **Federation Ready**: Preserve all institutional integration capabilities
- **Zero-Touch Compatible**: Support automatic configuration discovery

### **Success Criteria**
- ✅ All tests pass without modification
- ✅ Circuit breaker functionality preserved
- ✅ Connection pooling maintained
- ✅ TLS/mTLS support intact
- ✅ OAuth2 flows continue working
- ✅ Memory usage reduced by 15%+
- ✅ Request latency improved by 5-10%

## 🏗️ **Architecture Overview**

### **Current State (Reqwest)**
```rust
// src/communication/mod.rs
pub struct HttpCommunication {
    base_url: String,
    client: reqwest::Client,  // ← REMOVE
    timeout: Duration,
    service_registry: Option<Arc<dyn ServiceRegistry>>,
    circuit_breakers: Arc<DashMap<String, CircuitBreaker>>,
    metrics: Arc<HttpCommunicationMetrics>,
}
```

### **Target State (Hyper)**
```rust
// src/communication/mod.rs
pub struct HttpCommunication {
    base_url: String,
    client: Client<HttpsConnector<HttpConnector>>,  // ← NEW
    timeout: Duration,
    service_registry: Option<Arc<dyn ServiceRegistry>>,
    circuit_breakers: Arc<DashMap<String, CircuitBreaker>>,
    metrics: Arc<HttpCommunicationMetrics>,
}
```

## 📦 **Dependency Changes**

### **Cargo.toml Updates**

**Remove:**
```toml
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
```

**Add:**
```toml
hyper = { version = "1.0", features = ["client", "http1", "http2"] }
hyper-rustls = { version = "0.24", default-features = false, features = ["http1", "http2", "tls12", "logging"] }
hyper-timeout = "0.4"
http-body-util = "0.1"
```

### **Feature Flag Updates**
```toml
[features]
default = ["file-config", "hyper-communication", "built-in-observability"]
hyper-communication = ["hyper", "hyper-rustls", "hyper-timeout"]  # NEW
```

## 🔧 **Implementation Specification**

### **1. Core HTTP Client Replacement**

**File**: `src/communication/hyper_client.rs` (NEW)

```rust
use hyper::{Body, Client, Method, Request, Response, Uri};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_timeout::TimeoutConnector;
use http_body_util::{BodyExt, Full};
use std::time::Duration;

pub struct HyperHttpClient {
    client: Client<TimeoutConnector<HttpsConnector<hyper::client::HttpConnector>>>,
    default_timeout: Duration,
    user_agent: String,
}

impl HyperHttpClient {
    pub fn new(timeout: Duration) -> Result<Self, HyperClientError> {
        // Build HTTPS connector with rustls
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        
        // Add timeout wrapper
        let timeout_connector = TimeoutConnector::new(https);
        timeout_connector.set_connect_timeout(Some(timeout));
        timeout_connector.set_read_timeout(Some(timeout));
        timeout_connector.set_write_timeout(Some(timeout));
        
        // Build client with connection pooling
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .http2_only(false)
            .build(timeout_connector);
        
        Ok(Self {
            client,
            default_timeout: timeout,
            user_agent: "songbird-orchestrator/0.2.0".to_string(),
        })
    }
    
    pub async fn post_json(
        &self,
        url: &str,
        payload: serde_json::Value,
        headers: &HashMap<String, String>,
    ) -> Result<serde_json::Value, HyperClientError> {
        let uri: Uri = url.parse()?;
        let body_bytes = serde_json::to_vec(&payload)?;
        let body = Full::new(body_bytes.into());
        
        let mut request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .header("user-agent", &self.user_agent);
        
        // Add custom headers
        for (key, value) in headers {
            request = request.header(key, value);
        }
        
        let request = request.body(body)?;
        let response = self.client.request(request).await?;
        
        self.parse_json_response(response).await
    }
    
    async fn parse_json_response(
        &self,
        response: Response<hyper::body::Incoming>,
    ) -> Result<serde_json::Value, HyperClientError> {
        let status = response.status();
        let body_bytes = response.collect().await?.to_bytes();
        
        if status.is_success() {
            let json: serde_json::Value = serde_json::from_slice(&body_bytes)?;
            Ok(json)
        } else {
            let error_text = String::from_utf8_lossy(&body_bytes);
            Err(HyperClientError::HttpError {
                status: status.as_u16(),
                message: error_text.to_string(),
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HyperClientError {
    #[error("HTTP error {status}: {message}")]
    HttpError { status: u16, message: String },
    
    #[error("Request building error: {0}")]
    RequestError(#[from] hyper::http::Error),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] hyper::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("URI parsing error: {0}")]
    UriError(#[from] hyper::http::uri::InvalidUri),
}
```

### **2. HttpCommunication Migration**

**File**: `src/communication/mod.rs` (MODIFY)

```rust
// Replace reqwest imports
use crate::communication::hyper_client::{HyperHttpClient, HyperClientError};

impl HttpCommunication {
    pub fn new(base_url: String) -> Self {
        let timeout = Duration::from_secs(30);
        let client = HyperHttpClient::new(timeout)
            .unwrap_or_else(|e| {
                tracing::error!("Failed to create Hyper client: {}", e);
                panic!("Cannot initialize HTTP communication without client");
            });

        Self {
            base_url,
            client,
            timeout,
            service_registry: None,
            circuit_breakers: Arc::new(DashMap::new()),
            circuit_breaker_config: CircuitBreakerConfig::default(),
            metrics: Arc::new(HttpCommunicationMetrics::default()),
        }
    }
    
    // Update send_message implementation
    async fn send_message(
        &self,
        target: ServiceAddress,
        message: ServiceMessage,
    ) -> Result<CommunicationResponse> {
        let circuit_breaker = self.get_circuit_breaker(&target.service_id);
        
        if !circuit_breaker.should_allow_request() {
            // ... existing circuit breaker logic unchanged
        }
        
        let url = self.build_url(&target, None).await?;
        let start_time = Instant::now();
        
        // Build headers
        let mut headers = HashMap::new();
        headers.extend(message.headers.clone());
        headers.insert("x-message-id".to_string(), message.id.clone());
        if let Some(correlation_id) = &message.correlation_id {
            headers.insert("x-correlation-id".to_string(), correlation_id.clone());
        }
        
        // Send request using new Hyper client
        let response_result = self.client.post_json(&url, message.payload, &headers).await;
        
        // Handle response (existing logic preserved)
        match response_result {
            Ok(payload) => {
                let elapsed = start_time.elapsed();
                circuit_breaker.record_success();
                self.metrics.record_request_success(elapsed.as_millis() as u64, 0);
                
                Ok(CommunicationResponse {
                    message_id: message.id,
                    success: true,
                    payload: Some(payload),
                    error: None,
                    timestamp: chrono::Utc::now(),
                })
            }
            Err(e) => {
                let elapsed = start_time.elapsed();
                circuit_breaker.record_failure();
                self.metrics.record_request_failure();
                
                let (status_code, error_message) = match e {
                    HyperClientError::HttpError { status, message } => (status, message),
                    other => (500, other.to_string()),
                };
                
                Ok(CommunicationResponse {
                    message_id: message.id,
                    success: false,
                    payload: Some(serde_json::json!({
                        "error": error_message,
                        "status_code": status_code,
                        "elapsed_ms": elapsed.as_millis(),
                    })),
                    error: Some(error_message),
                    timestamp: chrono::Utc::now(),
                })
            }
        }
    }
}
```

### **3. OAuth2 Integration Migration**

**File**: `src/security/oauth.rs` (MODIFY)

```rust
use crate::communication::hyper_client::HyperHttpClient;

pub struct OAuth2Provider {
    client: HyperHttpClient,  // Replace reqwest::Client
    config: OAuth2Config,
}

impl OAuth2Provider {
    pub fn new(config: OAuth2Config) -> Result<Self, OAuth2Error> {
        let client = HyperHttpClient::new(Duration::from_secs(30))?;
        Ok(Self { client, config })
    }
    
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse, OAuth2Error> {
        let token_url = &self.config.token_endpoint;
        let payload = serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "client_id": self.config.client_id,
            "client_secret": self.config.client_secret,
            "redirect_uri": self.config.redirect_uri
        });
        
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        let response = self.client.post_json(token_url, payload, &headers).await?;
        let token_response: TokenResponse = serde_json::from_value(response)?;
        
        Ok(token_response)
    }
}
```

### **4. Proxy Module Migration**

**File**: `proxy/mod.rs` (MODIFY)

```rust
use crate::communication::hyper_client::HyperHttpClient;

impl SongbirdProxy {
    async fn forward_request(&self, service: &ServiceInfo, request: ProxyRequest) -> Result<ProxyResponse, SongbirdError> {
        let client = HyperHttpClient::new(Duration::from_secs(self.config.request_timeout))?;
        
        let target_url = self.build_target_url(service, &request)?;
        let payload = self.extract_request_payload(&request)?;
        let headers = self.build_proxy_headers(&request);
        
        let response = client.post_json(&target_url, payload, &headers).await
            .map_err(|e| SongbirdError::Communication(format!("Proxy request failed: {}", e)))?;
        
        Ok(ProxyResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: serde_json::to_vec(&response)?,
        })
    }
}
```

## 🧪 **Testing Strategy**

### **1. Unit Tests Migration**

**File**: `tests/hyper_communication_tests.rs` (NEW)

```rust
#[tokio::test]
async fn test_hyper_client_basic_request() {
    let client = HyperHttpClient::new(Duration::from_secs(5)).unwrap();
    let payload = serde_json::json!({"test": "data"});
    let headers = HashMap::new();
    
    // Mock server setup would go here
    let response = client.post_json("http://localhost:8080/test", payload, &headers).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_circuit_breaker_with_hyper() {
    let communication = HttpCommunication::new("http://localhost:8080".to_string());
    
    // Test circuit breaker functionality with new hyper client
    // ... existing circuit breaker tests should pass unchanged
}
```

### **2. Integration Tests**

**File**: `tests/http_server_tests.rs` (MODIFY)

```rust
// Update test client creation
async fn create_test_client() -> HyperHttpClient {
    HyperHttpClient::new(Duration::from_secs(5)).unwrap()
}

// All existing tests should pass with minimal changes
#[tokio::test]
async fn test_http_server_with_hyper_client() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/health", port);
    
    let response = client.post_json(&url, serde_json::json!({}), &HashMap::new()).await.unwrap();
    assert!(response.get("success").unwrap().as_bool().unwrap());
}
```

### **3. Performance Tests**

**File**: `tests/performance_comparison.rs` (NEW)

```rust
#[tokio::test]
async fn benchmark_hyper_vs_reqwest() {
    // Performance comparison tests
    // Target: 5-10% latency improvement
    // Target: 15-20% memory reduction
}
```

## 📊 **Performance Expectations**

### **Memory Usage Improvements**
- **Before (Reqwest)**: ~2.5MB baseline HTTP client memory
- **After (Hyper)**: ~2.0MB baseline HTTP client memory
- **Reduction**: 20% memory footprint reduction

### **Request Latency Improvements**
- **Before (Reqwest)**: ~5ms average request overhead
- **After (Hyper)**: ~4.5ms average request overhead  
- **Improvement**: 10% latency reduction

### **Connection Pooling**
- **Pool Size**: 10 connections per host (maintained)
- **Idle Timeout**: 30 seconds (maintained)
- **Keep-Alive**: HTTP/1.1 and HTTP/2 support (maintained)

## 🔒 **Security Considerations**

### **TLS Configuration**
```rust
// Maintain existing security standards
let https = HttpsConnectorBuilder::new()
    .with_native_roots()          // System certificate store
    .https_or_http()              // HTTPS preferred
    .enable_http1()               // HTTP/1.1 support
    .enable_http2()               // HTTP/2 support
    .build();
```

### **Certificate Validation**
- ✅ Maintain rustls-based TLS validation
- ✅ Support custom CA certificates
- ✅ mTLS support for institutional integration
- ✅ Certificate pinning capabilities

## 📝 **Migration Checklist**

### **Phase 1: Core Client (4 hours)**
- [ ] Create `src/communication/hyper_client.rs`
- [ ] Implement `HyperHttpClient` with connection pooling
- [ ] Add error handling and type definitions
- [ ] Update `Cargo.toml` dependencies

### **Phase 2: Communication Layer (2 hours)**
- [ ] Migrate `HttpCommunication::send_message()`
- [ ] Preserve circuit breaker integration
- [ ] Update metrics collection
- [ ] Maintain existing API contracts

### **Phase 3: Integration Points (1 hour)**
- [ ] Update OAuth2 provider
- [ ] Migrate proxy module
- [ ] Update test utilities
- [ ] Fix compilation errors

### **Phase 4: Testing & Validation (1 hour)**
- [ ] Run full test suite
- [ ] Performance benchmarking
- [ ] Memory usage validation
- [ ] Integration testing

## 🚀 **Deployment Strategy**

### **Feature Flag Rollout**
```toml
[features]
default = ["hyper-communication"]  # Enable by default
reqwest-fallback = ["reqwest"]     # Temporary fallback option
```

### **Rollback Plan**
- Keep reqwest dependency as optional feature for 1 release cycle
- Monitor performance metrics for 48 hours post-deployment
- Automatic rollback triggers if error rate > 1% increase

## 📋 **Success Metrics**

### **Functional Requirements**
- ✅ All existing tests pass
- ✅ Federation communication works
- ✅ OAuth2 flows functional
- ✅ Circuit breakers operational
- ✅ TLS/mTLS support intact

### **Performance Requirements**
- ✅ Memory usage reduced by ≥15%
- ✅ Request latency improved by ≥5%
- ✅ No increase in error rates
- ✅ Connection pooling efficiency maintained

### **Compatibility Requirements**
- ✅ Zero breaking API changes
- ✅ Institutional integration preserved
- ✅ Zero-touch deployment compatible
- ✅ All existing configuration options work

---

**Implementation Team**: Core HTTP Team  
**Review Required**: Architecture Team  
**Estimated Completion**: 1 Business Day  
**Risk Level**: Low (well-defined migration path) 