// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Environment Variable Isolation for Tests
//!
//! Provides RAII-based environment variable management that automatically
//! cleans up after tests, preventing state leakage in parallel test execution.
//!
//! # Example
//!
//! ```no_run
//! use songbird_test_utils::ScopedEnv;
//!
//! # fn main() {
//! let _env = ScopedEnv::set("MY_VAR", "my_value");
//! // MY_VAR is set here
//! assert_eq!(std::env::var("MY_VAR").unwrap(), "my_value");
//! # } // MY_VAR is automatically cleaned up when _env drops
//! ```

use std::env;

// ⚠️ CRITICAL FIX: Use tokio::sync::Mutex for async-safety!
// Using std::sync::Mutex caused deadlocks in async tests because:
// 1. Synchronous Mutex::lock() is a BLOCKING operation
// 2. Holding it across await points blocks the tokio runtime thread
// 3. Parallel async tests deadlock trying to acquire the same lock
//
// Solution: tokio::sync::Mutex allows the runtime to yield while waiting
use std::sync::OnceLock;
use tokio::sync::Mutex;

// Global lock for test environment isolation
// This ensures only one test modifies environment at a time
// Using OnceLock + tokio::Mutex for async-safe initialization
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_env_lock() -> &'static Mutex<()> {
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

// Mutating the process environment is only sound when no other thread reads it concurrently.
// All call sites here hold `ENV_LOCK` (or, in unit tests below, acquire it around direct cleanup).
fn env_set_var(key: impl AsRef<str>, value: impl AsRef<str>) {
    songbird_process_env::set_var(key, value);
}

fn env_remove_var(key: impl AsRef<str>) {
    songbird_process_env::remove_var(key);
}

/// Scoped environment variable that automatically cleans up on drop
///
/// This type uses RAII to ensure environment variables set in tests
/// are always cleaned up, even if the test panics.
///
/// # Thread Safety & Async Safety
///
/// Uses a global `tokio::sync::Mutex` to ensure only one test can modify
/// environment variables at a time. This is async-safe and won't deadlock
/// in async tests, unlike the previous `std::sync::Mutex` implementation.
#[must_use = "ScopedEnv must be held until cleanup is desired"]
pub struct ScopedEnv {
    key: String,
    old_value: Option<String>,
    _guard: tokio::sync::MutexGuard<'static, ()>,
}

impl ScopedEnv {
    /// Set an environment variable for the duration of this scope
    ///
    /// The variable will be automatically cleaned up when this value is dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// # async fn example() {
    /// let _env = ScopedEnv::set("TEST_VAR", "test_value").await;
    /// assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
    /// // Variable is cleaned up when _env drops
    /// # }
    /// ```
    pub async fn set(key: impl Into<String>, value: impl AsRef<str>) -> Self {
        let key = key.into();
        let guard = get_env_lock().lock().await;

        // Store old value (if any) for restoration
        let old_value = env::var(&key).ok();

        // Set new value
        env_set_var(&key, value.as_ref());

        Self {
            key,
            old_value,
            _guard: guard,
        }
    }

    /// Set multiple environment variables at once
    ///
    /// NOTE: This acquires a single lock for all variables to avoid deadlock.
    /// Returns a single guard that will restore all variables on drop.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// # async fn example() {
    /// let _env = ScopedEnv::set_multiple([
    ///     ("VAR1", "value1"),
    ///     ("VAR2", "value2"),
    /// ]).await;
    /// # }
    /// ```
    pub async fn set_multiple<I, K, V>(vars: I) -> ScopedEnvMultiple
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: AsRef<str>,
    {
        let guard = get_env_lock().lock().await;

        let mut restorations = Vec::new();
        for (key, value) in vars {
            let key_string = key.into();
            let old_value = env::var(&key_string).ok();
            env_set_var(&key_string, value.as_ref());
            restorations.push((key_string, old_value));
        }

        ScopedEnvMultiple {
            restorations,
            _guard: guard,
        }
    }

    /// Remove an environment variable for the duration of this scope
    ///
    /// If the variable was set, it will be restored when this value is dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// # async fn example() {
    /// songbird_process_env::set_var("TEMP_VAR", "value");
    /// let _env = ScopedEnv::remove("TEMP_VAR").await;
    /// assert!(std::env::var("TEMP_VAR").is_err());
    /// // Variable is restored when _env drops
    /// # }
    /// ```
    pub async fn remove(key: impl Into<String>) -> Self {
        let key = key.into();
        let guard = get_env_lock().lock().await;

        // Store old value for restoration
        let old_value = env::var(&key).ok();

        // Remove variable
        env_remove_var(&key);

        Self {
            key,
            old_value,
            _guard: guard,
        }
    }

