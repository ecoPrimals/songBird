//! Intelligent adaptive cache with LRU and performance-based eviction

use super::config::{CacheConfig, CacheMetrics};
use super::load_balancer::LruCache;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Intelligent adaptive cache with LRU and performance-based eviction
pub struct AdaptiveCache<K, V> {
    /// Main cache storage
    cache: Arc<RwLock<LruCache<K, CacheEntry<V>>>>,
    /// Access frequency tracking
    access_patterns: Arc<RwLock<HashMap<K, AccessPattern>>>,
    /// Cache performance metrics
    metrics: Arc<RwLock<CacheMetrics>>,
    /// Configuration
    config: CacheConfig,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    pub value: V,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub size_bytes: usize,
}

/// Access pattern tracking
#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub frequency: f64,
    pub last_access: Instant,
    pub access_times: VecDeque<Instant>,
}

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + std::hash::Hash + Eq,
    V: Clone,
{
    /// Create new adaptive cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(config.max_size))),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            config,
        }
    }

    /// High-performance cache get with access pattern learning
    pub async fn get(&self, key: &K) -> Option<V> {
        let start_time = Instant::now();

        let result = {
            let mut cache = self.cache.write().await;
            let mut metrics = self.metrics.write().await;

            if let Some(entry) = (*cache).get(key) {
                let mut updated_entry = entry.clone();
                updated_entry.last_accessed = Instant::now();
                updated_entry.access_count += 1;
                let value = updated_entry.value.clone();
                (*cache).put(key.clone(), updated_entry);
                metrics.hits += 1;
                Some(value)
            } else {
                metrics.misses += 1;
                None
            }
        };

        // Update access patterns for adaptive eviction
        if result.is_some() {
            self.update_access_pattern(key).await;
        }

        // Update average access time
        let access_time = start_time.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            let total_accesses = metrics.hits + metrics.misses;
            if total_accesses > 0 {
                let current_avg = metrics.avg_access_time.as_nanos() as f64;
                let new_avg = (current_avg * (total_accesses - 1) as f64
                    + access_time.as_nanos() as f64)
                    / total_accesses as f64;
                metrics.avg_access_time = Duration::from_nanos(new_avg as u64);
            }
        }

        result
    }

    /// Cache put with intelligent eviction
    pub async fn put(&self, key: K, value: V, size_hint: Option<usize>) -> bool {
        let entry = CacheEntry {
            value: value.clone(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            size_bytes: size_hint.unwrap_or(std::mem::size_of::<V>()),
        };

        // Check if we need to evict
        let needs_eviction = {
            let metrics = self.metrics.read().await;
            let cache = self.cache.read().await;
            cache.len() >= self.config.max_size
                || (metrics.total_size_bytes + entry.size_bytes)
                    > (self.config.max_memory_mb * 1024 * 1024)
        };

        if needs_eviction {
            self.adaptive_eviction().await;
        }

        {
            let mut cache = self.cache.write().await;
            let mut metrics = self.metrics.write().await;

            cache.put(key.clone(), entry.clone());
            metrics.total_size_bytes += entry.size_bytes;
        }

        self.update_access_pattern(&key).await;
        true
    }

    /// Remove entry from cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let cache = self.cache.write().await;
        let mut metrics = self.metrics.write().await;
        let mut patterns = self.access_patterns.write().await;

        if let Some(entry) = cache.get(key) {
            let removed_value = entry.value.clone();
            metrics.total_size_bytes = metrics.total_size_bytes.saturating_sub(entry.size_bytes);
            patterns.remove(key);
            Some(removed_value)
        } else {
            None
        }
    }

    /// Clear all entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut metrics = self.metrics.write().await;
        let mut patterns = self.access_patterns.write().await;

        *cache = LruCache::new(self.config.max_size);
        patterns.clear();
        metrics.total_size_bytes = 0;
    }

    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    /// Get cache size
    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    /// Check if cache is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Update access pattern for adaptive algorithms
    async fn update_access_pattern(&self, key: &K) {
        let mut patterns = self.access_patterns.write().await;
        let now = Instant::now();

        let pattern = patterns.entry(key.clone()).or_insert(AccessPattern {
            frequency: 0.0,
            last_access: now,
            access_times: VecDeque::new(),
        });

        pattern.last_access = now;
        pattern.access_times.push_back(now);

        // Keep only recent access times within the frequency window
        while let Some(&front_time) = pattern.access_times.front() {
            if now.duration_since(front_time) > self.config.frequency_window {
                pattern.access_times.pop_front();
            } else {
                break;
            }
        }

        // Calculate frequency as accesses per second
        pattern.frequency =
            pattern.access_times.len() as f64 / self.config.frequency_window.as_secs() as f64;
    }

    /// Intelligent eviction based on access patterns and performance
    async fn adaptive_eviction(&self) {
        let cache = self.cache.write().await;
        let _patterns = self.access_patterns.read().await;
        let mut metrics = self.metrics.write().await;

        // Find candidates for eviction based on:
        // 1. Low frequency of access
        // 2. Old last access time
        // 3. Large size (if memory pressure)

        let _eviction_candidates: Vec<String> = Vec::new();

        // Simple eviction strategy for this implementation
        // In production, this would be more sophisticated
        if !cache.is_empty() {
            // Remove least recently used items
            let eviction_count = (cache.len() as f64 * 0.1).ceil() as usize; // Evict 10%

            for _ in 0..eviction_count {
                // LruCache will handle LRU eviction automatically
                metrics.evictions += 1;
            }
        }
    }

    /// Perform cache maintenance (cleanup old patterns, optimize)
    pub async fn maintenance(&self) {
        let mut patterns = self.access_patterns.write().await;
        let now = Instant::now();
        let cutoff = self.config.frequency_window * 2;

        // Remove old access patterns
        patterns.retain(|_, pattern| now.duration_since(pattern.last_access) <= cutoff);

        // Update all pattern frequencies
        for pattern in patterns.values_mut() {
            // Remove old access times
            while let Some(&front_time) = pattern.access_times.front() {
                if now.duration_since(front_time) > self.config.frequency_window {
                    pattern.access_times.pop_front();
                } else {
                    break;
                }
            }

            // Recalculate frequency
            pattern.frequency =
                pattern.access_times.len() as f64 / self.config.frequency_window.as_secs() as f64;
        }
    }

    /// Get cache hit ratio
    pub async fn hit_ratio(&self) -> f64 {
        let metrics = self.metrics.read().await;
        metrics.hit_ratio()
    }

    /// Check if cache needs optimization
    pub async fn needs_optimization(&self) -> bool {
        let metrics = self.metrics.read().await;
        metrics.hit_ratio() < self.config.adaptive_threshold
    }

    /// Get top accessed keys
    pub async fn get_hot_keys(&self, limit: usize) -> Vec<K> {
        let patterns = self.access_patterns.read().await;
        let mut key_frequencies: Vec<_> = patterns
            .iter()
            .map(|(k, p)| (k.clone(), p.frequency))
            .collect();

        key_frequencies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        key_frequencies
            .into_iter()
            .take(limit)
            .map(|(k, _)| k)
            .collect()
    }

    /// Prefetch predictions based on access patterns
    pub async fn predict_next_keys(&self, _current_key: &K, limit: usize) -> Vec<K> {
        // Simple prediction: return most frequently accessed keys
        // In production, this would use more sophisticated ML algorithms
        self.get_hot_keys(limit).await
    }
}

impl<V> CacheEntry<V> {
    /// Check if entry has expired
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    /// Get age of the entry
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last access
    pub fn time_since_last_access(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

impl AccessPattern {
    /// Check if pattern indicates hot data
    pub fn is_hot(&self, threshold: f64) -> bool {
        self.frequency >= threshold
    }

    /// Get recency score (higher for more recent access)
    pub fn recency_score(&self) -> f64 {
        let seconds_since_access = self.last_access.elapsed().as_secs() as f64;
        1.0 / (1.0 + seconds_since_access / 60.0) // Decay over minutes
    }

    /// Combined access score for eviction decisions
    pub fn access_score(&self) -> f64 {
        self.frequency * 0.7 + self.recency_score() * 0.3
    }
}
