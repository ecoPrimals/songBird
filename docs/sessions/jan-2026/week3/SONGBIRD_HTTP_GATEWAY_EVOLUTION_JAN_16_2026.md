# Songbird HTTP Gateway Evolution - January 16, 2026

**Vision**: Songbird as UNIVERSAL HTTP gateway for entire ecosystem  
**Impact**: All primals achieve 100% pure Rust (no HTTP dependencies!)  
**Status**: Architecture designed, ready to implement  
**Alignment**: BiomeOS "Concentrated Gap" strategy PERFECTED

---

## 🎯 **The Opportunity**

### **Current State**

```
┌──────────┐              ┌──────────┐
│ Squirrel │──(HTTP/S)───→│ OpenAI   │
│          │              │HuggingFace│
└──────────┘              └──────────┘
Problem: HTTP dependencies!

┌──────────┐              ┌──────────┐
│  Other   │──(HTTP)─────→│ External │
│ Primals  │              │ Services │
└──────────┘              └──────────┘
Problem: Multiple HTTP gateways!
```

**Issues**:
- ❌ Squirrel has `reqwest` + transitive ring/openssl
- ❌ Multiple primals with HTTP clients
- ❌ Duplicated credential management
- ❌ No centralized rate limiting/caching

---

### **Evolved State** (Target)

```
┌──────────┐
│ Squirrel │
└────┬─────┘
     │ (Unix Socket)
     │
┌────┴─────┐     ┌──────────┐     ┌──────────┐
│  Other   │────→│ Songbird │────→│ External │
│ Primals  │     │  (HTTP   │ HTTPS│   APIs   │
└──────────┘     │ Gateway) │     │          │
                 └──────────┘     │ • OpenAI │
                                  │ • HuggingFace
                                  │ • Stripe │
                                  │ • GitHub │
                                  └──────────┘
```

**Benefits**:
- ✅ ALL primals: Unix sockets ONLY
- ✅ ALL primals: 100% pure Rust!
- ✅ ONE HTTP gateway: Songbird
- ✅ Centralized: credentials, rate limits, caching
- ✅ BiomeOS "Concentrated Gap": PERFECTED

---

## 🏗️ **Architecture**

### **HTTP Gateway Service**

Songbird exposes Unix socket proxies for external HTTP/HTTPS APIs:

```rust
// songbird/crates/songbird-orchestrator/src/http_gateway/mod.rs

pub struct HttpGatewayService {
    /// AI provider proxies (for Squirrel)
    ai_proxies: HashMap<String, AiProxyHandler>,
    
    /// Generic HTTP proxies (for any primal)
    http_proxies: HashMap<String, HttpProxyHandler>,
    
    /// Credential manager
    credentials: CredentialManager,
    
    /// Rate limiter
    rate_limiter: RateLimiter,
    
    /// Response cache
    cache: ResponseCache,
}

impl HttpGatewayService {
    /// Start the HTTP gateway service
    pub async fn start(&self) -> Result<()> {
        // 1. Start AI proxy listeners
        for (id, proxy) in &self.ai_proxies {
            let socket_path = format!("/run/user/1000/songbird-ai-{}.sock", id);
            tokio::spawn(proxy.listen(socket_path));
        }
        
        // 2. Start generic HTTP proxy listeners
        for (id, proxy) in &self.http_proxies {
            let socket_path = format!("/run/user/1000/songbird-http-{}.sock", id);
            tokio::spawn(proxy.listen(socket_path));
        }
        
        // 3. Register capabilities with discovery
        self.register_capabilities().await?;
        
        Ok(())
    }
}
```

---

### **AI Proxy Handler** (for Squirrel)

