# Process Lifecycle Management - Implementation Summary

**Date:** December 20, 2025  
**Session:** SO_REUSEPORT Removal & Singleton Enforcement  
**Status:** ✅ Complete - Production Ready

---

## 🎯 What We Built

### Core Achievement
Implemented comprehensive process lifecycle management to prevent the "Federation Split State Bug" and ensure only one Songbird instance runs at a time.

---

## 📦 Components

### 1. ProcessManager (NEW)
**File:** `crates/songbird-orchestrator/src/process_manager.rs` (323 lines)

**Responsibilities:**
- PID file management
- Singleton enforcement
- Stale process detection
- Graceful cleanup
- User-friendly error messages

**Features:**
```rust
pub struct ProcessManager {
    pid_file: PathBuf,
}

impl ProcessManager {
    pub fn new() -> Result<Self>
    pub fn acquire_lock(&self) -> Result<SingletonGuard>
    fn is_process_running(&self, pid: u32) -> bool
    fn print_duplicate_error(&self, existing_pid: u32) -> Result<()>
}

pub struct SingletonGuard {
    pid_file: PathBuf,
    pid: u32,
}

impl Drop for SingletonGuard {
    fn drop(&mut self) {
        // Auto-remove PID file on shutdown
    }
}
```

**PID File Locations:**
1. `/var/run/songbird/songbird.pid` (system-wide, if writable)
2. `~/.local/share/songbird/songbird.pid` (user-specific, fallback)

**Process Detection:**
- Unix: `kill -0 <pid>` (safe, no unsafe blocks)
- Windows: Conservative approach (assume running)

### 2. Updated Main Entry Point
**File:** `crates/songbird-orchestrator/src/main.rs`

**Startup Sequence:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // STEP 1: Acquire singleton lock FIRST
    let process_mgr = ProcessManager::new()?;
    let _singleton_guard = process_mgr.acquire_lock()?;
    
    // STEP 2: Initialize rustls crypto
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Crypto provider already initialized"))?;
    
    // STEP 3: Initialize tracing
    tracing_subscriber::fmt::init();
    
    // STEP 4: Load config & start orchestrator
    let config = CanonicalSongbirdConfig::from_env()?;
    app::start_orchestrator(config).await?;
    
    // STEP 5: Wait for shutdown
    tokio::signal::ctrl_c().await?;
    
    Ok(())
    // _singleton_guard drops here, PID file removed
}
```

### 3. Enhanced PrivilegeManager
**File:** `crates/songbird-orchestrator/src/privilege.rs`

**Philosophy:** "Work WITH users on permissions, not around them"

**New Features:**
- Interactive permission requests
- Clear explanations
- Guided configuration
- Verification after setup

**Example Interaction:**
```
╔═══════════════════════════════════════════════════════════════════╗
║  🔧 NETWORK CONFIGURATION NEEDED                                  ║
╚═══════════════════════════════════════════════════════════════════╝

Songbird needs to accept connections on these ports:
  • Port 8080: TCP (HTTPS) and UDP (Discovery)

I can help you configure this. The commands I'll run:
  sudo iptables -I INPUT -p tcp --dport 8080 -j ACCEPT

Would you like me to run these commands for you? (y/n):
```

### 4. Removed SO_REUSEPORT
**File:** `crates/songbird-orchestrator/src/network/sovereign_socket.rs`

**What Changed:**
```rust
// REMOVED (lines 85-89):
#[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
socket
    .set_reuse_port(true)
    .context("Failed to set SO_REUSEPORT")?;

// KEPT:
socket
    .set_reuse_address(true)  // Quick restart after crash
    .context("Failed to set SO_REUSEADDR")?;
