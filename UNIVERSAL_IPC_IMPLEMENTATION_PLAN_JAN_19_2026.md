# 🌍 Universal IPC Implementation Plan - Songbird

**Date**: January 19, 2026  
**Status**: 🚀 **READY TO IMPLEMENT**  
**Priority**: HIGH (Foundation for True Universality)

---

## 🎯 VISION

**Eliminate ALL platform-specific code from application primals** by centralizing IPC abstraction in Songbird.

**Before**:
```rust
// Every primal has this mess:
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(windows)]
use tokio::net::windows::named_pipe;
```

**After**:
```rust
// Clean, universal API:
let stream = songbird::ipc::connect("/primal/beardog").await?;
// Works EVERYWHERE!
```

---

## 📋 IMPLEMENTATION PHASES

### **Phase 1: Foundation** (Week 1) ⏱️ 15-20 hours

#### **Deliverable**: `songbird-universal-ipc` crate with Unix support

**Tasks**:
- [x] Create crate structure
- [ ] Define public API (`UniversalIPC`, `VirtualEndpoint`)
- [ ] Implement `PlatformIPC` trait
- [ ] Implement Unix backend (Linux, macOS, BSD)
- [ ] Implement `AsyncStream` trait wrapper
- [ ] Add in-memory service registry
- [ ] Write unit tests (Unix only)
- [ ] Integration tests (basic)

**Success Criteria**:
- ✅ `songbird::ipc::register("primal")` works on Unix
- ✅ `songbird::ipc::connect("/primal/name")` works on Unix
- ✅ Tests pass on Linux
- ✅ Zero unsafe code
- ✅ Clippy pedantic clean

---

### **Phase 2: Windows Support** (Week 2) ⏱️ 12-15 hours

#### **Deliverable**: Full Windows named pipe support

**Tasks**:
- [ ] Implement Windows `PlatformIPC`
- [ ] Add named pipe creation/connection
- [ ] Handle Windows-specific path formats
- [ ] Async named pipe wrapper
- [ ] Windows-specific tests
- [ ] Cross-platform integration tests

**Success Criteria**:
- ✅ Works on Windows 10/11
- ✅ Named pipes created correctly
- ✅ Connection handling robust
- ✅ Tests pass on Windows

---

### **Phase 3: Tower Atomic Integration** (Week 3) ⏱️ 10-12 hours

#### **Deliverable**: Tower Atomic using universal IPC

**Tasks**:
- [ ] Audit Tower Atomic IPC usage
- [ ] Replace Unix socket calls with `universal-ipc`
- [ ] Remove `#[cfg(unix)]` / `#[cfg(windows)]`
- [ ] Update tests
- [ ] Verify BearDog integration still works
- [ ] Performance benchmarks

**Success Criteria**:
- ✅ Tower Atomic works on ALL platforms
- ✅ Zero platform-specific code in Tower Atomic
- ✅ BearDog RPC still works
- ✅ No performance regression

---

### **Phase 4: NestGate Integration** (Week 4) ⏱️ 8-10 hours

#### **Deliverable**: Persistent service registry via NestGate

**Tasks**:
- [ ] Define `ServiceMetadata` struct
- [ ] Add NestGate client to `universal-ipc`
- [ ] Implement persistent storage on registration
- [ ] Add capability-based discovery
- [ ] Handle registry updates (heartbeat)
- [ ] Handle service removal (cleanup)

**Success Criteria**:
- ✅ Services persist across restarts
- ✅ Capability-based discovery works
- ✅ Registry survives crashes
- ✅ Clean shutdown handling

---

### **Phase 5: Advanced Features** (Week 5-6) ⏱️ 10-15 hours

#### **Deliverable**: Production-ready universal IPC

**Tasks**:
- [ ] Add health checks (detect dead connections)
- [ ] Implement reconnection logic
- [ ] Add metrics (connection count, latency)
- [ ] Load balancing (multiple instances)
- [ ] Security (authentication, encryption)
- [ ] Comprehensive error handling
- [ ] Production logging

**Success Criteria**:
- ✅ Robust error handling
- ✅ Auto-reconnect works
- ✅ Metrics observable
- ✅ Production-ready quality

---

### **Phase 6: Ecosystem Migration** (Month 2-3) ⏱️ 20-30 hours

#### **Deliverable**: All primals using universal IPC

**Tasks**:
- [ ] Migration guide for primal developers
- [ ] Update all songbird-orchestrator IPC calls
- [ ] Create examples for common patterns
- [ ] Update documentation
- [ ] Deprecation warnings for old API
- [ ] Migration tooling (if needed)

**Success Criteria**:
- ✅ All primals migrated
- ✅ Zero platform-specific IPC code
- ✅ Documentation complete
- ✅ Examples working

