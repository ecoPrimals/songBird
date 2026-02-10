//! Test utilities for config tests
//!
//! Provides ScopedEnv for serialized test execution of env var tests.
//!
//! **Concurrency**: Process env vars are global state. Tests that modify them
//! MUST serialize. The static mutex ensures mutual exclusion within this binary.

use std::collections::HashMap;
use std::env;
use std::sync::{Mutex, MutexGuard};

/// Global lock for env var serialization within this test binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard for environment variables
///
/// Acquires a global lock on construction and restores previous values on drop.
/// This ensures env var tests don't race with each other.
pub struct ScopedEnv {
    /// Variables to restore on drop
    restore: HashMap<String, Option<String>>,
    /// Mutex guard held for the lifetime of this scope
    _guard: MutexGuard<'static, ()>,
}

impl ScopedEnv {
    /// Create a new scoped environment manager (acquires global env lock)
    pub fn new() -> Self {
        Self {
            restore: HashMap::new(),
            _guard: ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }

    /// Set an environment variable (will be restored on drop)
    ///
    /// Returns self for method chaining.
    pub fn set(mut self, key: &str, value: &str) -> Self {
        // Save current value for restoration
        if !self.restore.contains_key(key) {
            self.restore.insert(key.to_string(), env::var(key).ok());
        }

        // Set new value
        env::set_var(key, value);

        self
    }

    /// Remove an environment variable (will be restored on drop)
    ///
    /// Returns self for method chaining.
    pub fn remove(mut self, key: &str) -> Self {
        // Save current value for restoration
        if !self.restore.contains_key(key) {
            self.restore.insert(key.to_string(), env::var(key).ok());
        }

        // Remove variable
        env::remove_var(key);

        self
    }

    /// Set multiple environment variables at once
    pub fn set_multiple(mut self, vars: &[(&str, &str)]) -> Self {
        for (key, value) in vars {
            self = self.set(key, value);
        }
        self
    }
}

impl Default for ScopedEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        // Restore all previous values
        for (key, value) in &self.restore {
            match value {
                Some(v) => env::set_var(key, v),
                None => env::remove_var(key),
            }
        }
    }
}
