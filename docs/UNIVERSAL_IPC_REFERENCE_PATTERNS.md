# 🎯 Universal IPC - Quick Reference Patterns

**Date**: February 3, 2026  
**Source**: Songbird Reference Implementation  
**Audience**: Other primal teams (BearDog, Toadstool, NestGate, Squirrel)  
**Purpose**: Copy-paste patterns for implementing Universal IPC Standard v3

---

## Quick Start

### 1. Core Types (Copy to your primal)

```rust
//! Your primal's IPC types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Platform-agnostic endpoint (ALL variants available on ALL platforms)
#[derive(Debug, Clone)]
pub enum NativeEndpoint {
    /// Unix domain socket (Linux, macOS, BSD)
    UnixSocket(PathBuf),
    
    /// Abstract Unix socket (Android, Linux) - SELinux-safe!
    AbstractSocket(String),
    
    /// Windows named pipe
    NamedPipe(String),
    
    /// TCP localhost (universal fallback - always works!)
    TcpLocal(u16),
}

impl NativeEndpoint {
    /// Get display string for logging
    pub fn display(&self) -> String {
        match self {
            NativeEndpoint::UnixSocket(path) => format!("unix://{}", path.display()),
            NativeEndpoint::AbstractSocket(name) => format!("abstract://{}", name),
            NativeEndpoint::NamedPipe(name) => format!("pipe://{}", name),
            NativeEndpoint::TcpLocal(port) => format!("tcp://127.0.0.1:{}", port),
        }
    }
    
    /// Check if this is the native transport for current platform
    pub fn is_native(&self) -> bool {
        #[cfg(target_os = "android")]
        return matches!(self, NativeEndpoint::AbstractSocket(_));
        
        #[cfg(all(unix, not(target_os = "android")))]
        return matches!(self, NativeEndpoint::UnixSocket(_));
        
        #[cfg(windows)]
        return matches!(self, NativeEndpoint::NamedPipe(_));
        
        #[cfg(not(any(unix, windows)))]
        return matches!(self, NativeEndpoint::TcpLocal(_));
    }
}
```

---

### 2. Platform-Specific Implementations

#### Unix Sockets (Linux, macOS, BSD)

```rust
use tokio::net::{UnixListener, UnixStream};
use std::path::PathBuf;

pub struct UnixIPC;

impl UnixIPC {
    /// Create Unix socket endpoint (XDG-compliant)
    pub async fn create_endpoint(primal_name: &str) -> Result<NativeEndpoint> {
        // XDG-compliant path
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/tmp"));
        
        let socket_dir = PathBuf::from(runtime_dir).join("biomeos");
        std::fs::create_dir_all(&socket_dir)?;
        
        let socket_path = socket_dir.join(format!("{}.sock", primal_name));
        
        // Remove stale socket if exists
        let _ = std::fs::remove_file(&socket_path);
        
        Ok(NativeEndpoint::UnixSocket(socket_path))
    }
    
    /// Listen on Unix socket
    pub async fn listen(endpoint: &NativeEndpoint) -> Result<UnixListener> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                let listener = UnixListener::bind(path)?;
                tracing::info!("📡 Listening on Unix socket: {}", path.display());
                Ok(listener)
            }
            _ => Err("UnixIPC requires UnixSocket endpoint"),
        }
    }
    
    /// Connect to Unix socket
    pub async fn connect(endpoint: &NativeEndpoint) -> Result<UnixStream> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                let stream = UnixStream::connect(path).await?;
                tracing::info!("🔌 Connected to Unix socket: {}", path.display());
                Ok(stream)
            }
            _ => Err("UnixIPC requires UnixSocket endpoint"),
        }
    }
}
```

---

#### Abstract Sockets (Android, Linux)