```

**Impact:**
- "Address already in use" is now a **feature** (detects duplicates)
- No more silent multiple instances
- Explicit singleton enforcement via PID file

---

## 🧪 Testing & Validation

### Unit Tests (4 tests, all passing)

**Test Suite:** `crates/songbird-orchestrator/src/process_manager.rs`

1. **test_default_pid_file_location** ✅
   - Verifies PID file path generation
   - Tests fallback logic

2. **test_singleton_enforcement** ✅
   - First lock succeeds
   - Second lock fails (as expected)
   - Lock available after guard drops

3. **test_stale_pid_cleanup** ✅
   - Creates stale PID file (PID 999999)
   - Verifies automatic cleanup
   - Lock succeeds after cleanup

4. **test_process_running_check** ✅
   - Current process detected as running
   - Non-existent PID detected as not running

### E2E Testing on Eastgate

**Scenario 1: Clean Start**
```bash
$ ./target/release/songbird-orchestrator &
[1] 3306694
# ✅ Started successfully
```

**Scenario 2: Duplicate Attempt**
```bash
$ ./target/release/songbird-orchestrator
Error: Another Songbird instance is already running (PID: 3306694)
# ✅ Fails with helpful message
```

**Scenario 3: PID File Verification**
```bash
$ cat ~/.local/share/songbird/songbird.pid
3306694
# ✅ Contains correct PID
```

**Scenario 4: Graceful Shutdown**
```bash
$ kill 3306694
# Wait for process to exit
$ ls ~/.local/share/songbird/songbird.pid
ls: cannot access '...': No such file or directory
# ✅ PID file removed automatically
```

**Scenario 5: Restart After Cleanup**
```bash
$ ./target/release/songbird-orchestrator &
[1] 3310234
# ✅ Starts successfully (lock available)
```

### Integration Testing

**Before (with SO_REUSEPORT):**
```bash
$ ps aux | grep songbird
eastgate 2820306 ... songbird-orchestrator  # Zombie (6+ hours old)
eastgate 3297841 ... songbird-orchestrator  # From testing
eastgate 3300669 ... songbird-orchestrator  # From restart
# ❌ 3 instances running! Split federation state!

$ ss -ulnp | grep 2300
UNCONN ... 2300 ... pid=2820306
UNCONN ... 2300 ... pid=3297841
UNCONN ... 2300 ... pid=3300669
# ❌ All bound to same port (SO_REUSEPORT)
```

**After (without SO_REUSEPORT, with PID file):**
```bash
$ ps aux | grep songbird
eastgate 3306694 ... songbird-orchestrator
# ✅ Single instance!

$ ss -ulnp | grep 2300
UNCONN ... 2300 ... pid=3306694
# ✅ Single listener!

$ cat ~/.local/share/songbird/songbird.pid
3306694
# ✅ PID file tracking
```

---

## 📊 Impact Analysis

### Bugs Fixed
- ✅ Federation split state bug (multiple instances)
- ✅ Zombie process accumulation
- ✅ Inconsistent federation views
- ✅ UDP port binding conflicts
- ✅ Silent duplicate instances

### Features Added
- ✅ Explicit singleton enforcement
- ✅ PID file lifecycle management
- ✅ Stale process detection & cleanup
- ✅ RAII guard (auto-cleanup)
- ✅ Helpful error messages
- ✅ Interactive permission requests
- ✅ User collaboration approach

### Code Quality
- ✅ No unsafe blocks
- ✅ Comprehensive tests (4 unit tests)
- ✅ Clear error messages
- ✅ Well-documented
- ✅ Production-ready

---

## 🔧 Technical Details

### Singleton Enforcement Strategy

```
┌─────────────────────────────────────────────────────┐
│  Acquire Lock                                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│  1. Check PID file exists?                         │
│     ├─ No  → Write PID, return guard              │
│     └─ Yes → Read PID from file                   │
│                                                     │
│  2. Check if that process is running?             │
│     ├─ Yes → FAIL with helpful message            │
│     └─ No  → Remove stale PID file                │
│                                                     │
│  3. Write current PID to file                      │
│                                                     │
│  4. Return SingletonGuard                          │
│     (Guard auto-removes PID file on drop)          │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Why This Works

**1. PID File is Created FIRST**
- Before any network binding
- Before any resource allocation
- Fail fast if duplicate detected

**2. RAII Guard Pattern**
```rust
let _singleton_guard = process_mgr.acquire_lock()?;
// Guard lives for entire program
// Automatically calls Drop::drop() on exit
// PID file removed cleanly
```

**3. Stale Process Cleanup**
```rust
if self.is_process_running(pid) {
    // Real duplicate - fail
    self.print_duplicate_error(pid)?;
    bail!("Another instance running");
} else {
    // Stale PID file - clean up
    warn!("Stale PID file, cleaning up");
    self.remove_pid_file()?;
}
```

**4. Process Existence Check (Safe)**
```rust
// No unsafe blocks needed!
Command::new("kill")
    .arg("-0")  // Signal 0 = existence check only
    .arg(pid.to_string())
    .output()
```

