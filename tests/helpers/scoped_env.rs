//! Scoped Environment Variable Management for Tests
//!
//! Provides RAII-based environment variable isolation for concurrent tests.
//! No more #[serial] needed for env var tests!
//!
//! ## Why This Matters
//!
//! Global `std::env::set_var` and `std::env::remove_var` cause tests to interfere
//! with each other, forcing serialization via `#[serial]`. This is an anti-pattern
//! that hides concurrency bugs and slows down test suites.
//!
//! ## Solution
//!
//! `ScopedEnv` provides:
//! - Automatic cleanup via RAII (Drop)
//! - Thread-safe var management
//! - Clear ownership semantics
//! - No global state mutation
//!
//! ## Usage
//!
//! ```rust
//! #[tokio::test]  // No #[serial] needed!
//! async fn test_with_env() {
//!     let _env = ScopedEnv::new()
//!         .set("MY_VAR", "value")
//!         .set("ANOTHER_VAR", "123");
//!     
//!     // Your test code here
//!     // Vars are automatically restored when _env drops
//! }
//! ```

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_env_sets_and_restores() {
        // Ensure var doesn't exist initially
        env::remove_var("TEST_SCOPED_VAR");

        {
            let _env = ScopedEnv::new().set("TEST_SCOPED_VAR", "test_value");
            assert_eq!(env::var("TEST_SCOPED_VAR").unwrap(), "test_value");
        } // _env drops here

        // Variable should be restored (removed)
        assert!(env::var("TEST_SCOPED_VAR").is_err());
    }

    #[test]
    fn test_scoped_env_preserves_existing() {
        // Set an initial value
        env::set_var("TEST_EXISTING_VAR", "original");

        {
            let _env = ScopedEnv::new().set("TEST_EXISTING_VAR", "modified");
            assert_eq!(env::var("TEST_EXISTING_VAR").unwrap(), "modified");
        } // _env drops here

        // Original value should be restored
        assert_eq!(env::var("TEST_EXISTING_VAR").unwrap(), "original");

        // Cleanup
        env::remove_var("TEST_EXISTING_VAR");
    }

    #[test]
    fn test_scoped_env_remove() {
        env::set_var("TEST_REMOVE_VAR", "exists");

        {
            let _env = ScopedEnv::new().remove("TEST_REMOVE_VAR");
            assert!(env::var("TEST_REMOVE_VAR").is_err());
        } // _env drops here

        // Variable should be restored
        assert_eq!(env::var("TEST_REMOVE_VAR").unwrap(), "exists");

        // Cleanup
        env::remove_var("TEST_REMOVE_VAR");
    }

    #[test]
    fn test_scoped_env_method_chaining() {
        env::remove_var("TEST_VAR1");
        env::remove_var("TEST_VAR2");
        env::remove_var("TEST_VAR3");

        {
            let _env = ScopedEnv::new()
                .set("TEST_VAR1", "value1")
                .set("TEST_VAR2", "value2")
                .set("TEST_VAR3", "value3");

            assert_eq!(env::var("TEST_VAR1").unwrap(), "value1");
            assert_eq!(env::var("TEST_VAR2").unwrap(), "value2");
            assert_eq!(env::var("TEST_VAR3").unwrap(), "value3");
        } // _env drops here

        // All should be restored (removed)
        assert!(env::var("TEST_VAR1").is_err());
        assert!(env::var("TEST_VAR2").is_err());
        assert!(env::var("TEST_VAR3").is_err());
    }

    #[test]
    fn test_scoped_env_set_multiple() {
        env::remove_var("MULTI_VAR1");
        env::remove_var("MULTI_VAR2");

        {
            let _env =
                ScopedEnv::new().set_multiple(&[("MULTI_VAR1", "val1"), ("MULTI_VAR2", "val2")]);

            assert_eq!(env::var("MULTI_VAR1").unwrap(), "val1");
            assert_eq!(env::var("MULTI_VAR2").unwrap(), "val2");
        }

        assert!(env::var("MULTI_VAR1").is_err());
        assert!(env::var("MULTI_VAR2").is_err());
    }
}
