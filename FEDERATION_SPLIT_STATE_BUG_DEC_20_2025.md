# Federation Split State Bug - December 20, 2025

## 🐛 THE BUG

**Symptom:** Eastgate shows 0 active nodes in federation, while Westgate and Strandgate see 3 nodes including Eastgate.

**Root Cause:** Multiple Songbird instances running simultaneously, causing split state.

## 🔍 INVESTIGATION

### Observed Behavior

```
Eastgate Status:
  Active nodes: 0
  Federation: Empty
  Discovery: Running
  Broadcasts: Working (others see Eastgate)

Westgate Status:
  Active nodes: 3 ✅
  Sees: Eastgate, Strandgate, itself

Strandgate Status:
  Active nodes: 3 ✅
  Sees: Eastgate, Westgate, itself
```

### The Paradox

- Eastgate IS in the federation (others see it)
- But Eastgate DOESN'T see others
- Discovery is broadcasting AND listening
- Health checks work (can reach all towers)

### Discovery Process

```
Process 2820306 (old):
  ✅ Discovery listener (UDP 2300)
  ✅ HTTPS server (8080)
  ⏱️  Started: 11:19
  
Process 2989238 (new):
  ✅ HTTPS server (8080)
  ❌ No discovery listener
  ⏱️  Started: 11:39
  
Process 2989236 (wrapper):
  Bash script wrapper
```

## 🎯 ROOT CAUSE

### Multiple Instances = Split State

**The Problem:**

1. **Instance A** (PID 2820306):
   - Has discovery listener
   - Receives peer broadcasts
   - Has federation state with peers
   - BUT not responding to API calls?

2. **Instance B** (PID 2989238):
   - Responding to API calls (8080)
   - Has federation state (self only)
   - NOT receiving discovery
   - Fresh state (just started)

**Result:** Federation state split between processes!

### SO_REUSEADDR/SO_REUSEPORT Strikes Again

Both processes can bind to port 8080 because we use `SO_REUSEADDR` and `SO_REUSEPORT` in the `SovereignSocket`:

```rust
// crates/songbird-orchestrator/src/network/sovereign_socket.rs
socket.set_reuse_address(true)?;
socket.set_reuse_port(true)?;
```

**Intention:** Allow graceful restart without "Address in use" errors

**Side Effect:** Allows multiple instances to coexist, causing state confusion!

### The Discovery Asymmetry

```
Timeline:

11:19 - Instance A starts
        ✅ Binds HTTPS 8080
        ✅ Binds UDP 2300
        ✅ Starts discovery
        ✅ Receives broadcasts from Westgate/Strandgate
        ✅ Registers peers

11:39 - Instance B starts (forgot to kill A)
        ✅ Binds HTTPS 8080 (REUSE_PORT allows!)
        ❌ Cannot bind UDP 2300 (Instance A has it)
        ❌ No discovery listener
        ❌ Empty federation state
        
Result:
- API calls → Instance B (empty state)
- Discovery → Instance A (has peers)
- SPLIT STATE!
```

## 📊 EVIDENCE

### Process List

```bash
ps aux | grep songbird-orchestrator | grep -v grep
# Output:
eastgate 2820306  ... 11:19 ... ./target/release/songbird-orchestrator
eastgate 2989238  ... 11:39 ... ./target/release/songbird-orchestrator
```

### Port Bindings

```bash
ss -tlnp | grep 8080
# Output:
LISTEN ... songbird-orches,pid=2989238,fd=11
LISTEN ... songbird-orches,pid=2820306,fd=11  # ← BOTH bound!

ss -ulnp | grep 2300
# Output:
UNCONN ... songbird-orches,pid=2820306,fd=14  # ← Only old instance
```

### Federation State

```bash
# From Eastgate (hits Instance B - new, empty state):
curl https://localhost:8080/api/federation/status
# Result: {"active_nodes": 0}

# From Westgate (sees Eastgate's Instance A):
curl https://192.168.1.123:8080/api/federation/status
# Result: {"active_nodes": 3, "nodes": [...]}
```

## 🔥 WHY THIS IS CRITICAL

### Silent Failure

- No error messages
- Both processes appear healthy
- Discovery seems to work
- But federation state is wrong

### Diagnosis Difficulty

- Hard to spot (need to check process list)
- Symptoms are confusing (paradoxical behavior)
- Users might not realize multiple instances

### Data Integrity

- Split state across processes
- API queries return incomplete data
- Workload distribution decisions based on wrong info

### Production Impact

- Load balancing breaks
- Task scheduling incorrect
- Monitoring shows wrong metrics
- Failover logic confused

## 🎯 THE DEEP DEBT

### Root Issue: No Process Lifecycle Management

This bug is a symptom of the architectural gap we documented:

1. **No PID File Management**
   - No way to detect existing instance
   - Can't prevent duplicate starts
   
2. **No Singleton Enforcement**
   - Multiple instances allowed
   - No automatic conflict detection

