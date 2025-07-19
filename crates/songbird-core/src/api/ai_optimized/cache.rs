use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{AccessPattern, AiWorkloadType};

/// AI-aware cache metrics
#[derive(Debug, Default, Clone, Serialize)]
pub struct AiCacheMetrics {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_rate: f64,
    pub memory_usage_bytes: u64,
    pub prediction_accuracy: f64,
    pub cache_size: usize,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl AiCacheMetrics {
    pub fn update_hit_rate(&mut self) {
        if self.total_requests > 0 {
            self.hit_rate = self.cache_hits as f64 / self.total_requests as f64;
            self.miss_rate = 1.0 - self.hit_rate;
        }
    }
}

/// Cached item with AI-aware metadata
#[derive(Debug, Clone)]
pub struct CachedItem {
    pub data: Arc<[u8]>,
    pub access_pattern: AccessPattern,
    pub size_bytes: usize,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
    pub ai_priority: f32,
}

/// AI-aware request cache with intelligent eviction
#[derive(Debug)]
pub struct AiAwareRequestCache {
    cache: HashMap<String, CachedItem>,
    access_patterns: HashMap<String, AccessPattern>,
    metrics: AiCacheMetrics,
    config: CacheConfig,
    total_memory_bytes: usize,
    last_cleanup: Instant,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_memory_bytes: usize,
    pub max_items: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub enable_predictive_caching: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_items: 10000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(60), // 1 minute
            enable_predictive_caching: true,
        }
    }
}

impl AiAwareRequestCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            access_patterns: HashMap::new(),
            metrics: AiCacheMetrics::default(),
            config,
            total_memory_bytes: 0,
            last_cleanup: Instant::now(),
        }
    }

    pub fn get(&mut self, key: &str, workload_type: Option<AiWorkloadType>) -> Option<Arc<[u8]>> {
        self.metrics.total_requests += 1;

        if let Some(item) = self.cache.get_mut(key) {
            // Check if item has expired
            if let Some(expires_at) = item.expires_at {
                if Instant::now() > expires_at {
                    self.cache.remove(key);
                    self.metrics.cache_misses += 1;
                    return None;
                }
            }

            // Update access pattern
            item.access_pattern.update_access(workload_type.clone());

            // Update global access patterns
            if let Some(pattern) = self.access_patterns.get_mut(key) {
                pattern.update_access(workload_type);
            }

            self.metrics.cache_hits += 1;
            self.metrics.update_hit_rate();
            Some(Arc::clone(&item.data))
        } else {
            self.metrics.cache_misses += 1;
            self.metrics.update_hit_rate();
            None
        }
    }

    pub fn put(
        &mut self,
        key: String,
        data: Vec<u8>,
        workload_type: Option<AiWorkloadType>,
    ) -> bool {
        let data_size = data.len();

        // Check memory limits
        if self.total_memory_bytes + data_size > self.config.max_memory_bytes {
            if !self.make_space_for(data_size) {
                return false;
            }
        }

        // Check item count limits
        if self.cache.len() >= self.config.max_items {
            if !self.evict_lru() {
                return false;
            }
        }

        let access_pattern = AccessPattern::new();
        let ai_priority = self.calculate_ai_priority(&workload_type, &access_pattern);

        let item = CachedItem {
            data: Arc::from(data.into_boxed_slice()),
            access_pattern: access_pattern.clone(),
            size_bytes: data_size,
            created_at: Instant::now(),
            expires_at: Some(Instant::now() + self.config.default_ttl),
            ai_priority,
        };

        self.cache.insert(key.clone(), item);
        self.access_patterns.insert(key, access_pattern);
        self.total_memory_bytes += data_size;
        self.metrics.cache_size = self.cache.len();

        true
    }

    pub fn invalidate(&mut self, key: &str) -> bool {
        if let Some(item) = self.cache.remove(key) {
            self.total_memory_bytes = self.total_memory_bytes.saturating_sub(item.size_bytes);
            self.access_patterns.remove(key);
            self.metrics.cache_size = self.cache.len();
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_patterns.clear();
        self.total_memory_bytes = 0;
        self.metrics.cache_size = 0;
    }

    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let mut expired_keys = Vec::new();

        for (key, item) in &self.cache {
            if let Some(expires_at) = item.expires_at {
                if now > expires_at {
                    expired_keys.push(key.clone());
                }
            }
        }

        for key in expired_keys {
            self.invalidate(&key);
        }

        self.last_cleanup = now;
    }

    pub fn get_metrics(&self) -> &AiCacheMetrics {
        &self.metrics
    }

    pub fn should_cleanup(&self) -> bool {
        Instant::now().duration_since(self.last_cleanup) > self.config.cleanup_interval
    }

    fn make_space_for(&mut self, required_bytes: usize) -> bool {
        let mut freed_bytes = 0;
        let mut candidates: Vec<_> = self
            .cache
            .iter()
            .map(|(k, v)| (k.clone(), v.ai_priority, v.size_bytes))
            .collect();

        // Sort by AI priority (lowest first for eviction)
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (key, _, size) in candidates {
            if freed_bytes >= required_bytes {
                break;
            }

            if self.invalidate(&key) {
                freed_bytes += size;
                self.metrics.eviction_rate += 1.0;
            }
        }

        freed_bytes >= required_bytes
    }

    fn evict_lru(&mut self) -> bool {
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = Instant::now();

        for (key, item) in &self.cache {
            if item.access_pattern.last_access < oldest_time {
                oldest_time = item.access_pattern.last_access;
                oldest_key = Some(key.clone());
            }
        }

        if let Some(key) = oldest_key {
            self.invalidate(&key);
            self.metrics.eviction_rate += 1.0;
            true
        } else {
            false
        }
    }

    fn calculate_ai_priority(
        &self,
        workload_type: &Option<AiWorkloadType>,
        pattern: &AccessPattern,
    ) -> f32 {
        let mut priority = 0.5; // Base priority

        // Adjust based on workload type
        if let Some(workload) = workload_type {
            priority += match workload {
                AiWorkloadType::ModelInference => 0.3,
                AiWorkloadType::ModelServing => 0.25,
                AiWorkloadType::StreamingProcessing => 0.2,
                AiWorkloadType::AgentCommunication => 0.15,
                AiWorkloadType::BatchProcessing => 0.1,
                AiWorkloadType::DataPreprocessing => 0.05,
                AiWorkloadType::Training => 0.0,
            };
        }

        // Adjust based on access frequency
        priority += (pattern.access_frequency * 0.2) as f32;

        // Adjust based on prediction accuracy
        priority += pattern.prediction_accuracy * 0.1;

        priority.clamp(0.0, 1.0)
    }
}

