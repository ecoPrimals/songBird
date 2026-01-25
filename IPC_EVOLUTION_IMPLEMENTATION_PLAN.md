# 🎯 Songbird IPC Evolution - Implementation Plan
**Date**: January 25, 2026  
**Priority**: HIGH - Blocks biomeOS HTTPS Integration  
**Status**: ⚠️ **ARCHITECTURAL GAP CONFIRMED**  
**Estimated Effort**: 7-9 hours (1-2 days)

---

## 📋 Acknowledgment

**The biomeOS team is absolutely correct!**

While Songbird has achieved the technical breakthrough of Pure Rust TLS 1.3 via Tower Atomic, we have **not exposed this capability via IPC** as required by the Primal IPC Protocol standard.

### Current Situation
- ✅ **Library Level**: `songbird-http-client` crate works perfectly
- ✅ **Example Code**: `cargo run --example test_https` succeeds
- ❌ **IPC Protocol**: No JSON-RPC Unix socket interface
- ❌ **Ecosystem Access**: biomeOS cannot orchestrate HTTPS requests

This is a **valid architectural gap** that must be resolved for TRUE PRIMAL compliance.

---

## 🎯 Implementation Plan

### Phase 1: IPC Handler Module (2-3 hours)

Create `crates/songbird-ipc/src/handlers/http_handler.rs`:

```rust
//! HTTP/HTTPS IPC Handler
//!
//! Exposes songbird-http-client functionality via JSON-RPC 2.0 over Unix sockets.
//! This is the bridge between the IPC protocol and our Pure Rust TLS implementation.

use crate::error::IpcError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use songbird_http_client::{BearDogClient, SongbirdHttpClient};
use std::time::Instant;
use tracing::{debug, error, info};

/// HTTP request parameters from JSON-RPC
#[derive(Debug, Deserialize)]
pub struct HttpRequestParams {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_timeout() -> u64 {
    30000 // 30 seconds
}

/// HTTP response for JSON-RPC
#[derive(Debug, Serialize)]
pub struct HttpResponseResult {
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub elapsed_ms: u128,
}

/// Handle http.request JSON-RPC method
pub async fn handle_http_request(
    params: HttpRequestParams,
    beardog_socket: &str,
) -> Result<HttpResponseResult, IpcError> {
    let start = Instant::now();
    
    info!("IPC http.request: {} {}", params.method, params.url);
    debug!("Headers: {:?}", params.headers);
    
    // Connect to BearDog crypto provider
    let beardog = BearDogClient::new_direct(beardog_socket);
    
    // Create HTTPS client with Tower Atomic pattern
    let client = SongbirdHttpClient::with_crypto_provider(beardog);
    
    // Make the request
    let response = match params.method.to_uppercase().as_str() {
        "GET" => client.get(&params.url).await,
        "POST" => {
            let body = params.body.as_deref().unwrap_or("");
            client.post(&params.url, body).await
        }
        "PUT" => {
            let body = params.body.as_deref().unwrap_or("");
            client.put(&params.url, body).await
        }
        "DELETE" => client.delete(&params.url).await,
        method => {
            error!("Unsupported HTTP method: {}", method);
            return Err(IpcError::invalid_params(format!(
                "Unsupported HTTP method: {}",
                method
            )));
        }
    }?;
    
    let elapsed = start.elapsed();
    
    info!(
        "IPC http.request completed: {} {} in {}ms",
        response.status,
        params.url,
        elapsed.as_millis()
    );
    
    Ok(HttpResponseResult {
        status_code: response.status,
        headers: response.headers,
        body: String::from_utf8_lossy(&response.body).to_string(),
        elapsed_ms: elapsed.as_millis(),
    })
}

/// Handle http.get convenience method
pub async fn handle_http_get(
    url: &str,
    beardog_socket: &str,
) -> Result<HttpResponseResult, IpcError> {
    let params = HttpRequestParams {
        url: url.to_string(),
        method: "GET".to_string(),
        headers: Default::default(),
        body: None,
        timeout_ms: 30000,
    };
    
    handle_http_request(params, beardog_socket).await
}

/// Handle http.post convenience method
pub async fn handle_http_post(
    url: &str,
    body: &str,
    content_type: Option<&str>,
    beardog_socket: &str,
) -> Result<HttpResponseResult, IpcError> {
    let mut headers = std::collections::HashMap::new();
    if let Some(ct) = content_type {
        headers.insert("Content-Type".to_string(), ct.to_string());
    }
    
    let params = HttpRequestParams {
        url: url.to_string(),
        method: "POST".to_string(),
        headers,
        body: Some(body.to_string()),
        timeout_ms: 30000,
    };
    
    handle_http_request(params, beardog_socket).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_request_params_deserialize() {
        let json = r#"{
            "url": "https://example.com",
            "method": "GET",
            "headers": {"User-Agent": "Test"},
            "timeout_ms": 5000
        }"#;
        
        let params: HttpRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.method, "GET");
        assert_eq!(params.timeout_ms, 5000);
    }
}
```

### Phase 2: JSON-RPC Router (1-2 hours)