```rust
// songbird/crates/songbird-orchestrator/src/http_gateway/ai_proxy.rs

pub struct AiProxyHandler {
    provider_id: String,  // "openai", "huggingface", etc.
    http_client: reqwest::Client,
    api_base_url: String,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<ResponseCache>,
}

impl AiProxyHandler {
    /// Listen on Unix socket and proxy requests to external AI API
    pub async fn listen(&self, socket_path: String) -> Result<()> {
        let listener = UnixListener::bind(&socket_path)?;
        info!("🤖 AI proxy ({}) listening on {}", self.provider_id, socket_path);
        
        loop {
            let (stream, _) = listener.accept().await?;
            let handler = self.clone();
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream).await {
                    warn!("AI proxy error: {}", e);
                }
            });
        }
    }
    
    /// Handle individual Unix socket connection
    async fn handle_connection(&self, mut stream: UnixStream) -> Result<()> {
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(&mut stream);
        
        // Read JSON-RPC request from primal (e.g., Squirrel)
        reader.read_until(b'\n', &mut buffer).await?;
        let request: JsonRpcRequest = serde_json::from_slice(&buffer)?;
        
        // Check rate limit
        self.rate_limiter.check(&self.provider_id).await?;
        
        // Check cache
        if let Some(cached) = self.cache.get(&request).await {
            debug!("📦 Cache hit for AI request");
            return self.send_response(&mut stream, cached).await;
        }
        
        // Translate JSON-RPC to HTTP API request
        let http_request = self.translate_request(&request)?;
        
        // Make HTTPS call to external AI API
        let api_key = std::env::var(format!("{}_API_KEY", self.provider_id.to_uppercase()))?;
        let response = self.http_client
            .post(&http_request.url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&http_request.body)
            .send()
            .await?;
        
        // Translate HTTP response to JSON-RPC
        let json_rpc_response = self.translate_response(response).await?;
        
        // Cache response
        self.cache.set(&request, &json_rpc_response).await;
        
        // Send response back via Unix socket
        self.send_response(&mut stream, json_rpc_response).await
    }
    
    /// Translate JSON-RPC request to provider-specific HTTP API
    fn translate_request(&self, request: &JsonRpcRequest) -> Result<HttpRequest> {
        match self.provider_id.as_str() {
            "openai" => self.translate_openai_request(request),
            "huggingface" => self.translate_huggingface_request(request),
            "anthropic" => self.translate_anthropic_request(request),
            _ => Err(anyhow!("Unknown AI provider: {}", self.provider_id)),
        }
    }
}
```

---

### **Generic HTTP Proxy** (for any external HTTP service)

```rust
// songbird/crates/songbird-orchestrator/src/http_gateway/http_proxy.rs

pub struct HttpProxyHandler {
    service_id: String,  // "stripe", "github", "slack", etc.
    http_client: reqwest::Client,
    base_url: String,
    auth_strategy: AuthStrategy,
}

#[derive(Clone)]
pub enum AuthStrategy {
    BearerToken { env_var: String },
    ApiKey { header: String, env_var: String },
    OAuth2 { /* ... */ },
    None,
}

impl HttpProxyHandler {
    /// Listen on Unix socket and proxy generic HTTP requests
    pub async fn listen(&self, socket_path: String) -> Result<()> {
        let listener = UnixListener::bind(&socket_path)?;
        info!("🌐 HTTP proxy ({}) listening on {}", self.service_id, socket_path);
        
        loop {
            let (stream, _) = listener.accept().await?;
            let handler = self.clone();
            tokio::spawn(async move {
                if let Err(e) = handler.handle_connection(stream).await {
                    warn!("HTTP proxy error: {}", e);
                }
            });
        }
    }
    
    /// Handle individual Unix socket connection
    async fn handle_connection(&self, mut stream: UnixStream) -> Result<()> {
        // Read JSON-RPC request
        let request: JsonRpcRequest = read_json_rpc(&mut stream).await?;
        
        // Extract HTTP method, path, headers, body from JSON-RPC params
        let method = request.params.get("method").and_then(|v| v.as_str()).unwrap_or("POST");
        let path = request.params.get("path").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing path"))?;
        let body = request.params.get("body");
        
        // Build URL
        let url = format!("{}{}", self.base_url, path);
        
        // Build HTTP request
        let mut http_request = self.http_client.request(
            method.parse()?,
            &url,
        );
        
        // Add authentication
        http_request = match &self.auth_strategy {
            AuthStrategy::BearerToken { env_var } => {
                let token = std::env::var(env_var)?;
                http_request.header("Authorization", format!("Bearer {}", token))
            }
            AuthStrategy::ApiKey { header, env_var } => {
                let key = std::env::var(env_var)?;
                http_request.header(header, key)
            }
            AuthStrategy::None => http_request,
            _ => http_request,
        };
        
        // Add body if present
        if let Some(body) = body {
            http_request = http_request.json(body);
        }
        
        // Execute HTTP request
        let response = http_request.send().await?;
        
        // Build JSON-RPC response
        let json_rpc_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "status": response.status().as_u16(),
                "headers": response.headers().clone(),
                "body": response.json::<serde_json::Value>().await?,
            })),
            error: None,
        };
        
        // Send response back via Unix socket
        write_json_rpc(&mut stream, &json_rpc_response).await
    }
}
```