---

## 🏗️ ARCHITECTURE DETAILS

### **Crate Structure**

```
crates/songbird-universal-ipc/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Public API
│   ├── endpoint.rs         # VirtualEndpoint, NativeEndpoint
│   ├── registry.rs         # In-memory service registry
│   ├── stream.rs           # AsyncStream trait + wrappers
│   ├── error.rs            # IpcError types
│   ├── platform/
│   │   ├── mod.rs          # PlatformIPC trait
│   │   ├── unix.rs         # Unix socket implementation
│   │   ├── windows.rs      # Named pipe implementation
│   │   └── fallback.rs     # TCP localhost (emergency)
│   └── nestgate/
│       ├── mod.rs          # NestGate integration
│       └── client.rs       # NestGate client
├── tests/
│   ├── unix_tests.rs       # Unix-specific tests
│   ├── windows_tests.rs    # Windows-specific tests
│   └── integration.rs      # Cross-platform integration
└── examples/
    ├── simple_server.rs
    ├── simple_client.rs
    └── discovery.rs
```

---

### **Public API Design**

```rust
// Public API (platform-agnostic!)
pub mod ipc {
    /// Initialize universal IPC (auto-detects platform)
    pub fn init() -> Result<()>;
    
    /// Register this primal (returns virtual endpoint)
    pub async fn register(name: &str) -> Result<VirtualEndpoint>;
    
    /// Listen on virtual endpoint
    pub async fn listen(endpoint: VirtualEndpoint) -> Result<Listener>;
    
    /// Connect to virtual endpoint
    pub async fn connect(path: &str) -> Result<Stream>;
    
    /// Get global IPC instance
    pub fn global() -> &'static UniversalIPC;
}

/// Virtual endpoint (always Unix-style path)
pub struct VirtualEndpoint {
    pub path: String,  // e.g., "/primal/beardog"
}

/// Unified stream interface
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}

/// Listener for incoming connections
pub struct Listener {
    // Platform-agnostic listener
}

impl Listener {
    pub async fn accept(&mut self) -> Result<Stream>;
}

/// Universal stream (wraps platform-specific)
pub struct Stream {
    inner: Box<dyn AsyncStream>,
}

impl AsyncRead for Stream { ... }
impl AsyncWrite for Stream { ... }
```

---

### **Platform Abstraction**

```rust
// Internal trait (not exposed)
trait PlatformIPC: Send + Sync {
    /// Create native endpoint for primal
    async fn create_endpoint(&self, name: &str) -> Result<NativeEndpoint>;
    
    /// Create listener on native endpoint
    async fn listen(&self, endpoint: &NativeEndpoint) -> Result<Box<dyn PlatformListener>>;
    
    /// Connect to native endpoint
    async fn connect(&self, endpoint: &NativeEndpoint) -> Result<Box<dyn AsyncStream>>;
    
    /// Cleanup endpoint
    async fn cleanup(&self, endpoint: &NativeEndpoint) -> Result<()>;
}

// Platform-specific endpoints
enum NativeEndpoint {
    #[cfg(unix)]
    UnixSocket(PathBuf),
    
    #[cfg(windows)]
    NamedPipe(String),
    
    TcpLocal(u16),  // Fallback
}
```

---

### **Service Registry**

```rust
/// In-memory service registry (with NestGate backing)
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEntry>,
    nestgate: Option<NestGateClient>,
}

struct ServiceEntry {
    virtual_endpoint: VirtualEndpoint,
    native_endpoint: NativeEndpoint,
    capabilities: Vec<String>,
    registered_at: Instant,
    last_seen: Instant,
}

impl ServiceRegistry {
    /// Register service
    pub async fn register(
        &mut self,
        name: &str,
        native: NativeEndpoint,
        capabilities: Vec<String>,
    ) -> Result<VirtualEndpoint> {
        let virtual = VirtualEndpoint {
            path: format!("/primal/{}", name),
        };
        
        let entry = ServiceEntry {
            virtual_endpoint: virtual.clone(),
            native_endpoint: native,
            capabilities,
            registered_at: Instant::now(),
            last_seen: Instant::now(),
        };
        
        self.services.insert(name.to_string(), entry.clone());
        
        // Persist to NestGate (if available)
        if let Some(ng) = &self.nestgate {
            ng.store_service(entry.to_metadata()).await?;
        }
        
        Ok(virtual)
    }
    
    /// Resolve virtual path to native endpoint
    pub async fn resolve(&self, path: &str) -> Result<&NativeEndpoint> {
        // Parse "/primal/name" -> "name"
        let name = path.strip_prefix("/primal/")
            .ok_or_else(|| anyhow!("Invalid virtual path: {}", path))?;
        
        self.services.get(name)
            .map(|e| &e.native_endpoint)
            .ok_or_else(|| anyhow!("Service not found: {}", name))
    }
    
    /// Find services by capability
    pub async fn find_by_capability(&self, cap: &str) -> Vec<String> {
        self.services.values()
            .filter(|e| e.capabilities.contains(&cap.to_string()))
            .map(|e| e.virtual_endpoint.path.clone())
            .collect()
    }
}
```

