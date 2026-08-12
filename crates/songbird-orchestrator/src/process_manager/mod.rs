// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Process lifecycle: PID file paths, instance locking, and stale process detection.

mod guard;
mod lock;
mod manager;
mod pid_path;
mod process_health;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use guard::SingletonGuard;
pub use manager::ProcessManager;