Update `crates/songbird-ipc/src/rpc_server.rs`:

```rust
//! JSON-RPC 2.0 Server for IPC
//!
//! Handles routing of JSON-RPC requests to appropriate handlers.

use crate::handlers::http_handler;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info, warn};

/// Handle a single JSON-RPC request
pub async fn handle_rpc_request(
    request: Value,
    beardog_socket: &str,
) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let params = &request["params"];
    let id = request["id"].clone();
    
    debug!("RPC method: {}, id: {:?}", method, id);
    
    match method {
        "http.request" => {
            match serde_json::from_value::<http_handler::HttpRequestParams>(params.clone()) {
                Ok(http_params) => {
                    match http_handler::handle_http_request(http_params, beardog_socket).await {
                        Ok(result) => json!({
                            "jsonrpc": "2.0",
                            "result": result,
                            "id": id
                        }),
                        Err(e) => json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32001,
                                "message": e.to_string()
                            },
                            "id": id
                        })
                    }
                }
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": format!("Invalid params: {}", e)
                    },
                    "id": id
                })
            }
        }
        
        "http.get" => {
            let url = params["url"].as_str().unwrap_or("");
            match http_handler::handle_http_get(url, beardog_socket).await {
                Ok(result) => json!({
                    "jsonrpc": "2.0",
                    "result": result,
                    "id": id
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32001,
                        "message": e.to_string()
                    },
                    "id": id
                })
            }
        }
        
        "http.post" => {
            let url = params["url"].as_str().unwrap_or("");
            let body = params["body"].as_str().unwrap_or("");
            let content_type = params["content_type"].as_str();
            
            match http_handler::handle_http_post(url, body, content_type, beardog_socket).await {
                Ok(result) => json!({
                    "jsonrpc": "2.0",
                    "result": result,
                    "id": id
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32001,
                        "message": e.to_string()
                    },
                    "id": id
                })
            }
        }
        
        _ => json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            },
            "id": id
        })
    }
}

/// Handle a single IPC connection
pub async fn handle_connection(
    mut stream: UnixStream,
    beardog_socket: String,
) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    loop {
        line.clear();
        
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let request: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Invalid JSON-RPC request: {}", e);
                        continue;
                    }
                };
                
                let response = handle_rpc_request(request, &beardog_socket).await;
                let response_str = serde_json::to_string(&response).unwrap();
                
                if let Err(e) = writer.write_all(response_str.as_bytes()).await {
                    error!("Failed to write response: {}", e);
                    break;
                }
                
                if let Err(e) = writer.write_all(b"\n").await {
                    error!("Failed to write newline: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Failed to read from socket: {}", e);
                break;
            }
        }
    }
    
    debug!("Connection closed");
}
```

### Phase 3: Update CLI (1 hour)

Update `src/bin/songbird/main.rs`:

```rust
#[derive(Parser)]
#[command(name = "songbird")]
#[command(about = "Songbird - Network Orchestration & Discovery Primal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Songbird server
    Server {
        /// HTTP port for federation/discovery
        #[arg(long, default_value = "8080")]
        port: u16,
        
        /// Unix socket for IPC (JSON-RPC)
        /// Example: /tmp/songbird-nat0.sock
        #[arg(long)]
        socket: Option<String>,
        
        /// Family ID for multi-instance deployments
        #[arg(long, default_value = "nat0")]
        family_id: String,
        
        /// BearDog socket path
        #[arg(long, env = "BEARDOG_SOCKET")]
        beardog_socket: Option<String>,
    },
    // ... other commands
}

async fn run_server(
    port: u16,
    socket: Option<String>,
    family_id: String,
    beardog_socket: Option<String>,
) -> Result<()> {
    // Determine BearDog socket
    let beardog = beardog_socket.unwrap_or_else(|| {
        format!("/tmp/beardog-{}.sock", family_id)
    });
    
    // Start HTTP server for federation (existing)
    let http_handle = tokio::spawn(async move {
        info!("Starting HTTP server on port {}", port);
        start_http_server(port).await
    });
    
    // Start IPC server if socket provided (NEW)
    if let Some(socket_path) = socket {
        info!("Starting IPC server on {}", socket_path);
        
        // Remove old socket if exists
        let _ = std::fs::remove_file(&socket_path);
        
        let listener = tokio::net::UnixListener::bind(&socket_path)?;
        info!("✅ IPC listening on {} (JSON-RPC 2.0)", socket_path);
        info!("✅ Using BearDog at {}", beardog);
        
        let ipc_handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let beardog_clone = beardog.clone();
                        tokio::spawn(async move {
                            songbird_ipc::rpc_server::handle_connection(
                                stream,
                                beardog_clone
                            ).await;
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });
        
        // Run both servers
        tokio::select! {
            res = http_handle => res?,
            res = ipc_handle => res?,
        }
    } else {
        // Run HTTP server only
        http_handle.await?
    }
    
    Ok(())
}
```

### Phase 4: Testing (2 hours)

Create `tests/ipc_http_integration.rs`:

