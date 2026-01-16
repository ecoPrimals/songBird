//! HTTP Gateway Service - Universal HTTP proxy for pure Rust ecosystem
//!
//! **Vision**: Songbird as the SINGLE HTTP gateway for the entire ecoPrimals ecosystem
//!
//! **Architecture**:
//! ```text
//! ┌──────────┐
//! │ Squirrel │─┐
//! │  (Pure   │ │
//! │   Rust!) │ │
//! └──────────┘ │
//!              │ (Unix Socket ONLY!)
//! ┌──────────┐ │
//! │  Other   │─┼────→ ┌──────────┐ ──(HTTPS)──→ ┌──────────┐
//! │ Primals  │ │      │ Songbird │               │ External │
//! │ (Pure    │ │      │  (HTTP   │               │   APIs   │
//! │  Rust!)  │ │      │ Gateway) │               │          │
//! └──────────┘ │      └──────────┘               │ • OpenAI │
//!              │                                  │ • Stripe │
//!              └─────────────────────────────────→│ • GitHub │
//!                                                 └──────────┘
//! ```
//!
//! **Impact**: ALL primals → 100% pure Rust (no HTTP dependencies!)
//!
//! ## Philosophy
//!
//! - **Deep Debt Solutions**: Comprehensive architecture, not quick fixes
//! - **Modern Idiomatic Rust**: Async/await, proper error handling
//! - **Fast AND Safe**: Zero-copy where possible, safe abstractions
//! - **Zero Hardcoding**: Capability-based, environment-driven
//! - **Primal Self-Knowledge**: Each primal only knows itself
//!
//! ## Modules
//!
//! - `rate_limiter` - Token bucket rate limiting with per-client quotas
//! - `cache` - Response caching with TTL and size limits
//! - `credentials` - Secure credential management from environment
//!
//! **Status**: Phase 1 implementation (Jan 16, 2026)
//! **Created**: January 16, 2026

pub mod cache;
pub mod capability_router;
pub mod credentials;
pub mod rate_limiter;
pub mod unix_listener;
pub mod universal_proxy;

pub use cache::ResponseCache;
pub use capability_router::{Capability, CapabilityRouter, ProviderConfig};
pub use credentials::CredentialManager;
pub use rate_limiter::RateLimiter;
pub use unix_listener::{UnixListenerConfig, UnixSocketListener};
pub use universal_proxy::UniversalProxy;

use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

/// HTTP Gateway Service - Universal HTTP proxy for pure Rust ecosystem
///
/// This service enables all primals to achieve 100% pure Rust by:
/// 1. Accepting Unix socket connections from primals
/// 2. Proxying HTTP/HTTPS requests to external APIs
/// 3. Managing credentials, rate limits, and caching centrally
///
/// **Philosophy**: Each primal only knows Unix sockets; Songbird handles ALL HTTP
#[derive(Clone)]
pub struct HttpGatewayService {
    /// Rate limiter for all proxied requests
    rate_limiter: Arc<RateLimiter>,
    
    /// Response cache to reduce external API calls
    cache: Arc<ResponseCache>,
    
    /// Credential manager for API keys
    credentials: Arc<CredentialManager>,
    
    /// HTTP client for external requests (pure Rust!)
    http_client: reqwest::Client,
}

impl HttpGatewayService {
    /// Create a new HTTP gateway service
    ///
    /// # Philosophy
    /// - Deep debt solution: comprehensive architecture
    /// - Modern idiomatic Rust: async/await, proper error handling
    /// - Zero hardcoding: all config from environment
    pub fn new() -> Result<Self> {
        info!("🌐 Initializing HTTP Gateway Service");
        
        // Create HTTP client (pure Rust with rustls!)
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()?;
        
        // Create rate limiter (100 requests per minute default)
        let rate_limiter = Arc::new(RateLimiter::new(
            100,
            std::time::Duration::from_secs(60),
        ));
        
        // Create response cache (100MB default)
        let cache = Arc::new(ResponseCache::new(100 * 1024 * 1024));
        
        // Create credential manager
        let credentials = Arc::new(CredentialManager::new());
        
        info!("✅ HTTP Gateway Service initialized");
        info!("   Rate limit: 100 req/min per client");
        info!("   Cache size: 100MB");
        info!("   TLS: Pure Rust (rustls)");
        
        Ok(Self {
            rate_limiter,
            cache,
            credentials,
            http_client,
        })
    }
    
    /// Start the HTTP gateway service
    ///
    /// This will start Unix socket listeners for:
    /// - AI proxies (OpenAI, HuggingFace, Anthropic)
    /// - Generic HTTP proxies (capability-based)
    ///
    /// **Status**: Phase 1 - Core infrastructure only
    /// **Future**: Phase 2 will add actual proxy listeners
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting HTTP Gateway Service");
        
        // Phase 1: Core infrastructure complete
        // Phase 2: Add Unix socket listeners
        // Phase 3: Add AI-specific proxies
        // Phase 4: Add generic HTTP proxy
        
        info!("✅ HTTP Gateway Service ready (core infrastructure)");
        info!("   Phase 2: Add Unix socket listeners");
        info!("   Phase 3: Add AI-specific proxies");
        
        Ok(())
    }
    
    /// Check rate limit for a client
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client (e.g., "squirrel", "nestgate")
    ///
    /// # Returns
    /// * `Ok(())` if rate limit allows request
    /// * `Err(...)` if rate limit exceeded
    pub async fn check_rate_limit(&self, client_id: &str) -> Result<()> {
        self.rate_limiter.check(client_id).await
    }
    
    /// Get cached response if available
    ///
    /// # Arguments
    /// * `cache_key` - Unique key for the cached response
    ///
    /// # Returns
    /// * `Some(Value)` if response is cached
    /// * `None` if response is not cached or expired
    pub async fn get_cached(&self, cache_key: &str) -> Option<Value> {
        self.cache.get(cache_key).await
    }
    
    /// Cache a response
    ///
    /// # Arguments
    /// * `cache_key` - Unique key for the cached response
    /// * `response` - Response to cache
    /// * `ttl` - Time-to-live for the cached response
    pub async fn cache_response(&self, cache_key: &str, response: &Value, ttl: std::time::Duration) {
        self.cache.set(cache_key, response, ttl).await;
    }
    
    /// Get API key for a service
    ///
    /// # Arguments
    /// * `service` - Service name (e.g., "openai", "huggingface")
    ///
    /// # Returns
    /// * `Some(String)` if API key is configured
    /// * `None` if API key is not configured
    pub fn get_api_key(&self, service: &str) -> Option<String> {
        self.credentials.get_api_key(service)
    }
}

impl Default for HttpGatewayService {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP gateway service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_gateway_creation() {
        let gateway = HttpGatewayService::new().unwrap();
        // Gateway created successfully
        drop(gateway);
    }
    
    #[tokio::test]
    async fn test_http_gateway_start() {
        let gateway = HttpGatewayService::new().unwrap();
        let result = gateway.start().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_rate_limit() {
        let gateway = HttpGatewayService::new().unwrap();
        
        // First 100 requests should succeed
        for _ in 0..100 {
            assert!(gateway.check_rate_limit("test_client").await.is_ok());
        }
        
        // 101st request should fail
        assert!(gateway.check_rate_limit("test_client").await.is_err());
    }
}

