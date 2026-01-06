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
        let family = std::env::var("SONGBIRD_FAMILY_ID")
            .or_else(|_| std::env::var("FAMILY_ID"))
            .ok();
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
    /// - `/var/run/songbird/songbird-nat0-tower1.pid` (multi-spore)
    /// - `/var/run/songbird/songbird-albatross-main.pid` (Albatross)
    /// - `/var/run/songbird/songbird-sparrow-iot1.pid` (Sparrow fleet)
    /// - `/var/run/songbird/songbird.pid` (legacy fallback)
    ///
    /// Priority:
    /// 1. /var/run/songbird/songbird-{family}-{node}.pid (system-wide, requires root)
    /// 2. ~/.local/share/songbird/songbird-{family}-{node}.pid (user-specific)
    fn default_pid_file() -> Result<PathBuf> {
        // Get FAMILY_ID and NODE_ID from environment
        let family_id = std::env::var("SONGBIRD_FAMILY_ID")
            .or_else(|_| std::env::var("FAMILY_ID"))
            .ok();
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
        
        // Try system-wide location first
        let system_path = PathBuf::from("/var/run/songbird").join(&filename);
        if let Some(parent) = system_path.parent() {
            if parent.exists() || fs::create_dir_all(parent).is_ok() {
                return Ok(system_path);
            }
        }

        // Fall back to user-specific location
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
                
                let identity_msg = self.node_identity.as_ref()
                    .map(|id| format!(" with NODE_ID={}", id))
                    .unwrap_or_default();
                
                bail!("Another Songbird instance{} is already running (PID: {})", identity_msg, existing_pid);
            } else {
                // Stale PID file - clean it up
                warn!("Found stale PID file (PID {} not running), cleaning up", existing_pid);
                self.remove_pid_file()?;
            }
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

    /// Check if a process is running
    ///
    /// This is platform-specific:
    /// - On Unix: Uses kill(pid, 0) to check existence
    /// - On Windows: Would need different approach
    fn is_process_running(&self, pid: u32) -> bool {
        #[cfg(unix)]
        {
            // Try to send signal 0 (existence check, doesn't actually send signal)
            // This is safe and doesn't require unsafe blocks
            let status = std::process::Command::new("kill").arg("-0").arg(pid.to_string()).output();

            match status {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, we'd need a different approach
            // For now, assume it's running (safer to reject than allow duplicates)
            warn!("Process existence check not implemented on this platform");
            true
        }
    }

    /// Print a helpful error message when a duplicate NODE_ID is detected
    fn print_duplicate_error(&self, existing_pid: u32) -> Result<()> {
        let identity_display = self.node_identity.as_ref()
            .map(|id| format!("NODE_ID: {}", id))
            .unwrap_or_else(|| "NODE_ID: (not set)".to_string());
        
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
        error!("   export SONGBIRD_FAMILY_ID=nat0");
        error!("   export SONGBIRD_NODE_ID=tower1");
        error!("   songbird &");
        error!("");
        error!("   # Spore 2 (different NODE_ID!)");
        error!("   export SONGBIRD_FAMILY_ID=nat0");
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
        Self::new().expect("Failed to create default ProcessManager")
    }
}

/// RAII guard for instance lock
///
/// Automatically releases the lock (removes PID file) when dropped.
/// This ensures clean shutdown even in case of panics.
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
}
