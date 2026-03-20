// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

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
            _guard: ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
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
        songbird_process_env::set_var(key, value);

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
        songbird_process_env::remove_var(key);

        self
    }

    /// Set multiple environment variables at once
    #[expect(dead_code, reason = "test assertions and harness ergonomics")] // helper for future tests
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
                Some(v) => songbird_process_env::set_var(key, v),
                None => songbird_process_env::remove_var(key),
            }
        }
    }
}
