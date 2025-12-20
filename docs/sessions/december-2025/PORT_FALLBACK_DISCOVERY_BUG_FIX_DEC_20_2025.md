# Port Fallback Discovery Bug Fix

**Date:** December 20, 2025  
**Status:** ✅ Fixed  
**Severity:** Critical - Prevented federation in deployment scenarios  

---

## 🐛 Bug Description

### Symptoms
- Songbird starts successfully with port fallback (e.g., 8080 → 8082)
- Discovery broadcasts the **configured** port (8080)
- But HTTP server listens on the **fallback** port (8082)
- **Result:** Other towers can't connect (port mismatch)

### Real-World Scenario (Eastgate)
```
Port 8080: Occupied by Cursor IDE (nestgate_bin process)
Songbird: Falls back to port 8082
Discovery: Broadcasts "eastgate available at 8080" ❌
Westgate: Tries to connect to eastgate:8080 → Connection refused
Federation: Eastgate invisible to other towers
```

---

## 🔍 Root Cause Analysis

### The Bug
**Startup Order Issue:**

```rust
// OLD (BROKEN) ORDER:
1. Start discovery with configured port (8080)
2. Start HTTP server (falls back to 8082)
3. Mismatch! Discovery says 8080, server on 8082
```

**Why It Happened:**

1. `start()` method started discovery first (line ~453)
2. Discovery read port from config: `SafeEnv::get_port("SONGBIRD_PORT", 8080)`
3. HTTP server started later (line ~573)
4. HTTP server used `SovereignBinder::bind_with_fallback()` which tries 8080, 8081, 8082...
5. Server bound to 8082 (first available)
6. But discovery was already broadcasting 8080!

**Code Locations:**

- `crates/songbird-orchestrator/src/app/mod.rs:457-460` - Discovery gets port from config
- `crates/songbird-orchestrator/src/app/http_server.rs:31` - Server does fallback binding
- `crates/songbird-orchestrator/src/app/http_server.rs:24` - Returns `Result<()>` (no port info!)

---

## ✅ The Fix

### Changes Made

#### 1. Make `start_http_server()` Return Actual Port

**File:** `crates/songbird-orchestrator/src/app/http_server.rs`

```rust
// BEFORE:
pub async fn start_http_server(...) -> Result<()> {
    let (listener, actual_addr) = bind_with_fallback(&addr).await?;
    let actual_port = actual_addr.port();
    // ... start server ...
    Ok(()) // ❌ Actual port lost!
}

// AFTER:
pub async fn start_http_server(...) -> Result<u16> {
    let (listener, actual_addr) = bind_with_fallback(&addr).await?;
    let actual_port = actual_addr.port();
    // ... start server ...
    Ok(actual_port) // ✅ Return actual port!
}
```

#### 2. Reorder Startup Sequence

**File:** `crates/songbird-orchestrator/src/app/mod.rs`

```rust
// BEFORE:
pub async fn start(&mut self) -> Result<()> {
    // 1. Start discovery (with configured port 8080)
    start_discovery(8080);
    
    // 2. Start HTTP server (falls back to 8082)
    start_http_server();
    
    // Result: Mismatch!
}

// AFTER:
pub async fn start(&mut self) -> Result<()> {
    // 1. Start HTTP server FIRST (get actual port)
    let actual_port = start_http_server().await?; // Returns 8082
    
    // 2. Start discovery WITH actual port
    start_discovery(actual_port); // Uses 8082 ✅
    
    // Result: Match!
}
```

#### 3. Use Actual Port for Node Identity

```rust
// AFTER (in start() method):
let actual_https_port = self.start_http_server().await?;
info!("✅ HTTP server started on port {}", actual_https_port);

// Initialize node identity with ACTUAL port
let mut node_identity = NodeIdentity::new_or_load(None)?;
node_identity.detect_all_endpoints(actual_https_port)?; // ✅ 8082

// Discovery now broadcasts correct port
let broadcaster = AnonymousDiscoveryBroadcaster::new_v3(
    node_identity.node_id,
    node_identity.node_name,
    endpoints, // All have port 8082 ✅
    ...
);
```

---

## 🧪 Testing

### Test Scenario: Port Conflict on Eastgate

**Setup:**
- Cursor IDE occupying port 8080
- Songbird configured for port 8080
- Expected fallback to 8082

