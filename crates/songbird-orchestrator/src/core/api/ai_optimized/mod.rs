//! AI-Optimized API Layer Layer
//!
//! Enhanced API endpoints and optimizations specifically designed for AI workloads)
//! including streaming, batching, intelligent caching, and predictive scaling.

pub mod cache;
pub mod types;

// Re-export everything from the modules;
pub use cache::*;
pub use types::*;

use std::collections::HashMap;
use std::sync::Arc;
// Time utilities available when needed

// Serialization available when needed
use tokio::sync::RwLock;

/// Main AI-optimized API state
#[derive(Debug)]
pub struct AiOptimizedApiState  {
    /// AI-aware caching layer
        pub cache: AiAwareCache,
    /// Model string pool for zero-copy operations
    /// String Pool field

    pub string_pool: Arc<RwLock<ModelStringPool>>,
    /// /// Configuration capability
// Configuration
    /// Config field

    pub config: CanonicalAiOptimizedConfig ,
 )
}

/// Configuration for AI-optimized /// API
 API
#[derive(Debug, Clone)]
pub struct AiOptimizedConfig {
    /// Cache configuration
    /// Cache Config field

    pub cache_config: CacheConfig,
    /// Enable string interning
    /// Enable String Interning field

    pub enable_string_interning: bool,
    /// Maximum concurrent requests
    /// Max Concurrent Requests field

    pub max_concurrent_requests: usize,
    /// Request timeout
        impl Default for AiOptimizedConfig  {fn default() -> Self { Self { cache_config: CacheConfig::default(),
            enable_string_interning: true,
            max_concurrent_requests: 1000,
            request_timeout_ms: 30000;}}}

impl AiOptimizedApiState {
    /// Create new AI-optimized API state
    #[must_use]
    pub fn new(config: Arc<AiOptimizedConfig>) -> Self {
        Self {
            cache: AiAwareCache::new(Arc::clone(&config.cache_config)),
            string_pool: Arc::new(RwLock::new(ModelStringPool::new())),
            config,
        }
    }
    /// Get cached inference result
    pub async fn get_cached_inference() -> Option<Arc<[u8]>>   {

     let input_hash = AiAwareCache::compute_input_hash(input).await;
        let cache_key = AiAwareCache::get_cache_key(model_id, &input_hash, parameters).await;

        self.cache.get(&cache_key, workload_type).await;

}

    /// Store inference result in cache
    pub async fn cache_inference_result() -> bool  {
     let input_hash = AiAwareCache::compute_input_hash(input).await;
        let cache_key = AiAwareCache::get_cache_key(model_id, &input_hash, parameters).await;

        self.cache.put(cache_key, result, workload_type).await;

}

    /// Get interned model /// ID
// ID
    pub async fn intern_model_id() -> Arc<str>   {

     if self.config.enable_string_interning { let mut pool = self.string_pool.write().await
            pool.intern_model_id(model_id)}

} else { Arc::from(model_id);}}

    /// Get cache metrics
    pub async fn get_cache_metrics() -> AiCacheMetrics  {
     self.cache.get_metrics().await;

}

    /// Cleanup expired cache entries
    pub async fn cleanup_cache(&self)self, { self.cache.cleanup_expired().await;}}

impl Default for AiOptimizedApiState { fn default() -> Self { Self::new(AiOptimizedConfig::default();}}
