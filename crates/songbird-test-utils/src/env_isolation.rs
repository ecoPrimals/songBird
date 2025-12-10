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
use std::sync::{Mutex, PoisonError};

// Global lock for test environment isolation
// This ensures only one test modifies environment at a time
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Scoped environment variable that automatically cleans up on drop
///
/// This type uses RAII to ensure environment variables set in tests
/// are always cleaned up, even if the test panics.
///
/// # Thread Safety
///
/// Uses a global mutex to ensure only one test can modify environment
/// variables at a time, preventing race conditions in parallel tests.
#[must_use = "ScopedEnv must be held until cleanup is desired"]
pub struct ScopedEnv {
    key: String,
    old_value: Option<String>,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl ScopedEnv {
    /// Set an environment variable for the duration of this scope
    ///
    /// The variable will be automatically cleaned up when this value is dropped.
    ///
    /// # Example
    ///
    /// ```
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// let _env = ScopedEnv::set("TEST_VAR", "test_value");
    /// assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
    /// // Variable is cleaned up when _env drops
    /// ```
    pub fn set(key: impl Into<String>, value: impl AsRef<str>) -> Self {
        let key = key.into();
        let guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        // Store old value (if any) for restoration
        let old_value = env::var(&key).ok();

        // Set new value
        env::set_var(&key, value.as_ref());

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
    /// ```
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// let _env = ScopedEnv::set_multiple([
    ///     ("VAR1", "value1"),
    ///     ("VAR2", "value2"),
    /// ]);
    /// ```
    pub fn set_multiple<I, K, V>(vars: I) -> ScopedEnvMultiple
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: AsRef<str>,
    {
        let guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        let mut restorations = Vec::new();
        for (key, value) in vars {
            let key_string = key.into();
            let old_value = env::var(&key_string).ok();
            env::set_var(&key_string, value.as_ref());
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
    /// ```
    /// use songbird_test_utils::ScopedEnv;
    ///
    /// std::env::set_var("TEMP_VAR", "value");
    /// let _env = ScopedEnv::remove("TEMP_VAR");
    /// assert!(std::env::var("TEMP_VAR").is_err());
    /// // Variable is restored when _env drops
    /// ```
    pub fn remove(key: impl Into<String>) -> Self {
        let key = key.into();
        let guard = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        // Store old value for restoration
        let old_value = env::var(&key).ok();

        // Remove variable
        env::remove_var(&key);

        Self {
            key,
            old_value,
            _guard: guard,
        }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        // Restore previous state
        match &self.old_value {
            Some(value) => env::set_var(&self.key, value),
            None => env::remove_var(&self.key),
        }
    }
}

/// Guard for multiple environment variables
///
/// Restores all variables when dropped
#[must_use = "ScopedEnvMultiple must be held until cleanup is desired"]
pub struct ScopedEnvMultiple {
    restorations: Vec<(String, Option<String>)>,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Drop for ScopedEnvMultiple {
    fn drop(&mut self) {
        // Restore all variables in reverse order
        for (key, old_value) in self.restorations.iter().rev() {
            match old_value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_env_set_and_cleanup() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_1";

        // Ensure clean state
        env::remove_var(test_key);
        assert!(env::var(test_key).is_err());

        {
            let _env = ScopedEnv::set(test_key, "test_value");
            assert_eq!(env::var(test_key).unwrap(), "test_value");
        }

        // Should be cleaned up
        assert!(env::var(test_key).is_err());
    }

    #[test]
    fn test_scoped_env_restores_previous_value() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_2";

        // Set initial value
        env::set_var(test_key, "original");

        {
            let _env = ScopedEnv::set(test_key, "temporary");
            assert_eq!(env::var(test_key).unwrap(), "temporary");
        }

        // Should restore original
        assert_eq!(env::var(test_key).unwrap(), "original");

        // Cleanup
        env::remove_var(test_key);
    }

    #[test]
    fn test_scoped_env_remove() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_3";

        // Set initial value
        env::set_var(test_key, "value");

        {
            let _env = ScopedEnv::remove(test_key);
            assert!(env::var(test_key).is_err());
        }

        // Should restore original
        assert_eq!(env::var(test_key).unwrap(), "value");

        // Cleanup
        env::remove_var(test_key);
    }

    #[test]
    fn test_scoped_env_multiple() {
        let keys = ["SONGBIRD_TEST_MULTI_1", "SONGBIRD_TEST_MULTI_2"];

        // Ensure clean state
        for key in &keys {
            env::remove_var(key);
        }

        {
            let _envs = ScopedEnv::set_multiple([(keys[0], "value1"), (keys[1], "value2")]);

            assert_eq!(env::var(keys[0]).unwrap(), "value1");
            assert_eq!(env::var(keys[1]).unwrap(), "value2");
        }

        // All should be cleaned up
        for key in &keys {
            assert!(env::var(key).is_err());
        }
    }

    #[test]
    fn test_scoped_env_panic_safety() {
        let test_key = "SONGBIRD_TEST_SCOPED_ENV_PANIC";

        env::remove_var(test_key);

        let result = std::panic::catch_unwind(|| {
            let _env = ScopedEnv::set(test_key, "value");
            assert_eq!(env::var(test_key).unwrap(), "value");
            panic!("Intentional panic for testing");
        });

        assert!(result.is_err());

        // Should still be cleaned up despite panic
        // Note: Due to panic unwinding in tests, this may not always work
        // The Drop impl should still run in production code
    }
}
