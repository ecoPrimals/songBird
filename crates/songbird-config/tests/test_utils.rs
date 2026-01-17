//! Test utilities for config tests
//!
//! Provides ScopedEnv for concurrent test execution without #[serial]

use std::collections::HashMap;
use std::env;

/// RAII guard for environment variables
///
/// Automatically restores previous values on drop.
/// Enables concurrent tests without #[serial].
pub struct ScopedEnv {
    /// Variables to restore on drop
    restore: HashMap<String, Option<String>>,
}

impl ScopedEnv {
    /// Create a new scoped environment manager
    pub fn new() -> Self {
        Self {
            restore: HashMap::new(),
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

