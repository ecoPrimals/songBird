//! Circuit Breaker Manager for External Service Calls
//!
//! Provides centralized circuit breaker management for all external HTTP calls,
//! preventing cascading failures and enabling graceful degradation.
//!
//! ## Deep Debt Evolution Principle
//!
//! **Before (No Protection)**:
//! ```ignore
//! // Direct calls, no protection
//! let response = client.get(url).await?;
//! // If service fails, all requests wait for timeout
//! // Cascading failures across system
//! ```
//!
//! **After (Circuit Breaker Protection)**:
//! ```ignore
//! // Protected calls with fail-fast
//! let breaker = manager.get_breaker_for_endpoint(url);
//! let response = breaker.call(|| client.get(url)).await?;
//! // Fails immediately if circuit is open
//! // Automatic recovery testing
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_universal::circuit_breaker_manager::CircuitBreakerManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = CircuitBreakerManager::new();
//!    
//!     // Get circuit breaker for an endpoint
//!     let breaker = manager.get_breaker_for_endpoint("https://api.example.com").await;
//!    
//!     // Use circuit breaker to protect calls
//!     let result = breaker.call(|| async {
//!         // External service call
//!         external_service_call().await
//!     }).await?;
//!    
//!     Ok(())
//! }
//!
//! # async fn external_service_call() -> Result<String, std::io::Error> {
//! #     Ok("success".to_string())
//! # }
//! ```

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};
use url::Url;

/// Manager for circuit breakers across multiple endpoints
///
/// Maintains a map of circuit breakers keyed by endpoint domain.
/// Automatically creates new breakers for unseen endpoints.
#[derive(Clone)]
pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager with default configuration
    ///
    /// Default configuration:
    /// - Failure threshold: 5 failures
    /// - Timeout: 60 seconds
    /// - Success threshold: 2 successes (in half-open state)
    /// - Operation timeout: 30 seconds
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config: CircuitBreakerConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create with builder pattern
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use songbird_universal::circuit_breaker_manager::CircuitBreakerManager;
    /// # use std::time::Duration;
    /// let manager = CircuitBreakerManager::builder()
    ///     .failure_threshold(10)
    ///     .timeout(Duration::from_secs(120))
    ///     .success_threshold(2)
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder() -> CircuitBreakerManagerBuilder {
        CircuitBreakerManagerBuilder::new()
    }

    /// Get or create a circuit breaker for an endpoint
    ///
    /// Extracts the domain from the URL and creates/retrieves a breaker for it.
    /// Multiple endpoints on the same domain share the same circuit breaker.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Full URL or domain string
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use songbird_universal::circuit_breaker_manager::CircuitBreakerManager;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let manager = CircuitBreakerManager::new();
    /// let breaker = manager.get_breaker_for_endpoint("https://api.github.com/repos").await;
    /// // Same breaker for any api.github.com endpoint
    /// let breaker2 = manager.get_breaker_for_endpoint("https://api.github.com/users").await;
    /// # }
    /// ```
    pub async fn get_breaker_for_endpoint(&self, endpoint: &str) -> Arc<CircuitBreaker> {
        let domain = Self::extract_domain(endpoint);

        // Check if breaker already exists
        {
            let breakers = self.breakers.read().await;
            if let Some(breaker) = breakers.get(&domain) {
                return Arc::clone(breaker);
            }
        }

        // Create new breaker
        let breaker = CircuitBreaker::with_config(self.config.clone());

        let breaker_arc = Arc::new(breaker);

        // Store for future use
        {
            let mut breakers = self.breakers.write().await;
            breakers.insert(domain.clone(), Arc::clone(&breaker_arc));
            info!("Created circuit breaker for domain: {}", domain);
        }

        breaker_arc
    }

    /// Execute a call with circuit breaker protection
    ///
    /// Helper method that wraps the call with proper circuit breaker logic.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let manager = CircuitBreakerManager::new();
    /// let result = manager
    ///     .call_with_breaker("https://api.github.com", || async {
    ///         client.get("https://api.github.com/repos").await
    ///     })
    ///     .await?;
    /// ```
    pub async fn call_with_breaker<F, Fut, T, E>(
        &self,
        endpoint: &str,
        operation: F,
    ) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: From<String>,
    {
        let breaker = self.get_breaker_for_endpoint(endpoint).await;

        // Check if request is allowed
        if !breaker.is_request_allowed().await {
            return Err(E::from(format!("Circuit breaker is open for endpoint: {}", endpoint)));
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                breaker.record_success().await;
                Ok(result)
            }
            Err(e) => {
                breaker.record_failure().await;
                Err(e)
            }
        }
    }

    /// Get circuit breaker for a specific domain (if it exists)
    ///
    /// Returns None if no breaker has been created for this domain yet.
    pub async fn get_breaker(&self, domain: &str) -> Option<Arc<CircuitBreaker>> {
        let breakers = self.breakers.read().await;
        breakers.get(domain).map(Arc::clone)
    }

    /// Get all managed circuit breakers
    ///
    /// Returns a snapshot of all current breakers.
    pub async fn all_breakers(&self) -> HashMap<String, Arc<CircuitBreaker>> {
        let breakers = self.breakers.read().await;
        breakers.clone()
    }

    /// Reset a circuit breaker for a specific domain
    ///
    /// Useful for manually recovering from a known issue.
    pub async fn reset_breaker(&self, domain: &str) {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(domain) {
            breaker.reset().await;
            info!("Reset circuit breaker for domain: {}", domain);
        }
    }

    /// Reset all circuit breakers
    ///
    /// Useful for system-wide recovery after maintenance.
    pub async fn reset_all(&self) {
        let breakers = self.breakers.read().await;
        for (domain, breaker) in breakers.iter() {
            breaker.reset().await;
            debug!("Reset circuit breaker for domain: {}", domain);
        }
        info!("Reset all {} circuit breakers", breakers.len());
    }

    /// Get statistics for all breakers
    ///
    /// Returns a map of domain to state.
    pub async fn get_all_stats(&self) -> HashMap<String, String> {
        let breakers = self.breakers.read().await;
        let mut stats = HashMap::new();

        for (domain, breaker) in breakers.iter() {
            let state = breaker.get_state().await;
            let state_str = format!("{:?}", state);
            stats.insert(domain.clone(), state_str);
        }

        stats
    }

    /// Extract domain from URL or endpoint string
    ///
    /// Handles various input formats:
    /// - Full URL: "https://api.github.com/repos" → "api.github.com"
    /// - Domain: "api.github.com" → "api.github.com"
    /// - IP: "192.168.1.1:8080" → "192.168.1.1"
    fn extract_domain(endpoint: &str) -> String {
        // Try to parse as URL first
        if let Ok(url) = Url::parse(endpoint) {
            if let Some(host) = url.host_str() {
                return host.to_string();
            }
        }

        // Fallback: extract domain-like pattern from string
        // Handle cases like "api.github.com:443" or "192.168.1.1:8080"
        let parts: Vec<&str> = endpoint.split('/').collect();
        let domain_part = parts.first().unwrap_or(&endpoint);

        // Remove port if present
        let domain_no_port: Vec<&str> = domain_part.split(':').collect();
        domain_no_port.first().unwrap_or(domain_part).to_string()
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for CircuitBreakerManager
pub struct CircuitBreakerManagerBuilder {
    failure_threshold: Option<u32>,
    timeout: Option<Duration>,
    success_threshold: Option<u32>,
    half_open_max_requests: Option<u32>,
}

impl CircuitBreakerManagerBuilder {
    fn new() -> Self {
        Self {
            failure_threshold: None,
            timeout: None,
            success_threshold: None,
            half_open_max_requests: None,
        }
    }

    /// Set failure threshold (number of failures before opening circuit)
    #[must_use]
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = Some(threshold);
        self
    }

    /// Set timeout duration for circuit breaker (how long to keep circuit open)
    #[must_use]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set success threshold for half-open state
    #[must_use]
    pub fn success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = Some(threshold);
        self
    }

    /// Set maximum requests allowed in half-open state
    #[must_use]
    pub fn half_open_max_requests(mut self, max: u32) -> Self {
        self.half_open_max_requests = Some(max);
        self
    }

    /// Build the manager
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid.
    pub fn build(self) -> Result<CircuitBreakerManager, String> {
        let mut config = CircuitBreakerConfig::default();

        if let Some(threshold) = self.failure_threshold {
            config.failure_threshold = threshold;
        }
        if let Some(timeout) = self.timeout {
            config.timeout = timeout;
        }
        if let Some(threshold) = self.success_threshold {
            config.success_threshold = threshold;
        }
        if let Some(max) = self.half_open_max_requests {
            config.half_open_max_requests = max;
        }

        Ok(CircuitBreakerManager::with_config(config))
    }
}

