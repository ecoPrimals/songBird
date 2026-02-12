// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Process Manager - Multi-Instance Support with NODE_ID Scoping
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
//  Purpose:
//    Ensures only one Songbird instance per NODE_ID runs at a time
//    Supports multi-instance deployments (Albatross, Sparrow flocks, etc.)
//
//  Evolution from v3.7.2:
//    - Was: Global singleton (blocked multi-spore)
//    - Now: NODE_ID-scoped singleton (enables fractal scaling)
//
//  Features:
//    - PID file management (scoped per FAMILY_ID + NODE_ID)
//    - Stale process detection
//    - Graceful takeover
//    - Multi-instance support (Albatross, Sparrow, etc.)
//    - Friendly error messages
//
//  Design Philosophy:
//    "Songbirds can take many forms: singleton Songbird, Albatross multiplexer,
//     or flocks of Sparrows for IoT. Each has its own identity but can coordinate,
//     form hierarchies, or subspawn as needed."
//
//  User Collaboration:
//    Instead of silently failing or allowing duplicates, we:
//    - Clearly explain what's wrong
//    - Show who's running
//    - Offer to help
//    - Guide next steps
//
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process;
use tracing::{debug, error, info, warn};

/// Process manager for multi-instance support
///
/// Each Songbird instance (identified by FAMILY_ID + NODE_ID) gets its own PID file.
/// This enables:
/// - Multi-spore deployments (multiple Songbirds per machine)
/// - Albatross multiplexer instances
/// - Sparrow IoT flocks
/// - Any other Songbird variant
pub struct ProcessManager {
    pid_file: PathBuf,
    node_identity: Option<String>, // For error messages
}

impl ProcessManager {
    /// Create a new process manager with NODE_ID-scoped PID file
    ///
    /// This automatically reads SONGBIRD_FAMILY_ID and SONGBIRD_NODE_ID from the environment
    /// to create a unique PID file per instance, enabling multi-instance deployments.
    pub fn new() -> Result<Self> {
        let pid_file = Self::default_pid_file()?;
        let node_identity = Self::get_node_identity();

        Ok(Self {
            pid_file,
            node_identity,
        })
    }

    /// Create a process manager with custom PID file location
    pub fn with_pid_file(pid_file: PathBuf) -> Self {
        Self {
            pid_file,
            node_identity: Self::get_node_identity(),
        }
    }

    /// Get node identity from environment (for error messages)
    fn get_node_identity() -> Option<String> {
        let family =
            std::env::var("SONGBIRD_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID")).ok();
        let node = std::env::var("SONGBIRD_NODE_ID")
            .or_else(|_| std::env::var("NODE_ID"))
            .or_else(|_| std::env::var("SPORE_ID"))
            .ok();