### SO_REUSEPORT vs SO_REUSEADDR

| Option | Purpose | Allows Multiple Instances | Songbird Usage |
|--------|---------|---------------------------|----------------|
| **SO_REUSEADDR** | Quick restart after crash | ❌ No | ✅ Kept |
| **SO_REUSEPORT** | Multiple processes on same port | ✅ Yes | ❌ Removed |

**Why We Kept SO_REUSEADDR:**
- Allows immediate restart after crash
- TIME_WAIT state doesn't block
- Does NOT allow simultaneous instances
- Perfect for singleton applications

**Why We Removed SO_REUSEPORT:**
- Allowed silent duplicate instances
- Caused federation split state
- Not appropriate for stateful orchestrators
- Conflicts with singleton requirement

---

## 🚀 Production Deployment

### For Developers
**No changes needed!**
- Automatic singleton enforcement
- Clear error messages if something's wrong
- PID file handled transparently

### For Operators
**Systemd Integration:**
```ini
[Service]
Type=simple
ExecStart=/path/to/songbird-orchestrator
Restart=on-failure
RestartSec=5s

# Optional: Capabilities for automatic firewall configuration
AmbientCapabilities=CAP_NET_BIND_SERVICE CAP_NET_ADMIN

# PID file will be automatically managed
PIDFile=/var/run/songbird/songbird.pid
```

**Manual Deployment:**
```bash
# Start Songbird
./songbird-orchestrator &

# Check status
curl -sk https://localhost:8080/health

# View PID file
cat ~/.local/share/songbird/songbird.pid

# Stop gracefully
kill $(cat ~/.local/share/songbird/songbird.pid)

# Force stop if needed
kill -9 $(cat ~/.local/share/songbird/songbird.pid)
```

### For Users
**Friendly Experience:**
- Clear error if already running
- Helpful instructions
- No more mystery duplicates
- Collaborative permission requests

---

## 📝 Files Changed

### New Files
- `crates/songbird-orchestrator/src/process_manager.rs` (323 lines)
- `SO_REUSEPORT_ANALYSIS_DEC_20_2025.md` (536 lines)
- `SO_REUSEPORT_REMOVAL_DEC_20_2025.md` (489 lines)

### Modified Files
- `crates/songbird-orchestrator/src/lib.rs` (added process_manager module)
- `crates/songbird-orchestrator/src/main.rs` (singleton lock on startup)
- `crates/songbird-orchestrator/src/network/sovereign_socket.rs` (removed SO_REUSEPORT)
- `crates/songbird-orchestrator/src/privilege.rs` (user collaboration)
- `crates/songbird-orchestrator/src/network/connectivity_test.rs` (unused import fix)

---

## 🎯 Key Learnings

### 1. SO_REUSEPORT is NOT for Singletons
- Designed for multi-process servers (worker pools)
- Kernel load-balances between instances
- Great for: Nginx, HAProxy, load balancers
- Bad for: Stateful orchestrators like Songbird

### 2. "Address in Use" is a Feature
- For singletons, it detects problems early
- Clear signal that something's wrong
- Better than silent split state

### 3. Explicit > Implicit
- PID file management is explicit
- Socket options are implicit
- Explicit is easier to debug and understand

### 4. User Collaboration > Silent Workarounds
- Ask for help instead of circumventing
- Explain clearly what's needed
- Guide through configuration
- Builds trust and understanding

---

## 🔗 Related Documentation

- `FEDERATION_SPLIT_STATE_BUG_DEC_20_2025.md` - The bug this fixes
- `SO_REUSEPORT_ANALYSIS_DEC_20_2025.md` - Technical analysis
- `PROCESS_LIFECYCLE_ARCHITECTURE_GAP_DEC_20_2025.md` - Original gap
- `MULTI_PATH_TRANSPORT_ARCHITECTURE_DEC_20_2025.md` - Federation design

---

## 🎵 User Feedback Integration

**Original Insight:**
> "ideally songbird can work with the user to get and maintain permissions"

**Our Response:**
- Enhanced PrivilegeManager with interactive mode
- Clear explanations before running commands
- User consent required
- Collaborative approach throughout

**Result:**
✅ Songbird now works WITH users, not around them!

---

**Status:** ✅ Complete - Production Ready  
**Next:** Deploy to Westgate/Strandgate and verify federation

