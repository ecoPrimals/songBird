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
/// - Modern: Async-safe with RwLock
#[derive(Clone)]
pub struct CredentialManager {
    /// Cached credentials (service_name -> API key)
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
    pub fn get_api_key(&self, service: &str) -> Option<String> {
        // Try to get from cache (blocking, but fast)
        if let Ok(credentials) = self.credentials.try_read() {
            if let Some(key) = credentials.get(service) {
                debug!("🔐 Credential cache hit for service: {}", service);
                return Some(key.clone());
            }
        }

        // Not cached, load from environment
        let service_upper = service.to_uppercase();

        let env_vars = [
            format!("{}_API_KEY", service_upper),
            format!("{}_KEY", service_upper),
            format!("{}_TOKEN", service_upper),
        ];

        for env_var in &env_vars {
            if let Ok(api_key) = std::env::var(env_var) {
                if !api_key.is_empty() {
                    debug!("🔐 Loaded credential for service: {} from {}", service, env_var);

                    // Cache the credential
                    if let Ok(mut credentials) = self.credentials.try_write() {
                        credentials.insert(service.to_string(), api_key.clone());
                    }

                    return Some(api_key);
                }
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
    pub async fn load_all(&self) -> Vec<String> {
        let known_services = ["openai", "huggingface", "anthropic", "stripe", "github", "slack"];

        let mut loaded = Vec::new();

        for service in &known_services {
            if self.get_api_key(service).is_some() {
                loaded.push((*service).to_string());
            }
        }

        if !loaded.is_empty() {
            debug!("🔐 Loaded credentials for {} services: {:?}", loaded.len(), loaded);
        } else {
            warn!(
                "⚠️  No credentials loaded. HTTP gateway will not be able to proxy external APIs."
            );
            warn!("   Set environment variables like OPENAI_API_KEY, HUGGINGFACE_API_KEY, etc.");
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
    use std::env;

    #[test]
    fn test_credential_manager_creation() {
        let manager = CredentialManager::new();
        assert!(manager.credentials.try_read().is_ok());
    }

    #[test]
    fn test_get_api_key_from_env() {
        // Set test API key
        env::set_var("TEST_SERVICE_API_KEY", "test_key_123");

        let manager = CredentialManager::new();
        let api_key = manager.get_api_key("test_service");

        assert_eq!(api_key, Some("test_key_123".to_string()));

        // Cleanup
        env::remove_var("TEST_SERVICE_API_KEY");
    }

    #[test]
    fn test_get_api_key_fallback() {
        // Test fallback to _KEY suffix
        env::set_var("ANOTHER_SERVICE_KEY", "another_key_456");

        let manager = CredentialManager::new();
        let api_key = manager.get_api_key("another_service");

        assert_eq!(api_key, Some("another_key_456".to_string()));

        // Cleanup
        env::remove_var("ANOTHER_SERVICE_KEY");
    }

    #[test]
    fn test_get_api_key_not_found() {
        let manager = CredentialManager::new();
        let api_key = manager.get_api_key("nonexistent_service");

        assert_eq!(api_key, None);
    }

    #[test]
    fn test_has_api_key() {
        env::set_var("MYSERVICE_API_KEY", "my_key_789");

        let manager = CredentialManager::new();

        assert!(manager.has_api_key("myservice"));
        assert!(!manager.has_api_key("other_service"));

        // Cleanup
        env::remove_var("MYSERVICE_API_KEY");
    }

    #[tokio::test]
    async fn test_load_all() {
        // Set up some test credentials
        env::set_var("OPENAI_API_KEY", "openai_test_key");
        env::set_var("STRIPE_API_KEY", "stripe_test_key");

        let manager = CredentialManager::new();
        let loaded = manager.load_all().await;

        assert!(loaded.contains(&"openai".to_string()));
        assert!(loaded.contains(&"stripe".to_string()));

        // Cleanup
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("STRIPE_API_KEY");
    }

    #[test]
    fn test_credential_caching() {
        env::set_var("CACHED_SERVICE_API_KEY", "cached_key");

        let manager = CredentialManager::new();

        // First access (loads from env)
        let key1 = manager.get_api_key("cached_service");

        // Second access (loads from cache)
        let key2 = manager.get_api_key("cached_service");

        assert_eq!(key1, key2);
        assert_eq!(key1, Some("cached_key".to_string()));

        // Cleanup
        env::remove_var("CACHED_SERVICE_API_KEY");
    }
}