        match (family, node) {
            (Some(f), Some(n)) => Some(format!("{}-{}", f, n)),
            (Some(f), None) => Some(f),
            (None, Some(n)) => Some(n),
            (None, None) => None,
        }
    }

    /// Get the default PID file location
    ///
    /// PID file path is scoped by FAMILY_ID and NODE_ID to allow multiple instances:
    ///
    /// Examples:
    /// - `/var/run/songbird/songbird-{family_id}-tower1.pid` (multi-spore)
    /// - `/var/run/songbird/songbird-albatross-main.pid` (Albatross)
    /// - `/var/run/songbird/songbird-sparrow-iot1.pid` (Sparrow fleet)
    /// - `/var/run/songbird/songbird.pid` (legacy fallback)
    ///
    /// Priority:
    /// 1. $SONGBIRD_PID_DIR/songbird-{family}-{node}.pid (explicit override, Android)
    /// 2. /var/run/songbird/songbird-{family}-{node}.pid (system-wide, requires root)
    /// 3. ~/.local/share/songbird/songbird-{family}-{node}.pid (user-specific)
    fn default_pid_file() -> Result<PathBuf> {
        // Get FAMILY_ID and NODE_ID from environment
        let family_id =
            std::env::var("SONGBIRD_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID")).ok();
        let node_id = std::env::var("SONGBIRD_NODE_ID")
            .or_else(|_| std::env::var("NODE_ID"))
            .or_else(|_| std::env::var("SPORE_ID"))
            .ok();

        // Build filename suffix based on available IDs
        let filename_suffix = match (family_id.as_ref(), node_id.as_ref()) {
            (Some(family), Some(node)) => format!("-{}-{}", family, node),
            (Some(family), None) => format!("-{}", family),
            (None, Some(node)) => format!("-{}", node),
            (None, None) => String::new(), // Legacy fallback
        };

        let filename = format!("songbird{}.pid", filename_suffix);

        // Priority 1: Explicit override via SONGBIRD_PID_DIR (for Android/restricted environments)
        if let Ok(pid_dir) = std::env::var("SONGBIRD_PID_DIR") {
            let custom_path = PathBuf::from(&pid_dir).join(&filename);
            if let Some(parent) = custom_path.parent() {
                if fs::create_dir_all(parent).is_ok() {
                    debug!("Using SONGBIRD_PID_DIR: {}", custom_path.display());
                    return Ok(custom_path);
                }
            }
        }

        // Priority 2: Try system-wide location first
        let system_path = PathBuf::from("/var/run/songbird").join(&filename);
        if let Some(parent) = system_path.parent() {
            if parent.exists() || fs::create_dir_all(parent).is_ok() {
                return Ok(system_path);
            }
        }

        // Priority 3: Fall back to user-specific location
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let user_path = home.join(".local/share/songbird").join(&filename);

        if let Some(parent) = user_path.parent() {
            fs::create_dir_all(parent).context("Failed to create PID file directory")?;
        }

        Ok(user_path)
    }

    /// Acquire instance lock (scoped per NODE_ID)
    ///
    /// This ensures only one instance **with this specific NODE_ID** can run at a time.
    /// Multiple instances with different NODE_IDs can run simultaneously.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Another instance **with the same NODE_ID** is already running
    /// - Cannot write PID file
    pub fn acquire_lock(&self) -> Result<SingletonGuard> {
        debug!("Attempting to acquire instance lock: {}", self.pid_file.display());
        if let Some(ref identity) = self.node_identity {
            debug!("   Node Identity: {}", identity);
        }

        // Check if PID file exists
        if self.pid_file.exists() {
            let existing_pid = self.read_pid_file()?;

            // Check if that process is actually running
            if self.is_process_running(existing_pid) {
                // A real instance is running - bail out with helpful message
                self.print_duplicate_error(existing_pid)?;

                let identity_msg = self
                    .node_identity
                    .as_ref()
                    .map(|id| format!(" with NODE_ID={}", id))
                    .unwrap_or_default();

                bail!(
                    "Another Songbird instance{} is already running (PID: {})",
                    identity_msg,
                    existing_pid
                );
            }
            // Stale PID file - clean it up
            warn!("Found stale PID file (PID {} not running), cleaning up", existing_pid);
            self.remove_pid_file()?;
        }

        // Write our PID
        let current_pid = process::id();
        self.write_pid_file(current_pid)?;

        info!("✅ Instance lock acquired (PID: {})", current_pid);
        if let Some(ref identity) = self.node_identity {
            info!("   Node Identity: {}", identity);
        }
        info!("   PID file: {}", self.pid_file.display());

        Ok(SingletonGuard {
            pid_file: self.pid_file.clone(),
            pid: current_pid,
        })
    }

    /// Read PID from file
    fn read_pid_file(&self) -> Result<u32> {
        let contents = fs::read_to_string(&self.pid_file).context("Failed to read PID file")?;

        let pid: u32 = contents.trim().parse().context("PID file contains invalid data")?;

        Ok(pid)
    }

    /// Write PID to file
    fn write_pid_file(&self, pid: u32) -> Result<()> {
        fs::write(&self.pid_file, pid.to_string()).context("Failed to write PID file")?;
        Ok(())
    }

    /// Remove PID file
    fn remove_pid_file(&self) -> Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file).context("Failed to remove PID file")?;
        }
        Ok(())
    }

    /// Check if a process is running and healthy (v3.17.0)
    ///
    /// **Modern Idiomatic Rust**: Distinguishes zombies from healthy processes
    ///
    /// **Deep Debt Solved** (Jan 7, 2026):
    /// Previous implementation used `kill -0` which returns success for zombie processes,
    /// blocking new deployments even when old process was defunct.
    ///
    /// **Evolution**:
    /// - Checks `/proc/{pid}/stat` for process state (Z = zombie)
    /// - Zombies are treated as "not running" (stale)
    /// - Enables automatic cleanup and graceful takeover
    /// - Production-ready: handles crashes, orphans, defunct processes
    ///
    /// This is platform-specific:
    /// - On Unix: Uses `/proc/{pid}/stat` to check state, falls back to `kill -0`
    /// - On Windows: Would need different approach (WMI or tasklist)
    fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            // Step 1: Check /proc/{pid}/stat for process state (v3.17.0)
            let stat_path = format!("/proc/{}/stat", pid);
            if let Ok(contents) = fs::read_to_string(&stat_path) {
                // Parse state from /proc/{pid}/stat
                // Format: pid (comm) state ppid pgrp session tty_nr tpgid flags ...
                // State: R (running), S (sleeping), D (disk sleep), Z (zombie), T (stopped), t (tracing stop), W (paging), X (dead), x (dead), K (wakekill), W (waking), P (parked)

                // Find the closing parenthesis after the command name
                // This is important because command name can contain spaces and special chars
                if let Some(state_pos) = contents.rfind(')') {
                    // State is the character after ") " (position + 2)
                    if let Some(state_char) = contents[state_pos + 2..].chars().next() {
                        match state_char {
                            'Z' => {
                                // Zombie process - treat as not running (defunct)
                                warn!(
                                    "⚠️  PID {} is a zombie process (defunct), treating as stale. \
                                    This allows graceful takeover by new deployment.",
                                    pid
                                );
                                return false; // ✅ Zombies are stale!
                            }
                            'X' | 'x' => {
                                // Dead process
                                debug!("PID {} is marked as dead", pid);
                                return false;
                            }
                            'R' | 'S' | 'D' | 'I' => {
                                // Real running process:
                                // R = running, S = sleeping (interruptible)
                                // D = sleeping (uninterruptible, usually I/O)
                                // I = idle (kernel thread)
                                debug!("✅ PID {} is healthy (state: {})", pid, state_char);
                                return true;
                            }
                            'T' | 't' => {
                                // Stopped (SIGSTOP) or tracing stop
                                // Treat as unhealthy for our purposes (can't serve requests)
                                warn!(
                                    "⚠️  PID {} is stopped/traced (state: {}), treating as unhealthy",
                                    pid, state_char
                                );
                                return false;
                            }
                            _ => {
                                // Unknown state - be conservative, check with kill
                                debug!(
                                    "Unknown process state '{}' for PID {}, using fallback check",
                                    state_char, pid
                                );
                            }
                        }
                    }
                }
            } else {
                // /proc/{pid}/stat doesn't exist - process is gone
                debug!("PID {} does not exist (/proc entry missing)", pid);
                return false;
            }

            // Step 2: Fallback to kill -0 if /proc parsing failed or state unknown
            // This maintains backward compatibility with non-Linux Unix systems
            debug!("Using fallback kill -0 check for PID {}", pid);
            let status = std::process::Command::new("kill").arg("-0").arg(pid.to_string()).output();

            match status {
                Ok(output) => {
                    let is_running = output.status.success();
                    if is_running {
                        // Note: This could be a zombie if /proc parsing failed
                        debug!("PID {} exists (kill -0 success)", pid);
                    }
                    is_running
                }
                Err(_) => false,
            }
        }

        #[cfg(not(unix))]
        {
            // Windows: Platform not supported (Linux/Unix focus)
            // Songbird targets Linux/Unix environments with XDG runtime directories.
            // For Windows support, see Phase 2 roadmap or use WSL2.
            // For now, assume it's running (safer to reject than allow duplicates)
            warn!("Process existence check not implemented on this platform, assuming running");
            true
        }
    }

    /// Print a helpful error message when a duplicate NODE_ID is detected
    fn print_duplicate_error(&self, existing_pid: u32) -> Result<()> {
        let identity_display = self
            .node_identity
            .as_ref()
            .map_or_else(|| "NODE_ID: (not set)".to_string(), |id| format!("NODE_ID: {}", id));

        error!("╔═══════════════════════════════════════════════════════════════════╗");
        error!("║                                                                   ║");
        error!("║  ⚠️  SONGBIRD INSTANCE ALREADY RUNNING                            ║");
        error!("║                                                                   ║");
        error!("╚═══════════════════════════════════════════════════════════════════╝");
        error!("");
        error!("Another Songbird instance with the same identity is running:");
        error!("  PID: {}", existing_pid);
        error!("  {}", identity_display);
        error!("  PID file: {}", self.pid_file.display());
        error!("");
        error!("This prevents multiple instances with the same NODE_ID from");
        error!("creating inconsistent state.");
        error!("");
        error!("💡 To run multiple Songbird instances on this machine:");
        error!("   Set unique SONGBIRD_NODE_ID for each instance:");
        error!("");
        error!("   # Spore 1");
        error!("   export SONGBIRD_FAMILY_ID=my-family");
        error!("   export SONGBIRD_NODE_ID=tower1");
        error!("   songbird &");
        error!("");
        error!("   # Spore 2 (different NODE_ID!)");
        error!("   export SONGBIRD_FAMILY_ID=my-family");
        error!("   export SONGBIRD_NODE_ID=tower2");
        error!("   songbird &");
        error!("");
        error!("Options for this instance:");
        error!("  1. Stop the existing instance:");
        error!("     kill {}", existing_pid);
        error!("");
        error!("  2. Check if it's healthy:");
        error!("     ps aux | grep {}", existing_pid);
        error!("     curl -k https://localhost:8080/health");
        error!("");
        error!("  3. Force kill if unresponsive:");
        error!("     kill -9 {}", existing_pid);
        error!("");

        Ok(())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        #[allow(clippy::expect_used)] // Default impl must succeed or is a fatal misconfiguration
        Self::new().expect("Failed to create default ProcessManager")
    }
}

