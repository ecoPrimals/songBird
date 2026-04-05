// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Test Environment Provider for Concurrent Testing
//!
//! Provides isolated environment variables for tests without using global `std::env`,
//! enabling truly concurrent test execution.

use std::collections::HashMap;

/// Test environment provider for isolated, concurrent testing
///
/// # Purpose
///
/// Allows tests to have their own isolated environment variables without
/// mutating global `std::env`, enabling tests to run concurrently.
///
/// # Example
///
/// ```no_run
/// use songbird_test_utils::TestEnv;
///
/// // NO #[serial] needed!
/// fn test_config() {
///     let mut env = TestEnv::new();
///     env.set("SONGBIRD_ENV", "production");
///     
///     // let mode = DeploymentMode::from_test_env(&env);
///     // assert!(matches!(mode, DeploymentMode::Production));
///     // No cleanup needed - env is local!
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct TestEnv {
    vars: HashMap<String, String>,
}

impl TestEnv {
    /// Create a new empty test environment
    #[must_use]
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    /// Create test environment with default development values
    #[must_use]
    pub fn development() -> Self {
        let mut env = Self::new();
        env.set("SONGBIRD_ENV", "development");
        env.set("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
        env.set("SONGBIRD_PORT", "8080");
        env
    }

    /// Create test environment with production values
    #[must_use]
    pub fn production() -> Self {
        let mut env = Self::new();
        env.set("SONGBIRD_ENV", "production");
        env.set("SONGBIRD_BIND_ADDRESS", "0.0.0.0");
        env.set("SONGBIRD_PORT", "80");
        env
    }

    /// Create test environment with staging values
    #[must_use]
    pub fn staging() -> Self {
        let mut env = Self::new();
        env.set("SONGBIRD_ENV", "staging");
        env.set("SONGBIRD_BIND_ADDRESS", "0.0.0.0");
        env.set("SONGBIRD_PORT", "8080");
        env
    }

    /// Create test environment with testing values
    #[must_use]
    pub fn testing() -> Self {
        let mut env = Self::new();
        env.set("SONGBIRD_ENV", "testing");
        env.set("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
        env.set("SONGBIRD_PORT", "0"); // Dynamic port
        env
    }

    /// Set an environment variable
    pub fn set(&mut self, key: &str, value: &str) -> &mut Self {
        self.vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Set multiple environment variables
    pub fn set_many(&mut self, pairs: &[(&str, &str)]) -> &mut Self {
        for (key, value) in pairs {
            self.set(key, value);
        }
        self
    }

    /// Get an environment variable
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    /// Get an environment variable with default value
    #[must_use]
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// Get an environment variable as a specific type
    #[must_use]
    pub fn get_parsed<T>(&self, key: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        self.get(key).and_then(|v| v.parse().ok())
    }

    /// Get port from environment with default
    #[must_use]
    pub fn get_port(&self, key: &str, default: u16) -> u16 {
        self.get_parsed(key).unwrap_or(default)
    }

    /// Get boolean from environment with default
    #[must_use]
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key)
            .and_then(|v| match v.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => v.parse::<bool>().ok(),
            })
            .unwrap_or(default)
    }

    /// Get integer from environment with default
    #[must_use]
    pub fn get_usize(&self, key: &str, default: usize) -> usize {
        self.get_parsed(key).unwrap_or(default)
    }

    /// Get u32 from environment with default
    #[must_use]
    pub fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get_parsed(key).unwrap_or(default)
    }

    /// Get u64 from environment with default
    #[must_use]
    pub fn get_u64(&self, key: &str, default: u64) -> u64 {
        self.get_parsed(key).unwrap_or(default)
    }

    /// Check if a variable is set
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    /// Remove a variable
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.vars.remove(key)
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.vars.clear();
    }

    /// Get all variables as a `HashMap`
    #[must_use]
    pub const fn as_map(&self) -> &HashMap<String, String> {
        &self.vars
    }

    /// Convert to `HashMap` (consuming self)
    #[must_use]
    pub fn into_map(self) -> HashMap<String, String> {
        self.vars
    }
}

/// Trait for types that can be created from a test environment
///
/// This enables dependency injection of environment variables for testing
/// without using global `std::env`.
pub trait FromTestEnv: Sized {
    /// Create from test environment
    fn from_test_env(env: &TestEnv) -> Self;
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test] // NO #[serial]!
    fn test_env_new() {
        let env = TestEnv::new();
        assert!(env.get("NONEXISTENT").is_none());
    }

    #[test] // NO #[serial]!
    fn test_env_set_get() {
        let mut env = TestEnv::new();
        env.set("KEY", "value");

        assert_eq!(env.get("KEY"), Some(&"value".to_string()));
    }

    #[test] // NO #[serial]!
    fn test_env_get_or() {
        let env = TestEnv::new();
        assert_eq!(env.get_or("MISSING", "default"), "default");
    }

    #[test] // NO #[serial]!
    fn test_env_parsed() {
        let mut env = TestEnv::new();
        env.set("PORT", "8080");

        assert_eq!(env.get_parsed::<u16>("PORT"), Some(8080));
    }

    #[test] // NO #[serial]!
    fn test_env_bool() {
        let mut env = TestEnv::new();
        env.set("ENABLED", "true");

        assert!(env.get_bool("ENABLED", false));
    }

    #[test] // NO #[serial]!
    fn test_env_presets() {
        let dev = TestEnv::development();
        assert_eq!(dev.get("SONGBIRD_ENV"), Some(&"development".to_string()));

        let prod = TestEnv::production();
        assert_eq!(prod.get("SONGBIRD_ENV"), Some(&"production".to_string()));

        let staging = TestEnv::staging();
        assert_eq!(staging.get("SONGBIRD_ENV"), Some(&"staging".to_string()));

        let testing = TestEnv::testing();
        assert_eq!(testing.get("SONGBIRD_ENV"), Some(&"testing".to_string()));
    }

    #[test] // NO #[serial]!
    fn test_env_set_many() {
        let mut env = TestEnv::new();
        env.set_many(&[("KEY1", "value1"), ("KEY2", "value2"), ("KEY3", "value3")]);

        assert_eq!(env.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(env.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(env.get("KEY3"), Some(&"value3".to_string()));
    }

    #[test] // NO #[serial]!
    fn test_env_concurrent_isolation() {
        // Each test gets its own environment
        let mut env1 = TestEnv::new();
        let mut env2 = TestEnv::new();

        env1.set("KEY", "value1");
        env2.set("KEY", "value2");

        // No conflicts!
        assert_eq!(env1.get("KEY"), Some(&"value1".to_string()));
        assert_eq!(env2.get("KEY"), Some(&"value2".to_string()));
    }
}
