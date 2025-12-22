// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Process Manager - Singleton Enforcement & PID File Management
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
//  Purpose:
//    Ensures only one Songbird orchestrator instance runs at a time
//    Prevents "Federation Split State Bug" (Dec 20, 2025)
//
//  Features:
//    - PID file management
//    - Stale process detection
//    - Graceful takeover
//    - Friendly error messages
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

/// Process manager for singleton enforcement
pub struct ProcessManager {
    pid_file: PathBuf,
}

impl ProcessManager {
    /// Create a new process manager with default PID file location
    pub fn new() -> Result<Self> {
        let pid_file = Self::default_pid_file()?;
        Ok(Self {
            pid_file,
        })
    }

    /// Create a process manager with custom PID file location
    pub fn with_pid_file(pid_file: PathBuf) -> Self {
        Self {
            pid_file,
        }
    }

    /// Get the default PID file location
    ///
    /// Priority:
    /// 1. /var/run/songbird/songbird.pid (system-wide, requires root)
    /// 2. ~/.local/share/songbird/songbird.pid (user-specific)
    fn default_pid_file() -> Result<PathBuf> {
        // Try system-wide location first
        let system_path = PathBuf::from("/var/run/songbird/songbird.pid");
        if let Some(parent) = system_path.parent() {
            if parent.exists() || fs::create_dir_all(parent).is_ok() {
                return Ok(system_path);
            }
        }

        // Fall back to user-specific location
        let home = dirs::home_dir().context("Could not determine home directory")?;
        let user_path = home.join(".local/share/songbird/songbird.pid");

        if let Some(parent) = user_path.parent() {
            fs::create_dir_all(parent).context("Failed to create PID file directory")?;
        }

        Ok(user_path)
    }

    /// Acquire singleton lock
    ///
    /// This ensures only one instance can run at a time.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Another instance is already running
    /// - Cannot write PID file
    pub fn acquire_lock(&self) -> Result<SingletonGuard> {
        debug!("Attempting to acquire singleton lock: {}", self.pid_file.display());

        // Check if PID file exists
        if self.pid_file.exists() {
            let existing_pid = self.read_pid_file()?;

            // Check if that process is actually running
            if self.is_process_running(existing_pid) {
                // A real instance is running - bail out with helpful message
                self.print_duplicate_error(existing_pid)?;
                bail!("Another Songbird instance is already running (PID: {})", existing_pid);
            } else {
                // Stale PID file - clean it up
                warn!("Found stale PID file (PID {} not running), cleaning up", existing_pid);
                self.remove_pid_file()?;
            }
        }

        // Write our PID
        let current_pid = process::id();
        self.write_pid_file(current_pid)?;

        info!("✅ Singleton lock acquired (PID: {})", current_pid);
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

    /// Print a helpful error message when a duplicate is detected
    fn print_duplicate_error(&self, existing_pid: u32) -> Result<()> {
        error!("╔═══════════════════════════════════════════════════════════════════╗");
        error!("║                                                                   ║");
        error!("║  ⚠️  SONGBIRD ALREADY RUNNING                                     ║");
        error!("║                                                                   ║");
        error!("╚═══════════════════════════════════════════════════════════════════╝");
        error!("");
        error!("Another Songbird instance is already running:");
        error!("  PID: {}", existing_pid);
        error!("  PID file: {}", self.pid_file.display());
        error!("");
        error!("This prevents the 'Federation Split State Bug' where multiple");
        error!("instances create inconsistent federation views.");
        error!("");
        error!("Options:");
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

/// RAII guard for singleton lock
///
/// Automatically releases the lock (removes PID file) when dropped
pub struct SingletonGuard {
    pid_file: PathBuf,
    pid: u32,
}

impl Drop for SingletonGuard {
    fn drop(&mut self) {
        debug!("Releasing singleton lock (PID: {})", self.pid);

        if self.pid_file.exists() {
            if let Err(e) = fs::remove_file(&self.pid_file) {
                warn!("Failed to remove PID file on shutdown: {}", e);
            } else {
                info!("✅ Singleton lock released cleanly");
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
