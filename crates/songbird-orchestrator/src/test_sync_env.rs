// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg(test)]

//! Re-exports the canonical process-wide env test lock from [`songbird_process_env`].
//!
//! All test modules in this crate that mutate or depend on overlay env state
//! must hold this lock for the duration of their assertions. Using a single
//! lock eliminates the class of flaky tests caused by parallel mutations
//! racing through independent per-module locks.

/// Process-wide lock for tests that mutate `songbird_process_env`.
///
/// Delegates to [`songbird_process_env::test_env_lock`] — the canonical
/// single serialization point shared by all test modules within this binary.
#[must_use = "hold until test completes"]
pub fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    songbird_process_env::test_env_lock()
}

/// RAII restore for a single env var.
pub struct VarGuard {
    key: &'static str,
    old: Option<String>,
}

impl VarGuard {
    /// Set `key` to `value`, restoring the previous value (or unset) on drop.
    pub fn set(key: &'static str, value: &str) -> Self {
        let old = songbird_process_env::var(key).ok();
        songbird_process_env::set_var(key, value);
        Self {
            key,
            old,
        }
    }

    /// Remove `key`, restoring the previous value on drop.
    pub fn remove(key: &'static str) -> Self {
        let old = songbird_process_env::var(key).ok();
        songbird_process_env::remove_var(key);
        Self {
            key,
            old,
        }
    }
}

impl Drop for VarGuard {
    fn drop(&mut self) {
        if let Some(ref v) = self.old {
            songbird_process_env::set_var(self.key, v);
        } else {
            songbird_process_env::remove_var(self.key);
        }
    }
}
