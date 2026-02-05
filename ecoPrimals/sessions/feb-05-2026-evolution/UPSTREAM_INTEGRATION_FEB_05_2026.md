# Upstream Integration Issues - February 5, 2026

**Status**: 🟡 Investigation Complete - Ready for Implementation  
**Version**: v3.21.0 → v3.22.0  
**Priority**: High (biomeOS Integration)

---

## Executive Summary

Three issues identified during biomeOS integration testing:

| Issue | Status | Complexity | Notes |
|-------|--------|------------|-------|
| 1. Standard methods in IPC | 🔴 **Needs Fix** | Simple | Methods exist in orchestrator, missing in IPC service |
| 2. BirdSong `family_id` | 🔴 **Needs Fix** | Simple | Provider has method, just need to call it |
| 3. TLS Protocol Detection | ✅ **Already Fixed** | N/A | Implemented in v3.21.0 (Feb 5, 2026) |

---

## Issue 1: Missing Standard Methods in `songbird-universal-ipc`

### Current State

✅ **Orchestrator Unix Server HAS these methods**:
- `crates/songbird-orchestrator/src/ipc/unix/handlers.rs:194` - `handle_health_standard()`
- `crates/songbird-orchestrator/src/ipc/unix/handlers.rs:386` - `handle_identity()`
- `crates/songbird-orchestrator/src/ipc/unix/handlers.rs:437` - `handle_rpc_discover()`

❌ **IPC Service MISSING route**:
- `crates/songbird-universal-ipc/src/service.rs:686-727` - No routes for `health`, `identity`, `rpc.discover`

### Root Cause

**Two different entry points**:
1. **Orchestrator's Unix Server** (`songbird-orchestrator/src/ipc/unix/server.rs`) - Routes these methods ✅
2. **Universal IPC Service** (`songbird-universal-ipc/src/service.rs`) - Does NOT route them ❌

The IPC service is used when Songbird is running as a standalone service (not embedded in orchestrator).

### Fix Required

Add routes and implementations to `crates/songbird-universal-ipc/src/service.rs`:

```rust
// In IpcServiceHandler struct, add:
start_time: Arc<RwLock<std::time::Instant>>,  // Track uptime

// In new():
pub fn new(registry: Arc<RwLock<ServiceRegistry>>) -> Self {
    Self {
        registry,
        start_time: Arc::new(RwLock::new(std::time::Instant::now())),
        // ... other fields
    }
}

// In handle() method, add BEFORE line 726 catch-all:
"health" => self.handle_health().await,
"identity" => self.handle_identity().await,
"rpc.discover" => self.handle_rpc_discover().await,

// Implement handlers:
async fn handle_health(&self) -> Result<Value, String> {
    let uptime_secs = self.start_time.read().await.elapsed().as_secs();
    let registry = self.registry.read().await;
    let services = registry.list_all();
    
    Ok(json!({
        "status": "healthy",
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime_secs,
        "services": services.len(),
    }))
}

async fn handle_identity(&self) -> Result<Value, String> {
    let family_id = std::env::var("SONGBIRD_FAMILY_ID")
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "nat0".to_string());
    
    Ok(json!({
        "primal": "songbird",
        "version": env!("CARGO_PKG_VERSION"),
        "family_id": family_id,
        "capabilities": [
            "ipc.register", "ipc.resolve", "ipc.discover", "ipc.list",
            "http.request", "http.get", "http.post",
            "stun.get_public_address", "stun.bind",
            "birdsong.generate_encrypted_beacon", "birdsong.decrypt_beacon",
            "birdsong.verify_lineage", "birdsong.get_lineage",
            "discovery.peers"
        ]
    }))
}

async fn handle_rpc_discover(&self) -> Result<Value, String> {
    Ok(json!({
        "methods": [
            "health", "identity", "rpc.discover",
            "primal.info", "primal.capabilities", "rpc.methods",
            "ipc.register", "ipc.resolve", "ipc.discover", "ipc.list",
            "http.request", "http.get", "http.post",
            "stun.get_public_address", "stun.bind",
            "birdsong.generate_encrypted_beacon", "birdsong.decrypt_beacon",
            "birdsong.verify_lineage", "birdsong.get_lineage",
            "discovery.peers",
            "rendezvous.register", "rendezvous.lookup",
            "peer.connect"
        ]
    }))
}
```

### Files to Modify
- `crates/songbird-universal-ipc/src/service.rs` (add routes + implementations)

---

## Issue 2: BirdSong `family_id` Not Passed to BearDog

### Current State

❌ **Current Code**:
```rust
// crates/songbird-universal-ipc/src/handlers/birdsong_handler.rs:151
let provider = BearDogBirdSongProvider::new(socket_path, None)  // <-- family_id is None!
```

✅ **Provider HAS the method**:
```rust
// crates/songbird-network-federation/src/beardog/production.rs:75-80
pub async fn with_family_id(
    socket_path: impl Into<PathBuf>,
    family_id: impl Into<String>,
) -> Result<Self> { ... }
```

### Root Cause

The `birdsong_handler.rs` doesn't read `family_id` from environment before creating the provider.

### Fix Required

**Option A: Environment Discovery** (RECOMMENDED)

```rust
// In birdsong_handler.rs get_provider() method, replace line 151:

// Discover family_id from environment (matches biomeOS pattern)
let family_id = std::env::var("FAMILY_ID")
    .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
    .or_else(|_| std::env::var("NODE_FAMILY_ID"))
    .ok();

let provider = if let Some(fid) = family_id {
    info!("🔒 Using family_id: {}", fid);
    BearDogBirdSongProvider::with_family_id(socket_path, fid)
        .await
        .map_err(|e| format!("Failed to create BirdSong provider: {e}"))?
} else {
    warn!("⚠️  No FAMILY_ID set - BearDog encryption may fail");
    BearDogBirdSongProvider::new(socket_path, None)
        .await
        .map_err(|e| format!("Failed to create BirdSong provider: {e}"))?
};
```