/// RAII guard for instance lock
///
/// Automatically releases the lock (removes PID file) when dropped.
/// This ensures clean shutdown even in case of panics.
#[derive(Debug)]
pub struct SingletonGuard {
    pid_file: PathBuf,
    pid: u32,
}

impl Drop for SingletonGuard {
    fn drop(&mut self) {
        debug!("Releasing instance lock (PID: {})", self.pid);

        if self.pid_file.exists() {
            if let Err(e) = fs::remove_file(&self.pid_file) {
                warn!("Failed to remove PID file on shutdown: {}", e);
            } else {
                info!("✅ Instance lock released cleanly");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_pid_file_location() {
        let path = ProcessManager::default_pid_file().unwrap();
        assert!(path.to_string_lossy().contains("songbird"));
        assert!(path.to_string_lossy().ends_with(".pid"));
    }

    #[test]
    fn test_singleton_enforcement() {
        let temp_dir = env::temp_dir();
        let pid_file = temp_dir.join(format!("songbird_test_{}.pid", process::id()));

        // Clean up any stale file
        let _ = fs::remove_file(&pid_file);

        let manager = ProcessManager::with_pid_file(pid_file.clone());

        // First lock should succeed
        let _guard1 = manager.acquire_lock().expect("First lock should succeed");

        // Second lock should fail
        let result = manager.acquire_lock();
        assert!(result.is_err());

        // Guard drops here, releasing lock
        drop(_guard1);

        // Now we should be able to acquire again
        let _guard2 = manager.acquire_lock().expect("Lock should be available after drop");
    }

    #[test]
    fn test_stale_pid_cleanup() {
        let temp_dir = env::temp_dir();
        let pid_file = temp_dir.join(format!("songbird_stale_{}.pid", process::id()));

        // Create a stale PID file with a definitely-not-running PID
        fs::write(&pid_file, "999999").unwrap();

        let manager = ProcessManager::with_pid_file(pid_file.clone());

        // Should succeed by cleaning up stale file
        let _guard = manager.acquire_lock().expect("Should clean up stale PID");
    }

    #[test]
    fn test_process_running_check() {
        let manager = ProcessManager::new().unwrap();

        // Current process should be running
        let current_pid = process::id();
        assert!(manager.is_process_running(current_pid));

        // PID 999999 should not be running
        assert!(!manager.is_process_running(999999));
    }

    /// Test zombie detection (v3.17.0)
    ///
    /// This test verifies the evolved `is_process_running()` can distinguish
    /// zombies from healthy processes.
    ///
    /// **Note**: Creating actual zombies in tests is complex (requires fork() and unsafe),
    /// so this test verifies the /proc parsing logic indirectly by testing known PIDs.
    #[test]
    #[cfg(unix)]
    fn test_zombie_detection_logic() {
        let manager = ProcessManager::new().unwrap();

        // Test 1: Current process should be running (not a zombie)
        let current_pid = process::id();
        assert!(
            manager.is_process_running(current_pid),
            "Current process should be detected as running"
        );

        // Test 2: Non-existent PID should not be running
        assert!(
            !manager.is_process_running(999999),
            "Non-existent PID should not be detected as running"
        );

        // Test 3: PID 1 (init/systemd) should always be running
        // (unless running in a container without PID 1)
        let pid_1_exists = fs::read_to_string("/proc/1/stat").is_ok();
        if pid_1_exists {
            assert!(manager.is_process_running(1), "PID 1 (init/systemd) should be running");
        }
    }

    /// Test /proc/pid/stat parsing for various process states (v3.17.0)
    #[test]
    #[cfg(unix)]
    fn test_proc_stat_parsing() {
        // This test verifies our parsing logic handles various /proc formats correctly

        // Example /proc/pid/stat format:
        // 12345 (process name) R 1 12345 12345 0 -1 4194304 ...
        //                        ^ state character (R = running, S = sleeping, Z = zombie, etc.)

        // Test case 1: Running process
        let stat_running = "12345 (bash) R 1 12345 12345 0 -1 4194304 123 456 0 0 10 20 0 0 20 0 1 0 1234567 8192 100 18446744073709551615";
        let state_pos = stat_running.rfind(')').unwrap();
        let state = stat_running[state_pos + 2..].chars().next().unwrap();
        assert_eq!(state, 'R', "Should parse running state");

        // Test case 2: Sleeping process
        let stat_sleeping = "12346 (sleep) S 1 12346 12346 0 -1 4194304 123 456 0 0 10 20 0 0 20 0 1 0 1234568 8192 100 18446744073709551615";
        let state_pos = stat_sleeping.rfind(')').unwrap();
        let state = stat_sleeping[state_pos + 2..].chars().next().unwrap();
        assert_eq!(state, 'S', "Should parse sleeping state");

        // Test case 3: Zombie process
        let stat_zombie = "12347 (defunct) Z 1 12347 12347 0 -1 4194304 0 0 0 0 0 0 0 0 20 0 1 0 1234569 0 0 18446744073709551615";
        let state_pos = stat_zombie.rfind(')').unwrap();
        let state = stat_zombie[state_pos + 2..].chars().next().unwrap();
        assert_eq!(state, 'Z', "Should parse zombie state");

        // Test case 4: Process name with spaces and special chars
        let stat_complex = "12348 (my (complex) name!) R 1 12348 12348 0 -1 4194304 123 456 0 0 10 20 0 0 20 0 1 0 1234570 8192 100 18446744073709551615";
        let state_pos = stat_complex.rfind(')').unwrap();
        let state = stat_complex[state_pos + 2..].chars().next().unwrap();
        assert_eq!(state, 'R', "Should handle complex process names");
    }

    /// Test that zombies block new instances (before cleanup) (v3.17.0)
    #[test]
    fn test_zombie_allows_new_deployment() {
        let temp_dir = env::temp_dir();
        let pid_file = temp_dir.join(format!("songbird_zombie_test_{}.pid", process::id()));

        // Clean up any stale file
        let _ = fs::remove_file(&pid_file);

        // Simulate a zombie scenario:
        // 1. Write a PID file with a "zombie" PID
        // 2. is_process_running() should return false for zombies
        // 3. New deployment should succeed (zombie treated as stale)

        // For this test, we use PID 999999 which definitely doesn't exist
        // In production, is_process_running() would detect a real zombie via /proc
        fs::write(&pid_file, "999999").unwrap();

        let manager = ProcessManager::with_pid_file(pid_file.clone());

        // Should succeed because is_process_running(999999) returns false
        let result = manager.acquire_lock();
        assert!(
            result.is_ok(),
            "Should acquire lock even with 'zombie' PID file (treats as stale)"
        );
    }

    /// Test graceful takeover message (v3.17.0)
    #[test]
    fn test_helpful_error_messages() {
        let temp_dir = env::temp_dir();
        let pid_file = temp_dir.join(format!("songbird_error_test_{}.pid", process::id()));

        // Clean up
        let _ = fs::remove_file(&pid_file);

        let manager = ProcessManager::with_pid_file(pid_file.clone());

        // First lock succeeds
        let _guard1 = manager.acquire_lock().expect("First lock should succeed");

        // Second lock fails with helpful error
        let result = manager.acquire_lock();
        assert!(result.is_err());

        // Error message should mention the PID and how to resolve
        let error_msg = format!("{}", result.unwrap_err());
        assert!(
            error_msg.contains("already running") || error_msg.contains("PID"),
            "Error should explain the conflict clearly"
        );
    }
}