```rust
use tokio::net::{UnixListener, UnixStream};

pub struct AndroidIPC;

impl AndroidIPC {
    /// Create abstract socket endpoint (SELinux-safe!)
    pub async fn create_endpoint(primal_name: &str) -> Result<NativeEndpoint> {
        // Abstract socket naming: @biomeos_{primal}
        // The @ prefix is automatically converted to null byte by Rust
        let abstract_name = format!("@biomeos_{}", primal_name);
        
        Ok(NativeEndpoint::AbstractSocket(abstract_name))
    }
    
    /// Listen on abstract socket
    pub async fn listen(endpoint: &NativeEndpoint) -> Result<UnixListener> {
        match endpoint {
            NativeEndpoint::AbstractSocket(name) => {
                // Rust automatically converts @name to \0name for abstract sockets
                let listener = UnixListener::bind(name)?;
                tracing::info!("📡 Listening on abstract socket: {}", name);
                Ok(listener)
            }
            _ => Err("AndroidIPC requires AbstractSocket endpoint"),
        }
    }
    
    /// Connect to abstract socket
    pub async fn connect(endpoint: &NativeEndpoint) -> Result<UnixStream> {
        match endpoint {
            NativeEndpoint::AbstractSocket(name) => {
                let stream = UnixStream::connect(name).await?;
                tracing::info!("🔌 Connected to abstract socket: {}", name);
                Ok(stream)
            }
            _ => Err("AndroidIPC requires AbstractSocket endpoint"),
        }
    }
}
```

**Why Abstract Sockets for Android?**
- ✅ No filesystem access (SELinux restrictions bypass)
- ✅ Automatic cleanup (no stale socket files)
- ✅ Same performance as filesystem Unix sockets
- ✅ Perfect for Pixel/GrapheneOS deployments

---

#### TCP Fallback (Universal)

```rust
use tokio::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, Ordering};

/// Port counter for automatic port assignment
static PORT_COUNTER: AtomicU16 = AtomicU16::new(50000);

pub struct TcpFallbackIPC;

impl TcpFallbackIPC {
    /// Create TCP localhost endpoint (always works!)
    pub async fn create_endpoint(primal_name: &str) -> Result<NativeEndpoint> {
        // Assign unique port
        let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        tracing::warn!("Using TCP fallback for '{}' - port {}", primal_name, port);
        
        Ok(NativeEndpoint::TcpLocal(port))
    }
    
    /// Listen on TCP localhost
    pub async fn listen(endpoint: &NativeEndpoint) -> Result<TcpListener> {
        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
                tracing::info!("📡 Listening on TCP localhost: 127.0.0.1:{}", port);
                Ok(listener)
            }
            _ => Err("TcpFallbackIPC requires TcpLocal endpoint"),
        }
    }
    
    /// Connect to TCP localhost
    pub async fn connect(endpoint: &NativeEndpoint) -> Result<TcpStream> {
        match endpoint {
            NativeEndpoint::TcpLocal(port) => {
                let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
                tracing::info!("🔌 Connected to TCP localhost: 127.0.0.1:{}", port);
                Ok(stream)
            }
            _ => Err("TcpFallbackIPC requires TcpLocal endpoint"),
        }
    }
}
```

**When to use TCP Fallback:**
- Platform lacks Unix sockets (rare)
- Windows (until named pipes implemented)
- Unknown/embedded platforms
- Testing/development (easy inspection)

---

### 3. Multi-Transport Strategy (CRITICAL!)

```rust
/// Try transports in priority order (native → fallback)
pub async fn create_endpoint_with_fallback(primal_name: &str) -> Result<NativeEndpoint> {
    // Platform-specific priority order
    #[cfg(target_os = "android")]
    let transports: Vec<Box<dyn Fn(&str) -> _>> = vec![
        Box::new(AndroidIPC::create_endpoint),  // Native (abstract sockets)
        Box::new(TcpFallbackIPC::create_endpoint),  // Fallback
    ];
    
    #[cfg(all(unix, not(target_os = "android")))]
    let transports: Vec<Box<dyn Fn(&str) -> _>> = vec![
        Box::new(UnixIPC::create_endpoint),  // Native (filesystem sockets)
        Box::new(AndroidIPC::create_endpoint),  // Alternative (abstract sockets)
        Box::new(TcpFallbackIPC::create_endpoint),  // Fallback
    ];
    
    #[cfg(not(unix))]
    let transports: Vec<Box<dyn Fn(&str) -> _>> = vec![
        Box::new(TcpFallbackIPC::create_endpoint),  // Fallback only
    ];
    
    // Try each transport in order
    let mut last_error = None;
    for create_fn in transports {
        match create_fn(primal_name).await {
            Ok(endpoint) => {
                tracing::info!("✅ Created endpoint: {}", endpoint.display());
                return Ok(endpoint);
            }
            Err(e) => {
                tracing::warn!("❌ Transport failed: {}", e);
                last_error = Some(e);
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| "No transports available".into()))
}
```