---

## 📊 **Configuration**

### **Songbird Configuration** (`songbird.yaml`)

```yaml
http_gateway:
  enabled: true
  
  # AI Proxies (for Squirrel and other AI-consuming primals)
  ai_proxies:
    - id: openai
      capability: "ai:text-generation:openai"
      socket: "/run/user/1000/songbird-ai-openai.sock"
      backend:
        url: "https://api.openai.com/v1/chat/completions"
        auth:
          type: bearer_token
          env_var: OPENAI_API_KEY
      rate_limit:
        requests_per_minute: 60
        burst: 10
      cache:
        enabled: true
        ttl_seconds: 300
        max_size_mb: 100
    
    - id: huggingface
      capability: "ai:text-generation:huggingface"
      socket: "/run/user/1000/songbird-ai-huggingface.sock"
      backend:
        url: "https://api-inference.huggingface.co/models"
        auth:
          type: bearer_token
          env_var: HUGGINGFACE_API_KEY
      retry:
        max_attempts: 3
        backoff: exponential
    
    - id: anthropic
      capability: "ai:text-generation:anthropic"
      socket: "/run/user/1000/songbird-ai-anthropic.sock"
      backend:
        url: "https://api.anthropic.com/v1/messages"
        auth:
          type: api_key
          header: "x-api-key"
          env_var: ANTHROPIC_API_KEY
  
  # Generic HTTP Proxies (for any primal needing external HTTP)
  http_proxies:
    - id: stripe
      capability: "payment:processing:stripe"
      socket: "/run/user/1000/songbird-http-stripe.sock"
      backend:
        url: "https://api.stripe.com/v1"
        auth:
          type: bearer_token
          env_var: STRIPE_API_KEY
      rate_limit:
        requests_per_second: 100
    
    - id: github
      capability: "git:hosting:github"
      socket: "/run/user/1000/songbird-http-github.sock"
      backend:
        url: "https://api.github.com"
        auth:
          type: bearer_token
          env_var: GITHUB_TOKEN
      headers:
        User-Agent: "ecoPrimals-Songbird/1.0"
        Accept: "application/vnd.github.v3+json"
```

---

## 🎯 **Use Cases**

### **1. Squirrel → OpenAI (via Songbird)**

```rust
// Squirrel code (production build, no HTTP!)
let ai_client = discover_capability("ai:text-generation:openai").await?;

let request = json!({
    "jsonrpc": "2.0",
    "method": "chat.completions.create",
    "params": {
        "model": "gpt-4",
        "messages": [
            {"role": "user", "content": "Explain quantum computing"}
        ]
    },
    "id": 1
});

// Connects to /run/user/1000/songbird-ai-openai.sock
let response = ai_client.send_request(request).await?;

// Squirrel never knows about HTTP!
// Songbird handled HTTPS → OpenAI API
```

---

### **2. NestGate → Stripe (via Songbird)**

```rust
// NestGate code (no HTTP dependencies!)
let payment_client = discover_capability("payment:processing:stripe").await?;

let request = json!({
    "jsonrpc": "2.0",
    "method": "http_request",
    "params": {
        "method": "POST",
        "path": "/charges",
        "body": {
            "amount": 2000,
            "currency": "usd",
            "source": "tok_visa"
        }
    },
    "id": 2
});

// Connects to /run/user/1000/songbird-http-stripe.sock
let response = payment_client.send_request(request).await?;

// NestGate: 100% pure Rust, no HTTP!
// Songbird handled HTTPS → Stripe API
```