3. **SO_REUSEPORT Intention vs Reality**
   - Intended: Graceful restart
   - Reality: Allows duplicates
   - Trade-off not managed

## ✅ SOLUTION DESIGN

### Phase 1: Detection & Prevention (Immediate)

#### 1.1 PID File Management

```rust
// crates/songbird-orchestrator/src/lifecycle/pid_manager.rs

pub struct PidManager {
    pid_file: PathBuf,
}

impl PidManager {
    /// Check if another instance is running
    pub fn check_existing() -> Result<Option<u32>> {
        let pid_file = Self::pid_path();
        
        if !pid_file.exists() {
            return Ok(None);
        }
        
        let pid_str = fs::read_to_string(&pid_file)?;
        let pid: u32 = pid_str.trim().parse()?;
        
        // Check if process actually exists
        if Self::process_exists(pid) {
            Ok(Some(pid))
        } else {
            // Stale PID file
            fs::remove_file(&pid_file)?;
            Ok(None)
        }
    }
    
    /// Check if process exists (without sending signals)
    fn process_exists(pid: u32) -> bool {
        Path::new(&format!("/proc/{}", pid)).exists()
    }
    
    /// Claim ownership by writing PID
    pub fn claim() -> Result<()> {
        let pid_file = Self::pid_path();
        
        // Create parent directory
        if let Some(parent) = pid_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write our PID
        fs::write(&pid_file, format!("{}", std::process::id()))?;
        
        info!("📝 PID file created: {}", pid_file.display());
        Ok(())
    }
    
    /// Release ownership on exit
    pub fn release() -> Result<()> {
        let pid_file = Self::pid_path();
        if pid_file.exists() {
            fs::remove_file(&pid_file)?;
            info!("🗑️  PID file removed");
        }
        Ok(())
    }
    
    fn pid_path() -> PathBuf {
        dirs::runtime_dir()
            .or_else(|| dirs::data_local_dir())
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("songbird")
            .join("songbird-orchestrator.pid")
    }
}
```

#### 1.2 Singleton Enforcement

```rust
// In main.rs startup:

use crate::lifecycle::PidManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider
    rustls::crypto::CryptoProvider::install_default_impl();
    
    // Check for existing instance
    match PidManager::check_existing()? {
        Some(existing_pid) => {
            eprintln!("❌ Songbird already running (PID: {})", existing_pid);
            eprintln!("   Options:");
            eprintln!("   - Stop existing: kill {}", existing_pid);
            eprintln!("   - Force start: SONGBIRD_FORCE_START=1");
            
            if std::env::var("SONGBIRD_FORCE_START").is_ok() {
                warn!("⚠️  Force starting despite existing instance");
            } else {
                std::process::exit(1);
            }
        }
        None => {
            // No existing instance, we can start
            PidManager::claim()?;
        }
    }
    
    // Setup cleanup on exit
    let _pid_guard = PidGuard::new();
    
    // Start orchestrator
    // ...
}

struct PidGuard;

impl PidGuard {
    fn new() -> Self {
        Self
    }
}

impl Drop for PidGuard {
    fn drop(&mut self) {
        let _ = PidManager::release();
    }
}
```

#### 1.3 Startup Check Script

```bash
#!/bin/bash
# check-songbird-running.sh

PID_FILE="${XDG_RUNTIME_DIR:-/tmp}/songbird/songbird-orchestrator.pid"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo "✅ Songbird running (PID: $PID)"
        exit 0
    else
        echo "⚠️  Stale PID file (process $PID not found)"
        rm -f "$PID_FILE"
        exit 1
    fi
else
    echo "❌ Songbird not running (no PID file)"
    exit 1
fi
```

### Phase 2: Improved SO_REUSEPORT Handling

#### Option A: Disable SO_REUSEPORT

```rust
// If we have PID file management, we don't need SO_REUSEPORT

socket.set_reuse_address(true)?;  // Keep for quick restart
// socket.set_reuse_port(true)?;   // ❌ Remove - allows duplicates!
```

**Pros:**
- Prevents duplicate instances
- "Address in use" error becomes a feature (alerts to existing instance)

**Cons:**
- Need to wait for port release on restart
- Slightly slower restart

#### Option B: Keep SO_REUSEPORT + Add Detection

```rust
// Keep SO_REUSEPORT but detect conflicts

socket.set_reuse_address(true)?;
socket.set_reuse_port(true)?;

// After binding, check if we're the only instance
if !PidManager::am_i_alone()? {
    error!("❌ Another Songbird instance detected!");
    error!("   Multiple instances will cause split federation state.");
    return Err(anyhow!("Duplicate instance detected"));
}
```

**Pros:**
- Fast restart capability
- Explicit conflict detection

**Cons:**
- More complex logic
- Race conditions possible

#### Recommendation: Option A (Disable SO_REUSEPORT)