**Why Multi-Transport is Critical:**
- ✅ Pixel 8a failure: BearDog tried filesystem socket → SELinux blocked
- ✅ Solution: Abstract socket fallback → SUCCESS
- ✅ Universal deployment: Works on ANY platform

---

### 4. JSON-RPC 2.0 Protocol (For Interoperability)

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,  // Always "2.0"
    pub method: String,   // e.g., "crypto.encrypt"
    pub params: Value,    // Method parameters
    pub id: Value,        // Request ID (can be number or string)
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,  // Always "2.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,  // Success result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,  // Error (if any)
    pub id: Value,  // Request ID (for correlation)
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,  // Error code (-32000 to -32099)
    pub message: String,  // Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,  // Optional error data
}

/// Send JSON-RPC request over any stream
pub async fn send_jsonrpc_request<S>(
    stream: &mut S,
    method: &str,
    params: Value,
    id: Value,
) -> Result<JsonRpcResponse>
where
    S: AsyncWrite + AsyncRead + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    // Create request
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id,
    };
    
    // Serialize
    let request_json = serde_json::to_string(&request)?;
    let request_bytes = format!("{}\n", request_json).into_bytes();
    
    // Send
    stream.write_all(&request_bytes).await?;
    
    // Read response (assuming newline-delimited)
    let mut response_buf = Vec::new();
    loop {
        let byte = stream.read_u8().await?;
        if byte == b'\n' {
            break;
        }
        response_buf.push(byte);
    }
    
    // Deserialize
    let response: JsonRpcResponse = serde_json::from_slice(&response_buf)?;
    
    Ok(response)
}
```

**Usage Example:**
```rust
// Connect to any primal
let mut stream = connect_to_primal("beardog").await?;

// Call method via JSON-RPC 2.0
let response = send_jsonrpc_request(
    &mut stream,
    "crypto.encrypt",
    json!({"data": "hello", "family_id": "pixel_tower"}),
    json!(1),
).await?;

// Check response
if let Some(result) = response.result {
    println!("Encrypted: {}", result);
} else if let Some(error) = response.error {
    println!("Error {}: {}", error.code, error.message);
}
```

---

### 5. Discovery File Pattern (For TCP Fallback)

```rust
use std::path::PathBuf;

/// Write discovery file for TCP endpoint
pub async fn write_discovery_file(primal_name: &str, port: u16) -> Result<()> {
    // XDG-compliant discovery directory
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    
    let discovery_file = PathBuf::from(runtime_dir)
        .join(format!("{}-ipc-port", primal_name));
    
    // Write: tcp:127.0.0.1:{port}
    let content = format!("tcp:127.0.0.1:{}", port);
    tokio::fs::write(&discovery_file, content.as_bytes()).await?;
    
    tracing::info!("📝 Wrote discovery file: {} → {}", 
        discovery_file.display(), content);
    
    Ok(())
}