```rust
//! Integration tests for HTTP over IPC

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[tokio::test]
async fn test_http_get_via_ipc() {
    // Assumes Songbird is running: songbird server --socket /tmp/test-songbird.sock
    
    let socket_path = "/tmp/test-songbird.sock";
    let mut stream = UnixStream::connect(socket_path).await.unwrap();
    
    // Send JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "http.get",
        "params": {
            "url": "https://cloudflare.com"
        },
        "id": 1
    });
    
    let request_str = serde_json::to_string(&request).unwrap();
    stream.write_all(request_str.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    
    // Read response
    let (reader, _) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await.unwrap();
    
    let response: serde_json::Value = serde_json::from_str(&response_line).unwrap();
    
    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["status_code"].as_u64().unwrap() == 200);
}
```

### Phase 5: Documentation (1 hour)

Create `docs/IPC_HTTPS_GUIDE.md` documenting the new capability.

---

## 📊 Timeline

| Phase | Task | Effort | Status |
|-------|------|--------|--------|
| 1 | IPC Handler Module | 2-3 hours | ⏳ TODO |
| 2 | JSON-RPC Router | 1-2 hours | ⏳ TODO |
| 3 | CLI Updates | 1 hour | ⏳ TODO |
| 4 | Integration Testing | 2 hours | ⏳ TODO |
| 5 | Documentation | 1 hour | ⏳ TODO |
| **Total** | **7-9 hours** | **1-2 days** | ⏳ TODO |

---

## ✅ Acceptance Criteria

### Songbird Side
- [ ] `songbird server --socket /tmp/songbird-nat0.sock` starts IPC listener
- [ ] JSON-RPC `http.request` method works via Unix socket
- [ ] Returns proper response: `status_code`, `headers`, `body`, `elapsed_ms`
- [ ] Connects to BearDog via socket path
- [ ] Graceful error handling with JSON-RPC error codes
- [ ] Concurrent request handling (multiple connections)

### biomeOS Side
- [ ] Neural API can spawn Songbird with `--socket` flag
- [ ] Neural API can route `http.request` to Songbird socket
- [ ] HTTPS request returns HTTP 200 OK
- [ ] Tower Atomic pattern works via IPC

---

## 🎯 Priority & Impact

**Priority**: **HIGH**  
**Blocker**: biomeOS HTTPS via Neural API  
**Impact**: Without this, the Pure Rust TLS achievement cannot be utilized by the ecosystem

### Why This Matters
1. **Standards Compliance**: Required by Primal IPC Protocol
2. **Ecosystem Access**: biomeOS needs IPC to orchestrate HTTPS
3. **Complete Story**: We achieved Pure Rust TLS - now make it accessible
4. **TRUE PRIMAL Status**: UniBin + ecoBin + IPC = Complete

---

## 📝 Implementation Notes

### Architecture Decisions
1. **Keep Library Separate**: `songbird-http-client` remains a reusable library
2. **IPC is a Bridge**: New `songbird-ipc` handlers bridge library to IPC
3. **No Breaking Changes**: Existing functionality unchanged
4. **Optional Mode**: `--socket` is optional (backward compatible)

### Error Handling
- Use JSON-RPC 2.0 error codes:
  - `-32700`: Parse error
  - `-32600`: Invalid Request
  - `-32601`: Method not found
  - `-32602`: Invalid params
  - `-32001`: TLS/HTTP errors (custom)

### Security Considerations
- Unix socket permissions (0600 recommended)
- Input validation on all JSON-RPC params
- Timeout enforcement
- Resource limits (concurrent connections)

---

## 🔗 Related Documents

- [Primal IPC Protocol](../../../wateringHole/PRIMAL_IPC_PROTOCOL.md) - The standard we must comply with
- [Tower Atomic Integration Guide](../../../wateringHole/SONGBIRD_TLS_TOWER_ATOMIC_INTEGRATION_GUIDE.md) - How Tower Atomic works
- [Songbird HTTP Client Docs](../crates/songbird-http-client/README.md) - Library we're exposing

---

## ✅ Validation

Once complete, test with:

```bash
# Terminal 1: Start Songbird with IPC
songbird server --socket /tmp/songbird-nat0.sock

# Terminal 2: Test via netcat
echo '{"jsonrpc":"2.0","method":"http.get","params":{"url":"https://cloudflare.com"},"id":1}' | \
  nc -U /tmp/songbird-nat0.sock

# Expected: JSON response with status_code: 200
```

---

## 🙏 Thank You biomeOS Team!

This is **excellent architectural feedback**. The gap identification is:
- ✅ **Accurate**: We indeed lack IPC exposure
- ✅ **Critical**: Blocks ecosystem integration
- ✅ **Actionable**: Clear implementation path
- ✅ **Well-Documented**: Thorough handoff document

**We'll implement this ASAP!** Estimated completion: 1-2 days

---

**Status**: ⚠️ **ACKNOWLEDGED - IMPLEMENTATION PLANNED**  
**Owner**: Songbird Team  
**ETA**: 1-2 days (7-9 hours development)  
**Blocker For**: biomeOS HTTPS via Neural API

---

*Response created: January 25, 2026*  
*Owner: Songbird Team*  
*In Response To: biomeOS IPC Evolution Request*