**Test Steps:**

1. **Kill all Songbird processes:**
   ```bash
   pkill -9 songbird-orchestrator
   sleep 3
   ```

2. **Start Songbird:**
   ```bash
   ./target/release/songbird-orchestrator &
   sleep 25
   ```

3. **Verify server port:**
   ```bash
   curl -sk https://localhost:8080/health  # Should fail
   curl -sk https://localhost:8082/health  # Should work ✅
   ```

4. **Check Eastgate's federation:**
   ```bash
   curl -sk https://localhost:8082/api/federation/status | \
     jq -r '.nodes[] | "\(.node_name) (\(.node_id[:12])...)"'
   
   # Should see westgate and pop-os
   ```

5. **Check Westgate's federation (CRITICAL):**
   ```bash
   # Run on Westgate:
   curl -sk https://localhost:8080/api/federation/status | \
     jq -r '.nodes[] | "\(.node_name) (\(.node_id[:12])...)"'
   
   # Should see:
   # westgate (526c1e31...)
   # pop-os (496fe99e...)
   # eastgate (SOME-UUID...)  ← Should appear! ✅
   ```

### Expected Results

✅ **Server binds to port 8082** (fallback)  
✅ **Discovery broadcasts port 8082** (actual)  
✅ **Westgate can connect to eastgate:8082**  
✅ **Eastgate appears in Westgate's federation**  
✅ **Identity-based routing works across all towers**

---

## 📊 Impact

### Before Fix
- **Deployment failures** in environments with port conflicts
- **Silent federation breakage** (node appears to start, but isn't discoverable)
- **Manual intervention required** (kill conflicting processes)
- **Poor developer experience** (confusing error messages)

### After Fix
- ✅ **Automatic port fallback** with correct discovery
- ✅ **Federation works** even with port conflicts
- ✅ **Zero manual intervention**
- ✅ **Clear logging** ("using port 8082 instead")
- ✅ **Deployment robustness** improved

---

## 🎯 Lessons Learned

### Deployment Realities
1. **Port conflicts are common** (IDEs, dev tools, other services)
2. **Fallback is necessary** but must be communicated
3. **Startup order matters** when components depend on each other
4. **Return values matter** - port info must propagate

### Architectural Insights
1. **Single source of truth** - Actual port must drive everything
2. **Late binding** - Bind early, configure late
3. **Propagate reality** - Don't broadcast config, broadcast reality
4. **Test edge cases** - Port conflicts reveal design flaws

---

## 🔮 Future Improvements

### 1. PID File Management (Pending)
Prevent multiple Songbird instances:
```rust
// On startup:
if pid_file_exists() && process_is_running() {
    error!("Songbird already running at PID {}", read_pid());
    exit(1);
}
write_pid_file();
```

### 2. Graceful Shutdown (Pending)
Handle SIGTERM/SIGINT properly:
```rust
tokio::spawn(async {
    tokio::signal::ctrl_c().await;
    info!("Shutting down gracefully...");
    cleanup_pid_file();
    broadcast_offline_message();
    exit(0);
});
```

### 3. Port Conflict Detection (Enhancement)
Better UX for port conflicts:
```rust
fn check_port_availability(port: u16) -> PortStatus {
    match get_port_owner(port) {
        Some(process) => PortStatus::Occupied {
            by: process.name,
            pid: process.pid,
        },
        None => PortStatus::Available,
    }
}
```

---

## 📚 Related Issues

- **Multi-Interface Identity Problem:** Solved with node_id coalescence
- **Session TTL Cleanup:** Prevents stale node accumulation  
- **Discovery Verification:** Ensures HTTPS connectivity before registration
- **Network Sovereignty:** Pure Rust, no manual firewall configs

---

## ✅ Verification Checklist

- [x] Server returns actual port from `start_http_server()`
- [x] Startup order: HTTP → Discovery
- [x] Node identity uses actual port
- [x] Discovery broadcasts actual port
- [x] Port fallback works (8080 → 8082)
- [x] Federation connection successful
- [x] Tested on Eastgate (real port conflict)
- [x] Westgate can see Eastgate
- [ ] PID file management (future)
- [ ] Graceful shutdown (future)

---

**Status:** Core fix complete and tested. Future enhancements (PID management, graceful shutdown) deferred for follow-up.