PID file management is more robust than socket-based detection.

### Phase 3: Diagnostic Endpoints

Add debugging endpoints to detect split state:

```rust
// GET /api/debug/process-info
{
    "pid": 2989238,
    "started_at": "2025-12-20T11:39:00Z",
    "uptime_seconds": 1234,
    "discovery_listener": false,  // ⚠️  Warning!
    "pid_file_owner": false        // ⚠️  Not the registered instance
}

// GET /api/debug/socket-status
{
    "https_port": 8080,
    "https_bound": true,
    "discovery_port": 2300,
    "discovery_bound": false,  // ⚠️  Should be true!
    "tarpc_port": 8091,
    "tarpc_bound": true
}
```

### Phase 4: Health Check Enhancement

```rust
// Update health check to detect issues

#[get("/health")]
async fn health(state: web::Data<AppState>) -> impl Responder {
    let warnings = vec![];
    
    // Check if we own the PID file
    if !PidManager::am_i_owner() {
        warnings.push("Not the registered instance");
    }
    
    // Check if discovery is running
    if !state.discovery_listener.is_listening() {
        warnings.push("Discovery listener not bound");
    }
    
    // Check for split state indicators
    if state.federation.active_nodes() == 0 && uptime() > 60 {
        warnings.push("No peers discovered (possible split state)");
    }
    
    HttpResponse::Ok().json(json!({
        "status": if warnings.is_empty() { "OK" } else { "DEGRADED" },
        "warnings": warnings,
        "uptime_seconds": uptime(),
    }))
}
```

## 📈 IMPLEMENTATION PLAN

### Immediate (This Session if Time)

1. ✅ Document the bug
2. ⏳ Kill duplicate processes on Eastgate
3. ⏳ Verify federation recovery
4. ⏳ Test prevention (try to start duplicate)

### Next Session (High Priority)

1. Implement PID file management
2. Add singleton enforcement in main.rs
3. Decide on SO_REUSEPORT strategy
4. Add process info debug endpoint
5. Update health check with warnings

### Future (Medium Priority)

6. Automatic duplicate detection in health check
7. Admin UI showing process conflicts
8. Monitoring alerts for split state
9. Documentation for operators

## 🎓 LESSONS LEARNED

### 1. SO_REUSEPORT is a Double-Edged Sword

**Good for:**
- Load balancing across workers
- Zero-downtime restarts
- High-performance servers

**Bad for:**
- Single-instance applications
- Stateful services
- Federation coordination

**Lesson:** Use SO_REUSEPORT only when you WANT multiple instances.

### 2. Silent Failures are the Worst

The split state bug had:
- No error messages
- Healthy-looking processes
- Confusing symptoms
- Hard to diagnose

**Lesson:** Add explicit checks and warnings for abnormal states.

### 3. Process Lifecycle is Foundational

This is the 3rd time in this session we've hit zombie/duplicate process issues:
1. Port conflicts on startup
2. Manual cleanup required
3. Split federation state

**Lesson:** Process lifecycle management isn't optional infrastructure - it's foundational architecture.

### 4. Test in Production Conditions

This bug only appeared after:
- Multiple restarts
- Forgot to kill old instance
- Real federation deployment

**Lesson:** Test failure scenarios (forgot to stop, restart during operation, etc.)

## 🔧 IMMEDIATE WORKAROUND

```bash
# Always kill ALL instances before starting
pkill -9 songbird-orchestrator
sleep 2

# Verify all stopped
ps aux | grep songbird-orchestrator

# Then start ONE instance
./target/release/songbird-orchestrator
```

## 🎯 SUCCESS CRITERIA

Implementation is complete when:

1. ✅ Only ONE Songbird instance can run at a time
2. ✅ Duplicate start attempts are rejected with clear error
3. ✅ PID file tracks running instance
4. ✅ Health check warns about split state
5. ✅ Debug endpoint shows process ownership
6. ✅ Documentation updated with operational procedures

## 📊 IMPACT

### Before Fix:
- ❌ Silent duplicate instances
- ❌ Split federation state
- ❌ Confusing behavior
- ❌ Manual detection required
- ❌ No warnings

### After Fix:
- ✅ Singleton enforcement
- ✅ Clear error messages
- ✅ Consistent federation state
- ✅ Automatic detection
- ✅ Health check warnings

## 🏆 ARCHITECTURAL DEBT RESOLVED

This fix closes a major gap in the Process Lifecycle Management architecture documented in `PROCESS_LIFECYCLE_ARCHITECTURE_GAP_DEC_20_2025.md`.

**Status:** Deep debt identified and solution designed. Ready for implementation.

---

*"The hardest bugs to find are those that don't generate errors - they just silently corrupt state. Proactive detection and prevention is worth a thousand diagnostic sessions."*

**Session:** Federation Deployment - December 20, 2025  
**Bug Severity:** Critical (data integrity)  
**Fix Priority:** High (next session)

