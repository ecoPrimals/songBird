# 🔄 BTSP Evolution Plan - January 16, 2026

**BTSP**: BearDog Tunnel Security Protocol  
**Joint Project**: BearDog + Songbird  
**Purpose**: Tower atomic enclave communication  
**Status**: Evolution required for "Concentrated Gap" strategy

---

## 🎯 **BiomeOS Guidance**

**HTTP Deprecation for Inter-Primal Communication**:
- ✅ HTTP is now EXTERNAL ONLY
- ✅ Songbird = ONLY primal with HTTP server
- ✅ All inter-primal communication via Unix sockets
- ✅ "Concentrated Gap" strategy completion

**Impact on BTSP**:
- Current: BearDog HTTP server → Songbird HTTP client
- Target: BearDog Unix socket server → Songbird Unix socket client
- Result: BearDog achieves 100% pure Rust!

---

## 📊 **Current Architecture**

### **BTSP Today** (HTTP-based):
```
BearDog (BTSP Server)
    ↓ HTTP API
Songbird (BTSP Client)
    ↓ External HTTP/TLS
External Systems
```

**Components**:
- BearDog: Provides BTSP HTTP API (tower-http, hyper)
- Songbird: Consumes BTSP via HTTP client (reqwest)
- Purpose: Tower atomic security tunnels
- Integration: Works with BirdSong P2P for tower broadcast

**Current Dependencies** (BearDog):
```toml
# HTTP/API
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "cors"] }
hyper = { version = "1.1", features = ["full"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

**Issue**: BearDog has HTTP server → Still has `ring` via `reqwest`/`rustls` (transitive)

---

## 🎯 **Target Architecture**

### **BTSP Evolution** (Unix Socket-based):
```
BearDog (BTSP via Unix Socket)
    ↓ Unix Socket IPC
Songbird (BTSP Client + HTTP Gateway)
    ↓ External HTTP/TLS ONLY
External Systems
```

**Evolution**:
1. ✅ BearDog: Unix socket BTSP server (NO HTTP)
2. ✅ Songbird: Unix socket BTSP client (connects to BearDog)
3. ✅ Songbird: HTTP server for external (ONLY primal with HTTP)
4. ✅ Result: Songbird = single controlled HTTP gateway to NUCLEUS

**Benefits**:
- ✅ BearDog: 100% pure Rust (no HTTP → no transitive `ring`)
- ✅ Concentrated Gap: HTTP only in Songbird
- ✅ NUCLEUS Access Control: Single HTTP entry point
- ✅ Tower Atomic Security: Maintained via Unix socket BTSP
- ✅ BirdSong P2P: Unchanged (UDP, already pure)

---

## 🔧 **Evolution Steps**

### **Phase 1: BearDog Side** (2-4 hours)

**Actions**:
1. ✅ Migrate BTSP server from HTTP to Unix socket
   - Remove `tower`, `tower-http`, `hyper`
   - Implement Unix socket JSON-RPC BTSP server
   - Use existing `beardog-ipc` patterns

2. ✅ Remove HTTP client (`reqwest`)
   - No longer needed for internal communication
   - All primal communication via Unix sockets

3. ✅ Update BTSP protocol
   - Keep protocol semantics (tunnels, security)
   - Change transport: HTTP → Unix socket JSON-RPC
   - Maintain BirdSong P2P integration

4. ✅ Update tests
   - Test Unix socket BTSP server
   - Integration tests with mock Songbird client

**Result**: BearDog 100% pure Rust (no HTTP → no transitive `ring`)!

---

### **Phase 2: Songbird Side** (2-4 hours)

**Actions**:
1. ✅ Migrate BTSP client from HTTP to Unix socket
   - Remove HTTP client for BearDog communication
   - Implement Unix socket JSON-RPC BTSP client
   - Connect to BearDog's Unix socket

2. ✅ Keep HTTP server for external
   - Songbird is the ONLY primal with HTTP server
   - Handles all external HTTP/TLS for ecosystem
   - BTSP tunnels exposed via HTTP endpoints (if needed)

3. ✅ Update BTSP integration
   - Internal: Unix socket to BearDog
   - External: HTTP API for tower access (if needed)
   - BirdSong P2P: Unchanged

4. ✅ Update tests
   - Test Unix socket BTSP client
   - Integration tests with BearDog
   - E2E tower atomic tests

**Result**: Songbird = single HTTP gateway, BTSP via Unix sockets!

---

## 📋 **Detailed Implementation**

### **BTSP Protocol Over Unix Sockets**

**Current** (HTTP JSON):
```http
POST /btsp/tunnel/create
Content-Type: application/json