---

### **3. ToadStool → GitHub (via Songbird)**

```rust
// ToadStool code (deploy to GitHub Packages)
let git_client = discover_capability("git:hosting:github").await?;

let request = json!({
    "jsonrpc": "2.0",
    "method": "http_request",
    "params": {
        "method": "POST",
        "path": "/repos/ecoprimals/nucleus/dispatches",
        "body": {
            "event_type": "deploy",
            "client_payload": {
                "artifact": "nucleus-v1.0.0"
            }
        }
    },
    "id": 3
});

let response = git_client.send_request(request).await?;

// ToadStool: 100% pure Rust!
// Songbird handled HTTPS → GitHub API
```

---

## ✅ **Benefits**

### **For Primals (Squirrel, NestGate, ToadStool, etc.)**

1. ✅ **100% Pure Rust** (including transitive deps!)
   - No `reqwest` → no `rustls` → no `ring`
   - Zero HTTP dependencies
   - Perfect ARM cross-compilation

2. ✅ **Simpler Architecture**
   - One adapter type: `UniversalAdapter` (Unix socket)
   - No vendor-specific HTTP code
   - TRUE PRIMAL infant pattern: **PERFECTED**

3. ✅ **No Credential Management**
   - No API keys in primal configs
   - Songbird handles all external credentials
   - Better security isolation

4. ✅ **Smaller Binaries**
   - No HTTP stack
   - Faster compile times
   - Reduced attack surface

---

### **For Songbird**

1. ✅ **Centralized HTTP Gateway**
   - Single point for ALL external HTTP/HTTPS
   - BiomeOS "Concentrated Gap": **PERFECTED**
   - Clear architectural role

2. ✅ **Unified Rate Limiting**
   - Ecosystem-wide rate limits
   - Per-primal quotas
   - Burst protection

3. ✅ **Response Caching**
   - Reduce external API costs
   - Faster responses
   - Better resilience

4. ✅ **Credential Security**
   - All API keys in ONE place
   - BearDog audit surface: minimized
   - Rotation: simplified

---

### **For Ecosystem**

1. ✅ **Concentrated Gap: PERFECTED**
   - HTTP: Songbird ONLY
   - All primals: Unix sockets ONLY
   - 100% pure Rust everywhere!

2. ✅ **Security**
   - Minimal attack surface
   - Centralized audit point
   - Clear trust boundaries

3. ✅ **Observability**
   - All external calls via Songbird
   - Unified logging/tracing
   - Better debugging

4. ✅ **Cost Optimization**
   - Caching reduces API calls
   - Rate limiting prevents overages
   - Better resource utilization

---

## 🔧 **Implementation Plan**

### **Phase 1: Core HTTP Gateway** (6-8 hours)

**Tasks**:

1. ✅ Create `http_gateway` module
   ```
   songbird/crates/songbird-orchestrator/src/http_gateway/
   ├── mod.rs              (HttpGatewayService)
   ├── ai_proxy.rs         (AiProxyHandler)
   ├── http_proxy.rs       (HttpProxyHandler)
   ├── rate_limiter.rs     (RateLimiter)
   ├── cache.rs            (ResponseCache)
   └── credentials.rs      (CredentialManager)
   ```

2. ✅ Implement JSON-RPC ↔ HTTP translation
3. ✅ Add rate limiting and caching
4. ✅ Integrate with capability discovery
5. ✅ Configuration loading from YAML

---

### **Phase 2: AI Proxies for Squirrel** (4-6 hours)

**Tasks**:

1. ✅ OpenAI proxy (`songbird-ai-openai.sock`)
2. ✅ HuggingFace proxy (`songbird-ai-huggingface.sock`)
3. ✅ Anthropic proxy (`songbird-ai-anthropic.sock`)
4. ✅ Register AI capabilities with discovery
5. ✅ Test with Squirrel (production build, no HTTP!)

---

### **Phase 3: Generic HTTP Proxies** (2-4 hours)

**Tasks**:

1. ✅ Generic proxy handler
2. ✅ Auth strategy implementations (Bearer, API Key, OAuth2)
3. ✅ Test with example services (Stripe, GitHub)

---

### **Phase 4: Testing & Validation** (4-6 hours)

**Tasks**:

1. ✅ Unit tests (rate limiter, cache, translators)
2. ✅ Integration tests (Unix socket ↔ HTTP)
3. ✅ E2E tests (Squirrel → Songbird → OpenAI)
4. ✅ Chaos tests (API failures, timeouts)
5. ✅ Fault injection tests (network errors)

---

### **Phase 5: Documentation** (2-3 hours)

**Tasks**:

1. ✅ Update `SONGBIRD_OVERVIEW.md`
2. ✅ Create `HTTP_GATEWAY_GUIDE.md`
3. ✅ Update primal integration guides
4. ✅ Configuration examples

---

## 📋 **Testing Strategy**

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        
        // Should allow 10 requests
        for _ in 0..10 {
            assert!(limiter.check("test").await.is_ok());
        }
        
        // 11th should fail
        assert!(limiter.check("test").await.is_err());
        
        // After 1 second, should allow again
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(limiter.check("test").await.is_ok());
    }
    
    #[tokio::test]
    async fn test_response_cache() {
        let cache = ResponseCache::new(100);
        
        let request = json!({"method": "test"});
        let response = json!({"result": "cached"});
        
        cache.set(&request, &response).await;
        
        let cached = cache.get(&request).await.unwrap();
        assert_eq!(cached, response);
    }
}
```

---

### **Integration Tests**

```rust
// tests/http_gateway_integration_tests.rs

#[tokio::test]
async fn test_ai_proxy_openai() {
    // Start Songbird HTTP gateway
    let gateway = HttpGatewayService::new().await.unwrap();
    gateway.start().await.unwrap();
    
    // Connect to Unix socket
    let mut stream = UnixStream::connect("/run/user/1000/songbird-ai-openai.sock")
        .await
        .unwrap();
    
    // Send JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "chat.completions.create",
        "params": {
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello"}]
        },
        "id": 1
    });
    
    write_json_rpc(&mut stream, &request).await.unwrap();
    
    // Read response
    let response: JsonRpcResponse = read_json_rpc(&mut stream).await.unwrap();
    
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}
```

---

### **E2E Tests** (with Squirrel)

```rust
// tests/squirrel_ai_proxy_e2e.rs

#[tokio::test]
#[ignore = "Requires Songbird + Squirrel running"]
async fn test_squirrel_openai_via_songbird() {
    // This test requires:
    // 1. Songbird running with HTTP gateway enabled
    // 2. Squirrel running (production build, no HTTP!)
    // 3. OPENAI_API_KEY set in Songbird's env
    
    // Squirrel makes AI request via Unix socket
    let client = SquirrelAiClient::new().await.unwrap();
    let response = client.generate_text("openai", "Explain quantum computing").await.unwrap();
    
    assert!(response.contains("quantum"));
    
    // Verify: Squirrel has NO HTTP dependencies!
    // (cargo tree in Squirrel should show no reqwest/rustls)
}
```

---

### **Chaos Tests**

```rust
// tests/http_gateway_chaos_tests.rs

#[tokio::test]
async fn test_external_api_failure() {
    let gateway = HttpGatewayService::new().await.unwrap();
    
    // Simulate OpenAI API failure
    let mock_server = MockServer::start().await;
    mock_server.register_error("/v1/chat/completions", 500).await;
    
    // Request should fail gracefully
    let result = gateway.proxy_ai_request("openai", request).await;
    assert!(result.is_err());
    
    // Should not crash Songbird!
    assert!(gateway.is_healthy().await);
}

#[tokio::test]
async fn test_timeout_handling() {
    let gateway = HttpGatewayService::new().await.unwrap();
    
    // Simulate slow API
    let mock_server = MockServer::start().await;
    mock_server.register_delay("/v1/chat/completions", Duration::from_secs(60)).await;
    
    // Should timeout gracefully
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        gateway.proxy_ai_request("openai", request)
    ).await;
    
    assert!(result.is_err()); // Timeout
}
```

---

### **Fault Injection Tests**

```rust
// tests/http_gateway_fault_injection.rs