/// Read discovery file to find primal endpoint
pub async fn read_discovery_file(primal_name: &str) -> Result<NativeEndpoint> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    
    let discovery_file = PathBuf::from(runtime_dir)
        .join(format!("{}-ipc-port", primal_name));
    
    // Read file
    let content = tokio::fs::read_to_string(&discovery_file).await?;
    let content = content.trim();
    
    // Parse: tcp:127.0.0.1:{port}
    if let Some(port_str) = content.strip_prefix("tcp:127.0.0.1:") {
        let port: u16 = port_str.parse()?;
        tracing::info!("📖 Found {} via discovery file: port {}", primal_name, port);
        Ok(NativeEndpoint::TcpLocal(port))
    } else {
        Err("Invalid discovery file format".into())
    }
}
```

**Why Discovery Files?**
- ✅ Enables TCP fallback discovery (no Unix socket scan)
- ✅ XDG-compliant (`$XDG_RUNTIME_DIR/{primal}-ipc-port`)
- ✅ Works on Android, Windows, embedded
- ✅ Already implemented in BearDog and Songbird

---

### 6. Complete Server Example

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create endpoint with fallback
    let endpoint = create_endpoint_with_fallback("myprimal").await?;
    
    // 2. Write discovery file (if TCP)
    if let NativeEndpoint::TcpLocal(port) = endpoint {
        write_discovery_file("myprimal", port).await?;
    }
    
    // 3. Listen
    let listener = match &endpoint {
        NativeEndpoint::UnixSocket(_) => {
            UnixIPC::listen(&endpoint).await?
        }
        NativeEndpoint::AbstractSocket(_) => {
            AndroidIPC::listen(&endpoint).await?
        }
        NativeEndpoint::TcpLocal(_) => {
            TcpFallbackIPC::listen(&endpoint).await?
        }
        _ => panic!("Unsupported endpoint type"),
    };
    
    println!("🚀 Listening on: {}", endpoint.display());
    
    // 4. Accept connections
    loop {
        let (mut stream, _addr) = listener.accept().await?;
        
        tokio::spawn(async move {
            // Read JSON-RPC request
            let mut buf = Vec::new();
            loop {
                let byte = stream.read_u8().await?;
                if byte == b'\n' {
                    break;
                }
                buf.push(byte);
            }
            
            let request: JsonRpcRequest = serde_json::from_slice(&buf)?;
            
            // Handle request
            let result = handle_method(&request.method, &request.params).await?;
            
            // Send JSON-RPC response
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request.id,
            };
            
            let response_json = serde_json::to_string(&response)?;
            stream.write_all(format!("{}\n", response_json).as_bytes()).await?;
            
            Ok::<(), Box<dyn std::error::Error>>(())
        });
    }
}

async fn handle_method(method: &str, params: &Value) -> Result<Value> {
    match method {
        "health" => Ok(json!({"status": "healthy"})),
        "echo" => Ok(params.clone()),
        _ => Err(format!("Unknown method: {}", method).into()),
    }
}
```

---

### 7. Complete Client Example

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Try to discover endpoint
    let endpoint = discover_primal_endpoint("beardog").await?;
    
    // 2. Connect
    let mut stream = match &endpoint {
        NativeEndpoint::UnixSocket(_) => {
            UnixIPC::connect(&endpoint).await?
        }
        NativeEndpoint::AbstractSocket(_) => {
            AndroidIPC::connect(&endpoint).await?
        }
        NativeEndpoint::TcpLocal(_) => {
            TcpFallbackIPC::connect(&endpoint).await?
        }
        _ => panic!("Unsupported endpoint type"),
    };
    
    println!("✅ Connected to BearDog: {}", endpoint.display());
    
    // 3. Send JSON-RPC request
    let response = send_jsonrpc_request(
        &mut stream,
        "crypto.encrypt",
        json!({"data": "hello", "family_id": "pixel_tower"}),
        json!(1),
    ).await?;
    
    // 4. Handle response
    if let Some(result) = response.result {
        println!("🎉 Result: {}", result);
    } else if let Some(error) = response.error {
        eprintln!("❌ Error {}: {}", error.code, error.message);
    }
    
    Ok(())
}