impl Default for AiAwareRequestCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

/// Global AI-aware cache
#[derive(Debug)]
pub struct AiAwareCache {
    inner: Arc<RwLock<AiAwareRequestCache>>,
}

impl Default for AiAwareCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

impl AiAwareCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AiAwareRequestCache::new(config))),
        }
    }

    pub async fn get(&self, key: &str, workload_type: Option<AiWorkloadType>) -> Option<Arc<[u8]>> {
        let mut cache = self.inner.write().await;
        cache.get(key, workload_type)
    }

    pub async fn put(
        &self,
        key: String,
        data: Vec<u8>,
        workload_type: Option<AiWorkloadType>,
    ) -> bool {
        let mut cache = self.inner.write().await;
        cache.put(key, data, workload_type)
    }

    pub async fn invalidate(&self, key: &str) -> bool {
        let mut cache = self.inner.write().await;
        cache.invalidate(key)
    }

    pub async fn clear(&self) {
        let mut cache = self.inner.write().await;
        cache.clear();
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.inner.write().await;
        if cache.should_cleanup() {
            cache.cleanup_expired();
        }
    }

    pub async fn get_metrics(&self) -> AiCacheMetrics {
        let cache = self.inner.read().await;
        cache.get_metrics().clone()
    }

    pub async fn get_cache_key(
        model_id: &str,
        input_hash: &str,
        parameters: &Option<HashMap<String, serde_json::Value>>,
    ) -> String {
        let params_hash = if let Some(params) = parameters {
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by_key(|(k, _)| *k);

            let params_str = sorted_params
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join(",");

            format!("{:x}", md5::compute(params_str.as_bytes()))
        } else {
            "no_params".to_string()
        };

        format!("{}:{}:{}", model_id, input_hash, params_hash)
    }

    pub async fn compute_input_hash(input: &serde_json::Value) -> String {
        let input_str = serde_json::to_string(input).unwrap_or_default();
        format!("{:x}", md5::compute(input_str.as_bytes()))
    }
}
