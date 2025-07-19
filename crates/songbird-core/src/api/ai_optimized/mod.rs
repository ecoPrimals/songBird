//! AI-Optimized API Layer
//!
//! Enhanced API endpoints and optimizations specifically designed for AI workloads,
//! including streaming, batching, intelligent caching, and predictive scaling.

pub mod cache;
pub mod types;

// Re-export everything from the modules
pub use cache::*;
pub use types::*;

use songbird_errors::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main AI-optimized API state
#[derive(Debug)]
pub struct AiOptimizedApiState {
    /// AI-aware caching layer
    pub cache: AiAwareCache,
    /// Model string pool for zero-copy operations
    pub string_pool: Arc<RwLock<ModelStringPool>>,
    /// Configuration
    pub config: AiOptimizedConfig,
}

/// Configuration for AI-optimized API
#[derive(Debug, Clone)]
pub struct AiOptimizedConfig {
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Enable string interning
    pub enable_string_interning: bool,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Request timeout
    pub request_timeout_ms: u64,
}

impl Default for AiOptimizedConfig {
    fn default() -> Self {
        Self {
            cache_config: CacheConfig::default(),
            enable_string_interning: true,
            max_concurrent_requests: 1000,
            request_timeout_ms: 30000,
        }
    }
}

impl AiOptimizedApiState {
    /// Create new AI-optimized API state
    pub fn new(config: AiOptimizedConfig) -> Self {
        Self {
            cache: AiAwareCache::new(config.cache_config.clone()),
            string_pool: Arc::new(RwLock::new(ModelStringPool::new())),
            config,
        }
    }

    /// Get cached inference result
    pub async fn get_cached_inference(
        &self,
        model_id: &str,
        input: &serde_json::Value,
        parameters: &Option<HashMap<String, serde_json::Value>>,
        workload_type: Option<AiWorkloadType>,
    ) -> Option<Arc<[u8]>> {
        let input_hash = AiAwareCache::compute_input_hash(input).await;
        let cache_key = AiAwareCache::get_cache_key(model_id, &input_hash, parameters).await;

        self.cache.get(&cache_key, workload_type).await
    }

    /// Store inference result in cache
    pub async fn cache_inference_result(
        &self,
        model_id: &str,
        input: &serde_json::Value,
        parameters: &Option<HashMap<String, serde_json::Value>>,
        result: Vec<u8>,
        workload_type: Option<AiWorkloadType>,
    ) -> bool {
        let input_hash = AiAwareCache::compute_input_hash(input).await;
        let cache_key = AiAwareCache::get_cache_key(model_id, &input_hash, parameters).await;

        self.cache.put(cache_key, result, workload_type).await
    }

    /// Get interned model ID
    pub async fn intern_model_id(&self, model_id: &str) -> Arc<str> {
        if self.config.enable_string_interning {
            let mut pool = self.string_pool.write().await;
            pool.intern_model_id(model_id)
        } else {
            Arc::from(model_id)
        }
    }

    /// Get cache metrics
    pub async fn get_cache_metrics(&self) -> AiCacheMetrics {
        self.cache.get_metrics().await
    }

    /// Cleanup expired cache entries
    pub async fn cleanup_cache(&self) {
        self.cache.cleanup_expired().await;
    }
}

impl Default for AiOptimizedApiState {
    fn default() -> Self {
        Self::new(AiOptimizedConfig::default())
    }
}
