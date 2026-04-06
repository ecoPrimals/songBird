// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Synchronous env locking for async unit tests (avoids holding `tokio::sync::Mutex` across `await`).

use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Process-wide lock for tests that mutate `songbird_process_env`.
#[must_use = "hold until test completes"]
#[allow(clippy::expect_used)]
pub fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().expect("test env lock poisoned")
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
