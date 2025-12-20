# Process Lifecycle Management - Architectural Debt
**Date:** December 20, 2025  
**Priority:** High  
**Category:** Deployment Robustness

## 🎯 Problem Statement

During federation testing and deployment, we repeatedly encountered issues with:
1. **Zombie processes** - Old Songbird instances not properly terminated
2. **Port conflicts** - Multiple instances trying to bind to same ports
3. **Duplicate instances** - No singleton enforcement
4. **Manual cleanup required** - `pkill -9` as primary recovery mechanism

### User's Vision

The user identified a key architectural evolution:
> "maybe a songbird can id and take over/kill another songbird. even spawn new subinstances."

This points to **self-healing, self-managing processes** - a fundamental capability for production systems.

## 🔍 Current Gaps

### 1. No PID File Management
**Missing:** Standard Unix daemon pattern
```rust
// What we need:
// - Write PID to /var/run/songbird.pid (or ~/.local/share/songbird/songbird.pid)
// - Check PID file on startup
// - Verify process is actually running
// - Clean up stale PID files
```

### 2. No Singleton Enforcement
**Missing:** Automatic duplicate detection
```rust
// Desired behavior:
// 1. On startup, check if another instance exists
// 2. If yes:
//    a) Is it healthy? → Exit gracefully
//    b) Is it zombie? → Take over (kill + replace)
// 3. If no: Proceed normally
```

### 3. No Graceful Shutdown
**Missing:** Signal handlers for clean termination
```rust
// What we need:
// - SIGTERM handler: Graceful shutdown
// - SIGINT handler: Graceful shutdown
// - Cleanup: Close sockets, update PID file, notify federation
```

### 4. No Port Conflict Resolution
**Current:** Port fallback works, but doesn't clean up conflict source
**Desired:** Identify and optionally terminate conflicting process

### 5. No Sub-Instance Management
**Missing:** Ability to spawn and manage worker processes
```rust
// User's vision:
// - Main Songbird spawns sub-instances for subsystems
// - Each sub-instance has own identity but reports to parent
// - Parent manages lifecycle of children
// - Federation of federations pattern
```

## 📋 Real-World Impact

### Issues Encountered in Session

1. **Eastgate Zombie Processes (Multiple Times)**
   ```bash
   # Found 2-4 Songbird processes running simultaneously
   ps aux | grep songbird-orchestrator
   # Had to manually kill with pkill -9
   ```
   **Impact:** Port conflicts, federation duplication, deployment confusion

2. **Port 2300 Already in Use**
   ```
   ERROR: Anonymous discovery listener error: Address already in use (os error 98)
   ```
   **Cause:** Old instance still holding UDP discovery port  
   **Resolution:** Manual `kill -9`

3. **Silent Failures**
   - Processes starting then immediately exiting
   - No logs captured (redirected to /dev/null or lost)
   - Had to run in foreground to see errors

4. **Restart Ambiguity**
   - Is the new process the only one?
   - Did the old one actually stop?
   - Which PID is the "real" Songbird?

## ✅ Proposed Solutions

### Phase 1: Basic Lifecycle Management

#### 1.1 PID File Implementation
```rust
// crates/songbird-orchestrator/src/lifecycle/pid_manager.rs

pub struct PidManager {
    pid_file: PathBuf,
}

impl PidManager {
    /// Check if another instance is running
    pub fn check_existing() -> Result<Option<u32>> {
        // Read PID file
        // Check if process exists (kill -0)
        // Return PID if alive, None if stale
    }
    
    /// Claim ownership (write our PID)
    pub fn claim() -> Result<()> {
        // Write current PID to file
        // Set file permissions
    }
    
    /// Release ownership on clean exit
    pub fn release() -> Result<()> {
        // Remove PID file
    }
}
```

#### 1.2 Singleton Enforcement
```rust
// In main.rs startup:

match PidManager::check_existing()? {
    Some(existing_pid) => {
        println!("⚠️  Songbird already running (PID: {})", existing_pid);
        println!("   Checking health...");
        
        if is_healthy(existing_pid) {
            eprintln!("❌ Healthy instance already running. Exiting.");
            std::process::exit(1);
        } else {
            println!("⚠️  Existing instance is zombie. Taking over...");
            kill_process(existing_pid)?;
            PidManager::claim()?;
        }
    }
    None => {
        PidManager::claim()?;
    }
}
```

#### 1.3 Signal Handlers
```rust
use tokio::signal;

async fn graceful_shutdown(pid_manager: PidManager) {
    let mut sigterm = signal::unix::signal(SignalKind::terminate())?;
    let mut sigint = signal::unix::signal(SignalKind::interrupt())?;
    
    tokio::select! {
        _ = sigterm.recv() => info!("Received SIGTERM, shutting down..."),
        _ = sigint.recv() => info!("Received SIGINT, shutting down..."),
    }
    
    // Cleanup
    pid_manager.release()?;
    // Notify federation we're leaving
    // Close all connections
    // Exit gracefully
}
```

### Phase 2: Advanced Process Management