#[tokio::test]
async fn test_network_partition() {
    let gateway = HttpGatewayService::new().await.unwrap();
    
    // Simulate network partition (DNS failure)
    inject_fault(FaultType::DnsFailure).await;
    
    let result = gateway.proxy_ai_request("openai", request).await;
    assert!(result.is_err());
    
    // Should recover after network restored
    clear_faults().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    let result = gateway.proxy_ai_request("openai", request).await;
    assert!(result.is_ok());
}
```

---

## 🎯 **Success Criteria**

### **Songbird**
- [ ] HTTP gateway module implemented
- [ ] AI proxies working (OpenAI, HuggingFace, Anthropic)
- [ ] Generic HTTP proxy working
- [ ] Rate limiting functional
- [ ] Response caching functional
- [ ] All tests passing (unit, integration, E2E, chaos, fault)

### **Squirrel Integration**
- [ ] Squirrel production build: Zero HTTP dependencies
- [ ] `cargo tree | grep reqwest` → Empty
- [ ] `cargo tree | grep ring` → Empty (in production build)
- [ ] Squirrel → Songbird → OpenAI: Working
- [ ] Squirrel → Songbird → HuggingFace: Working

### **Ecosystem**
- [ ] Only 1 HTTP gateway: Songbird
- [ ] All primals: Unix sockets ONLY
- [ ] Concentrated gap: **PERFECTED**
- [ ] Documentation: Complete

---

## 🚀 **Timeline**

| Phase | Component | Time | Complexity |
|-------|-----------|------|------------|
| **Phase 1** | Core HTTP Gateway | 6-8 hours | Medium |
| **Phase 2** | AI Proxies | 4-6 hours | Medium |
| **Phase 3** | Generic Proxies | 2-4 hours | Low |
| **Phase 4** | Testing | 4-6 hours | Medium |
| **Phase 5** | Documentation | 2-3 hours | Low |
| **TOTAL** | - | **18-27 hours** | Medium |

**Coordination**: High (Songbird + Squirrel teams)  
**Risk**: Medium (new architecture, requires testing)  
**Value**: **VERY HIGH** (all primals → 100% pure Rust!)

---

## 🎊 **Impact**

### **Before**
```
HTTP Gateways: 2+ (Songbird + Squirrel + others)
Pure Rust Primals: 2/5 (NestGate, BearDog)
Concentrated Gap: Good
```

### **After**
```
HTTP Gateways: 1 (Songbird ONLY!)
Pure Rust Primals: 5/5 (ALL! 🎉)
Concentrated Gap: PERFECTED ✨
```

---

## 📝 **Next Steps**

### **Immediate** (This Session)
1. ✅ Document HTTP gateway architecture (this doc)
2. ⏳ Create testing evolution plan
3. ⏳ Update `MASTER_EVOLUTION_HANDOFF.md`

### **Week 2** (Joint Work)
1. Implement HTTP gateway core
2. Implement AI proxies for Squirrel
3. Comprehensive testing (unit, E2E, chaos, fault)
4. Squirrel integration

### **Week 3** (Expansion)
1. Generic HTTP proxies
2. More AI providers (Gemini, Claude Opus)
3. Production deployment
4. Documentation

---

## 🎯 **Conclusion**

**YES** - Songbird can manage HTTP connections for other primals!

This evolution:
- ✅ Makes ALL primals 100% pure Rust (including transitive deps!)
- ✅ Perfects BiomeOS "Concentrated Gap" (1 gateway, not many)
- ✅ Centralizes credentials, rate limiting, caching
- ✅ Simplifies architecture (TRUE PRIMAL infant pattern)
- ✅ Reduces attack surface (security++)
- ✅ **Game changer for ecosystem purity!** 🦀🌱✨

**This is the TRUE PRIMAL way!**

---

**Created**: January 16, 2026  
**Author**: Songbird Team  
**Status**: Architecture designed, ready to implement  
**Impact**: 🏆 **ECOSYSTEM TRANSFORMATION**

