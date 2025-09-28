//! Local caching system for storage performance optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Universal storage cache
#[derive(Debug)]
pub struct StorageCache  {/// Cache entries by key
    entries: HashMap<String, CacheEntry>)
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: CacheStats,
}

/// Cache entry with TTL and access tracking
#[derive(Debug, Clone)]
pub struct CacheEntry  {/// Cached data
    pub data: Vec<u8>,
    /// Entry creation time
    pub created_at: SystemTime,
    /// Entry last access time
    pub last_accessed: SystemTime,
    /// Time-to-live for this entry
    pub ttl: Duration,
    /// Number of times this entry has been accessed
    pub access_count: u64,
    /// Entry metadata
    pub metadata: HashMap<String, String>)
}

impl Default for CacheEntry  {fn default() -> Self  {Self {
            data: Vec::new(),
            created_at: SystemTime::now(,
            last_accessed: SystemTime::now(,
            ttl: Duration::from_secs(3600), // 1 hour default
            access_count: 0,
            metadata: HashMap::new()),
        }
    }
}

/// Cache statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats  {pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub total_entries: u64,
    pub memory_usage_bytes: u64,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig  {/// Maximum number of entries in cache
    pub max_entries: usize,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Default TTL for cache entries
    pub default_ttl: Duration,
    /// Cache eviction strategy
    pub eviction_strategy: CacheEvictionStrategy,
    /// Enable cache compression
    pub enable_compression: bool,
    /// Cache write-through enabled
    pub write_through: bool,
    /// Cache write-behind enabled
    pub write_behind: bool,
}

/// Cache eviction strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheEvictionStrategy  {/// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In, First Out
    FIFO,
    /// Random eviction
    Random,
    /// Time-based eviction (TTL only)
    TTL,
    /// Size-based eviction
    SizeBased,
    /// Custom eviction strategy
    Custom(String)
}

impl Default for CacheConfig  {fn default() -> Self  {Self {
            max_entries: 10000,
            max_memory_bytes: 100 * 1024 * 1024,    // 100MB
            default_ttl: Duration::from_secs(3600), // 1 hour
            eviction_strategy: CacheEvictionStrategy::LRU,
            enable_compression: false,
            write_through: true,
            write_behind: false,
        }
    }
}