**Option B: RPC Parameter** (Breaking Change)

Require `family_id` in the RPC params. Not recommended as it breaks existing clients.

### Files to Modify
- `crates/songbird-universal-ipc/src/handlers/birdsong_handler.rs:151` (add family_id discovery)

---

## Issue 3: TLS Protocol Detection (HTTP/HTTPS Same Port)

### Current State

✅ **ALREADY IMPLEMENTED** in v3.21.0 (Feb 5, 2026)!

**Evidence**:
```rust
// crates/songbird-orchestrator/src/app/http_server.rs:292-347
// PROTOCOL DETECTION: Peek at first byte to detect TLS vs HTTP
// TLS ClientHello starts with 0x16 (Handshake content type)
let mut peek_buf = [0u8; 1];
let peek_result = tcp_stream.peek(&mut peek_buf).await;

let is_tls = match peek_result {
    Ok(1) => peek_buf[0] == 0x16, // TLS Handshake content type
    // ...
}
```

### What Was Fixed

In the Deep Debt Evolution (v3.21.0, Feb 5 2026):
- ✅ HTTP and HTTPS on **same port**
- ✅ Peek first byte: `0x16` = TLS handshake, ASCII = HTTP
- ✅ Eliminates "Server responded with HTTP instead of TLS" errors
- ✅ Graceful degradation when clients don't support TLS

### Verification Needed

The implementation exists, but we should verify it works cross-device:

```bash
# From Tower to Pixel (192.168.1.80:8080):
curl -k -v https://192.168.1.80:8080/.well-known/songbird

# Should work without "HTTP instead of TLS" error
```

### Possible Issues (If Still Failing)

1. **Pixel not using new code** - Ensure Pixel is running v3.21.0+
2. **Certificate issues** - Self-signed cert not generated on Android
3. **Port configuration** - Verify port 8080 is actually bound

### Debug Commands

```bash
# Check Pixel Songbird version
adb shell "/data/local/tmp/biomeos/songbird --version"

# Check if TLS cert exists
adb shell "ls -la /data/local/tmp/biomeos/*.pem"

# Test with openssl
openssl s_client -connect 192.168.1.80:8080 -servername pixel8a 2>&1 | head -30
```

---

## Implementation Plan

### Phase 1: Fix Issue 1 & 2 (2 hours)

```bash
# 1. Add standard methods to IPC service
# File: crates/songbird-universal-ipc/src/service.rs
# - Add start_time field
# - Add health/identity/rpc.discover routes
# - Implement handler methods

# 2. Add family_id to BirdSong provider
# File: crates/songbird-universal-ipc/src/handlers/birdsong_handler.rs
# - Read FAMILY_ID from env
# - Use with_family_id() constructor
```

### Phase 2: Verify Issue 3 (30 min)

```bash
# 3. Test TLS protocol detection cross-device
# - Deploy v3.21.0 to Pixel
# - Test HTTPS from Tower
# - Confirm protocol detection works
```

### Phase 3: Testing (1 hour)

```bash
# Test standard methods
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
  nc -U /run/user/1000/biomeos/songbird-nat0.sock

echo '{"jsonrpc":"2.0","method":"identity","params":{},"id":1}' | \
  nc -U /run/user/1000/biomeos/songbird-nat0.sock

# Test BirdSong with family_id
FAMILY_ID=nat0 ./songbird server --socket /tmp/test.sock &
echo '{"jsonrpc":"2.0","method":"birdsong.generate_encrypted_beacon","params":{"node_id":"test","capabilities":[]},"id":1}' | \
  nc -U /tmp/test.sock

# Test TLS protocol detection
curl -k -v https://192.168.1.80:8080/.well-known/songbird
```

---

## Test Matrix

| Test | Entry Point | Expected Result |
|------|------------|-----------------|
| `health` on orchestrator Unix socket | `songbird-orchestrator/ipc/unix` | ✅ Works (already implemented) |
| `health` on IPC service socket | `songbird-universal-ipc/service` | ❌ Fails (needs fix) |
| `identity` on orchestrator | `songbird-orchestrator/ipc/unix` | ✅ Works |
| `identity` on IPC service | `songbird-universal-ipc/service` | ❌ Fails |
| BirdSong without FAMILY_ID | Both | ❌ BearDog fails |
| BirdSong with FAMILY_ID=nat0 | Both | ❌ Fails (needs fix) |
| HTTPS cross-device | HTTP server | ❓ Should work (v3.21.0) |

---

## Success Criteria

After fixes:
- [ ] `health` works on IPC service socket
- [ ] `identity` works on IPC service socket
- [ ] `rpc.discover` works on IPC service socket
- [ ] BirdSong encryption succeeds with FAMILY_ID env var
- [ ] HTTPS works cross-device (Tower ↔ Pixel)
- [ ] All existing tests pass
- [ ] Build: 0 errors

---

## Version Bump

After implementation:
- **Current**: v3.21.0 (Deep Debt Evolution Complete)
- **Next**: v3.22.0 (biomeOS Integration Complete)

---

## Related Documentation

- Deep Debt Evolution: `CLEANUP_COMPLETED_FEB_05_2026.md`
- v3.21.0 Changelog: `CHANGELOG.md:10-67`
- TLS Protocol Detection: `evolution_feb_2026_tests.rs:618-667` (protocol detection tests)

---

**Created**: February 5, 2026 @ 05:00 UTC  
**Investigator**: Cursor Agent  
**Status**: 🟡 Ready for Implementation

