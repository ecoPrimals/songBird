// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Process manager struct and constructors.

use anyhow::Result;
use std::path::PathBuf;

use super::pid_path;

/// Process manager for multi-instance support
pub struct ProcessManager {
    pub(super) pid_file: PathBuf,
    pub(super) node_identity: Option<String>,
}

impl ProcessManager {
    /// Create a new process manager with NODE_ID-scoped PID file
    ///
    /// # Errors
    ///
    /// Returns an error if the default PID file path cannot be resolved.
    pub fn new() -> Result<Self> {
        let pid_file = pid_path::default_pid_file()?;
        let node_identity = Self::get_node_identity();

        pid_path::cleanup_legacy_pid_files(&pid_file);

        Ok(Self {
            pid_file,
            node_identity,
        })
    }

    /// Create a process manager with custom PID file location
    #[must_use]
    pub fn with_pid_file(pid_file: PathBuf) -> Self {
        Self {
            pid_file,
            node_identity: Self::get_node_identity(),
        }
    }

    fn get_node_identity() -> Option<String> {
        let family = songbird_process_env::var("SONGBIRD_FAMILY_ID")
            .or_else(|_| songbird_process_env::var("FAMILY_ID"))
            .ok();
        let node = songbird_process_env::var("SONGBIRD_NODE_ID")
            .or_else(|_| songbird_process_env::var("NODE_ID"))
            .or_else(|_| songbird_process_env::var("SPORE_ID"))
            .ok();

        match (family, node) {
            (Some(f), Some(n)) => Some(format!("{f}-{n}")),
            (Some(f), None) => Some(f),
            (None, Some(n)) => Some(n),
            (None, None) => None,
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        #[expect(
            clippy::expect_used,
            reason = "`Default` cannot return `Result`; use `ProcessManager::new()` when errors must propagate"
        )]
        Self::new().expect("Failed to create default ProcessManager")
    }
}