---

## 🔧 IMPLEMENTATION DETAILS

### **Unix Implementation**

```rust
// crates/songbird-universal-ipc/src/platform/unix.rs

use tokio::net::{UnixListener, UnixStream};

pub struct UnixIPC;

impl PlatformIPC for UnixIPC {
    async fn create_endpoint(&self, name: &str) -> Result<NativeEndpoint> {
        // Use /tmp/primal-{name}.sock
        let path = PathBuf::from(format!("/tmp/primal-{}.sock", name));
        
        // Clean up old socket if exists
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        
        Ok(NativeEndpoint::UnixSocket(path))
    }
    
    async fn listen(&self, endpoint: &NativeEndpoint) -> Result<Box<dyn PlatformListener>> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                let listener = UnixListener::bind(path)?;
                Ok(Box::new(UnixListenerWrapper { inner: listener }))
            }
            _ => Err(anyhow!("Invalid endpoint for Unix platform"))
        }
    }
    
    async fn connect(&self, endpoint: &NativeEndpoint) -> Result<Box<dyn AsyncStream>> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                let stream = UnixStream::connect(path).await?;
                Ok(Box::new(stream))
            }
            _ => Err(anyhow!("Invalid endpoint for Unix platform"))
        }
    }
    
    async fn cleanup(&self, endpoint: &NativeEndpoint) -> Result<()> {
        match endpoint {
            NativeEndpoint::UnixSocket(path) => {
                if path.exists() {
                    tokio::fs::remove_file(path).await?;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }
}

// Wrapper to implement AsyncStream for UnixStream
impl AsyncStream for UnixStream {}
```

---

### **Windows Implementation**

```rust
// crates/songbird-universal-ipc/src/platform/windows.rs

use tokio::net::windows::named_pipe::{ServerOptions, ClientOptions};

pub struct WindowsIPC;

impl PlatformIPC for WindowsIPC {
    async fn create_endpoint(&self, name: &str) -> Result<NativeEndpoint> {
        // Use \\.\pipe\primal-{name}
        let pipe_name = format!(r"\\.\pipe\primal-{}", name);
        Ok(NativeEndpoint::NamedPipe(pipe_name))
    }
    
    async fn listen(&self, endpoint: &NativeEndpoint) -> Result<Box<dyn PlatformListener>> {
        match endpoint {
            NativeEndpoint::NamedPipe(name) => {
                let server = ServerOptions::new()
                    .first_pipe_instance(true)
                    .create(name)?;
                Ok(Box::new(NamedPipeListener { 
                    server,
                    pipe_name: name.clone(),
                }))
            }
            _ => Err(anyhow!("Invalid endpoint for Windows platform"))
        }
    }
    
    async fn connect(&self, endpoint: &NativeEndpoint) -> Result<Box<dyn AsyncStream>> {
        match endpoint {
            NativeEndpoint::NamedPipe(name) => {
                let client = ClientOptions::new()
                    .open(name)?;
                Ok(Box::new(NamedPipeStream { inner: client }))
            }
            _ => Err(anyhow!("Invalid endpoint for Windows platform"))
        }
    }
    
    async fn cleanup(&self, _endpoint: &NativeEndpoint) -> Result<()> {
        // Windows named pipes auto-cleanup
        Ok(())
    }
}

// Wrapper for named pipe
struct NamedPipeStream {
    inner: NamedPipeClient,
}

impl AsyncStream for NamedPipeStream {}
impl AsyncRead for NamedPipeStream { ... }
impl AsyncWrite for NamedPipeStream { ... }
```

---

## 🧪 TESTING STRATEGY

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[cfg(unix)]
    async fn test_unix_register_and_connect() {
        let ipc = UniversalIPC::new().unwrap();
        
        // Register
        let endpoint = ipc.register("test-primal").await.unwrap();
        assert_eq!(endpoint.path, "/primal/test-primal");
        
        // Listen
        let mut listener = ipc.listen(endpoint).await.unwrap();
        
        // Connect
        tokio::spawn(async move {
            let stream = ipc.connect("/primal/test-primal").await.unwrap();
            // Use stream...
        });
        
        // Accept
        let conn = listener.accept().await.unwrap();
        // Use conn...
    }
    
    #[tokio::test]
    #[cfg(windows)]
    async fn test_windows_named_pipes() {
        // Similar test for Windows
    }
}
```

### **Integration Tests**

```rust
// tests/integration.rs