{
  "tunnel_id": "tower-001",
  "security_level": "high",
  "endpoints": [...]
}
```

**Evolved** (Unix Socket JSON-RPC):
```json
{
  "jsonrpc": "2.0",
  "method": "btsp.tunnel.create",
  "params": {
    "tunnel_id": "tower-001",
    "security_level": "high",
    "endpoints": [...]
  },
  "id": 1
}
```

**Transport**:
- BearDog socket: `/tmp/beardog-{family_id}.sock`
- Songbird connects as client
- Same JSON-RPC 2.0 protocol used elsewhere
- Maintains full BTSP semantics

---

### **BirdSong P2P Integration**

**BirdSong** (UDP multicast for tower broadcast):
- ✅ Already pure Rust
- ✅ Already uses UDP (not HTTP)
- ✅ No changes needed!

**Integration**:
- BTSP: Tower security tunnels (Unix socket)
- BirdSong: Tower discovery/broadcast (UDP)
- Together: Complete tower atomic communication

**No Impact**: BirdSong evolution already complete!

---

## 🏗️ **Code Changes**

### **BearDog Changes**

**Remove** (from `Cargo.toml`):
```toml
# DELETE these
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "cors"] }
hyper = { version = "1.1", features = ["full"] }
reqwest = { version = "0.12", ... }
```

**Keep**:
```toml
# BearDog IPC (already have this)
beardog-ipc = { workspace = true }

# Pure Rust crypto (already migrated)
sha2 = "0.10"
hmac = "0.12"
# ... other RustCrypto crates
```

**Add** (if needed):
```toml
# Unix socket JSON-RPC (likely already have)
serde_json = { workspace = true }
tokio = { workspace = true, features = ["net"] }
```

**Code Evolution**:
```rust
// OLD: HTTP server
let app = Router::new()
    .route("/btsp/tunnel/create", post(create_tunnel))
    .route("/btsp/tunnel/close", post(close_tunnel));

let listener = TcpListener::bind("127.0.0.1:8080").await?;
axum::serve(listener, app).await?;

// NEW: Unix socket JSON-RPC server
let socket_path = PathBuf::from(format!("/tmp/beardog-{}.sock", family_id));
let listener = UnixListener::bind(&socket_path)?;

loop {
    let (stream, _) = listener.accept().await?;
    tokio::spawn(handle_btsp_request(stream));
}

async fn handle_btsp_request(stream: UnixStream) {
    // JSON-RPC 2.0 handler
    // Methods: btsp.tunnel.create, btsp.tunnel.close, etc.
}
```

---

### **Songbird Changes**

**Current**:
```rust
// HTTP client to BearDog BTSP
let client = reqwest::Client::new();
let response = client
    .post("http://localhost:8080/btsp/tunnel/create")
    .json(&request)
    .send()
    .await?;
```

**Evolved**:
```rust
// Unix socket client to BearDog BTSP
let socket_path = PathBuf::from(format!("/tmp/beardog-{}.sock", family_id));
let stream = UnixStream::connect(&socket_path).await?;

let request = json!({
    "jsonrpc": "2.0",
    "method": "btsp.tunnel.create",
    "params": tunnel_params,
    "id": 1
});