/// Discover primal endpoint (try multiple strategies)
async fn discover_primal_endpoint(primal_name: &str) -> Result<NativeEndpoint> {
    // Strategy 1: Try Unix socket (Linux, macOS)
    #[cfg(unix)]
    {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        let unix_path = PathBuf::from(runtime_dir)
            .join("biomeos")
            .join(format!("{}.sock", primal_name));
        
        if unix_path.exists() {
            return Ok(NativeEndpoint::UnixSocket(unix_path));
        }
    }
    
    // Strategy 2: Try abstract socket (Android, Linux)
    #[cfg(target_os = "android")]
    {
        // Can't check if abstract socket exists without connecting
        // Try connecting immediately
        let abstract_name = format!("@biomeos_{}", primal_name);
        return Ok(NativeEndpoint::AbstractSocket(abstract_name));
    }
    
    // Strategy 3: Try discovery file (TCP fallback)
    if let Ok(endpoint) = read_discovery_file(primal_name).await {
        return Ok(endpoint);
    }
    
    Err(format!("Could not discover primal: {}", primal_name).into())
}
```

---

## Testing Your Implementation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_endpoint_creation() {
        let endpoint = create_endpoint_with_fallback("test").await.unwrap();
        
        // Should create valid endpoint
        assert!(!endpoint.display().is_empty());
    }
    
    #[tokio::test]
    async fn test_roundtrip() {
        let endpoint = create_endpoint_with_fallback("test").await.unwrap();
        
        // Start server
        let listener = /* ... */;
        
        // Connect client
        let stream = /* ... */;
        
        // Send/receive
        let response = send_jsonrpc_request(
            &mut stream,
            "echo",
            json!({"msg": "hello"}),
            json!(1),
        ).await.unwrap();
        
        assert_eq!(response.result.unwrap(), json!({"msg": "hello"}));
    }
}
```

### Integration Tests (Cross-Primal)

```bash
# Terminal 1: Start BearDog
./beardog server --listen  # Ensures TCP fallback available

# Terminal 2: Connect from your primal
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/1000/biomeos/beardog.sock

# Should get:
# {"jsonrpc":"2.0","result":{"status":"healthy"},"id":1}
```

---

## Common Pitfalls (Avoid These!)

### ❌ DON'T: Hardcode transport

```rust
// BAD: Only works on Unix!
let listener = UnixListener::bind("/tmp/myprimal.sock")?;
```

**FIX**: Use multi-transport with fallback (see Pattern 3)

---

### ❌ DON'T: Hardcode paths

```rust
// BAD: Breaks on non-standard systems
let socket_path = "/tmp/myprimal.sock";
```

**FIX**: Use XDG-compliant paths (see Pattern 2)

---

### ❌ DON'T: Ignore platform differences

```rust
// BAD: Assumes filesystem sockets work (fails on Android!)
#[cfg(unix)]
let path = format!("/tmp/{}.sock", primal_name);
```

**FIX**: Use abstract sockets on Android (see Pattern 2)

---

### ❌ DON'T: Forget discovery files

```rust
// BAD: TCP endpoint with no discovery mechanism
let listener = TcpListener::bind("127.0.0.1:50000")?;
// How will other primals find you?
```

**FIX**: Write discovery file (see Pattern 5)

---

### ❌ DON'T: Use custom protocol

```rust
// BAD: Custom protocol breaks interoperability
stream.write_all(b"MYPRIMAL:encrypt:hello")?;
```

**FIX**: Use JSON-RPC 2.0 (see Pattern 4)

---

## Quick Checklist

Before claiming Universal IPC compliance, verify:

- [ ] All transport types available on all platforms (runtime selection)
- [ ] Multi-transport fallback (native → alternative → TCP)
- [ ] XDG-compliant paths (`$XDG_RUNTIME_DIR/biomeos/`)
- [ ] Abstract socket support (Android/SELinux-safe)
- [ ] TCP fallback with discovery files
- [ ] JSON-RPC 2.0 protocol (interoperability)
- [ ] Comprehensive error handling
- [ ] Unit tests (endpoint creation, roundtrip)
- [ ] Integration tests (cross-primal)
- [ ] Platform tests (Linux, Android, etc.)

---

## Need Help?

1. **Reference Implementation**: `phase1/songbird/crates/songbird-universal-ipc/`
2. **Full Audit**: `UNIVERSAL_IPC_AUDIT_FEB_03_2026.md`
3. **Standard Spec**: `wateringHole/UNIVERSAL_IPC_STANDARD_V3.md`
4. **Contact**: Songbird team or wateringHole Core Team

---

**Created**: February 3, 2026  
**Version**: 1.0  
**Status**: Production-Ready Reference Patterns

---

🦀🌍✨ **Copy, Adapt, Deploy!** ✨🌍🦀