#[tokio::test]
async fn test_cross_primal_communication() {
    // Simulate two primals communicating
    let ipc = UniversalIPC::init().unwrap();
    
    // Primal 1: Register and listen
    let endpoint = ipc.register("primal1").await.unwrap();
    let mut listener = ipc.listen(endpoint).await.unwrap();
    
    // Primal 2: Connect and send message
    tokio::spawn(async move {
        let mut stream = ipc.connect("/primal/primal1").await.unwrap();
        stream.write_all(b"hello").await.unwrap();
    });
    
    // Primal 1: Receive message
    let mut conn = listener.accept().await.unwrap();
    let mut buf = [0u8; 5];
    conn.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"hello");
}
```

---

## 📊 SUCCESS METRICS

### **Technical**
- ✅ Zero unsafe code
- ✅ Clippy pedantic clean
- ✅ 90%+ test coverage
- ✅ Works on Linux, macOS, Windows
- ✅ No performance regression vs raw sockets

### **API Quality**
- ✅ Simple API (< 10 public functions)
- ✅ Clear error messages
- ✅ Comprehensive documentation
- ✅ Multiple examples

### **Adoption**
- ✅ Tower Atomic migrated
- ✅ All songbird-orchestrator IPC migrated
- ✅ Migration guide published
- ✅ Zero platform-specific code in applications

---

## ⚠️ RISKS & MITIGATION

### **Risk 1: Windows Named Pipe Complexity**
**Mitigation**: Start simple, iterate based on testing

### **Risk 2: Performance Overhead**
**Mitigation**: Benchmark early, optimize hot paths

### **Risk 3: Breaking Changes**
**Mitigation**: 
- Keep old API deprecated (not removed)
- Provide migration period
- Automated migration tooling

### **Risk 4: Platform-Specific Bugs**
**Mitigation**:
- Extensive testing on all platforms
- CI/CD for Windows + Linux + macOS
- Beta period with early adopters

---

## 📅 TIMELINE

**Total Estimated Time**: 55-75 hours over 6-8 weeks

### **Weeks 1-2**: Foundation (Phase 1-2)
- Create crate, Unix + Windows implementations
- **Deliverable**: `songbird-universal-ipc` v0.1.0

### **Weeks 3-4**: Integration (Phase 3-4)
- Tower Atomic migration, NestGate integration
- **Deliverable**: Songbird v4.0.0 (Tower Atomic universal)

### **Weeks 5-6**: Advanced Features (Phase 5)
- Production hardening, metrics, health checks
- **Deliverable**: `songbird-universal-ipc` v1.0.0

### **Weeks 7-8**: Ecosystem Migration (Phase 6)
- Documentation, examples, migration guide
- **Deliverable**: Ecosystem-wide adoption

---

## 🎯 FIRST MILESTONE

### **Phase 1 Complete** (Week 1)

**Goal**: Unix-only universal IPC working

**Deliverables**:
1. ✅ `songbird-universal-ipc` crate created
2. ✅ Public API defined
3. ✅ Unix implementation complete
4. ✅ Tests passing on Linux
5. ✅ Example server/client working

**Demo**:
```rust
// example: Two primals talking (Unix only)
// Server:
let endpoint = ipc::register("demo").await?;
let mut listener = ipc::listen(endpoint).await?;

// Client:
let stream = ipc::connect("/primal/demo").await?;
// Works!
```

---

## 📚 REFERENCES

- **Architecture Doc**: `UNIVERSAL_IPC_ARCHITECTURE_HANDOFF_JAN_19_2026.md`
- **Tower Atomic**: `crates/songbird-orchestrator/src/ipc/server_pure_rust.rs`
- **Current IPC**: `crates/songbird-orchestrator/src/ipc/unix_socket.rs`
- **Standards**: `ecoPrimals/wateringHole/GENOMEBIN_ARCHITECTURE_STANDARD.md`

---

## ✅ READY TO START

**Status**: 🚀 **APPROVED FOR IMPLEMENTATION**

**Next Steps**:
1. Create `crates/songbird-universal-ipc/` structure
2. Define public API in `lib.rs`
3. Implement Unix backend
4. Write tests
5. Iterate!

---

**Document**: UNIVERSAL_IPC_IMPLEMENTATION_PLAN_JAN_19_2026.md  
**Date**: January 19, 2026  
**Status**: Implementation Ready  
**Estimated Time**: 55-75 hours over 6-8 weeks

🌍🦀✨ **Let's make Songbird truly universal!** ✨🦀🌍

