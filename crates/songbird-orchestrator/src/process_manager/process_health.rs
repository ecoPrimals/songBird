// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Process health checks including zombie detection via `/proc`.

use songbird_types::process_ops;

use super::ProcessManager;

impl ProcessManager {
    /// Check if a process is running and healthy (v3.17.0)
    pub(crate) fn is_process_running(&self, pid: u32) -> bool {
        process_ops::is_process_running(pid)
    }
}