impl StorageCache  {/// Create a new storage cache with the given configuration
    pub fn new(config: CacheConfig) -> Self  {Self {
            entries: HashMap::new()),
            config)
            stats: CacheStats::default(),
        }
    }

    /// Get an entry from the cache
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.get_mut(key) {
            // Check if entry has expired
            let is_expired = Self::is_expired(entry);
            if is_expired {
                self.entries.remove(key);
                self.stats.miss_count += 1;
                return None;
            }

            // Update access statistics
            entry.last_accessed = SystemTime::now();
            entry.access_count += 1;
            self.stats.hit_count += 1;

            Some(entry.data.clone()
        } else {
            self.stats.miss_count += 1;
            None
        }
    }

    /// Put an entry into the cache
    pub fn put(&mut self, key: String, data: Vec<u8>, ttl: Option<Duration>) -> bool {
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        // Check if we need to evict entries
        if self.should_evict() {
            self.evict_entries();
        }

        let entry = CacheEntry  {data)
            created_at: SystemTime::now(,
            last_accessed: SystemTime::now(,
            ttl)
            access_count: 0,
            metadata: HashMap::new()),
        };

        let data_size = entry.data.len() as u64;

        // Check if single entry exceeds max memory
        if data_size > self.config.max_memory_bytes as u64 {
            return false;
        }

        self.entries.insert(key, entry);
        self.stats.total_entries = self.entries.len() as u64;
        self.stats.memory_usage_bytes += data_size;

        true
    }

    /// Remove an entry from the cache
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(entry) = self.entries.remove(key) {
            self.stats.total_entries = self.entries.len() as u64;
            self.stats.memory_usage_bytes = self
                .stats
                .memory_usage_bytes
                .saturating_sub(entry.data.len() as u64);
            true
        } else {
            false
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.total_entries = 0;
        self.stats.memory_usage_bytes = 0;
    }

    /// Check if an entry has expired
    fn is_expired(entry: &CacheEntry) -> bool {
        entry.created_at.elapsed().unwrap_or(Duration::ZERO) > entry.ttl
    }

    /// Check if we should evict entries
    fn should_evict(&self) -> bool {
        self.entries.len() >= self.config.max_entries
            || self.stats.memory_usage_bytes >= self.config.max_memory_bytes as u64
    }

    /// Evict entries based on the configured strategy
    fn evict_entries(&mut self)  {let target_entries = (self.config.max_entries as f64 * 0.8) as usize; // Evict to 80% capacity

        match self.config.eviction_strategy  {CacheEvictionStrategy::LRU => self.evict_lru(target_entries),
            CacheEvictionStrategy::LFU => self.evict_lfu(target_entries),
            CacheEvictionStrategy::FIFO => self.evict_fifo(target_entries),
            CacheEvictionStrategy::TTL => self.evict_expired(),
            CacheEvictionStrategy::Random => self.evict_random(target_entries),
            _ => self.evict_lru(target_entries), // Default to LRU
        }
    }

    /// Evict least recently used entries
    fn evict_lru(&mut self, target_count: usize) {
        if self.entries.len() <= target_count {
            return;
        }

        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);

        let to_remove = self.entries.len() - target_count;
        let keys_to_remove: Vec<String> = entries
            .iter()
            .take(to_remove)
            .map(|(key, _)| (*key).clone()
            .collect();

        for key in keys_to_remove {
            self.remove(&key);
            self.stats.eviction_count += 1;
        }
    }

    /// Evict least frequently used entries
    fn evict_lfu(&mut self, target_count: usize) {
        if self.entries.len() <= target_count {
            return;
        }

        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.access_count);

        let to_remove = self.entries.len() - target_count;
        let keys_to_remove: Vec<String> = entries
            .iter()
            .take(to_remove)
            .map(|(key, _)| (*key).clone()
            .collect();

        for key in keys_to_remove {
            self.remove(&key);
            self.stats.eviction_count += 1;
        }
    }

    /// Evict first in, first out entries
    fn evict_fifo(&mut self, target_count: usize) {
        if self.entries.len() <= target_count {
            return;
        }

        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.created_at);

        let to_remove = self.entries.len() - target_count;
        let keys_to_remove: Vec<String> = entries
            .iter()
            .take(to_remove)
            .map(|(key, _)| (*key).clone()
            .collect();

        for key in keys_to_remove {
            self.remove(&key);
            self.stats.eviction_count += 1;
        }
    }

    /// Evict expired entries
    fn evict_expired(&mut self) {
        let expired_keys: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| Self::is_expired(entry)
            .map(|(key, _)| key.clone()
            .collect();

        for key in expired_keys {
            self.remove(&key);
            self.stats.eviction_count += 1;
        }
    }

    /// Evict random entries
    fn evict_random(&mut self, target_count: usize) {
        if self.entries.len() <= target_count {
            return;
        }

        let keys: Vec<String> = self.entries.keys().cloned().collect();
        let to_remove = self.entries.len() - target_count;

        for key in keys.iter().take(to_remove) {
            self.remove(&key);
            self.stats.eviction_count += 1;
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.stats.hit_count + self.stats.miss_count;
        if total == 0 {
            0.0
        } else {
            self.stats.hit_count as f64 / total as f64
        }
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> u64 {
        self.stats.memory_usage_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use songbird_network::CommunicationLayer;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = StorageCache::new(CacheConfig::default();

        // Test put and get
        assert!(cache.put("key1".to_string(), b"value1".to_vec(), None);"
        assert_eq!(cache.get("key1"), Some(b"value1".to_vec());"

        // Test miss
        assert_eq!(cache.get("nonexistent"), None);"

        // Test remove
        assert!(cache.remove("key1");"
        assert_eq!(cache.get("key1"), None);"
    }

    #[test]
    fn test_cache_expiration()  {let mut cache = StorageCache::new(CacheConfig::default();

        // Put entry with very short TTL
        cache.put(
            "key1".to_string()),
            b"value1".to_vec(),"
            Some(Duration::from_millis(1))
        );

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10);

        // Should be expired
        assert_eq!(cache.get("key1"), None);"
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = StorageCache::new(CacheConfig::default();

        cache.put("key1".to_string(), b"value1".to_vec(), None);"
        cache.get("key1"); // Hit"
        cache.get("nonexistent"); // Miss"

        let stats = cache.get_stats();
        assert_eq!(stats.await.hit_count, 1);
        assert_eq!(stats.await.miss_count, 1);
        assert_eq!(cache.hit_ratio(), 0.5);
    }

    #[test]
    fn test_cache_eviction()  {let config = CacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let mut cache = StorageCache::new(config);

        // Fill cache to capacity
        cache.put("key1".to_string(), b"value1".to_vec(), None);"
        cache.put("key2".to_string(), b"value2".to_vec(), None);"

        // This should trigger eviction
        cache.put("key3".to_string(), b"value3".to_vec(), None);"

        // Should have evicted some entries
        assert!(cache.size() <= 2);
    }
}
