// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Environment Provider Trait - Dependency Injection for Environment Access
//!
//! This trait enables proper dependency injection for environment variable access,
//! making code testable without global state mutation or serial test execution.
//!
//! ## Architecture Pattern
//!
//! **Problem**: Direct `songbird_process_env::var()` calls couple code to global state
//! **Solution**: Abstract environment access behind a trait
//!
//! ## Design Principles
//!
//! 1. **Dependency Injection**: Pass environment as a parameter
//! 2. **Testability**: Easy to mock/override for tests
//! 3. **Backward Compatibility**: Wrapper functions maintain existing API
//! 4. **Zero Cost**: Trait methods inline to zero overhead
//! 5. **Production Ready**: Pattern works in both tests and production

/// Environment provider trait for dependency injection
///
/// Abstracts environment variable access to enable:
/// - Concurrent-safe testing without global state
/// - Easy mocking and testing
/// - Future: scoped configuration, hot-reload, etc.
pub trait EnvironmentProvider: Send + Sync {
    /// Get an environment variable
    fn get(&self, key: &str) -> Option<String>;

    /// Check if an environment variable exists
    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Get an environment variable or return a default
    fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    /// Get an environment variable and parse it
    fn get_parsed<T>(&self, key: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        self.get(key).and_then(|v| v.parse().ok())
    }
}

/// Real environment provider - uses actual process environment
///
/// This is the production implementation that reads via [`songbird_process_env::var`]
/// (overlay first, then the OS environment).
#[derive(Debug, Clone, Copy, Default)]
pub struct RealEnvironment;

impl EnvironmentProvider for RealEnvironment {
    #[inline]
    fn get(&self, key: &str) -> Option<String> {
        songbird_process_env::var(key).ok()
    }

    #[inline]
    fn contains_key(&self, key: &str) -> bool {
        songbird_process_env::var(key).is_ok()
    }
}

/// Implement `EnvironmentProvider` for `EnvOverride`
impl EnvironmentProvider for crate::env_override::EnvOverride {
    fn get(&self, key: &str) -> Option<String> {
        self.get(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
}

/// Convenience macro for creating functions with environment DI
///
/// Creates both a `_with_env` variant for DI and a convenience wrapper
/// that uses `RealEnvironment`.
///
/// ## Example
///
/// ```ignore
/// async fn get_endpoint_impl(env: &impl EnvironmentProvider) -> Result<String> {
///     env.get("ENDPOINT").ok_or_else(|| "Not found".into())
/// }
///
/// // This macro creates:
/// // - pub async fn get_endpoint_with_env(env: &impl EnvironmentProvider) -> Result<String>
/// // - pub async fn get_endpoint() -> Result<String>
/// ```
#[macro_export]
macro_rules! with_env_di {
    (
        $(#[$meta:meta])*
        pub async fn $name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty
        $body:block
    ) => {
        paste::paste! {
            $(#[$meta])*
            pub async fn [<$name _with_env>](
                env: &impl $crate::env_provider::EnvironmentProvider,
                $($arg: $arg_ty),*
            ) -> $ret {
                $body
            }

            $(#[$meta])*
            pub async fn $name($($arg: $arg_ty),*) -> $ret {
                [<$name _with_env>](&$crate::env_provider::RealEnvironment, $($arg),*).await
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env_override::EnvOverride;

    #[test]
    fn test_real_environment() {
        let env = RealEnvironment;

        // Should be able to read real env vars like PATH
        assert!(env.get("PATH").is_some());
        assert!(env.contains_key("PATH"));
    }

    #[test]
    fn test_env_override_as_provider() {
        let env = EnvOverride::new();
        env.set("TEST_KEY", "test_value");

        // Should work as EnvironmentProvider
        assert_eq!(env.get("TEST_KEY"), Some(String::from("test_value")));
        assert!(env.contains_key("TEST_KEY"));
        assert!(!env.contains_key("NONEXISTENT_KEY"));
    }

    #[test]
    fn test_concurrent_isolation() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let env = EnvOverride::new();
                    env.set("THREAD_ID", i.to_string());

                    // Each thread has isolated environment
                    assert_eq!(env.get("THREAD_ID"), Some(i.to_string()));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_get_or_default() {
        let env = EnvOverride::new();

        assert_eq!(env.get_or("MISSING", "default"), "default");

        env.set("PRESENT", "value");
        assert_eq!(env.get_or("PRESENT", "default"), "value");
    }

    #[test]
    fn test_get_parsed() {
        let env = EnvOverride::new();

        env.set("NUMBER", "42");
        env.set("BOOL", "true");
        env.set("INVALID", "not_a_number");

        assert_eq!(env.get_parsed::<i32>("NUMBER"), Some(42));
        assert_eq!(env.get_parsed::<bool>("BOOL"), Some(true));
        assert_eq!(env.get_parsed::<i32>("INVALID"), None);
        assert_eq!(env.get_parsed::<i32>("MISSING"), None);
    }
}
