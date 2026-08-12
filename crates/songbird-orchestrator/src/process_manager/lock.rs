// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Instance lock acquisition, PID file I/O, and duplicate-instance error reporting.

use anyhow::{Context, Result, bail};
use std::fs;
use std::process;
use tracing::{debug, error, info, warn};

use super::ProcessManager;
use super::guard::SingletonGuard;
use super::pid_path;

impl ProcessManager {
    /// Acquire instance lock (scoped per `NODE_ID`)
    ///
    /// # Errors
    ///
    /// Returns an error if another instance with the same identity is running,
    /// or if the PID file cannot be written.
    pub fn acquire_lock(&self) -> Result<SingletonGuard> {
        debug!("Attempting to acquire instance lock: {}", self.pid_file.display());
        if let Some(ref identity) = self.node_identity {
            debug!("   Node Identity: {}", identity);
        }

        if self.pid_file.exists() {
            let existing_pid = self.read_pid_file()?;

            if self.is_process_running(existing_pid) {
                self.print_duplicate_error(existing_pid)?;

                let identity_msg = self
                    .node_identity
                    .as_ref()
                    .map(|id| format!(" with NODE_ID={id}"))
                    .unwrap_or_default();

                bail!(
                    "Another Songbird instance{identity_msg} is already running (PID: {existing_pid})"
                );
            }
            warn!("Found stale PID file (PID {} not running), cleaning up", existing_pid);
            self.remove_pid_file()?;
        }

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

    fn read_pid_file(&self) -> Result<u32> {
        let contents = fs::read_to_string(&self.pid_file).context("Failed to read PID file")?;

        contents.trim().parse::<u32>().context("PID file contains invalid data")
    }

    fn write_pid_file(&self, pid: u32) -> Result<()> {
        fs::write(&self.pid_file, pid.to_string()).context("Failed to write PID file")?;
        Ok(())
    }

    fn remove_pid_file(&self) -> Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file).context("Failed to remove PID file")?;
        }
        Ok(())
    }

    fn print_duplicate_error(&self, existing_pid: u32) -> Result<()> {
        let identity_display = self
            .node_identity
            .as_ref()
            .map_or_else(|| String::from("NODE_ID: (not set)"), |id| format!("NODE_ID: {id}"));

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
        error!(
            "     curl -k https://{}:{}/health",
            songbird_types::constants::LOCALHOST,
            songbird_types::defaults::ports::DEFAULT_HTTP_PORT
        );
        error!("");
        error!("  3. Force kill if unresponsive:");
        error!("     kill -9 {}", existing_pid);
        error!("");

        Ok(())
    }
}

/// Expose PID path helpers for tests.
impl ProcessManager {
    #[cfg_attr(not(test), expect(dead_code, reason = "used by process_manager unit tests"))]
    pub(crate) fn default_pid_file() -> Result<std::path::PathBuf> {
        pid_path::default_pid_file()
    }
}
