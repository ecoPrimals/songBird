// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Credential manager for HTTP gateway
//!
//! **Philosophy**: Secure credential management from environment
//!
//! ## Design
//!
//! - Zero hardcoding: All credentials from environment variables
//! - Secure: No credentials in code or logs
//! - Capability-based: Discover credentials by service capability
//! - Lazy loading: Only load credentials when needed
//!
//! ## Security
//!
//! - Credentials never logged or exposed
//! - Environment variable sanitization
//! - Clear error messages without leaking secrets
//!
//! **Created**: January 16, 2026

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Credential manager for external API keys
///
/// **Philosophy**:
/// - Zero hardcoding: All from environment
/// - Secure: No logging of actual credentials
/// - Modern: Async-safe with `RwLock`
#[derive(Clone)]
pub struct CredentialManager {
    /// Cached credentials (`service_name` -> API key)
    credentials: Arc<RwLock<HashMap<String, String>>>,
}

impl CredentialManager {
    /// Create a new credential manager
    ///
    /// Credentials are loaded lazily from environment variables on first access.
    ///
    /// # Philosophy
    /// - Zero hardcoding: No credentials in code
    /// - Secure: Credentials never logged
    /// - Flexible: Works with any service
    #[must_use]
    pub fn new() -> Self {
        debug!("🔐 Initializing credential manager");

        Self {
            credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get API key for a service
    ///
    /// # Arguments
    /// * `service` - Service name (e.g., "openai", "huggingface", "stripe")
    ///
    /// # Returns
    /// * `Some(String)` if API key is configured
    /// * `None` if API key is not configured
    ///
    /// # Environment Variables
    /// The manager will look for credentials in this order:
    /// 1. `{SERVICE}_API_KEY` (e.g., `OPENAI_API_KEY`)
    /// 2. `{SERVICE}_KEY` (e.g., `OPENAI_KEY`)
    /// 3. `{SERVICE}_TOKEN` (e.g., `OPENAI_TOKEN`)
    ///
    /// # Philosophy
    /// - Zero hardcoding: Credentials from environment
    /// - Secure: No logging of actual keys
    /// - Flexible: Multiple naming conventions supported
    #[must_use]
    pub fn get_api_key(&self, service: &str) -> Option<String> {
        self.get_api_key_with(service, |name| songbird_process_env::var(name).ok())
    }

    /// Get API key with injectable env reader (concurrent-safe, testable)
    pub fn get_api_key_with<F>(&self, service: &str, env_reader: F) -> Option<String>
    where
        F: Fn(&str) -> Option<String>,
    {
        // Try to get from cache (blocking, but fast)
        if let Ok(credentials) = self.credentials.try_read()
            && let Some(key) = credentials.get(service)
        {
            debug!("🔐 Credential cache hit for service: {}", service);
            return Some(key.clone());
        }

        // Not cached, load from environment
        let service_upper = service.to_uppercase();

        let env_vars = [
            format!("{service_upper}_API_KEY"),
            format!("{service_upper}_KEY"),
            format!("{service_upper}_TOKEN"),
        ];

        for env_var in &env_vars {
            if let Some(api_key) = env_reader(env_var)
                && !api_key.is_empty()
            {
                debug!("🔐 Loaded credential for service: {} from {}", service, env_var);

                // Cache the credential
                if let Ok(mut credentials) = self.credentials.try_write() {
                    credentials.insert(service.to_string(), api_key.clone());
                }

                return Some(api_key);
            }
        }

        warn!("⚠️  No API key found for service '{}'. Tried: {:?}", service, env_vars);

        None
    }

    /// Check if a service has a configured API key
    ///
    /// # Arguments
    /// * `service` - Service name
    ///
    /// # Returns
    /// * `true` if API key is configured
    /// * `false` otherwise
    #[must_use]
    pub fn has_api_key(&self, service: &str) -> bool {
        self.get_api_key(service).is_some()
    }

    /// Load all credentials from environment
    ///
    /// This will proactively load credentials for known services.
    /// Useful for startup validation.
    ///
    /// # Returns
    /// * List of services with configured credentials
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn load_all(&self) -> Vec<String> {
        let known_services = ["openai", "huggingface", "anthropic", "stripe", "github", "slack"];

        let mut loaded = Vec::new();

        for service in &known_services {
            if self.get_api_key(service).is_some() {
                loaded.push((*service).to_string());
            }
        }

        if loaded.is_empty() {
            warn!(
                "⚠️  No credentials loaded. HTTP gateway will not be able to proxy external APIs."
            );
            warn!("   Set environment variables like OPENAI_API_KEY, HUGGINGFACE_API_KEY, etc.");
        } else {
            debug!("🔐 Loaded credentials for {} services: {:?}", loaded.len(), loaded);
        }

        loaded
    }

    /// Clear credential cache (for testing)
    #[cfg(test)]
    pub async fn clear_cache(&self) {
        let mut credentials = self.credentials.write().await;
        credentials.clear();
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Create a mock env reader from key-value pairs
    fn mock_env(vars: HashMap<String, String>) -> impl Fn(&str) -> Option<String> {
        move |name: &str| vars.get(name).cloned()
    }

    #[test]
    fn test_credential_manager_creation() {
        let manager = CredentialManager::new();
        assert!(manager.credentials.try_read().is_ok());
    }

    #[test]
    fn test_get_api_key_from_env() {
        // ✅ Concurrent-safe: Uses injectable env reader
        let env = mock_env(HashMap::from([(
            String::from("TEST_SERVICE_API_KEY"),
            String::from("test_key_123"),
        )]));

        let manager = CredentialManager::new();
        let api_key = manager.get_api_key_with("test_service", env);
        assert_eq!(api_key, Some(String::from("test_key_123")));
    }

    #[test]
    fn test_get_api_key_fallback() {
        // ✅ Concurrent-safe: Tests _KEY suffix fallback
        let env = mock_env(HashMap::from([(
            String::from("ANOTHER_SERVICE_KEY"),
            String::from("another_key_456"),
        )]));

        let manager = CredentialManager::new();
        let api_key = manager.get_api_key_with("another_service", env);
        assert_eq!(api_key, Some(String::from("another_key_456")));
    }

    #[test]
    fn test_get_api_key_not_found() {
        let env = mock_env(HashMap::new());
        let manager = CredentialManager::new();
        let api_key = manager.get_api_key_with("nonexistent_service", env);
        assert_eq!(api_key, None);
    }

    #[test]
    fn test_has_api_key() {
        // ✅ Concurrent-safe: Pre-load into cache to test has_api_key
        let env = mock_env(HashMap::from([(
            String::from("MYSERVICE_API_KEY"),
            String::from("my_key_789"),
        )]));

        let manager = CredentialManager::new();
        // Load key into cache via get_api_key_with
        let _ = manager.get_api_key_with("myservice", env);
        // has_api_key checks cache first, so this will work
        assert!(manager.has_api_key("myservice"));
        assert!(!manager.has_api_key("other_service"));
    }

    #[tokio::test]
    async fn test_load_all_with_env() {
        // ✅ Concurrent-safe: Pre-load credentials into cache
        let manager = CredentialManager::new();

        // Pre-load specific credentials via injectable env
        let openai_env = mock_env(HashMap::from([(
            String::from("OPENAI_API_KEY"),
            String::from("openai_test_key"),
        )]));
        let stripe_env = mock_env(HashMap::from([(
            String::from("STRIPE_API_KEY"),
            String::from("stripe_test_key"),
        )]));

        let _ = manager.get_api_key_with("openai", openai_env);
        let _ = manager.get_api_key_with("stripe", stripe_env);

        // load_all checks cache + real env; cached entries will be found
        let loaded = manager.load_all().await;
        assert!(loaded.contains(&String::from("openai")));
        assert!(loaded.contains(&String::from("stripe")));
    }

    #[test]
    fn test_credential_caching() {
        // ✅ Concurrent-safe: Tests cache behavior
        let env = mock_env(HashMap::from([(
            String::from("CACHED_SERVICE_API_KEY"),
            String::from("cached_key"),
        )]));

        let manager = CredentialManager::new();

        // First access (loads from mock env)
        let key1 = manager.get_api_key_with("cached_service", &env);

        // Second access (should hit cache — pass empty env to prove cache works)
        let empty_env = mock_env(HashMap::new());
        let key2 = manager.get_api_key_with("cached_service", empty_env);

        assert_eq!(key1, key2);
        assert_eq!(key1, Some(String::from("cached_key")));
    }
}