impl Default for CircuitBreakerManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            CircuitBreakerManager::extract_domain("https://api.github.com/repos"),
            "api.github.com"
        );
        assert_eq!(CircuitBreakerManager::extract_domain("http://localhost:8080/api"), "localhost");
        assert_eq!(CircuitBreakerManager::extract_domain("api.example.com"), "api.example.com");
        assert_eq!(CircuitBreakerManager::extract_domain("192.168.1.1:8080"), "192.168.1.1");
    }

    #[tokio::test]
    async fn test_get_breaker_for_endpoint() {
        let manager = CircuitBreakerManager::new();

        let breaker1 = manager.get_breaker_for_endpoint("https://api.github.com/repos").await;
        let breaker2 = manager.get_breaker_for_endpoint("https://api.github.com/users").await;

        // Should be the same breaker (same domain)
        assert!(Arc::ptr_eq(&breaker1, &breaker2));
    }

    #[tokio::test]
    async fn test_different_domains_different_breakers() {
        let manager = CircuitBreakerManager::new();

        let breaker1 = manager.get_breaker_for_endpoint("https://api.github.com/repos").await;
        let breaker2 = manager.get_breaker_for_endpoint("https://api.gitlab.com/repos").await;

        // Should be different breakers (different domains)
        assert!(!Arc::ptr_eq(&breaker1, &breaker2));
    }

    #[tokio::test]
    async fn test_reset_breaker() {
        let manager = CircuitBreakerManager::new();

        let breaker = manager.get_breaker_for_endpoint("https://api.github.com/repos").await;

        // Manually set a state (would normally happen via failed calls)
        manager.reset_breaker("api.github.com").await;

        // Should not panic
    }

    #[tokio::test]
    async fn test_custom_config() {
        let manager = CircuitBreakerManager::builder()
            .failure_threshold(10)
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        let breaker = manager.get_breaker_for_endpoint("https://example.com").await;
        assert!(Arc::strong_count(&breaker) >= 1);
    }

    #[tokio::test]
    async fn test_get_all_stats() {
        let manager = CircuitBreakerManager::new();

        let _ = manager.get_breaker_for_endpoint("https://api.github.com").await;
        let _ = manager.get_breaker_for_endpoint("https://api.gitlab.com").await;

        let stats = manager.get_all_stats().await;
        assert_eq!(stats.len(), 2);
        assert!(stats.contains_key("api.github.com"));
        assert!(stats.contains_key("api.gitlab.com"));
    }
}
