// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Thread-safe environment override for concurrent testing
//!
//! This module provides a way to test environment-dependent code
//! without polluting the global process environment or requiring
//! serial test execution.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe environment override for testing
///
/// Allows concurrent tests to work with different "environments"
/// without race conditions or global state pollution.
///
/// ## Design
/// - Uses `Arc<RwLock<>>` for interior mutability
/// - Each test gets its own `EnvOverride` instance
/// - Falls back to real environment when key not found
/// - Zero global state - fully concurrent-safe
#[derive(Debug, Clone, Default)]
pub struct EnvOverride {
    vars: Arc<RwLock<HashMap<String, String>>>,
}

impl EnvOverride {
    /// Create a new environment override
    #[must_use]
    pub fn new() -> Self {
        Self {
            vars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a variable in this override
    ///
    /// # Panics
    /// Panics if the internal lock is poisoned (only happens if a thread panicked while holding the lock)
    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) {
        // Note: Lock poisoning only occurs if a panic happens while holding the lock.
        // In test code, this is acceptable. For production, use proper error handling.
        let mut vars = self.vars.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        vars.insert(key.into(), value.into());
    }

    /// Get a variable from this override, falling back to real env
    ///
    /// Returns `None` if the key is not in the override and not in the real environment.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        let vars = self.vars.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        vars.get(key).cloned().or_else(|| songbird_process_env::var(key).ok())
    }

    /// Remove a variable from this override
    ///
    /// # Panics
    /// Panics if the internal lock is poisoned (only happens if a thread panicked while holding the lock)
    pub fn remove(&self, key: &str) {
        let mut vars = self.vars.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        vars.remove(key);
    }

    /// Clear all overrides
    ///
    /// # Panics
    /// Panics if the internal lock is poisoned (only happens if a thread panicked while holding the lock)
    pub fn clear(&self) {
        let mut vars = self.vars.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        vars.clear();
    }

    /// Check if a key exists in override or real environment
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        let vars = self.vars.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        vars.contains_key(key) || songbird_process_env::var(key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_override_isolation() {
        let env1 = EnvOverride::new();
        let env2 = EnvOverride::new();

        env1.set("TEST_VAR", "value1");
        env2.set("TEST_VAR", "value2");

        assert_eq!(env1.get("TEST_VAR"), Some("value1".to_string()));
        assert_eq!(env2.get("TEST_VAR"), Some("value2".to_string()));
    }

    #[test]
    fn test_env_override_fallback() {
        let env = EnvOverride::new();

        // Should fall back to real PATH variable
        assert!(env.get("PATH").is_some());
    }

    #[test]
    fn test_env_override_concurrent() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let env = EnvOverride::new();
                    env.set("THREAD_ID", i.to_string());
                    assert_eq!(env.get("THREAD_ID"), Some(i.to_string()));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