#### 2.1 Port Conflict Detection & Resolution
```rust
/// Check if port is in use and by whom
pub fn check_port_conflict(port: u16) -> Option<ProcessInfo> {
    // Use lsof or /proc/net/tcp to identify process
    // Return process info if found
}

/// Optionally terminate conflicting process
pub fn resolve_port_conflict(port: u16, force: bool) -> Result<()> {
    if let Some(process) = check_port_conflict(port) {
        if process.name == "songbird-orchestrator" {
            // It's us! Zombie instance. Safe to kill.
            kill_process(process.pid)?;
        } else if force {
            // It's something else. Kill if forced.
            warn!("Killing {} (PID {}) to free port {}", process.name, process.pid, port);
            kill_process(process.pid)?;
        } else {
            // Let port fallback handle it
            return Err(anyhow!("Port conflict: {} using port {}", process.name, port));
        }
    }
    Ok(())
}
```

#### 2.2 Sub-Instance Spawning
```rust
pub struct SongbirdParent {
    children: HashMap<String, ChildInstance>,
}

pub struct ChildInstance {
    name: String,
    pid: u32,
    purpose: InstancePurpose,
    status: InstanceStatus,
}

pub enum InstancePurpose {
    Orchestrator,      // Main instance
    ComputeWorker,     // CPU-bound tasks
    GpuWorker,         // GPU tasks
    StorageManager,    // I/O tasks
    FederationBridge,  // Inter-node comms
}

impl SongbirdParent {
    /// Spawn a specialized sub-instance
    pub fn spawn_child(&mut self, purpose: InstancePurpose) -> Result<ChildInstance> {
        // Fork or spawn new process
        // Configure child for specific purpose
        // Track in parent
    }
    
    /// Monitor child health
    pub fn monitor_children(&self) {
        // Check each child's status
        // Restart if crashed
        // Report to federation
    }
}
```

### Phase 3: Self-Healing Capabilities

#### 3.1 Health Monitoring
```rust
pub struct HealthMonitor {
    last_heartbeat: Instant,
    error_count: u32,
    memory_usage: u64,
}

impl HealthMonitor {
    /// Check if this instance is healthy
    pub fn self_check(&self) -> HealthStatus {
        // Check memory usage
        // Check response time
        // Check error rate
        // Check federation connectivity
    }
    
    /// Determine if we should self-terminate
    pub fn should_restart(&self) -> bool {
        // Too many errors? Restart.
        // Memory leak detected? Restart.
        // Federation connection lost? Restart.
    }
}
```

#### 3.2 Automatic Recovery
```rust
/// Self-healing loop
async fn self_healing_loop(health_monitor: Arc<HealthMonitor>) {
    loop {
        sleep(Duration::from_secs(60)).await;
        
        if health_monitor.should_restart() {
            warn!("🔄 Self-healing: Restarting due to health issues");
            
            // Spawn replacement
            spawn_replacement_instance()?;
            
            // Wait for replacement to be healthy
            sleep(Duration::from_secs(5)).await;
            
            // Gracefully exit this instance
            std::process::exit(0);
        }
    }
}
```

## 📈 Benefits

### Immediate (Phase 1)
- ✅ No more zombie processes
- ✅ Clean restarts without manual intervention
- ✅ Port conflicts auto-detected
- ✅ Graceful shutdowns

### Medium-term (Phase 2)
- ✅ Automatic port conflict resolution
- ✅ Sub-instance specialization
- ✅ Better resource utilization
- ✅ Federation of federations

### Long-term (Phase 3)
- ✅ Self-healing capabilities
- ✅ Zero-downtime updates
- ✅ Automatic recovery from failures
- ✅ Production-grade reliability

## 🎯 Implementation Priority

### High Priority (Next Session)
1. PID file management
2. Singleton enforcement
3. Graceful shutdown handlers

### Medium Priority
4. Port conflict detection
5. Better error logging
6. Health monitoring

### Future Evolution
7. Sub-instance spawning
8. Self-healing capabilities
9. Zero-downtime updates

## 🎓 Architectural Principles

### 1. **Defensive by Default**
Assume there might be other instances. Check first, act second.

### 2. **Self-Aware**
Songbird should know:
- Is it the only instance?
- Is it healthy?
- Should it restart?

### 3. **Cooperative**
Multiple instances should:
- Detect each other
- Negotiate (if appropriate)
- Hand off gracefully

### 4. **Fail Loudly**
Don't fail silently:
- Log errors
- Notify federation
- Leave audit trail

## 📚 References

### Inspiration
- **systemd:** Modern service management
- **supervisord:** Process monitoring
- **PM2:** Node.js process manager
- **Docker:** Container lifecycle management

### Rust Ecosystem
- `daemonize` crate: Daemon infrastructure
- `tokio::signal`: Async signal handling
- `sysinfo` crate: Process information
- `nix` crate: Unix system calls

## 🎉 Conclusion

Process lifecycle management is NOT just a "nice to have" - it's **essential for production deployments**. The issues we encountered (zombies, port conflicts, manual cleanup) are symptoms of missing infrastructure.

The user's vision of "songbird taking over another songbird" is architecturally sound and points toward a self-managing, self-healing system - exactly what production systems need.

**Status:** Architectural debt documented. Ready for implementation in next iteration.

---

*"A system that cannot manage itself is a system that requires constant human intervention. True autonomy requires self-awareness and self-management."*

