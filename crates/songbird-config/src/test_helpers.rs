// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Test helpers for concurrent, safe environment variable testing
//!
//! **DEPRECATED**: This module is deprecated. Use `songbird_test_utils::ScopedEnv` instead.
//!
//! This implementation has been superseded by the async-safe version in `songbird-test-utils`
//! which properly handles tokio async tests and prevents deadlocks.
//!
//! # Migration Guide
//!
//! ```ignore
//! // Old (deprecated):
//! use songbird_config::test_helpers::ScopedEnv;
//! let _env = ScopedEnv::new("KEY", "value");
//!
//! // New (recommended):
//! use songbird_test_utils::ScopedEnv;
//! let _env = ScopedEnv::set("KEY", "value").await;
//! ```

#![deprecated(
    since = "0.1.0",
    note = "Use `songbird_test_utils::ScopedEnv` instead for async-safe environment variable testing"
)]

use songbird_process_env;
use std::sync::{Mutex, MutexGuard};

/// Global lock for environment variable tests
///
/// This ensures that tests modifying environment variables don't interfere
/// with each other when running concurrently.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard for environment variable testing
///
/// Acquires a global lock to ensure environment variable modifications
/// don't interfere with concurrent tests.
///
/// # Examples
/// ```no_run
/// use songbird_config::test_helpers::EnvironmentLock;
///
/// fn test_with_env_var() {
///     let _guard = EnvironmentLock::new();
///     songbird_process_env::set_var("MY_VAR", "value");
///     // Test code here...
///     songbird_process_env::remove_var("MY_VAR");
///     // Lock automatically released when _guard drops
/// }
/// ```
pub struct EnvironmentLock {
    _guard: MutexGuard<'static, ()>,
}

impl EnvironmentLock {
    /// Acquire the environment lock
    ///
    /// This will block if another test is currently holding the lock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _guard: ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
        }
    }
}

impl Default for EnvironmentLock {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped environment variable setter
///
/// Sets an environment variable and automatically removes it when dropped.
///
/// # Examples
/// ```no_run
/// use songbird_config::test_helpers::ScopedEnv;
///
/// fn test_with_scoped_env() {
///     let _env = ScopedEnv::new("MY_VAR", "value");
///     assert_eq!(std::env::var("MY_VAR").unwrap(), "value");
///     // Variable automatically removed when _env drops
/// }
/// ```
pub struct ScopedEnv {
    key: String,
    original: Option<String>,
}

impl ScopedEnv {
    /// Set an environment variable for the scope
    pub fn new(key: impl Into<String>, value: impl AsRef<str>) -> Self {
        let key = key.into();
        let original = std::env::var(&key).ok();
        songbird_process_env::set_var(&key, value.as_ref());
        Self {
            key,
            original,
        }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => songbird_process_env::set_var(&self.key, value),
            None => songbird_process_env::remove_var(&self.key),
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions and harness ergonomics")] // Test code - unwraps are acceptable for clarity

    use super::*;

    #[test]
    fn test_scoped_env_sets_and_removes() {
        let key = "SONGBIRD_TEST_SCOPED_ENV";
        songbird_process_env::remove_var(key);

        {
            let _env = ScopedEnv::new(key, "test_value");
            assert_eq!(std::env::var(key).unwrap(), "test_value");
        }

        assert!(std::env::var(key).is_err());
    }

    #[test]
    fn test_scoped_env_restores_original() {
        let key = "SONGBIRD_TEST_SCOPED_RESTORE";
        songbird_process_env::set_var(key, "original");

        {
            let _env = ScopedEnv::new(key, "temporary");
            assert_eq!(std::env::var(key).unwrap(), "temporary");
        }

        assert_eq!(std::env::var(key).unwrap(), "original");
        songbird_process_env::remove_var(key);
    }

    #[test]
    fn test_environment_lock_serializes_access() {
        // This test verifies that the lock works, though we can't
        // truly test concurrency in a single thread
        let _lock1 = EnvironmentLock::new();
        // If this didn't block, a second lock would deadlock
        drop(_lock1);
        let _lock2 = EnvironmentLock::new();
    }
}