stream.write_all(request.to_string().as_bytes()).await?;
let response = read_json_rpc_response(&stream).await?;
```

**Keep**:
- HTTP server for external access (Songbird-only!)
- BTSP tunnels accessible via HTTP endpoints (if needed for external)
- BirdSong P2P integration (unchanged)

---

## 📊 **Impact Analysis**

### **Benefits**

**BearDog**:
- ✅ 100% pure Rust (no transitive `ring`)
- ✅ No HTTP dependencies
- ✅ Unix socket communication only
- ✅ Simpler, more secure

**Songbird**:
- ✅ Single HTTP gateway (concentrated gap)
- ✅ NUCLEUS access control
- ✅ Clear separation of concerns
- ✅ Internal: Unix sockets, External: HTTP

**Ecosystem**:
- ✅ BearDog achieves 100% pure Rust
- ✅ Songbird maintains its role as communication primal
- ✅ BTSP security maintained
- ✅ Tower atomic communication intact
- ✅ BirdSong P2P unaffected

---

### **Effort Estimate**

| Team | Task | Estimated Time |
|------|------|----------------|
| **BearDog** | Migrate BTSP server to Unix socket | 2-3 hours |
| **BearDog** | Remove HTTP dependencies | 1 hour |
| **BearDog** | Update tests | 1 hour |
| **Songbird** | Migrate BTSP client to Unix socket | 2-3 hours |
| **Songbird** | Update integration | 1 hour |
| **Songbird** | Update tests | 1 hour |
| **Joint** | E2E tower atomic testing | 2 hours |

**Total**: ~8-10 hours (joint effort)  
**Coordination**: High (joint project)  
**Complexity**: Medium (protocol unchanged, transport evolved)

---

## ✅ **Success Criteria**

### **BearDog**

- [ ] No HTTP dependencies (tower, hyper, reqwest removed)
- [ ] BTSP server on Unix socket (`/tmp/beardog-{family_id}.sock`)
- [ ] All tests passing
- [ ] 100% pure Rust (including transitives)
- [ ] `cargo tree | grep ring` → No results (except in tests)

### **Songbird**

- [ ] BTSP client uses Unix socket to BearDog
- [ ] HTTP server maintained (for external only)
- [ ] BirdSong P2P integration intact
- [ ] All tests passing
- [ ] Tower atomic E2E tests passing

### **Integration**

- [ ] BearDog ←→ Songbird: Unix socket communication
- [ ] BTSP protocol semantics maintained
- [ ] Tower atomic security intact
- [ ] BirdSong P2P working
- [ ] External access via Songbird HTTP (if needed)

---

## 🎯 **Timeline Integration**

### **Week 1** (Jan 14-16, 2026) - ✅ COMPLETE
- BiomeOS integration
- Pure Rust runtime evolution
- RustCrypto dependencies added
- Documentation world-class

### **Week 2** (Jan 24-30, 2026) - UPDATED PLAN

**Original Plan**:
- RustCrypto migration (internal crypto)
- BTSP tunnels → AES-GCM, X25519, Ed25519
- BirdSong → Ed25519, SHA-2, HMAC
- Auth → Argon2, SHA-2

**Updated Plan** (with BTSP Evolution):
- **Mon-Tue**: RustCrypto migration (internal crypto)
- **Wed-Thu**: BTSP evolution to Unix sockets (joint with BearDog)
- **Fri**: Testing, documentation, BiomeOS handoff

**New Priority**:
1. RustCrypto internal crypto (Mon-Tue, 6-8 hours)
2. BTSP Unix socket evolution (Wed-Thu, 8-10 hours)
3. Testing & documentation (Fri, 4-6 hours)

**Total Week 2**: ~18-24 hours (aggressive but achievable)

---

## 📚 **References**

**Architecture**:
- Tower Atomic: NUCLEUS atomic for enclave deployment
- BTSP: Security protocol for tower communication
- BirdSong: P2P UDP protocol for tower discovery/broadcast
- Concentrated Gap: HTTP deprecated for primals, Songbird-only

**Documentation**:
- `PURE_RUST_STRATEGY_CONCENTRATED_GAP_JAN_16_2026.md` - HTTP deprecation
- `PRIMAL_TEAMS_EVOLUTION_REVIEW_JAN_16_2026.md` - Current status
- `NUCLEUS_BONDING_MODEL.md` - Interaction patterns
- `MASTER_EVOLUTION_HANDOFF_JAN_16_2026.md` - Overall roadmap

**Week 1 Achievements**:
- `WEEK1_COMPLETE_JAN_16_2026.md` - Complete summary
- `HANDOFF_READY_WEEK1_JAN_16_2026.md` - Production readiness

---

## 🎊 **Conclusion**

**BTSP Evolution**: Clear path to Unix socket-based architecture!

**Benefits**:
- ✅ BearDog: 100% pure Rust
- ✅ Songbird: Single HTTP gateway
- ✅ NUCLEUS: Controlled access point
- ✅ Tower Atomic: Security maintained

**Timeline**: ~8-10 hours (joint BearDog + Songbird)  
**Week**: Week 2 (Jan 24-30, 2026)  
**Impact**: Completes the "Concentrated Gap" strategy! 🏆

**Coordination**:
- BearDog team: Server evolution
- Songbird team: Client evolution
- Joint: E2E testing

---

**Created**: January 16, 2026  
**Source**: BiomeOS upstream guidance  
**Purpose**: BTSP evolution to Unix sockets  
**Result**: Clear path to pure Rust + controlled HTTP gateway!

🦀🐻🐦✨ **BTSP Evolution: Tower Atomic Security via Pure Rust!** ✨🐦🐻🦀