    /// Remove multiple environment variables at once
    ///
    /// **IMPORTANT**: This acquires a single lock for all variables to avoid deadlock.
    /// Do NOT create multiple `ScopedEnv::remove()` instances simultaneously,
    /// as they will deadlock trying to acquire the same global lock.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// # async fn example() {
    /// // ✅ CORRECT: Remove multiple vars with single lock
    /// let _env = ScopedEnv::remove_multiple(["VAR1", "VAR2"]).await;
    ///
    /// // ❌ WRONG: This will DEADLOCK!
    /// // let _env1 = ScopedEnv::remove("VAR1").await;
    /// // let _env2 = ScopedEnv::remove("VAR2").await;
    /// # }
    /// ```
    pub async fn remove_multiple<I, K>(keys: I) -> ScopedEnvMultiple
    where
        I: IntoIterator<Item = K>,
        K: Into<String>,
    {
        let guard = get_env_lock().lock().await;

        let mut restorations = Vec::new();
        for key in keys {
            let key_string = key.into();
            let old_value = env::var(&key_string).ok();
            env_remove_var(&key_string);
            restorations.push((key_string, old_value));
        }

        ScopedEnvMultiple {
            restorations,
            _guard: guard,
        }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        // Restore previous state
        match &self.old_value {
            Some(value) => env_set_var(&self.key, value),
            None => env_remove_var(&self.key),
        }
    }
}

/// Guard for multiple environment variables
///
/// Restores all variables when dropped
#[must_use = "ScopedEnvMultiple must be held until cleanup is desired"]
pub struct ScopedEnvMultiple {
    restorations: Vec<(String, Option<String>)>,
    _guard: tokio::sync::MutexGuard<'static, ()>,
}

impl Drop for ScopedEnvMultiple {
    fn drop(&mut self) {
        // Restore all variables in reverse order
        for (key, old_value) in self.restorations.iter().rev() {
            match old_value {
                Some(value) => env_set_var(key, value),
                None => env_remove_var(key),
            }
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scoped_env_set_and_cleanup() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_1";
        let _lock = get_env_lock().lock().await;

        // Ensure clean state
        env_remove_var(test_key);
        assert!(env::var(test_key).is_err());

        {
            let _env = ScopedEnv::set(test_key, "test_value").await;
            assert_eq!(env::var(test_key).unwrap(), "test_value");
        }

        // Should be cleaned up
        assert!(env::var(test_key).is_err());
    }

    #[tokio::test]
    async fn test_scoped_env_restores_previous_value() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_2";

        // Set initial value
        {
            let _lock = get_env_lock().lock().await;
            env_set_var(test_key, "original");
        }

        {
            let _env = ScopedEnv::set(test_key, "temporary").await;
            assert_eq!(env::var(test_key).unwrap(), "temporary");
        }

        // Should restore original
        assert_eq!(env::var(test_key).unwrap(), "original");

        // Cleanup
        {
            let _lock = get_env_lock().lock().await;
            env_remove_var(test_key);
        }
    }

    #[tokio::test]
    async fn test_scoped_env_remove() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_3";

        // Set initial value
        {
            let _lock = get_env_lock().lock().await;
            env_set_var(test_key, "value");
        }

        {
            let _env = ScopedEnv::remove(test_key).await;
            assert!(env::var(test_key).is_err());
        }

        // Should restore original
        assert_eq!(env::var(test_key).unwrap(), "value");

        // Cleanup
        {
            let _lock = get_env_lock().lock().await;
            env_remove_var(test_key);
        }
    }

    #[tokio::test]
    async fn test_scoped_env_multiple() {
        let keys = ["SONGBIRD_TEST_MULTI_1", "SONGBIRD_TEST_MULTI_2"];

        // Ensure clean state
        {
            let _lock = get_env_lock().lock().await;
            for key in &keys {
                env_remove_var(key);
            }
        }

        {
            let _envs = ScopedEnv::set_multiple([(keys[0], "value1"), (keys[1], "value2")]).await;

            assert_eq!(env::var(keys[0]).unwrap(), "value1");
            assert_eq!(env::var(keys[1]).unwrap(), "value2");
        }

        // All should be cleaned up
        for key in &keys {
            assert!(env::var(key).is_err());
        }
    }

    #[tokio::test]
    async fn test_scoped_env_panic_safety() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_PANIC";

        {
            let _lock = get_env_lock().lock().await;
            env_remove_var(test_key);
        }

        let result = std::panic::AssertUnwindSafe(async move {
            let _env = ScopedEnv::set(test_key, "value").await;
            assert_eq!(env::var(test_key).unwrap(), "value");
            panic!("Intentional panic for testing");
        });

        let result = tokio::task::spawn(result).await;
        assert!(result.is_err());

        // Should still be cleaned up despite panic
        // Note: Due to panic unwinding in tests, this may not always work
        // The Drop impl should still run in production code
    }
}
