// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! RAII guard that releases the instance lock on drop.

use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// RAII guard for instance lock
///
/// Automatically releases the lock (removes PID file) when dropped.
#[derive(Debug)]
pub struct SingletonGuard {
    pub(super) pid_file: PathBuf,
    pub(super) pid: u32,
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
