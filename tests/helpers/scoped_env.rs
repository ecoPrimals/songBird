// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Scoped Environment Variable Management for Tests
//!
//! Provides RAII-based environment variable isolation for concurrent tests
//! using `songbird_process_env` (in-memory overlay, zero `unsafe`).
//!
//! `std::env::set_var` / `std::env::remove_var` are `unsafe` in Rust 2024.
//! This module avoids them entirely by routing through the process-env overlay.

use std::collections::HashMap;

/// RAII guard for environment variables
///
/// Automatically restores previous overlay values on drop.
pub struct ScopedEnv {
    restore: HashMap<String, Option<String>>,
}

impl ScopedEnv {
    pub fn new() -> Self {
        Self {
            restore: HashMap::new(),
        }
    }

    pub fn set(mut self, key: &str, value: &str) -> Self {
        if !self.restore.contains_key(key) {
            self.restore.insert(key.to_string(), songbird_process_env::var(key).ok());
        }
        songbird_process_env::set_var(key, value);
        self
    }

    pub fn remove(mut self, key: &str) -> Self {
        if !self.restore.contains_key(key) {
            self.restore.insert(key.to_string(), songbird_process_env::var(key).ok());
        }
        songbird_process_env::remove_var(key);
        self
    }

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
        for (key, value) in &self.restore {
            match value {
                Some(v) => songbird_process_env::set_var(key, v),
                None => songbird_process_env::remove_var(key),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_scoped_env_sets_and_restores() {
        songbird_process_env::remove_var("TEST_SCOPED_VAR");

        {
            let _env = ScopedEnv::new().set("TEST_SCOPED_VAR", "test_value");
            assert_eq!(songbird_process_env::var("TEST_SCOPED_VAR").unwrap(), "test_value");
        }

        assert!(songbird_process_env::var("TEST_SCOPED_VAR").is_err());
    }

    #[test]
    fn test_scoped_env_preserves_existing() {
        songbird_process_env::set_var("TEST_EXISTING_VAR", "original");

        {
            let _env = ScopedEnv::new().set("TEST_EXISTING_VAR", "modified");
            assert_eq!(songbird_process_env::var("TEST_EXISTING_VAR").unwrap(), "modified");
        }

        assert_eq!(songbird_process_env::var("TEST_EXISTING_VAR").unwrap(), "original");
        songbird_process_env::remove_var("TEST_EXISTING_VAR");
    }

    #[test]
    fn test_scoped_env_remove() {
        songbird_process_env::set_var("TEST_REMOVE_VAR", "exists");

        {
            let _env = ScopedEnv::new().remove("TEST_REMOVE_VAR");
            assert!(songbird_process_env::var("TEST_REMOVE_VAR").is_err());
        }

        assert_eq!(songbird_process_env::var("TEST_REMOVE_VAR").unwrap(), "exists");
        songbird_process_env::remove_var("TEST_REMOVE_VAR");
    }

    #[test]
    fn test_scoped_env_method_chaining() {
        songbird_process_env::remove_var("TEST_VAR1");
        songbird_process_env::remove_var("TEST_VAR2");
        songbird_process_env::remove_var("TEST_VAR3");

        {
            let _env = ScopedEnv::new()
                .set("TEST_VAR1", "value1")
                .set("TEST_VAR2", "value2")
                .set("TEST_VAR3", "value3");

            assert_eq!(songbird_process_env::var("TEST_VAR1").unwrap(), "value1");
            assert_eq!(songbird_process_env::var("TEST_VAR2").unwrap(), "value2");
            assert_eq!(songbird_process_env::var("TEST_VAR3").unwrap(), "value3");
        }

        assert!(songbird_process_env::var("TEST_VAR1").is_err());
        assert!(songbird_process_env::var("TEST_VAR2").is_err());
        assert!(songbird_process_env::var("TEST_VAR3").is_err());
    }

    #[test]
    fn test_scoped_env_set_multiple() {
        songbird_process_env::remove_var("MULTI_VAR1");
        songbird_process_env::remove_var("MULTI_VAR2");

        {
            let _env =
                ScopedEnv::new().set_multiple(&[("MULTI_VAR1", "val1"), ("MULTI_VAR2", "val2")]);

            assert_eq!(songbird_process_env::var("MULTI_VAR1").unwrap(), "val1");
            assert_eq!(songbird_process_env::var("MULTI_VAR2").unwrap(), "val2");
        }

        assert!(songbird_process_env::var("MULTI_VAR1").is_err());
        assert!(songbird_process_env::var("MULTI_VAR2").is_err());
    }
}
